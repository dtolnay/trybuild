use difference::{Changeset, Difference};
use termcolor::Color::{self, *};

use super::{Expected, Test};
use crate::error::Error;
use crate::normalize;
use crate::term;

use std::cmp::min;
use std::path::Path;
use std::process::Output;

const EXPECTED_COLOR: Color = Blue;
const ACTUAL_COLOR: Color = Red;
const WIP_COLOR: Color = Yellow;
const WARN_COLOR: Color = Yellow;
const ERR_COLOR: Color = Red;
const OK_COLOR: Color = Green;

macro_rules! impl_print {
    ($macro_:ident, $func:ident, $color:expr, $fmt:expr, $($args:expr),*) => {{
        term::$func($color);
        $macro_!($fmt, $($args),*);
        term::reset();
    }};
}

macro_rules! println_color {
    ($color:expr, $fmt:expr, $($args:expr),*) => {{
        impl_print!(println, color, $color, $fmt, $($args),*)
    }};

    ($color:expr, $fmt:expr) => { println_color!($color, $fmt,) };
}

macro_rules! println_bold {
    ($color:expr, $fmt:expr, $($args:expr),*) => {{
        impl_print!(println, bold_color, $color, $fmt, $($args),*)
    }};

    ($color:expr, $fmt:expr) => { println_bold!($color, $fmt,) };
}

macro_rules! print_color {
    ($color:expr, $fmt:expr, $($args:expr),*) => {{
        impl_print!(print, color, $color, $fmt, $($args),*)
    }};

    ($color:expr, $fmt:expr) => { print_color!($color, $fmt,) };
}

macro_rules! print_bold {
    ($color:expr, $fmt:expr, $($args:expr),*) => {{
        impl_print!(print, bold_color, $color, $fmt, $($args),*)
    }};

    ($color:expr, $fmt:expr) => { print_bold!($color, $fmt,) };
}

macro_rules! print_bg_color {
    ($color:expr, $fmt:expr, $($args:expr),*) => {{
        impl_print!(print, bg_color, $color, $fmt, $($args),*)
    }};

    ($color:expr, $fmt:expr) => { print_bg_color!($color, $fmt,) };
}

pub(crate) enum Level {
    Fail,
    Warn,
}

pub(crate) use self::Level::*;

pub(crate) fn prepare_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    print_bold!(ERR_COLOR, "ERROR");
    println!(": {}", err);
    println!();
}

pub(crate) fn test_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    println_bold!(ERR_COLOR, "error");
    println_color!(ERR_COLOR, "{}", err);
    println!();
}

pub(crate) fn no_tests_enabled() {
    println_color!(WARN_COLOR, "There are no tests enabled yet.");
}

pub(crate) fn ok() {
    println_color!(OK_COLOR, "ok");
}

pub(crate) fn begin_test(test: &Test, show_expected: bool) {
    let display_name = if show_expected {
        test.path
            .file_name()
            .unwrap_or_else(|| test.path.as_os_str())
            .to_string_lossy()
    } else {
        test.path.as_os_str().to_string_lossy()
    };

    print!("test ");
    term::bold();
    print!("{}", display_name);
    term::reset();

    if show_expected {
        match test.expected {
            Expected::Pass => print!(" [should pass]"),
            Expected::CompileFail => print!(" [should fail to compile]"),
        }
    }

    print!(" ... ");
}

pub(crate) fn failed_to_build(stderr: &str) {
    println_bold!(ERR_COLOR, "error");
    snippet(ERR_COLOR, stderr);
    println!();
}

pub(crate) fn should_not_have_compiled() {
    println_bold!(ERR_COLOR, "error");
    println_color!(
        ERR_COLOR,
        "Expected test case to fail to compile, but it succeeded."
    );
    println!();
}

pub(crate) fn write_stderr_wip(wip_path: &Path, stderr_path: &Path, stderr: &str) {
    let wip_path = wip_path.to_string_lossy();
    let stderr_path = stderr_path.to_string_lossy();

    println_bold!(WIP_COLOR, "wip");
    println!();
    print_bold!(WIP_COLOR, "NOTE");
    println!(": writing the following output to `{}`.", wip_path);
    println!(
        "Move this file to `{}` to accept it as correct.",
        stderr_path,
    );
    snippet(WIP_COLOR, stderr);
    println!();
}

pub(crate) fn overwrite_stderr(stderr_path: &Path, stderr: &str) {
    let stderr_path = stderr_path.to_string_lossy();

    println_bold!(WIP_COLOR, "wip");
    println!();
    print_bold!(WIP_COLOR, "NOTE");
    println!(": writing the following output to `{}`.", stderr_path);
    snippet(WIP_COLOR, stderr);
    println!();
}

