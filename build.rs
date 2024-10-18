fn main() {
    // Warning: build.rs is not published to crates.io.

    println!("cargo:rerun-if-changed=src/tests");
    println!("cargo:rustc-check-cfg=cfg(trybuild_no_target)");
}
