Trybuild
========

[![Build Status](https://api.travis-ci.com/dtolnay/trybuild.svg?branch=master)](https://travis-ci.com/dtolnay/trybuild)
[![Latest Version](https://img.shields.io/crates/v/trybuild.svg)](https://crates.io/crates/trybuild)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/trybuild)

Trybuild is a test harness for invoking rustc on a set of test cases and
asserting that any resulting error messages are the ones intended.

<p align="center">
<a href="#compile-fail-tests">
<img src="https://user-images.githubusercontent.com/1940490/57186574-76469e00-6e96-11e9-8cb5-b63b657170c9.png" width="700">
</a>
</p>

Such tests are commonly useful for testing error reporting involving procedural
macros. We would write test cases triggering either errors detected by the macro
or errors detected by the Rust compiler in the resulting expanded code, and
compare against the expected errors to ensure that they remain user-friendly.

This style of testing is sometimes called *ui tests* because they test aspects
of the user's interaction with a library outside of what would be covered by
ordinary API tests.

Nothing here is specific to macros; trybuild would work equally well for testing
misuse of non-macro APIs.

```toml
[dev-dependencies]
trybuild = "0.0"
```

*Compiler support: requires rustc 1.33+*

<br>

## Compile-fail tests

A minimal trybuild setup looks like this:

```rust
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
```

The test can be run with `cargo test`. It will individually compile each of the
source files matching the glob pattern, expect them to fail to compile, and
assert that the compiler's error message matches an adjacently named _*.stderr_
file containing the expected output (same file name as the test except with a
different extension). If it matches, the test case is considered to succeed.

Dependencies listed under `[dev-dependencies]` in the project's Cargo.toml are
accessible from within the test cases.

Failing tests display the expected vs actual compiler output inline.

<p align="center">
<a href="#compile-fail-tests">
<img src="https://user-images.githubusercontent.com/1940490/57186575-79418e80-6e96-11e9-9478-c9b3dc10327f.png" width="700">
</a>
</p>

A compile\_fail test that fails to fail to compile is also a failure.

<p align="center">
<a href="#compile-fail-tests">
<img src="https://user-images.githubusercontent.com/1940490/57186576-7b0b5200-6e96-11e9-8bfd-2de705125108.png" width="700">
</a>
</p>

<br>

## Pass tests

The same test harness is able to run tests that are expected to pass, too.
Ordinarily you would just have Cargo run such tests directly, but being able to
combine modes like this could be useful for workshops in which participants work
through test cases enabling one at a time. Trybuild was originally developed for
my [procedural macros workshop at Rust Latam][workshop].

[workshop]: https://github.com/dtolnay/proc-macro-workshop

```rust
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-header.rs");
    t.pass("tests/02-parse-body.rs");
    t.compile_fail("tests/03-expand-four-errors.rs");
    t.pass("tests/04-paste-ident.rs");
    t.pass("tests/05-repeat-section.rs");
    //t.pass("tests/06-make-work-in-function.rs");
    //t.pass("tests/07-init-array.rs");
    //t.compile_fail("tests/08-ident-span.rs");
}
```

Pass tests are considered to succeed if they compile successfully and have a
`main` function that does not panic when the compiled binary is executed.

<p align="center">
<a href="#pass-tests">
<img src="https://user-images.githubusercontent.com/1940490/57186580-7f376f80-6e96-11e9-9cae-8257609269ef.png" width="700">
</a>
</p>

<br>

## Details

That's the entire API.

<br>

## Workflow

There are two ways to update the _*.stderr_ files as you iterate on your test
cases or your library; handwriting them is not recommended.

First, if a test case is being run as compile\_fail but a corresponding
_*.stderr_ file does not exist, the test runner will save the actual compiler
output with the right filename into a directory called *wip* within the
directory containing Cargo.toml. So you can update these files by deleting them,
running `cargo test`, and moving all the files from *wip* into your testcase
directory.

<p align="center">
<a href="#workflow">
<img src="https://user-images.githubusercontent.com/1940490/57186579-7cd51580-6e96-11e9-9f19-54dcecc9fbba.png" width="700">
</a>
</p>

Alternatively, run `cargo test` with the environment variable
`TRYBUILD=overwrite` to skip the *wip* directory and write all compiler output
directly in place.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
