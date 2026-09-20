use crate::engines::{
    exec_error::{ExecErrorKind, ExecResult},
    interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
};

macro_rules! local_access {
    ($load:ident, $store:ident, $slot:ident, $pop:ident) => {
        pub fn $load(f: &mut InterpreterFrame, idx: usize) -> ExecResult<StepControl> {
            let value = f.get_local(idx)?;
            if !matches!(value, Slot::$slot(_)) {
                return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
            }
            f.push(value);
            Ok(StepControl::Continue)
        }

        pub fn $store(f: &mut InterpreterFrame, idx: usize) -> ExecResult<StepControl> {
            let value = f.$pop()?;
            f.set_local(idx, Slot::$slot(value))?;
            Ok(StepControl::Continue)
        }
    };
}

local_access!(iload, istore, Int, pop_int);
local_access!(fload, fstore, Float, pop_float);
local_access!(aload, astore, Ref, pop_ref);

macro_rules! wide_local_access {
    ($load:ident, $store:ident, $high:ident, $low:ident) => {
        pub fn $load(f: &mut InterpreterFrame, idx: usize) -> ExecResult<StepControl> {
            let high = f.get_local(idx)?;
            let low = f.get_local(idx + 1)?;
            if !matches!((high, low), (Slot::$high(_), Slot::$low(_))) {
                return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
            }
            f.push(high);
            f.push(low);
            Ok(StepControl::Continue)
        }

        pub fn $store(f: &mut InterpreterFrame, idx: usize) -> ExecResult<StepControl> {
            let low = f.pop()?;
            let high = f.pop()?;
            if !matches!((high, low), (Slot::$high(_), Slot::$low(_))) {
                return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
            }
            f.set_wide_local(idx, high, low)?;
            Ok(StepControl::Continue)
        }
    };
}

wide_local_access!(lload, lstore, LongHigh, LongLow);
wide_local_access!(dload, dstore, DoubleHigh, DoubleLow);
