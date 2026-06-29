use rust::oops::symbol_table::SymbolTable;

#[test]
fn test_intern_same_string_returns_same_pointer() {
    let a = SymbolTable::intern("Hello".to_string());
    let b = SymbolTable::intern("Hello".to_string());

    // Same content → same pointer (interning)
    assert!(a.equals(&b));
}

#[test]
fn test_intern_different_strings_return_different_pointers() {
    let a = SymbolTable::intern("foo".to_string());
    let b = SymbolTable::intern("bar".to_string());

    // Different content → different pointers
    assert!(!a.equals(&b));
}

#[test]
fn test_intern_same_content_different_cases() {
    let a = SymbolTable::intern("Abc".to_string());
    let b = SymbolTable::intern("abc".to_string());

    // Case-sensitive: different strings
    assert!(!a.equals(&b));
    assert_eq!(a.utf8(), "Abc");
    assert_eq!(b.utf8(), "abc");
}

#[test]
fn test_utf8_returns_correct_string() {
    let h = SymbolTable::intern("HelloWorld".to_string());

    assert_eq!(h.utf8(), "HelloWorld");
}

#[test]
fn test_empty_string() {
    let a = SymbolTable::intern("".to_string());
    let b = SymbolTable::intern("".to_string());

    assert!(a.equals(&b));
    assert_eq!(a.utf8(), "");
}

#[test]
fn test_clone_shares_pointer() {
    let a = SymbolTable::intern("shared".to_string());
    let b = a.clone();

    // Clone should share the same pointer
    assert!(a.equals(&b));
    assert_eq!(a.utf8(), b.utf8());
}

#[test]
fn test_many_distinct_strings() {
    // Intern a bunch of distinct strings; they should all be retrievable
    let handles: Vec<_> = (0..100)
        .map(|i| SymbolTable::intern(format!("string_{}", i)))
        .collect();

    for (i, h) in handles.iter().enumerate() {
        assert_eq!(h.utf8(), &format!("string_{}", i));
    }

    // Re-intern should hit existing entries
    for (i, h) in handles.iter().enumerate() {
        let h2 = SymbolTable::intern(format!("string_{}", i));
        assert!(h.equals(&h2));
    }
}

#[test]
fn test_unicode_string() {
    let a = SymbolTable::intern("你好世界".to_string());
    let b = SymbolTable::intern("你好世界".to_string());

    assert!(a.equals(&b));
    assert_eq!(a.utf8(), "你好世界");
}

#[test]
fn test_long_string() {
    let long = "a".repeat(10000);
    let a = SymbolTable::intern(long.clone());
    let b = SymbolTable::intern(long);

    assert!(a.equals(&b));
    assert_eq!(a.utf8().len(), 10000);
}

#[test]
fn test_drop_and_reintern() {
    // Intern, then drop all handles, then re-intern.
    // The old node may or may not be recycled yet, so we just verify
    // the content is still correct.
    let s = "drop_and_reintern_test";
    {
        let _a = SymbolTable::intern(s.to_string());
    } // dropped

    let b = SymbolTable::intern(s.to_string());
    assert_eq!(b.utf8(), s);
}

#[test]
fn test_multiple_clones_then_drop() {
    let a = SymbolTable::intern("multi_clone".to_string());
    let b = a.clone();
    let c = a.clone();
    let d = b.clone();

    assert!(a.equals(&c));
    assert!(b.equals(&d));
    assert_eq!(c.utf8(), "multi_clone");

    // All handles dropped; ref_cnt should go back to 0 without panicking
    drop(a);
    drop(b);
    drop(c);
    drop(d);
}

#[test]
fn test_intern_keeps_original_string() {
    // The symbol table should store the first interned variant
    let mut s = String::from("original");
    let h = SymbolTable::intern(s.clone());

    // Mutate the local string; symbol should keep original content
    s.push_str("_modified");
    assert_eq!(h.utf8(), "original");

    // Re-interning the modified string creates a new entry
    let h2 = SymbolTable::intern(s);
    assert!(!h.equals(&h2));
    assert_eq!(h2.utf8(), "original_modified");
}
