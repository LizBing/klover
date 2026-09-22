use super::support::{fixture_root, invoke, run};
use klover::{
    engines::{exec_dispatcher::MethodReturn, exec_error::ExecErrorKind},
    oops::jvalue::JValue::*,
};

fn check_switch(method: &str, key: i32, expected: i32) {
    let desc = if method.ends_with("Indexed") {
        "(IIIII)I"
    } else {
        "(I)I"
    };
    let args = if method.ends_with("Indexed") {
        vec![Int(0), Int(0), Int(0), Int(0), Int(key)]
    } else {
        vec![Int(key)]
    };
    for fuel in [1, 2, 3, 10000] {
        let actual = run("SwitchOps", method, desc, &args, fuel);
        assert!(
            matches!(actual, MethodReturn::Value(Int(value)) if value == expected),
            "SwitchOps.{method}: key={key}, fuel={fuel}, expected={expected}"
        );
    }
}

#[test]
fn switch_fixtures_cover_both_opcodes_and_all_four_alignments() {
    let root = fixture_root();
    let bytes = std::fs::read(std::path::Path::new(&root).join("SwitchOps.class")).unwrap();
    let mut options = cafebabe::ParseOptions::default();
    options.parse_bytecode(true);
    let class = cafebabe::parse_class_with_options(&bytes, &options).unwrap();
    for prefix in ["dense", "sparse"] {
        let mut alignments = [false; 4];
        for suffix in ["", "Indexed", "AddOne", "AddSix"] {
            let name = format!("{prefix}{suffix}");
            let method = class
                .methods
                .iter()
                .find(|method| method.name.as_ref() == name)
                .unwrap();
            let code = method
                .attributes
                .iter()
                .find_map(|attr| match &attr.data {
                    cafebabe::attributes::AttributeData::Code(code) => Some(code),
                    _ => None,
                })
                .unwrap();
            let (bci, opcode) = code
                .bytecode
                .as_ref()
                .unwrap()
                .opcodes
                .iter()
                .find(|(_, opcode)| {
                    matches!(
                        opcode,
                        cafebabe::bytecode::Opcode::Tableswitch(_)
                            | cafebabe::bytecode::Opcode::Lookupswitch(_)
                    )
                })
                .unwrap();
            assert!(
                matches!(
                    (prefix, opcode),
                    ("dense", cafebabe::bytecode::Opcode::Tableswitch(_))
                        | ("sparse", cafebabe::bytecode::Opcode::Lookupswitch(_))
                ),
                "{name}"
            );
            alignments[bci % 4] = true;
        }
        assert_eq!(alignments, [true; 4], "{prefix} switch padding coverage");
    }
}

#[test]
fn tableswitch_all_cases_defaults_and_padding() {
    for (method, offset) in [
        ("dense", 0),
        ("denseIndexed", 0),
        ("denseAddOne", 1),
        ("denseAddSix", 6),
    ] {
        for (key, expected) in [(-2, 99), (-1, 11), (0, 22), (1, 33), (2, 44), (3, 99)] {
            check_switch(method, key - offset, expected);
        }
    }
    check_switch("dense", i32::MIN, 99);
    check_switch("dense", i32::MAX, 99);
}

#[test]
fn tableswitch_holes_fallthrough_and_shared_targets() {
    for (key, expected) in [
        (-3, 99),
        (-2, 10),
        (-1, 20),
        (0, 99),
        (1, 30),
        (2, 40),
        (3, 99),
    ] {
        check_switch("denseHole", key, expected);
    }
    for (key, expected) in [(-1, 99), (0, 7), (1, 6), (2, 4), (3, 4), (4, 99)] {
        check_switch("denseFallThrough", key, expected);
    }
}

#[test]
fn tableswitch_ranges_at_int_limits() {
    for (key, expected) in [
        (i32::MIN, 1),
        (i32::MIN + 1, 2),
        (i32::MIN + 2, 3),
        (i32::MIN + 3, 99),
        (i32::MAX, 99),
    ] {
        check_switch("denseMin", key, expected);
    }
    for (key, expected) in [
        (i32::MAX - 3, 99),
        (i32::MAX - 2, 1),
        (i32::MAX - 1, 2),
        (i32::MAX, 3),
        (i32::MIN, 99),
    ] {
        check_switch("denseMax", key, expected);
    }
}

