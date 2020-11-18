use std::fs;
use std::path::Path;

fn main() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/res/content.txt");
    assert_eq!(fs::read_to_string(&file).unwrap(), "hello world");
}
