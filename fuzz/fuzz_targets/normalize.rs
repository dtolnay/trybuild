#![no_main]

#[path = "../../src/directory.rs"]
#[allow(dead_code)]
mod directory;
#[path = "../../src/normalize.rs"]
#[allow(dead_code)]
mod normalize;

use crate::directory::Directory;
use crate::normalize::Context;
use crate::run::PathDependency;
use libfuzzer_sys::fuzz_target;
use std::path::Path;

mod run {
    pub struct PathDependency {
        pub name: String,
        pub normalized_path: super::Directory,
    }
}

fuzz_target!(|string: &str| {
    if string.len() > 500 {
        return;
    }
    let context = Context {
        krate: "trybuild000",
        input_file: Path::new("tests/ui/error.rs"),
        source_dir: &Directory::new("/git/trybuild/test_suite"),
        workspace: &Directory::new("/git/trybuild"),
        target_dir: &Directory::new("/git/trybuild/target"),
        path_dependencies: &[PathDependency {
            name: String::from("diesel"),
            normalized_path: Directory::new("/home/user/documents/rust/diesel/diesel"),
        }],
    };
    let _ = normalize::diagnostics(string, context);
});