#[test]
fn lookupswitch_first_middle_and_last_case_at_all_alignments() {
    for (method, offset) in [
        ("sparse", 0),
        ("sparseIndexed", 0),
        ("sparseAddOne", 1),
        ("sparseAddSix", 6),
    ] {
        for (key, expected) in [(-100, 11), (7, 22), (1000, 33)] {
            check_switch(method, key - offset, expected);
        }
    }
}

#[test]
fn lookupswitch_misses_and_empty_table_use_default() {
    for key in [i32::MIN, -101, -99, 0, 6, 8, 999, 1001, i32::MAX] {
        check_switch("sparse", key, 99);
        check_switch("defaultOnly", key, 99);
    }
}

#[test]
fn lookupswitch_handles_signed_extreme_keys() {
    for (key, expected) in [
        (i32::MIN, 1),
        (i32::MIN + 1, 99),
        (-1, 99),
        (0, 2),
        (1, 99),
        (i32::MAX - 1, 99),
        (i32::MAX, 3),
    ] {
        check_switch("sparseExtremes", key, expected);
    }
}

#[test]
fn lookupswitch_fallthrough_and_shared_targets() {
    for (key, expected) in [(-100, 7), (7, 6), (1000, 4), (10000, 4), (8, 99)] {
        check_switch("sparseFallThrough", key, expected);
    }
}

#[test]
fn tableswitch_in_loop_resumes_across_budget_exhaustion() {
    for (n, expected) in [(0, 0), (1, 1), (4, 10), (8, 20), (9, 21)] {
        check_switch("denseLoop", n, expected);
    }
}

#[test]
fn lookupswitch_in_loop_resumes_across_budget_exhaustion() {
    for (n, expected) in [(0, 0), (1, 4), (4, 14), (8, 28), (9, 32)] {
        check_switch("sparseLoop", n, expected);
    }
}

fn check_bytecode(method: &str, key: i32, expected: i32) {
    for fuel in [1, 2, 3, 10000] {
        let actual = run("SwitchBytecodeOps", method, "(I)I", &[Int(key)], fuel);
        assert!(
            matches!(actual, MethodReturn::Value(Int(value)) if value == expected),
            "{method}: key={key}, fuel={fuel}, expected={expected}"
        );
    }
}

#[test]
fn tableswitch_direct_back_edges_and_shared_targets() {
    for (key, expected) in [(-2, 3), (-1, 1), (0, 2), (1, 1), (2, 3)] {
        check_bytecode("tableBackEdge", key, expected);
    }
}

#[test]
fn lookupswitch_direct_back_edges_and_shared_targets() {
    for (key, expected) in [(-101, 3), (-100, 1), (0, 3), (7, 2), (1000, 1), (1001, 3)] {
        check_bytecode("lookupBackEdge", key, expected);
    }
}

#[test]
fn switch_single_entry_tables_at_int_boundaries() {
    for kind in ["table", "lookup"] {
        for (suffix, matched) in [("Min", i32::MIN), ("Max", i32::MAX)] {
            for key in [i32::MIN, -1, 0, 1, i32::MAX] {
                check_bytecode(
                    &format!("{kind}Single{suffix}"),
                    key,
                    if key == matched { 1 } else { 2 },
                );
            }
        }
    }
}

#[test]
fn lookupswitch_zero_pair_bytecode_uses_default() {
    for key in [i32::MIN, -1, 0, 1, i32::MAX] {
        check_bytecode("lookupEmpty", key, 2);
    }
}

#[test]
fn switch_consumes_key_and_preserves_stack_prefix() {
    for method in ["tableStackPrefix", "lookupStackPrefix"] {
        for key in [0, 1, -1] {
            check_bytecode(method, key, 42);
        }
    }
}

#[test]
fn malformed_switch_operand_reports_execution_error() {
    for kind in ["table", "lookup"] {
        for suffix in ["MissingKey", "FloatKey"] {
            for fuel in [1, 10000] {
                let error = match invoke(
                    "SwitchBytecodeOps",
                    &format!("{kind}{suffix}"),
                    "(I)I",
                    &[Int(0)],
                    fuel,
                ) {
                    Err(error) => error,
                    Ok(_) => panic!("malformed switch operand completed normally"),
                };
                assert!(matches!(
                    (suffix, error.kind),
                    ("MissingKey", ExecErrorKind::StackUnderflow)
                        | ("FloatKey", ExecErrorKind::MismatchSlotType)
                ));
            }
        }
    }
}
