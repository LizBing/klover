use std::sync::Once;

use klover::{
    class_loader::bootstrap_cld::BootstrapCLD, engine::{
        call::Invocation, exec_dispatcher::ExecDispatcher, outcome::{RetValue, RunOutcome, ThreadExit}, resolved_method::ResolvedMethod, slot::Slot,
    }, runtime::{
        arguments::Arguments,
        thread_manager::ThreadManager,
        vm::vm_init,
    },
};

static VM_INIT: Once = Once::new();

fn init_vm() {
    VM_INIT.call_once(|| {
        vm_init(Arguments {
            bs_class_path: format!(
                "{}/../test_data/classes",
                env!("CARGO_MANIFEST_DIR")
            ),
            xmx: 64 * 1024 * 1024,
        });
    });
}

#[test]
fn run_simple_addition() {
    init_vm();

    let klass = BootstrapCLD::find_class("SimpleAddition").unwrap();
    let holder = klass.as_normal_ref().unwrap();

    let method = holder
        .find_declared_method("add", "(II)I")
        .unwrap();

    let target = ResolvedMethod::new(holder, method);

    let mut manager = ThreadManager::new(1024);
    let mut thread = manager.create_thread().unwrap();
    thread.start().unwrap();

    let mut dispatcher = ExecDispatcher::new();
    dispatcher
        .enter_root(
            &mut thread,
            Invocation {
                target,
                args: vec![Slot::int(1), Slot::int(2)],
            },
        )
        .unwrap();

    loop {
        match dispatcher.run_quantum(&mut thread, 64).unwrap() {
            RunOutcome::QuantumExpired => continue,

            RunOutcome::Terminated(ThreadExit::Returned(
                RetValue::Int(value),
            )) => {
                assert_eq!(value, 3);
                break;
            }

            outcome => panic!("unexpected outcome: {outcome:?}"),
        }
    }
}
