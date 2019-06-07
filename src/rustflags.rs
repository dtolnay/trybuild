use std::env;
use std::process::Command;

const RUSTFLAGS: &str = "RUSTFLAGS";
const IGNORED_LINTS: &[&str] = &["dead_code"];

pub fn make_vec() -> Vec<String> {
    let mut rustflags = Vec::new();

    for &lint in IGNORED_LINTS {
        rustflags.push("-A".to_owned());
        rustflags.push(lint.to_owned());
    }

    rustflags
}

pub fn set_env(cmd: &mut Command) {
    let mut rustflags = match env::var_os(RUSTFLAGS) {
        Some(rustflags) => rustflags,
        None => return,
    };

    for flag in make_vec() {
        rustflags.push(" ");
        rustflags.push(flag);
    }

    cmd.env(RUSTFLAGS, rustflags);
}
