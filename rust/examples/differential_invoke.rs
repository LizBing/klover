//! Test-only process adapter. No invocation, loading, or interpreter semantics live here.
use klover::oops::symbol_table::SymbolTable;
use klover::{
    class_loader::bs_cld::BootstrapCLD,
    engines::{
        exec_dispatcher::{DispatchOutcome, ExecBudget, ExecDispatcher, MethodReturn},
        invocation::Invocation,
    },
    oops::jvalue::JValue,
    runtime::{arguments::Arguments, java_thread::JavaThread, vm},
};
use std::{env, fs, path::Path, process};

fn descriptor(desc: &str) -> Result<(&str, u8), String> {
    let (params, ret) = desc.split_once(')').ok_or("invalid descriptor")?;
    let params = params.strip_prefix('(').ok_or("invalid descriptor")?;
    if !params.bytes().all(|b| b"ZBCSIJFD".contains(&b))
        || ret.len() != 1
        || !b"ZBCSIJFDV".contains(&ret.as_bytes()[0])
    {
        return Err("only primitive parameters and primitive/void returns are supported".into());
    }
    Ok((params, ret.as_bytes()[0]))
}

fn argument(kind: u8, text: &str) -> Result<JValue, String> {
    let bad = || format!("invalid {} argument: {text}", kind as char);
    Ok(match kind {
        b'Z' => match text {
            "0" => JValue::Boolean(0),
            "1" => JValue::Boolean(1),
            _ => return Err(bad()),
        },
        b'B' => JValue::Byte(text.parse().map_err(|_| bad())?),
        b'C' => JValue::Char(text.parse().map_err(|_| bad())?),
        b'S' => JValue::Short(text.parse().map_err(|_| bad())?),
        b'I' => JValue::Int(text.parse().map_err(|_| bad())?),
        b'J' => JValue::Long(text.parse().map_err(|_| bad())?),
        b'F' if text.len() == 8 => JValue::Float(f32::from_bits(
            u32::from_str_radix(text, 16).map_err(|_| bad())?,
        )),
        b'D' if text.len() == 16 => JValue::Double(f64::from_bits(
            u64::from_str_radix(text, 16).map_err(|_| bad())?,
        )),
        _ => return Err(bad()),
    })
}

fn result(kind: u8, value: MethodReturn) -> Result<String, String> {
    Ok(match (kind, value) {
        (b'V', MethodReturn::Void) => "void".into(),
        (b'Z' | b'B' | b'C' | b'S' | b'I', MethodReturn::Value(JValue::Int(v))) => v.to_string(),
        (b'J', MethodReturn::Value(JValue::Long(v))) => v.to_string(),
        (b'F', MethodReturn::Value(JValue::Float(v))) => {
            if v.is_nan() {
                "nan".into()
            } else {
                format!("{:08x}", v.to_bits())
            }
        }
        (b'D', MethodReturn::Value(JValue::Double(v))) => {
            if v.is_nan() {
                "nan".into()
            } else {
                format!("{:016x}", v.to_bits())
            }
        }
        _ => return Err("return value does not match descriptor".into()),
    })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 6 {
        return Err(
            "usage: differential_invoke CLASSPATH CLASS METHOD DESCRIPTOR FUEL QUANTUM [ARGS...]"
                .into(),
        );
    }
    let (params, ret) = descriptor(&args[3])?;
    if args.len() - 6 != params.len() {
        return Err("argument count does not match descriptor".into());
    }
    let values: Vec<_> = params
        .bytes()
        .zip(&args[6..])
        .map(|(kind, value)| argument(kind, value))
        .collect::<Result<_, _>>()?;
    let mut fuel: u64 = args[4].parse().map_err(|_| "invalid fuel")?;
    let quantum: u64 = args[5].parse().map_err(|_| "invalid quantum")?;
    if fuel == 0 || quantum == 0 {
        return Err("fuel and quantum must be positive".into());
    }
    if args[1]
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
        || args[1].contains(['.', '\\'])
    {
        return Err("class must be a JVM internal name".into());
    }

    // Invocation currently trusts its caller. Validate test-interface inputs here,
    // using the existing parser rather than widening the production VM API.
    let bytes = fs::read(Path::new(&args[0]).join(format!("{}.class", args[1])))
        .map_err(|e| format!("read class: {e}"))?;
    let cf = cafebabe::parse_class(&bytes).map_err(|e| format!("parse class: {e:?}"))?;
    if &*cf.this_class != args[1] {
        return Err("class name does not match class file".into());
    }
    if cf.super_class.as_deref() != Some("java/lang/Object")
        || !cf.interfaces.is_empty()
        || cf.methods.iter().any(|m| m.name == "<clinit>")
    {
        return Err(
            "test entry must extend Object, have no interfaces and no class initializer".into(),
        );
    }
    let method = cf
        .methods
        .iter()
        .find(|m| m.name == args[2] && m.descriptor.to_string() == args[3])
        .ok_or("method not found")?;
    if !method
        .access_flags
        .contains(cafebabe::MethodAccessFlags::STATIC)
        || !method
            .access_flags
            .contains(cafebabe::MethodAccessFlags::PUBLIC)
    {
        return Err("test entry must be public static".into());
    }

    vm::try_init(Arguments {
        bs_class_path: args[0].clone(),
        ms_pretouch: false,
        xmx: 64 * 1024 * 1024,
    })
    .map_err(|e| format!("VM initialization: {e:?}"))?;
    let owner = BootstrapCLD::find_class(&SymbolTable::intern(&args[1]))
        .map_err(|e| format!("load: {e:?}"))?
        .as_normal_klass()
        .ok_or("not a normal class")?;
    let invocation = Invocation::try_new(owner, &args[2], &args[3], &values)
        .map_err(|e| format!("invocation: {e:?}"))?;
    let slots: usize = params
        .bytes()
        .map(|b| if b == b'J' || b == b'D' { 2 } else { 1 })
        .sum();
    if slots > invocation.code().max_locals() {
        return Err("arguments exceed method locals".into());
    }
    let mut thread = JavaThread::new();
    ExecDispatcher::start(&mut thread, invocation).map_err(|e| format!("start: {e:?}"))?;
    while fuel > 0 {
        let slice = fuel.min(quantum);
        fuel -= slice;
        match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(slice))
            .map_err(|e| format!("execute: {e:?}"))?
        {
            DispatchOutcome::Yielded => {}
            DispatchOutcome::Completed(value) => {
                println!("KLOVER_DIFF_V1\t{}\t{}", ret as char, result(ret, value)?);
                return Ok(());
            }
        }
    }
    Err("execution fuel exhausted".into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("differential_invoke: {error}");
        process::exit(2);
    }
}
