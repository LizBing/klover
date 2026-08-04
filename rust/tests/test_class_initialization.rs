mod test_harness;

use klover::{
    engine::{
        call::Invocation,
        exec_dispatcher::ExecDispatcher,
        exec_error::{ExecError, JavaExceptionKind},
        outcome::{PendingException, ThreadExit},
        resolved_method::ResolvedMethod,
        slot::Slot,
    },
    runtime::{runtime_error::StackError, thread_manager::ThreadManager},
};
use test_harness::{expect_int, load_class, run};

#[test]
fn class_init_frames_resume_suspended_static_field_operations() {
    let holder = load_class("ClassInitScenarios");

    // ClassInitChild.<clinit> observes the value installed by its superclass,
    // proving that the parent ClassInitFrame completed first.
    assert_eq!(
        expect_int(run(&holder, "readChildField", "()I", vec![])),
        42
    );

    // The putstatic operand must remain on the caller stack while
    // PutStaticNeedsInit.<clinit> runs, then be committed afterwards.
    assert_eq!(
        expect_int(run(
            &holder,
            "writeThenRead",
            "(I)I",
            vec![Slot::int(77)],
        )),
        77
    );
}

#[test]
fn default_method_interfaces_are_initialized_in_recursive_order() {
    let holder = load_class("ClassInitScenarios");

    // RootDefaultInterface runs before ChildDefaultInterface, then the class.
    // PlainParentInterface declares no default method and must remain untouched.
    assert_eq!(
        expect_int(run(
            &holder,
            "defaultInterfaceInitializationOrder",
            "()I",
            vec![],
        )),
        123
    );
}

#[test]
fn initializing_an_interface_does_not_initialize_its_parent() {
    let holder = load_class("ClassInitScenarios");

    assert_eq!(
        expect_int(run(
            &holder,
            "initializeInterfaceWithoutParent",
            "()I",
            vec![],
        )),
        4
    );
}

#[test]
fn failed_clinit_marks_the_class_erroneous() {
    let holder = load_class("ClassInitScenarios");

    let first = run(&holder, "readFailingClass", "()I", vec![]);
    assert!(matches!(
        first,
        ThreadExit::UncaughtException(PendingException::JVMGen(
            JavaExceptionKind::ArithmeticException
        ))
    ));

    let second = run(&holder, "readFailingClass", "()I", vec![]);
    assert!(matches!(
        second,
        ThreadExit::UncaughtException(PendingException::JVMGen(
            JavaExceptionKind::NoClassDefFoundError
        ))
    ));
}

#[test]
fn prerequisite_failure_marks_the_child_erroneous() {
    let holder = load_class("ClassInitScenarios");

    let first = run(&holder, "readFailingChild", "()I", vec![]);
    assert!(matches!(
        first,
        ThreadExit::UncaughtException(PendingException::JVMGen(
            JavaExceptionKind::ArithmeticException
        ))
    ));

    let second = run(&holder, "readFailingChild", "()I", vec![]);
    assert!(matches!(
        second,
        ThreadExit::UncaughtException(PendingException::JVMGen(
            JavaExceptionKind::NoClassDefFoundError
        ))
    ));
}

#[test]
fn initialized_class_is_not_run_twice() {
    let holder = load_class("ClassInitScenarios");

    assert_eq!(expect_int(run(&holder, "readOnceClass", "()I", vec![])), 1);
    assert_eq!(expect_int(run(&holder, "readOnceClass", "()I", vec![])), 1);
}

#[test]
fn same_thread_recursive_request_observes_in_progress_state() {
    let holder = load_class("ClassInitScenarios");

    assert_eq!(
        expect_int(run(&holder, "readRecursiveClass", "()I", vec![])),
        1
    );
}

#[test]
fn engine_error_releases_the_initialization_claim() {
    let holder = load_class("AbortableClassInit");
    let method = holder.find_declared_method("read", "()I").unwrap();
    let target = ResolvedMethod::new(holder.clone(), method);

    let mut manager = ThreadManager::new(0);
    let mut thread = manager.create_thread().unwrap();
    thread.start().unwrap();

    let error = ExecDispatcher::new()
        .enter_root(
            &mut thread,
            Invocation {
                target,
                args: vec![],
            },
        )
        .unwrap_err();
    assert!(matches!(error, ExecError::Stack(StackError::Overflow)));

    assert_eq!(expect_int(run(&holder, "read", "()I", vec![])), 6);
}
