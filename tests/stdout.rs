#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/stdout/print-pass.rs");
    t.compile_fail("tests/stdout/print-fail.rs");
    t.compile_fail("tests/stdout/run-fail.rs");
}
