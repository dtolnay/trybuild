const IGNORED_LINTS: &[&str] = &["dead_code"];

pub(crate) fn toml() -> toml::Value {
    let mut rustflags = vec!["--cfg", "trybuild", "--verbose"];

    for &lint in IGNORED_LINTS {
        rustflags.push("-A");
        rustflags.push(lint);
    }

    if let Ok(flags) = std::env::var("RUSTFLAGS") {
        // TODO: could parse this properly and allowlist or blocklist certain
        // flags. This is good enough to at least support cargo-llvm-cov.
        if flags.contains("-C instrument-coverage") {
            rustflags.extend(["-C", "instrument-coverage"]);
        }
    }

    toml::Value::try_from(rustflags).unwrap()
}
