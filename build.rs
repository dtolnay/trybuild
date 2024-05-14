use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src/tests");

    println!("cargo:rustc-check-cfg=cfg(trybuild_no_target)");
    println!("cargo:rustc-check-cfg=cfg(host_os, values(\"windows\"))");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let target = env::var("TARGET").ok();
    let path = Path::new(&out_dir).join("target");
    let value = match target {
        Some(target) => format!(r#"Some("{}")"#, target.escape_debug()),
        None => "None".to_owned(),
    };
    fs::write(path, value)?;

    let host = env::var_os("HOST").unwrap();
    if let Some("windows") = host.to_str().unwrap().split('-').nth(2) {
        println!("cargo:rustc-cfg=host_os=\"windows\"");
    }

    Ok(())
}
