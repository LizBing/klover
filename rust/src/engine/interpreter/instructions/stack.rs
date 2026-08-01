use crate::engine::{
    exec_error::{ExecError, ExecResult},
    interpreter::{
        interpreter_frame::InterpreterFrame,
        operand_stack::{OperandStack, StackValue},
    },
    outcome::StepOutcome,
};

fn invalid(opcode: u8) -> ExecError {
    ExecError::InvalidStackOperation { opcode }
}

fn require_category1(value: StackValue, opcode: u8) -> ExecResult<()> {
    if value.is_category1() {
        Ok(())
    } else {
        Err(invalid(opcode))
    }
}

fn pop_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(1)?;
    require_category1(values[0], 0x57)?;
    stack.replace_top_values(1, &[])
}

fn pop2_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let top = stack.top_values(1)?[0];
    if top.is_category2() {
        return stack.replace_top_values(1, &[]);
    }

    let values = stack.top_values(2)?;
    require_category1(values[1], 0x58)?;
    stack.replace_top_values(2, &[])
}

fn dup_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let value = stack.top_values(1)?[0];
    require_category1(value, 0x59)?;
    stack.replace_top_values(1, &[value, value])
}

fn dup_x1_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(2)?;
    let [value1, value2] = [values[0], values[1]];
    require_category1(value1, 0x5a)?;
    require_category1(value2, 0x5a)?;
    stack.replace_top_values(2, &[value1, value2, value1])
}

fn dup_x2_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(2)?;
    let [value1, value2] = [values[0], values[1]];
    require_category1(value1, 0x5b)?;

    if value2.is_category2() {
        return stack.replace_top_values(2, &[value1, value2, value1]);
    }

    let values = stack.top_values(3)?;
    let value3 = values[2];
    require_category1(value2, 0x5b)?;
    require_category1(value3, 0x5b)?;
    stack.replace_top_values(3, &[value1, value3, value2, value1])
}

fn dup2_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let value1 = stack.top_values(1)?[0];
    if value1.is_category2() {
        return stack.replace_top_values(1, &[value1, value1]);
    }

    let values = stack.top_values(2)?;
    let value2 = values[1];
    require_category1(value1, 0x5c)?;
    require_category1(value2, 0x5c)?;
    stack.replace_top_values(2, &[value2, value1, value2, value1])
}

fn dup2_x1_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(2)?;
    let [value1, value2] = [values[0], values[1]];

    if value1.is_category2() {
        require_category1(value2, 0x5d)?;
        return stack.replace_top_values(2, &[value1, value2, value1]);
    }

    require_category1(value1, 0x5d)?;
    require_category1(value2, 0x5d)?;
    let values = stack.top_values(3)?;
    let value3 = values[2];
    require_category1(value3, 0x5d)?;
    stack.replace_top_values(3, &[value2, value1, value3, value2, value1])
}

fn dup2_x2_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(2)?;
    let [value1, value2] = [values[0], values[1]];

    if value1.is_category2() && value2.is_category2() {
        return stack.replace_top_values(2, &[value1, value2, value1]);
    }

    if value1.is_category2() {
        require_category1(value2, 0x5e)?;
        let values = stack.top_values(3)?;
        let value3 = values[2];
        require_category1(value3, 0x5e)?;
        return stack.replace_top_values(3, &[value1, value3, value2, value1]);
    }

    require_category1(value1, 0x5e)?;
    require_category1(value2, 0x5e)?;
    let values = stack.top_values(3)?;
    let value3 = values[2];

    if value3.is_category2() {
        return stack.replace_top_values(3, &[value2, value1, value3, value2, value1]);
    }

    let values = stack.top_values(4)?;
    let value4 = values[3];
    require_category1(value3, 0x5e)?;
    require_category1(value4, 0x5e)?;
    stack.replace_top_values(4, &[value2, value1, value4, value3, value2, value1])
}

fn swap_impl(stack: &mut OperandStack) -> ExecResult<()> {
    let values = stack.top_values(2)?;
    let [value1, value2] = [values[0], values[1]];
    require_category1(value1, 0x5f)?;
    require_category1(value2, 0x5f)?;
    stack.replace_top_values(2, &[value1, value2])
}

