use std::env;

const IGNORED_LINTS: &[&str] = &["dead_code"];

pub(crate) fn toml(extra_rustflags: &[&'static str]) -> toml::Value {
    let mut rustflags = vec!["--cfg", "trybuild", "--verbose"];

    for &lint in IGNORED_LINTS {
        rustflags.push("-A");
        rustflags.push(lint);
    }

    if let Some(flags) = env::var_os("RUSTFLAGS") {
        // TODO: could parse this properly and allowlist or blocklist certain
        // flags. This is good enough to at least support cargo-llvm-cov.
        if flags.to_string_lossy().contains("-C instrument-coverage") {
            rustflags.extend(["-C", "instrument-coverage"]);
        }
    }

    rustflags.extend(extra_rustflags);

    toml::Value::try_from(rustflags).unwrap()
}
