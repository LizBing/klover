// Minimal Java 8 class files for switch shapes that javac does not reliably emit.
// Every method is static (I)I. The negative tests deliberately omit a valid int
// operand; the current VM has no verifier and must report execution errors.
fn u2(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend(value.to_be_bytes());
}
fn u4(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_be_bytes());
}
fn utf8(bytes: &mut Vec<u8>, value: &str) {
    bytes.push(1);
    u2(bytes, value.len() as u16);
    bytes.extend(value.as_bytes());
}
fn int(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_be_bytes());
}

fn switch_end(prefix_len: usize, table: bool, count: usize) -> usize {
    ((prefix_len + 4) & !3) + if table { 12 + count * 4 } else { 8 + count * 8 }
}

fn switch(prefix: &[u8], table: bool, keys: &[i32], targets: &[usize], default: usize) -> Vec<u8> {
    assert_eq!(keys.len(), targets.len());
    let bci = prefix.len() as i32;
    let mut code = prefix.to_vec();
    code.push(if table { 0xaa } else { 0xab });
    while code.len() % 4 != 0 {
        code.push(0);
    }
    int(&mut code, default as i32 - bci);
    if table {
        int(&mut code, keys[0]);
        int(&mut code, *keys.last().unwrap());
        for target in targets {
            int(&mut code, *target as i32 - bci);
        }
    } else {
        int(&mut code, keys.len() as i32);
        for (key, target) in keys.iter().zip(targets) {
            int(&mut code, *key);
            int(&mut code, *target as i32 - bci);
        }
    }
    code
}

pub fn build() -> Vec<u8> {
    let mut methods = Vec::<(String, Vec<u8>)>::new();
    for (kind, table) in [("table", true), ("lookup", false)] {
        // iload_0; goto switch; iconst_1; ireturn; switch ...
        // Cases can jump backward directly to iconst_1, not just through goto.
        let prefix = [0x1a, 0xa7, 0x00, 0x05, 0x04, 0xac];
        let end = switch_end(prefix.len(), table, 3);
        let keys = if table { [-1, 0, 1] } else { [-100, 7, 1000] };
        let mut code = switch(&prefix, table, &keys, &[4, end, 4], end + 2);
        code.extend([0x05, 0xac, 0x06, 0xac]); // return 2; return 3
        methods.push((format!("{kind}BackEdge"), code));

        for (suffix, key) in [("Min", i32::MIN), ("Max", i32::MAX)] {
            let end = switch_end(1, table, 1);
            let mut code = switch(&[0x1a], table, &[key], &[end], end + 2);
            code.extend([0x04, 0xac, 0x05, 0xac]);
            methods.push((format!("{kind}Single{suffix}"), code));
        }
        // Keep 41 below the key; correct switch consumption leaves it for iadd.
        let prefix = [0x10, 41, 0x1a];
        let end = switch_end(prefix.len(), table, 1);
        let mut code = switch(&prefix, table, &[0], &[end], end);
        code.extend([0x04, 0x60, 0xac]); // 41 + 1 -> 42
        methods.push((format!("{kind}StackPrefix"), code));

        for (suffix, prefix) in [("MissingKey", &[][..]), ("FloatKey", &[0x0b][..])] {
            let end = switch_end(prefix.len(), table, 1);
            let mut code = switch(prefix, table, &[0], &[end], end);
            code.extend([0x04, 0xac]);
            methods.push((format!("{kind}{suffix}"), code));
        }
    }
    // Also cover a zero-pair lookupswitch independent of javac optimizations.
    let end = switch_end(1, false, 0);
    let mut code = switch(&[0x1a], false, &[], &[], end);
    code.extend([0x05, 0xac]);
    methods.push(("lookupEmpty".into(), code));

    let mut bytes = Vec::new();
    u4(&mut bytes, 0xcafebabe);
    u2(&mut bytes, 0);
    u2(&mut bytes, 52);
    u2(&mut bytes, 7 + methods.len() as u16);
    utf8(&mut bytes, "SwitchBytecodeOps"); // #1
    bytes.push(7);
    u2(&mut bytes, 1); // #2 Class
    utf8(&mut bytes, "java/lang/Object"); // #3
    bytes.push(7);
    u2(&mut bytes, 3); // #4 superclass
    utf8(&mut bytes, "Code"); // #5
    utf8(&mut bytes, "(I)I"); // #6
    for (name, _) in &methods {
        utf8(&mut bytes, name);
    }
    u2(&mut bytes, 0x0021); // public, super
    u2(&mut bytes, 2);
    u2(&mut bytes, 4);
    u2(&mut bytes, 0); // interfaces
    u2(&mut bytes, 0); // fields
    u2(&mut bytes, methods.len() as u16);
    for (index, (_, code)) in methods.iter().enumerate() {
        u2(&mut bytes, 0x0009); // public static
        u2(&mut bytes, 7 + index as u16);
        u2(&mut bytes, 6); // descriptor
        u2(&mut bytes, 1); // attributes
        u2(&mut bytes, 5); // Code
        u4(&mut bytes, 12 + code.len() as u32);
        u2(&mut bytes, 3); // max_stack
        u2(&mut bytes, 1); // max_locals
        u4(&mut bytes, code.len() as u32);
        bytes.extend(code);
        u2(&mut bytes, 0); // exception table
        u2(&mut bytes, 0); // Code attributes
    }
    u2(&mut bytes, 0); // class attributes
    bytes
}
