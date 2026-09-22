mod classfile;

use klover::{
    class_loader::bs_cld::BootstrapCLD,
    engines::{
        exec_dispatcher::{DispatchOutcome, ExecBudget, ExecDispatcher, MethodReturn},
        exec_error::ExecResult,
        invocation::Invocation,
    },
    oops::jvalue::JValue,
    runtime::{arguments::Arguments, java_thread::JavaThread, vm},
};
use std::{path::PathBuf, sync::Once};

pub fn fixture_root() -> PathBuf {
    std::env::var_os("KLOVER_TEST_CLASSES")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../build/test-classes"))
}

pub fn init_vm() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let source = fixture_root();
        // Separate generated test artifacts from the fixture builder's tracked
        // output. The integration test binary owns one VM and one class path.
        let root = source
            .parent()
            .unwrap()
            .join("switch-integration-classes")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(root.join("java/lang")).unwrap();
        for entry in std::fs::read_dir(&source).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|value| value.to_str()) == Some("class") {
                std::fs::copy(entry.path(), root.join(entry.file_name())).unwrap();
            }
        }
        std::fs::copy(
            source.join("java/lang/Object.class"),
            root.join("java/lang/Object.class"),
        )
        .unwrap();
        std::fs::write(root.join("SwitchBytecodeOps.class"), classfile::build()).unwrap();
        vm::try_init(Arguments {
            bs_class_path: root.to_string_lossy().into_owned(),
            xmx: 64 * 1024 * 1024,
        })
        .unwrap();
        // Avoid racing the bootstrap loader's non-atomic class definition.
        for class in ["SwitchOps", "SwitchBytecodeOps"] {
            BootstrapCLD::find_class(class).unwrap();
        }
    });
}

pub fn invoke(
    class: &str,
    method: &str,
    desc: &str,
    args: &[JValue],
    fuel: u64,
) -> ExecResult<MethodReturn> {
    init_vm();
    let owner = BootstrapCLD::find_class(class)
        .unwrap()
        .as_normal_klass_ref()
        .unwrap();
    let mut thread = JavaThread::new();
    ExecDispatcher::start(&mut thread, Invocation::try_new(owner, method, desc, args)?)?;
    for _ in 0..10000 {
        // Interleave zero-budget polls with actual progress, including after switches.
        assert!(matches!(
            ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(0))?,
            DispatchOutcome::Yielded
        ));
        match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel))? {
            DispatchOutcome::Yielded => {}
            DispatchOutcome::Completed(value) => return Ok(value),
        }
    }
    panic!("{class}.{method}{desc} did not terminate (fuel={fuel})");
}

pub fn run(class: &str, method: &str, desc: &str, args: &[JValue], fuel: u64) -> MethodReturn {
    invoke(class, method, desc, args, fuel)
        .unwrap_or_else(|error| panic!("{class}.{method}{desc}: {error:?}"))
}
