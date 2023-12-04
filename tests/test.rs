#[test]
fn test_api() {
    let t = trybuild2::TestCases::new();
    t.pass("tests/ui/run-pass-0.rs");
    t.pass("tests/ui/print-stdout.rs");
    t.pass("tests/ui/run-pass-1.rs");
    t.pass("tests/ui/print-stderr.rs");
    t.pass("tests/ui/run-pass-2.rs");
    t.pass("tests/ui/print-both.rs");
    t.pass("tests/ui/run-pass-4.rs");
    t.compile_fail("tests/ui/run-pass-3.rs");
    t.pass("tests/ui/run-pass-5.rs");
    t.pass("tests/ui/compile-fail-0.rs");
    t.pass("tests/ui/run-pass-6.rs");
    t.pass("tests/ui/run-pass-7.rs");
    t.pass("tests/ui/run-pass-8.rs");
    t.compile_fail("tests/ui/compile-fail-1.rs");
    t.pass("tests/ui/run-fail.rs");
    t.pass("tests/ui/run-pass-9.rs");
    t.compile_fail("tests/ui/compile-fail-2.rs");
    t.compile_fail("tests/ui/compile-fail-3.rs");
    t.pass_inline("inline_pass_main", "fn main() {}");
    // Should be a mismatch.
    t.pass_inline_check_sub("inline_check_sub", "fn main() { let x = 12; }", "wuwu");
    // Should be ok.
    t.pass_inline_check_sub(
        "inline_check_sub2",
        "fn main() { let x = 12; }",
        "warning: unused variable: `x`",
    );
    t.compile_fail_inline(
        "inline_compile_fail",
        "fn main() { compile_error!(\"ERROR\"); }",
        "tests/ui/inline_compile_fail.stderr",
    );
    t.compile_fail_check_sub("tests/ui/compile-fail-0.rs", "compile_error!");
    // This one should be a mismatch!
    t.compile_fail_check_sub("tests/ui/compile-fail-0.rs", "I'm not here!");
    t.compile_fail_inline_check_sub(
        "inline_compile_fail_sub",
        "fn main() { compile_error!(\"tadam\"); }",
        "compile_error!(\"tadam",
    );
}

#[test]
fn with_suffix() {
    let t = trybuild2::TestCases::with_suffix("suffixed");
    t.compile_fail("tests/ui/compile-fail-1.rs");
}