macro_rules! stack_instruction {
    ($name:ident, $implementation:ident) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
            $implementation(f.operand_stack_mut())?;
            Ok(StepOutcome::Continue)
        }
    };
}

stack_instruction!(pop, pop_impl);
stack_instruction!(pop2, pop2_impl);
stack_instruction!(dup, dup_impl);
stack_instruction!(dup_x1, dup_x1_impl);
stack_instruction!(dup_x2, dup_x2_impl);
stack_instruction!(dup2, dup2_impl);
stack_instruction!(dup2_x1, dup2_x1_impl);
stack_instruction!(dup2_x2, dup2_x2_impl);
stack_instruction!(swap, swap_impl);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::slot::Slot;

    fn category1(value: i32) -> StackValue {
        StackValue::Category1(Slot::int(value))
    }

    fn category2(value: i64) -> StackValue {
        StackValue::Category2(Slot::long_high(value), Slot::long_low(value))
    }

    fn category2_double(value: f64) -> StackValue {
        StackValue::Category2(Slot::double_high(value), Slot::double_low(value))
    }

    fn stack(values_bottom_to_top: &[StackValue]) -> OperandStack {
        let mut stack = OperandStack::new(32);
        for value in values_bottom_to_top {
            stack.push_value(*value).unwrap();
        }
        stack
    }

    fn assert_stack(stack: &OperandStack, expected_bottom_to_top: &[StackValue]) {
        let actual_top_to_bottom = stack.top_values(expected_bottom_to_top.len()).unwrap();
        let expected_top_to_bottom: Vec<_> = expected_bottom_to_top.iter().rev().copied().collect();
        assert_eq!(actual_top_to_bottom, expected_top_to_bottom);
    }

    fn assert_invalid(result: ExecResult<()>, opcode: u8) {
        assert!(matches!(
            result,
            Err(ExecError::InvalidStackOperation { opcode: actual }) if actual == opcode
        ));
    }

    #[test]
    fn pop_forms() {
        let [a, b, wide] = [category1(1), category1(2), category2(3)];

        let mut values = stack(&[a, b]);
        pop_impl(&mut values).unwrap();
        assert_stack(&values, &[a]);

        let mut invalid_values = stack(&[a, wide]);
        assert_invalid(pop_impl(&mut invalid_values), 0x57);
        assert_stack(&invalid_values, &[a, wide]);
    }

    #[test]
    fn pop2_forms() {
        let [a, b, c, wide] = [category1(1), category1(2), category1(3), category2(4)];

        let mut category1_values = stack(&[a, b, c]);
        pop2_impl(&mut category1_values).unwrap();
        assert_stack(&category1_values, &[a]);

        let mut category2_value = stack(&[a, wide]);
        pop2_impl(&mut category2_value).unwrap();
        assert_stack(&category2_value, &[a]);

        let mut invalid_values = stack(&[wide, a]);
        assert_invalid(pop2_impl(&mut invalid_values), 0x58);
        assert_stack(&invalid_values, &[wide, a]);
    }

    #[test]
    fn dup_form() {
        let [a, wide] = [category1(1), category2(2)];

        let mut values = stack(&[a]);
        dup_impl(&mut values).unwrap();
        assert_stack(&values, &[a, a]);

        let mut invalid_values = stack(&[wide]);
        assert_invalid(dup_impl(&mut invalid_values), 0x59);
        assert_stack(&invalid_values, &[wide]);
    }

    #[test]
    fn dup_x1_form() {
        let [a, b, wide] = [category1(1), category1(2), category2(3)];

        let mut values = stack(&[b, a]);
        dup_x1_impl(&mut values).unwrap();
        assert_stack(&values, &[a, b, a]);

        let mut invalid_values = stack(&[wide, a]);
        assert_invalid(dup_x1_impl(&mut invalid_values), 0x5a);
        assert_stack(&invalid_values, &[wide, a]);
    }

    #[test]
    fn dup_x2_forms() {
        let [a, b, c, wide] = [category1(1), category1(2), category1(3), category2(4)];

        let mut three_category1 = stack(&[c, b, a]);
        dup_x2_impl(&mut three_category1).unwrap();
        assert_stack(&three_category1, &[a, c, b, a]);

        let mut over_category2 = stack(&[wide, a]);
        dup_x2_impl(&mut over_category2).unwrap();
        assert_stack(&over_category2, &[a, wide, a]);

        let mut invalid_values = stack(&[a, wide]);
        assert_invalid(dup_x2_impl(&mut invalid_values), 0x5b);
        assert_stack(&invalid_values, &[a, wide]);
    }

    #[test]
    fn dup2_forms() {
        let [a, b, wide] = [category1(1), category1(2), category2(3)];

        let mut two_category1 = stack(&[b, a]);
        dup2_impl(&mut two_category1).unwrap();
        assert_stack(&two_category1, &[b, a, b, a]);

        let mut category2_value = stack(&[wide]);
        dup2_impl(&mut category2_value).unwrap();
        assert_stack(&category2_value, &[wide, wide]);
    }

    #[test]
    fn dup2_x1_forms() {
        let [a, b, c, wide] = [category1(1), category1(2), category1(3), category2(4)];

        let mut three_category1 = stack(&[c, b, a]);
        dup2_x1_impl(&mut three_category1).unwrap();
        assert_stack(&three_category1, &[b, a, c, b, a]);

        let mut category2_over_category1 = stack(&[b, wide]);
        dup2_x1_impl(&mut category2_over_category1).unwrap();
        assert_stack(&category2_over_category1, &[wide, b, wide]);

        let mut invalid_values = stack(&[wide, a]);
        assert_invalid(dup2_x1_impl(&mut invalid_values), 0x5d);
        assert_stack(&invalid_values, &[wide, a]);
    }

    #[test]
    fn dup2_x2_forms() {
        let [a, b, c, d, wide1, wide2] = [
            category1(1),
            category1(2),
            category1(3),
            category1(4),
            category2(5),
            category2(6),
        ];

        let mut four_category1 = stack(&[d, c, b, a]);
        dup2_x2_impl(&mut four_category1).unwrap();
        assert_stack(&four_category1, &[b, a, d, c, b, a]);

        let mut category2_over_two_category1 = stack(&[c, b, wide1]);
        dup2_x2_impl(&mut category2_over_two_category1).unwrap();
        assert_stack(&category2_over_two_category1, &[wide1, c, b, wide1]);

        let mut two_category1_over_category2 = stack(&[wide1, b, a]);
        dup2_x2_impl(&mut two_category1_over_category2).unwrap();
        assert_stack(&two_category1_over_category2, &[b, a, wide1, b, a]);

        let mut two_category2 = stack(&[wide2, wide1]);
        dup2_x2_impl(&mut two_category2).unwrap();
        assert_stack(&two_category2, &[wide1, wide2, wide1]);
    }

    #[test]
    fn swap_form() {
        let [a, b, wide] = [category1(1), category1(2), category2(3)];

        let mut values = stack(&[b, a]);
        swap_impl(&mut values).unwrap();
        assert_stack(&values, &[a, b]);

        let mut invalid_values = stack(&[wide, a]);
        assert_invalid(swap_impl(&mut invalid_values), 0x5f);
        assert_stack(&invalid_values, &[wide, a]);
    }

    #[test]
    fn failed_operation_does_not_mutate_stack() {
        let [a, b] = [category1(1), category1(2)];
        let mut values = OperandStack::new(2);
        values.push_value(a).unwrap();
        values.push_value(b).unwrap();

        assert!(matches!(
            dup_impl(&mut values),
            Err(ExecError::OperandStackOverflow)
        ));
        assert_stack(&values, &[a, b]);
    }

    #[test]
    fn malformed_category2_value_is_rejected() {
        let mut values = OperandStack::new(2);
        let malformed = StackValue::Category2(Slot::int(1), Slot::int(2));
        assert!(matches!(
            values.push_value(malformed),
            Err(ExecError::InvalidOperandStackShape)
        ));
        assert!(matches!(
            values.top_values(1),
            Err(ExecError::OperandStackUnderflow)
        ));
    }

    #[test]
    fn double_category2_bits_are_preserved() {
        let value = category2_double(f64::from_bits(0x7ff8_0000_0000_1234));
        let mut values = stack(&[value]);
        dup2_impl(&mut values).unwrap();
        assert_stack(&values, &[value, value]);
    }
}
