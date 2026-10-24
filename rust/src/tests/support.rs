use std::{
    fs::{self, File},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    engines::{
        exec_dispatcher::ExecDispatcher,
        invocation::Invocation,
        protocol::{DispatchOutcome, ExecBudget, MethodReturn},
    },
    oops::{
        jvalue::JValue,
        normal_klass::{NormalKlass, UnlinkedNormalKlass},
    },
    runtime::{arguments::Arguments, java_thread::JavaThread, ms_api, vm},
};

/// Automatically removed files for child logs or class-path fixtures.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "klover-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test directory {}: {error}", path.display()),
            }
        }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn is_child(test_name: &str) -> bool {
    let test_name = test_name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap_or(test_name);
    std::env::var("KLOVER_TEST_CHILD").as_deref() == Ok(test_name)
}

/// Re-execute one exact test, giving it fresh process-wide VM state.
/// Files avoid pipe-buffer deadlocks; a deadline bounds native hangs too.
pub fn isolated(test_name: &str, body: impl FnOnce()) {
    let test_name = test_name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap_or(test_name);
    if is_child(test_name) {
        body();
        return;
    }
    let dir = TempDir::new();
    let log_path = dir.0.join("child.log");
    let log = File::create(&log_path).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            test_name,
            "--nocapture",
            "--test-threads=1",
            "--include-ignored",
        ])
        .env("KLOVER_TEST_CHILD", test_name)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .expect("start isolated test");
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let output = fs::read_to_string(&log_path).unwrap();
            assert!(status.success(), "{test_name}: {status}\n{output}");
            // libtest returns success even when an exact name selects no tests.
            assert!(
                output.contains("1 passed"),
                "child did not run its test: {output}"
            );
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "{test_name} timed out\n{}",
                fs::read_to_string(&log_path).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn ensure_metaspace() {
    assert!(matches!(
        ms_api::initialize(),
        Ok(()) | Err(ms_api::MsInitError::AlreadyInitialized)
    ));
}

pub fn fixture_dir() -> PathBuf {
    let path = std::env::var_os("KLOVER_TEST_CLASSES")
        .map(PathBuf::from)
        .expect("Java fixtures missing: run `make test` or `make test-classes`, then set KLOVER_TEST_CLASSES");
    assert!(path.is_absolute(), "KLOVER_TEST_CLASSES must be absolute");
    assert!(
        path.join("java/lang/Object.class").is_file(),
        "compile fixtures with `make test-classes`"
    );
    path
}

pub fn init_vm(path: PathBuf) {
    vm::try_init(Arguments {
        bs_class_path: path.to_str().unwrap().into(),
        ms_pretouch: false,
        xmx: 1024 * 1024,
    })
    .unwrap();
}

pub fn parse_class<'a>(cld: &'a ClassLoaderData<'a>, bytes: &[u8]) -> NormalKlass<'a> {
    let parsed = crate::class_parser::parse_class(bytes).expect("parse test class");
    NormalKlass::link(UnlinkedNormalKlass::build(parsed, Some(cld)), None).unwrap()
}

/// Minimal, single-method root class. Bytecode remains explicit at call sites.
/// This deliberately does not implement a general class-file assembler.
pub fn class_bytes(desc: &str, code: &[u8], max_stack: u16, max_locals: u16) -> Vec<u8> {
    class_bytes_with_flags(desc, code, max_stack, max_locals, 0x9)
}

pub fn class_bytes_with_flags(
    desc: &str,
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
    method_flags: u16,
) -> Vec<u8> {
    fn u2(out: &mut Vec<u8>, n: u16) {
        out.extend_from_slice(&n.to_be_bytes());
    }
    let mut out = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52];
    u2(&mut out, 6);
    for text in ["java/lang/Object"] {
        out.push(1);
        u2(&mut out, text.len() as u16);
        out.extend_from_slice(text.as_bytes());
    }
    out.extend_from_slice(&[7, 0, 1]);
    for text in ["test", desc, "Code"] {
        out.push(1);
        u2(&mut out, text.len() as u16);
        out.extend_from_slice(text.as_bytes());
    }
    for n in [0x21, 2, 0, 0, 0, 1, method_flags, 3, 4, 1, 5] {
        u2(&mut out, n);
    }
    out.extend_from_slice(&(12u32 + code.len() as u32).to_be_bytes());
    u2(&mut out, max_stack);
    u2(&mut out, max_locals);
    out.extend_from_slice(&(code.len() as u32).to_be_bytes());
    out.extend_from_slice(code);
    for _ in 0..3 {
        u2(&mut out, 0);
    }
    out
}

pub fn run_method(
    owner: &NormalKlass<'_>,
    name: &str,
    desc: &str,
    args: &[JValue],
    slice: u64,
) -> MethodReturn {
    assert!(slice > 0);
    let invocation = Invocation::try_new(owner, name, desc, args).unwrap();
    let mut thread = JavaThread::new();
    ExecDispatcher::start(&mut thread, invocation).unwrap();
    for _ in 0..100_000 {
        let mut budget = ExecBudget::new(slice);
        match ExecDispatcher::poll(&mut thread, &mut budget)
            .unwrap_or_else(|e| panic!("{name}{desc} args={args:?} slice={slice}: {e:?}"))
        {
            DispatchOutcome::Yielded => assert_eq!(budget.remaining(), 0),
            DispatchOutcome::Completed(value) => return value,
        }
    }
    panic!("{name}{desc} args={args:?} exceeded test execution limit");
}

pub fn assert_value(actual: MethodReturn, expected: JValue) {
    let MethodReturn::Value(actual) = actual else {
        panic!("expected {expected:?}, got void")
    };
    let equal = match (actual, expected) {
        (JValue::Int(a), JValue::Int(b)) => a == b,
        (JValue::Long(a), JValue::Long(b)) => a == b,
        (JValue::Float(a), JValue::Float(b)) => {
            (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
        }
        (JValue::Double(a), JValue::Double(b)) => {
            (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
        }
        (JValue::Ref(a), JValue::Ref(b)) => a.as_u32() == b.as_u32(),
        _ => false,
    };
    assert!(equal, "expected {expected:?}, got {actual:?}");
}
