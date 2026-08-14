use crate::engines::slot::Slot;

pub enum StepOutcome {
    Continue,
    Return(Slot),
    Invoke,
}
