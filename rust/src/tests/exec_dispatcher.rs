use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use super::ExecDispatcher;
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        invocation::Invocation,
        java_frame::JavaFrame,
        protocol::{DispatchOutcome, EngineExit, ExecBudget, MethodReturn},
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::jvalue::JValue,
    runtime::java_thread::JavaThread,
    tests::support::{
        assert_value, class_bytes, ensure_metaspace, parse_class,
        parse_class_with_runtime_max_locals,
    },
};

// A caller that requests real interpreted callees without depending on bytecode
// lowering. Each scripted transition consumes one unit of the shared budget.
struct Caller<'exec, 'meta: 'exec> {
    exits: VecDeque<EngineExit<'exec, 'meta>>,
    received: Rc<RefCell<Vec<MethodReturn>>>,
}

impl<'exec, 'meta: 'exec> JavaFrame<'exec, 'meta> for Caller<'exec, 'meta> {
    fn resume(&mut self, budget: &mut ExecBudget) -> ExecResult<EngineExit<'exec, 'meta>> {
        if !budget.is_startable() {
            return Ok(EngineExit::BudgetExhausted);
        }
        budget.charge(1);
        Ok(self
            .exits
            .pop_front()
            .expect("caller resumed past its return"))
    }

    fn accept_return(&mut self, value: MethodReturn) -> ExecResult<()> {
        self.received.borrow_mut().push(value);
        Ok(())
    }
}

fn assert_poll_rejected(thread: &mut JavaThread<'_, '_>) {
    assert!(thread.is_terminated());
    assert!(!thread.is_new());
    assert!(!thread.is_runnable());
    for fuel in [0, 1, 100] {
        let mut budget = ExecBudget::new(fuel);
        let error = ExecDispatcher::poll(thread, &mut budget)
            .err()
            .expect("terminated thread must reject polling");
        assert!(matches!(error.kind, ExecErrorKind::ThreadTerminated));
        assert_eq!(budget.remaining(), fuel);
        assert!(thread.is_terminated());
    }
}

#[test]
fn call_and_return_boundaries_preserve_budget_and_deliver_once() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let leaf = parse_class(&cld, &class_bytes("()I", &[0x10, 42, 0xac], 1, 0));
    let received = Rc::new(RefCell::new(Vec::new()));
    let caller = Caller {
        exits: VecDeque::from([
            EngineExit::Invoke(Invocation::lookup_declared(&leaf, "test", "()I", &[]).unwrap()),
            EngineExit::Invoke(Invocation::lookup_declared(&leaf, "test", "()I", &[]).unwrap()),
            EngineExit::Return(MethodReturn::Void),
        ]),
        received: received.clone(),
    };
    let mut thread = JavaThread::new();
    thread.start_with_frame(caller);
    for (fuel, returns) in [(0, 0), (1, 0), (2, 1), (0, 1), (1, 1), (2, 2)] {
        let mut budget = ExecBudget::new(fuel);
        assert!(matches!(
            ExecDispatcher::poll(&mut thread, &mut budget).unwrap(),
            DispatchOutcome::Yielded
        ));
        assert_eq!(budget.remaining(), 0);
        assert!(thread.is_runnable());
        assert_eq!(received.borrow().len(), returns);
    }
    assert!(matches!(
        received.borrow().as_slice(),
        [
            MethodReturn::Value(JValue::Int(42)),
            MethodReturn::Value(JValue::Int(42))
        ]
    ));
    let mut budget = ExecBudget::new(1);
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut budget).unwrap(),
        DispatchOutcome::Completed(MethodReturn::Void)
    ));
    assert_eq!(budget.remaining(), 0);
    assert!(thread.is_terminated());
    assert!(thread.current_frame().is_none());
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(10))
            .err()
            .unwrap()
            .kind,
        ExecErrorKind::ThreadTerminated
    ));
    assert_poll_rejected(&mut thread);
    assert_eq!(received.borrow().len(), 2);
}

#[test]
fn nested_frames_return_to_their_immediate_caller() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let leaf = parse_class(&cld, &class_bytes("()V", &[0xb1], 0, 0));
    let outer_returns = Rc::new(RefCell::new(Vec::new()));
    let inner_returns = Rc::new(RefCell::new(Vec::new()));
    let mut thread = JavaThread::new();
    thread.start_with_frame(Caller {
        exits: VecDeque::from([EngineExit::Return(MethodReturn::Value(JValue::Int(7)))]),
        received: outer_returns.clone(),
    });
    thread.push_frame(Caller {
        exits: VecDeque::from([
            EngineExit::Invoke(Invocation::lookup_declared(&leaf, "test", "()V", &[]).unwrap()),
            EngineExit::Return(MethodReturn::Value(JValue::Long(123))),
        ]),
        received: inner_returns.clone(),
    });
    let mut budget = ExecBudget::new(20);
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut budget).unwrap(),
        DispatchOutcome::Completed(MethodReturn::Value(JValue::Int(7)))
    ));
    assert_eq!(budget.remaining(), 16); // call, leaf return, inner return, entry return
    assert!(matches!(
        inner_returns.borrow().as_slice(),
        [MethodReturn::Void]
    ));
    assert!(matches!(
        outer_returns.borrow().as_slice(),
        [MethodReturn::Value(JValue::Long(123))]
    ));
    assert!(thread.is_terminated());
    assert!(thread.current_frame().is_none());
}

