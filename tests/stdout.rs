#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/stdout/println.rs");
}
