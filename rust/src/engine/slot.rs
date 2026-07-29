use crate::{gc_bindings::oop_handle::NObjPtr, engine::engine_error::{ExecError, ExecResult}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    Empty,
    Int,
    Float,
    Ref,
    LongHigh,
    LongLow,
    DoubleHigh,
    DoubleLow,
    ReturnAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    bits: u32,
    kind: SlotKind,
}

impl Slot {
    pub const EMPTY: Self = Self {
        bits: 0,
        kind: SlotKind::Empty,
    };

    pub fn int(value: i32) -> Self {
        Self {
            bits: value as u32,
            kind: SlotKind::Int,
        }
    }

    pub fn float(value: f32) -> Self {
        Self {
            bits: value.to_bits(),
            kind: SlotKind::Float,
        }
    }

    pub fn reference(value: NObjPtr) -> Self {
        Self {
            bits: value,
            kind: SlotKind::Ref,
        }
    }

    pub fn long_high(value: i64) -> Self {
        Self {
            bits: ((value as u64) >> 32) as u32,
            kind: SlotKind::LongHigh,
        }
    }

    pub fn long_low(value: i64) -> Self {
        Self {
            bits: value as u32,
            kind: SlotKind::LongLow,
        }
    }

    pub fn double_high(value: f64) -> Self {
        let bits = value.to_bits();

        Self {
            bits: (bits >> 32) as u32,
            kind: SlotKind::DoubleHigh,
        }
    }

    pub fn double_low(value: f64) -> Self {
        Self {
            bits: value.to_bits() as u32,
            kind: SlotKind::DoubleLow,
        }
    }

    pub fn as_int(self) -> ExecResult<i32> {
        match self.kind {
            SlotKind::Int => Ok(self.bits as i32),
            actual => Err(ExecError::SlotTypeMismatch {
                expected: SlotKind::Int,
                actual,
            }),
        }
    }

    pub fn as_ref(self) -> ExecResult<NObjPtr> {
        match self.kind {
            SlotKind::Ref => Ok(self.bits),
            actual => Err(ExecError::SlotTypeMismatch {
                expected: SlotKind::Ref,
                actual,
            }),
        }
    }
}
