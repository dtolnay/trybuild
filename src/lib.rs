//! #### &emsp;A compiler diagnostics testing library in just 3 functions.
//!
//! Trybuild is a test harness for invoking rustc on a set of test cases and
//! asserting that any resulting error messages are the ones intended.
//!
//! Such tests are commonly useful for testing error reporting involving
//! procedural macros. We would write test cases triggering either errors
//! detected by the macro or errors detected by the Rust compiler in the
//! resulting expanded code, and compare against the expected errors to ensure
//! that they remain user-friendly.
//!
//! This style of testing is sometimes called *ui tests* because they test
//! aspects of the user's interaction with a library outside of what would be
//! covered by ordinary API tests.
//!
//! Nothing here is specific to macros; trybuild would work equally well for
//! testing misuse of non-macro APIs.
//!
//! <br>
//!
//! # Compile-fail tests
//!
//! A minimal trybuild setup looks like this:
//!
//! ```
//! #[test]
//! fn ui() {
//!     let t = trybuild::TestCases::new();
//!     t.compile_fail("tests/ui/*.rs");
//! }
//! ```
//!
//! The test can be run with `cargo test`. It will individually compile each of
//! the source files matching the glob pattern, expect them to fail to compile,
//! and assert that the compiler's error message matches an adjacently named
//! _*.stderr_ file containing the expected output (same file name as the test
//! except with a different extension). If it matches, the test case is
//! considered to succeed.
//!
//! Dependencies listed under `[dev-dependencies]` in the project's Cargo.toml
//! are accessible from within the test cases.
//!
//! <p align="center">
//! <img src="compile-fail.png" width="700">
//! </p>
//!
//! Failing tests display the expected vs actual compiler output inline.
//!
//! <p align="center">
//! <img src="mismatch.png" width="700">
//! </p>
//!
//! A compile_fail test that fails to fail to compile is also a failure.
//!
//! <p align="center">
//! <img src="should-fail.png" width="700">
//! </p>
//!
//! <br>
//!
//! # Pass tests
//!
//! The same test harness is able to run tests that are expected to pass, too.
//! Ordinarily you would just have Cargo run such tests directly, but being able
//! to combine modes like this could be useful for workshops in which
//! participants work through test cases enabling one at a time. Trybuild was
//! originally developed for my [procedural macros workshop at Rust
//! Latam][workshop].
//!
//! [workshop]: https://github.com/dtolnay/proc-macro-workshop
//!
//! ```
//! #[test]
//! fn ui() {
//!     let t = trybuild::TestCases::new();
//!     t.pass("tests/01-parse-header.rs");
//!     t.pass("tests/02-parse-body.rs");
//!     t.compile_fail("tests/03-expand-four-errors.rs");
//!     t.pass("tests/04-paste-ident.rs");
//!     t.pass("tests/05-repeat-section.rs");
//!     //t.pass("tests/06-make-work-in-function.rs");
//!     //t.pass("tests/07-init-array.rs");
//!     //t.compile_fail("tests/08-ident-span.rs");
//! }
//! ```
//!
//! Pass tests are considered to succeed if they compile successfully and have a
//! `main` function that does not panic when the compiled binary is executed.
//!
//! <p align="center">
//! <img src="workshop.png" width="700">
//! </p>
//!
//! <br>
//!
//! # Details
//!
//! That's the entire API.
//!
//! <br>
//!
//! # Workflow
//!
//! There are two ways to update the _*.stderr_ files as you iterate on your
//! test cases or your library; handwriting them is not recommended.
//!
//! First, if a test case is being run as compile_fail but a corresponding
//! _*.stderr_ file does not exist, the test runner will save the actual
//! compiler output with the right filename into a directory called *wip* within
//! the directory containing Cargo.toml. So you can update these files by
//! deleting them, running `cargo test`, and moving all the files from *wip*
//! into your testcase directory.
//!
//! <p align="center">
//! <img src="wip.png" width="700">
//! </p>
//!
//! Alternatively, run `cargo test` with the environment variable
//! `TRYBUILD=overwrite` to skip the *wip* directory and write all compiler
//! output directly in place.

#[macro_use]
mod term;

#[macro_use]
mod path;

mod banner;
mod cargo;
mod dependencies;
mod env;
mod error;
mod manifest;
mod message;
mod normalize;
mod run;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::thread;

pub struct TestCases {
    runner: RefCell<Runner>,
}

struct Runner {
    tests: Vec<Test>,
}

#[derive(Clone)]
struct Test {
    path: PathBuf,
    expected: Expected,
}

#[derive(Copy, Clone)]
enum Expected {
    Pass,
    CompileFail,
}

impl TestCases {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TestCases {
            runner: RefCell::new(Runner { tests: Vec::new() }),
        }
    }

    pub fn pass<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().tests.push(Test {
            path: path.as_ref().to_owned(),
            expected: Expected::Pass,
        });
    }

    pub fn compile_fail<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().tests.push(Test {
            path: path.as_ref().to_owned(),
            expected: Expected::CompileFail,
        });
    }
}

#[doc(hidden)]
impl Drop for TestCases {
    fn drop(&mut self) {
        if !thread::panicking() {
            self.runner.borrow_mut().run();
        }
    }
}