#[test]
fn interpreted_callers_accept_all_return_kinds_across_yields() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let cases = [
        ("I", 0x1a, 0xac, JValue::Int(-123)),
        ("J", 0x1e, 0xad, JValue::Long(i64::MIN + 123)),
        ("F", 0x22, 0xae, JValue::Float(-0.0)),
        ("D", 0x26, 0xaf, JValue::Double(-0.0)),
        (
            "Ljava/lang/Object;",
            0x2a,
            0xb0,
            JValue::Ref(NObjPtr::null()),
        ),
    ];
    for (kind, load, ret, value) in cases {
        let entry_desc = format!("(){kind}");
        let leaf_desc = format!("({kind}){kind}");
        let entry = parse_class(&cld, &class_bytes(&entry_desc, &[ret], 2, 0));
        let leaf = parse_class(&cld, &class_bytes(&leaf_desc, &[load, ret], 2, 2));
        for fuel in [1, 100] {
            let mut thread = JavaThread::new();
            ExecDispatcher::start(
                &mut thread,
                Invocation::lookup_declared(&entry, "test", &entry_desc, &[]).unwrap(),
            )
            .unwrap();
            thread.push_frame(
                InterpreterFrame::try_new(
                    Invocation::lookup_declared(&leaf, "test", &leaf_desc, &[value]).unwrap(),
                )
                .unwrap(),
            );
            let mut yielded = 0;
            loop {
                match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel)).unwrap() {
                    DispatchOutcome::Yielded => {
                        yielded += 1;
                        assert!(yielded < 4);
                    }
                    DispatchOutcome::Completed(result) => {
                        assert_value(result, value);
                        break;
                    }
                }
            }
            assert_eq!(yielded, if fuel == 1 { 2 } else { 0 });
            assert!(thread.current_frame().is_none());
        }
    }
    // Void returns must not add an operand to the interpreter caller.
    let owner = parse_class(&cld, &class_bytes("()V", &[0xb1], 0, 0));
    let mut thread = JavaThread::new();
    ExecDispatcher::start(
        &mut thread,
        Invocation::lookup_declared(&owner, "test", "()V", &[]).unwrap(),
    )
    .unwrap();
    thread.push_frame(
        InterpreterFrame::try_new(Invocation::lookup_declared(&owner, "test", "()V", &[]).unwrap())
            .unwrap(),
    );
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(2)).unwrap(),
        DispatchOutcome::Completed(MethodReturn::Void)
    ));
}

#[test]
fn invocation_creation_and_callee_errors_keep_their_context() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let invalid = parse_class_with_runtime_max_locals(&cld, &class_bytes("(I)V", &[0xb1], 0, 1), 0);
    let received = Rc::new(RefCell::new(Vec::new()));
    let mut thread = JavaThread::new();
    thread.start_with_frame(Caller {
        exits: VecDeque::from([EngineExit::Invoke(
            Invocation::lookup_declared(&invalid, "test", "(I)V", &[JValue::Int(1)]).unwrap(),
        )]),
        received: received.clone(),
    });
    let error = ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(10))
        .err()
        .unwrap();
    assert!(matches!(
        error.kind,
        ExecErrorKind::InsufficientLocalSlots {
            required: 1,
            available: 0
        }
    ));
    assert_eq!(error.brief.unwrap().desc, "(I)V");
    assert_poll_rejected(&mut thread);
    assert!(received.borrow().is_empty());

    let failing = parse_class(&cld, &class_bytes("()I", &[0x04, 0x03, 0x6c, 0xac], 2, 0));
    let mut thread = JavaThread::new();
    thread.start_with_frame(Caller {
        exits: VecDeque::from([EngineExit::Invoke(
            Invocation::lookup_declared(&failing, "test", "()I", &[]).unwrap(),
        )]),
        received: received.clone(),
    });
    let mut budget = ExecBudget::new(10);
    let error = ExecDispatcher::poll(&mut thread, &mut budget)
        .err()
        .unwrap();
    assert!(matches!(error.kind, ExecErrorKind::DivisionByZero));
    let brief = error.brief.unwrap();
    assert_eq!(brief.desc, "()I");
    assert_eq!(brief.instruction_index, Some(2));
    assert_eq!(budget.remaining(), 6);
    assert_poll_rejected(&mut thread);
    assert!(received.borrow().is_empty());
}

#[test]
fn return_delivery_error_terminates_thread_and_is_not_retried() {
    struct RejectReturn {
        deliveries: Rc<Cell<usize>>,
    }
    impl<'exec, 'meta: 'exec> JavaFrame<'exec, 'meta> for RejectReturn {
        fn resume(&mut self, _: &mut ExecBudget) -> ExecResult<EngineExit<'exec, 'meta>> {
            panic!("caller must not resume after rejecting the return")
        }
        fn accept_return(&mut self, _: MethodReturn) -> ExecResult<()> {
            self.deliveries.set(self.deliveries.get() + 1);
            Err(ExecError::new(ExecErrorKind::MismatchSlotType))
        }
    }
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let leaf = parse_class(&cld, &class_bytes("()V", &[0xb1], 0, 0));
    let deliveries = Rc::new(Cell::new(0));
    let mut thread = JavaThread::new();
    thread.start_with_frame(RejectReturn {
        deliveries: deliveries.clone(),
    });
    thread.push_frame(
        InterpreterFrame::try_new(Invocation::lookup_declared(&leaf, "test", "()V", &[]).unwrap())
            .unwrap(),
    );
    // Return delivery must fail immediately even if the callee used the last
    // unit of budget. Later polls must neither resume the caller nor redeliver.
    let mut budget = ExecBudget::new(1);
    let error = ExecDispatcher::poll(&mut thread, &mut budget)
        .err()
        .unwrap();
    assert!(matches!(error.kind, ExecErrorKind::MismatchSlotType));
    assert_eq!(budget.remaining(), 0);
    assert_eq!(deliveries.get(), 1);
    assert_poll_rejected(&mut thread);
    assert_eq!(deliveries.get(), 1);
}
