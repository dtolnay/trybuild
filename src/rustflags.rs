const IGNORED_LINTS: &[&str] = &["dead_code"];

pub(crate) fn toml() -> toml::Value {
    let mut rustflags = vec!["--cfg", "trybuild", "--verbose"];

    for &lint in IGNORED_LINTS {
        rustflags.push("-A");
        rustflags.push(lint);
    }

    toml::Value::try_from(rustflags).unwrap()
}
