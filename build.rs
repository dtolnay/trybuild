use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let mut target = env::var("TARGET").ok();

    // When --target flag is passed, cargo does not pass RUSTFLAGS to rustc when
    // building proc-macro and build script even if the host and target triples
    // are the same. Therefore, if we always pass --target to cargo, tools such
    // as coverage that require RUSTFLAGS do not work for tests run by trybuild.
    //
    // To avoid that problem, do not pass --target to cargo if we know that it
    // has not been passed.
    //
    // Cargo does not have a way to tell the build script whether --target has
    // been passed or not, so we use the following heuristic:
    //
    // - The host and target triples are the same.
    // - And RUSTFLAGS is available when *building* the build script.
    //
    // Note that the second is when building, not when running. This is due to:
    //
    // - After rust-lang/cargo#9601, cargo does not pass RUSTFLAGS to the build
    //   script when running.
    // - CARGO_ENCODED_RUSTFLAGS, which was introduced in rust-lang/cargo#9601,
    //   cannot be used for this purpose because it contains the value of
    //   RUSTFLAGS even if --target is passed and the host and target triples
    //   are the same.
    if target == env::var("HOST").ok() && option_env!("RUSTFLAGS").is_some() {
        target = None;
    }

    let path = Path::new(&out_dir).join("target.rs");
    let value = match target {
        Some(target) => format!("Some({:?})", target),
        None => "None".to_owned(),
    };
    let content = format!("const TARGET: Option<&'static str> = {};", value);
    fs::write(path, content)
}