pub(crate) fn mismatch(expected: &str, actual: &str) {
    println_bold!(ERR_COLOR, "mismatch");
    println!();

    if need_diff(expected, actual) {
        fn trim_end_lf(s: &str) -> &str {
            if s.chars().last() == Some('\n') {
                &s[..s.len() - 1]
            } else {
                s
            }
        };
        let expected = trim_end_lf(expected);
        let actual = trim_end_lf(actual);
        let diffs = Changeset::new(expected, actual, " ").diffs;
        print_diff(diffs);
    } else {
        println_bold!(EXPECTED_COLOR, "EXPECTED:");
        snippet(EXPECTED_COLOR, expected);
        println!();
        println_bold!(ACTUAL_COLOR, "ACTUAL OUTPUT:");
        snippet(ACTUAL_COLOR, actual);
        println!();
    }
}

pub(crate) fn output(warnings: &str, output: &Output) {
    let success = output.status.success();
    let stdout = normalize::trim(&output.stdout);
    let stderr = normalize::trim(&output.stderr);
    let has_output = !stdout.is_empty() || !stderr.is_empty();

    if success {
        ok();
        if has_output || !warnings.is_empty() {
            println!();
        }
    } else {
        println_bold!(ERR_COLOR, "error");
        if has_output {
            println_color!(ERR_COLOR, "Test case failed at runtime.");
        } else {
            println_color!(
                ERR_COLOR,
                "Execution of the test case was unsuccessful but there was no output."
            );
        }
        println!();
    }

    self::warnings(warnings);

    let color = if success { WARN_COLOR } else { ERR_COLOR };

    for (name, content) in &[("STDOUT", stdout), ("STDERR", stderr)] {
        if !content.is_empty() {
            println_bold!(color, "{}:", name);
            snippet(color, &normalize::trim(content));
            println!();
        }
    }
}

pub(crate) fn fail_output(level: Level, stdout: &[u8]) {
    let color = match level {
        Fail => ERR_COLOR,
        Warn => WARN_COLOR,
    };

    if !stdout.is_empty() {
        println_bold!(color, "STDOUT:");
        snippet(color, &normalize::trim(stdout));
        println!();
    }
}

pub(crate) fn warnings(warnings: &str) {
    if warnings.is_empty() {
        return;
    }

    println_bold!(WARN_COLOR, "WARNINGS:");
    snippet(WARN_COLOR, warnings);
    println!();
}

fn snippet(color: Color, content: &str) {
    dotted_line(color);

    // Color one line at a time because Travis does not preserve color setting
    // across output lines.
    for line in content.lines() {
        println_color!(color, "{}", line);
    }

    dotted_line(color);
}

fn dotted_line(color: Color) {
    println_color!(color, "{}", "┈".repeat(60));
}

fn need_diff(expected: &str, actual: &str) -> bool {
    let diffs = Changeset::new(expected, actual, "\n").diffs;

    let lines_changed: usize = diffs
        .iter()
        .map(|d| match d {
            Difference::Same(_) => 0,
            Difference::Add(ref added) => added.split('\n').count(),
            Difference::Rem(ref removed) => removed.split('\n').count(),
        })
        .sum();

    let min_lines = min(expected.split('\n').count(), actual.split('\n').count()) as f64;
    let diff_fraction = (lines_changed as f64) / min_lines;

    // we print the diff only if diff lines count is 6 or less
    // OR overall number of lines changed is 10% or less of
    // `min(expected_lines, actual_lines)`

    lines_changed <= 6 || diff_fraction <= 0.1
}

// github-like diff
fn print_diff(diffs: Vec<Difference>) {
    macro_rules! print_diff_snippet {
        ($title:expr, $ty:ident, $color:expr, $diffs:expr) => {{
            println_bold!($color, $title);
            dotted_line($color);

            for (i, chunk) in $diffs.iter().enumerate() {
                match chunk {
                    Difference::$ty(ref chunk) => {
                        if i != 0 {
                            print!(" ");
                        }

                        // LF symbols should not be colored
                        let trimmed = chunk.trim_end_matches('\n');
                        print_bg_color!($color, "{}", trimmed);
                        print!("{}", "\n".repeat(chunk.len() - trimmed.len()));
                    }

                    Difference::Same(ref chunk) => {
                        if i != 0 {
                            print!(" ");
                        }
                        print_color!($color, "{}", chunk);
                    }

                    _ => {}
                }
            }

            println!();
            dotted_line($color);
            println!();
        }};
    }

    print_diff_snippet!("EXPECTED:", Rem, EXPECTED_COLOR, &diffs);
    println!();
    print_diff_snippet!("ACTUAL OUTPUT:", Add, ACTUAL_COLOR, &diffs);
    println!();
}
