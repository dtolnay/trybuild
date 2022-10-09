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
use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::fmt::{self, Debug};
use std::path::Path;
use std::str;

mod run {
    pub struct PathDependency {
        pub name: String,
        pub normalized_path: super::Directory,
    }
}

struct Input<'a>(&'a str);

impl<'a> Debug for Input<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.0, formatter)
    }
}

impl<'a> Arbitrary<'a> for Input<'a> {
    fn arbitrary(_u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        unreachable!()
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> arbitrary::Result<Self> {
        match str::from_utf8(u.take_rest()) {
            Ok(s) => Ok(Input(s)),
            Err(_) => Err(arbitrary::Error::IncorrectFormat),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, Some(500))
    }
}

fuzz_target!(|input: Input| {
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
    let _ = normalize::diagnostics(input.0, context);
});
