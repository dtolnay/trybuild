use derive_mac::Print;

#[derive(Print)]
pub struct Field {
    name: &'static str,
    bitmask: u16,
}

fn main() {}