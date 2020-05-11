#[test]
#[cfg(target_os = "linux")]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/compile-pass-stdout.rs");
}
