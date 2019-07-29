use derive_mac::Print;

#[allow(dead_code)]
#[derive(Print)]
pub struct Field {
    name: &'static str,
    bitmask: u16,
}

fn main() {
    assert!(false)
}
