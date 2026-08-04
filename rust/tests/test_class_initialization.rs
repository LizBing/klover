mod test_harness;

use klover::{
    engine::{
        exec_error::{ExecError, JavaExceptionKind},
        outcome::{PendingException, ThreadExit},
        slot::Slot,
    },
    oops::oops_errors::ClassInitError,
};
use test_harness::{expect_int, load_class, run, try_run};

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

    let second = try_run(&holder, "readFailingClass", "()I", vec![]).unwrap_err();
    assert!(matches!(
        second,
        ExecError::ClassInitialization(ClassInitError::Erroneous)
    ));
}
