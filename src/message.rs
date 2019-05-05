use termcolor::Color::{self, *};

use super::{Expected, Test};
use crate::banner;
use crate::error::Error;
use crate::normalize;
use crate::term;

use std::path::Path;
use std::process::Output;

pub(crate) fn prepare_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    term::bold_color(Red);
    print!("ERROR");
    term::reset();
    println!(": {}", err);
    println!();
}

pub(crate) fn test_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    term::bold_color(Red);
    println!("error");
    term::color(Red);
    println!("{}", err);
    term::reset();
    println!();
}

pub(crate) fn no_tests_enabled() {
    term::color(Yellow);
    println!("There are no tests enabled yet.");
    term::reset();
}

pub(crate) fn nice() {
    term::color(Green);
    println!("nice!");
    term::reset();
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

    print!("Testing ");
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
    term::bold_color(Red);
    println!("error");
    snippet(Red, stderr);
    println!();
}

pub(crate) fn should_not_have_compiled() {
    term::bold_color(Red);
    println!("error");
    term::color(Red);
    println!("Expected test case to fail to compile, but it succeeded.");
    term::reset();
    println!();
}

pub(crate) fn write_stderr_wip(wip_path: &Path, stderr_path: &Path, stderr: &str) {
    let wip_path = wip_path.to_string_lossy();
    let stderr_path = stderr_path.to_string_lossy();

    term::bold_color(Yellow);
    println!("wip");
    println!();
    print!("NOTE");
    term::reset();
    println!(": writing the following output to `{}`.", wip_path);
    println!(
        "Move this file to `{}` to accept it as correct.",
        stderr_path,
    );
    snippet(Yellow, stderr);
    println!();
}

pub(crate) fn overwrite_stderr(stderr_path: &Path, stderr: &str) {
    let stderr_path = stderr_path.to_string_lossy();

    term::bold_color(Yellow);
    println!("wip");
    println!();
    print!("NOTE");
    term::reset();
    println!(": writing the following output to `{}`.", stderr_path);
    snippet(Yellow, stderr);
    println!();
}

pub(crate) fn mismatch(expected: &str, actual: &str) {
    term::bold_color(Red);
    println!("mismatch");
    term::reset();
    println!();
    term::bold_color(Blue);
    println!("EXPECTED:");
    snippet(Blue, expected);
    println!();
    term::bold_color(Red);
    println!("ACTUAL OUTPUT:");
    snippet(Red, actual);
    println!();
}

pub(crate) fn output(warnings: &str, output: &Output) {
    let success = output.status.success();
    let stdout = normalize::trim(&output.stdout);
    let stderr = normalize::trim(&output.stderr);
    let has_output = !stdout.is_empty() || !stderr.is_empty();

    if success {
        nice();
        if has_output || !warnings.is_empty() {
            println!();
        }
    } else {
        term::bold_color(Red);
        println!("error");
        term::color(Red);
        if has_output {
            println!("Test case failed at runtime.");
        } else {
            println!("Execution of the test case was unsuccessful but there was no output.");
        }
        term::reset();
        println!();
    }

    self::warnings(warnings);

    let color = if success { Yellow } else { Red };

    for (name, content) in &[("STDOUT", stdout), ("STDERR", stderr)] {
        if !content.is_empty() {
            term::bold_color(color);
            println!("{}:", name);
            snippet(color, &normalize::trim(content));
            println!();
        }
    }
}

pub(crate) fn warnings(warnings: &str) {
    if warnings.is_empty() {
        return;
    }

    term::bold_color(Yellow);
    println!("WARNINGS:");
    snippet(Yellow, warnings);
    println!();
}

fn snippet(color: Color, content: &str) {
    term::color(color);
    banner::dotted();

    // Color one line at a time because Travis does not preserve color setting
    // across output lines.
    for line in content.lines() {
        term::color(color);
        println!("{}", line);
    }

    term::color(color);
    banner::dotted();
    term::reset();
}
