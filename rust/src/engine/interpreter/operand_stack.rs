use crate::engine::{
    engine_error::{ExecError, ExecResult},
    slot::{Slot, SlotKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StackValue {
    Category1(Slot),
    Category2(Slot, Slot),
}

impl StackValue {
    pub(crate) fn is_category1(self) -> bool {
        matches!(self, Self::Category1(_))
    }

    pub(crate) fn is_category2(self) -> bool {
        matches!(self, Self::Category2(_, _))
    }

    fn slot_count(self) -> usize {
        match self {
            Self::Category1(_) => 1,
            Self::Category2(_, _) => 2,
        }
    }

    fn validate(self) -> ExecResult<()> {
        let valid = match self {
            Self::Category1(slot) => matches!(
                slot.kind(),
                SlotKind::Int | SlotKind::Float | SlotKind::Ref | SlotKind::ReturnAddress
            ),
            Self::Category2(high, low) => matches!(
                (high.kind(), low.kind()),
                (SlotKind::LongHigh, SlotKind::LongLow)
                    | (SlotKind::DoubleHigh, SlotKind::DoubleLow)
            ),
        };

        if valid {
            Ok(())
        } else {
            Err(ExecError::InvalidOperandStackShape)
        }
    }
}

#[derive(Debug)]
pub(crate) struct OperandStack {
    slots: Vec<Slot>,
    max_slots: usize,
}

impl OperandStack {
    pub(crate) fn new(max_slots: usize) -> Self {
        Self {
            slots: Vec::with_capacity(max_slots),
            max_slots,
        }
    }

    fn ensure_capacity(&self, additional_slots: usize) -> ExecResult<()> {
        let new_len = self
            .slots
            .len()
            .checked_add(additional_slots)
            .ok_or(ExecError::OperandStackOverflow)?;

        if new_len > self.max_slots {
            return Err(ExecError::OperandStackOverflow);
        }

        Ok(())
    }

    pub(crate) fn push_slot(&mut self, slot: Slot) -> ExecResult<()> {
        match slot.kind() {
            SlotKind::Int | SlotKind::Float | SlotKind::Ref | SlotKind::ReturnAddress => {}
            _ => return Err(ExecError::InvalidOperandStackShape),
        }

        self.ensure_capacity(1)?;
        self.slots.push(slot);
        Ok(())
    }

    pub(crate) fn pop_slot(&mut self) -> ExecResult<Slot> {
        let value = self.top_values(1)?[0];
        let StackValue::Category1(slot) = value else {
            return Err(ExecError::InvalidOperandStackShape);
        };
        self.replace_top_values(1, &[])?;
        Ok(slot)
    }

    pub(crate) fn push_value(&mut self, value: StackValue) -> ExecResult<()> {
        value.validate()?;
        self.ensure_capacity(value.slot_count())?;
        self.append_value(value);
        Ok(())
    }

    pub(crate) fn top_values(&self, count: usize) -> ExecResult<Vec<StackValue>> {
        let mut values = Vec::with_capacity(count);
        let mut cursor = self.slots.len();

        for _ in 0..count {
            if cursor == 0 {
                return Err(ExecError::OperandStackUnderflow);
            }

            let top = self.slots[cursor - 1];
            match top.kind() {
                SlotKind::Int | SlotKind::Float | SlotKind::Ref | SlotKind::ReturnAddress => {
                    values.push(StackValue::Category1(top));
                    cursor -= 1;
                }
                SlotKind::LongLow => {
                    if cursor < 2 || self.slots[cursor - 2].kind() != SlotKind::LongHigh {
                        return Err(ExecError::InvalidOperandStackShape);
                    }
                    values.push(StackValue::Category2(self.slots[cursor - 2], top));
                    cursor -= 2;
                }
                SlotKind::DoubleLow => {
                    if cursor < 2 || self.slots[cursor - 2].kind() != SlotKind::DoubleHigh {
                        return Err(ExecError::InvalidOperandStackShape);
                    }
                    values.push(StackValue::Category2(self.slots[cursor - 2], top));
                    cursor -= 2;
                }
                SlotKind::Empty | SlotKind::LongHigh | SlotKind::DoubleHigh => {
                    return Err(ExecError::InvalidOperandStackShape);
                }
            }
        }

        Ok(values)
    }

    pub(crate) fn replace_top_values(
        &mut self,
        consumed_values: usize,
        produced_bottom_to_top: &[StackValue],
    ) -> ExecResult<()> {
        let consumed = self.top_values(consumed_values)?;
        let consumed_slots: usize = consumed.iter().map(|value| value.slot_count()).sum();
        let produced_slots: usize = produced_bottom_to_top
            .iter()
            .map(|value| value.slot_count())
            .sum();

        for value in produced_bottom_to_top {
            value.validate()?;
        }
        let retained_slots = self.slots.len() - consumed_slots;
        let new_len = retained_slots
            .checked_add(produced_slots)
            .ok_or(ExecError::OperandStackOverflow)?;

        if new_len > self.max_slots {
            return Err(ExecError::OperandStackOverflow);
        }

        self.slots.truncate(retained_slots);
        for value in produced_bottom_to_top {
            self.append_value(*value);
        }
        Ok(())
    }

    fn append_value(&mut self, value: StackValue) {
        match value {
            StackValue::Category1(slot) => self.slots.push(slot),
            StackValue::Category2(high, low) => {
                self.slots.push(high);
                self.slots.push(low);
            }
        }
    }

    pub(crate) fn take_top_slots(&mut self, slot_count: usize) -> ExecResult<Vec<Slot>> {
        if slot_count > self.slots.len() {
            return Err(ExecError::OperandStackUnderflow);
        }

        let start = self.slots.len() - slot_count;
        if start < self.slots.len() {
            match self.slots[start].kind() {
                SlotKind::LongLow | SlotKind::DoubleLow => {
                    return Err(ExecError::InvalidOperandStackShape);
                }
                _ => {}
            }
        }

        Ok(self.slots.split_off(start))
    }
}
