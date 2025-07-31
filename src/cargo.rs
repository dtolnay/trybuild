use crate::directory::Directory;
use crate::error::{Error, Result};
use crate::manifest::Name;
use crate::run::Project;
use crate::rustflags;
use serde_derive::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::{env, io, iter};
use target_triple::TARGET;

#[derive(Deserialize)]
pub(crate) struct Metadata {
    pub target_directory: Directory,
    pub workspace_root: Directory,
    pub packages: Vec<PackageMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct PackageMetadata {
    pub name: String,
    pub targets: Vec<BuildTarget>,
    pub manifest_path: PathBuf,
}

#[derive(Deserialize)]
pub(crate) struct BuildTarget {
    pub crate_types: Vec<String>,
}

fn raw_cargo() -> Command {
    match env::var_os("CARGO") {
        Some(cargo) => Command::new(cargo),
        None => Command::new("cargo"),
    }
}

fn cargo(project: &Project) -> Command {
    let mut cmd = raw_cargo();
    cmd.current_dir(&project.dir);
    cmd.envs(cargo_target_dir(project));
    cmd.env_remove("RUSTFLAGS");
    cmd.env("CARGO_INCREMENTAL", "0");
    cmd.arg("--offline");

    let rustflags = rustflags::toml();
    cmd.arg(format!("--config=build.rustflags={rustflags}"));
    cmd.arg(format!("--config=target.{TARGET}.rustflags={rustflags}"));

    cmd
}

fn cargo_target_dir(project: &Project) -> impl Iterator<Item = (&'static str, PathBuf)> {
    iter::once((
        "CARGO_TARGET_DIR",
        path!(project.target_dir / "tests" / "trybuild"),
    ))
}

pub(crate) fn manifest_dir() -> Result<Directory> {
    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        return Ok(Directory::from(manifest_dir));
    }
    let mut dir = Directory::current()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Ok(dir);
        }
        dir = dir.parent().ok_or(Error::ProjectDir)?;
    }
}

pub(crate) fn build_dependencies(project: &mut Project) -> Result<()> {
    // Try copying or generating lockfile.
    match File::open(path!(project.workspace / "Cargo.lock")) {
        Ok(mut workspace_cargo_lock) => {
            if let Ok(mut new_cargo_lock) = File::create(path!(project.dir / "Cargo.lock")) {
                // Not fs::copy in order to avoid producing a read-only destination
                // file if the source file happens to be read-only.
                let _ = io::copy(&mut workspace_cargo_lock, &mut new_cargo_lock);
            }
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                let _ = cargo(project).arg("generate-lockfile").status();
            }
        }
    }

    let mut command = cargo(project);
    command
        .arg(if project.has_pass { "build" } else { "check" })
        .args(target())
        .arg("--bin")
        .arg(&project.name)
        .args(features(project));

    let status = command.status().map_err(Error::Cargo)?;
    if !status.success() {
        return Err(Error::CargoFail);
    }

    // Check if this Cargo contains https://github.com/rust-lang/cargo/pull/10383
    project.keep_going = command
        .arg("--keep-going")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    Ok(())
}

pub(crate) fn build_test(project: &Project, name: &Name) -> Result<Output> {
    let _ = cargo(project)
        .arg("clean")
        .arg("--package")
        .arg(&project.name)
        .arg("--color=never")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    cargo(project)
        .arg(if project.has_pass { "build" } else { "check" })
        .args(target())
        .arg("--bin")
        .arg(name)
        .args(features(project))
        .arg("--quiet")
        .arg("--color=never")
        .arg("--message-format=json")
        .output()
        .map_err(Error::Cargo)
}

pub(crate) fn build_all_tests(project: &Project) -> Result<Output> {
    let _ = cargo(project)
        .arg("clean")
        .arg("--package")
        .arg(&project.name)
        .arg("--color=never")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    cargo(project)
        .arg(if project.has_pass { "build" } else { "check" })
        .args(target())
        .arg("--bins")
        .args(features(project))
        .arg("--quiet")
        .arg("--color=never")
        .arg("--message-format=json")
        .arg("--keep-going")
        .output()
        .map_err(Error::Cargo)
}

pub(crate) fn run_test(project: &Project, name: &Name) -> Result<Output> {
    cargo(project)
        .arg("run")
        .args(target())
        .arg("--bin")
        .arg(name)
        .args(features(project))
        .arg("--quiet")
        .arg("--color=never")
        .output()
        .map_err(Error::Cargo)
}

pub(crate) fn metadata() -> Result<Metadata> {
    let output = raw_cargo()
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output()
        .map_err(Error::Cargo)?;

    serde_json::from_slice(&output.stdout).map_err(|err| {
        print!("{}", String::from_utf8_lossy(&output.stderr));
        Error::Metadata(err)
    })
}

fn features(project: &Project) -> Vec<String> {
    match &project.features {
        Some(features) => vec![
            "--no-default-features".to_owned(),
            "--features".to_owned(),
            features.join(","),
        ],
        None => vec![],
    }
}

fn target() -> Vec<&'static str> {
    // When --target flag is passed, cargo does not pass RUSTFLAGS to rustc when
    // building proc-macro and build script even if the host and target triples
    // are the same. Therefore, if we always pass --target to cargo, tools such
    // as coverage that require RUSTFLAGS do not work for tests run by trybuild.
    //
    // To avoid that problem, do not pass --target to cargo if we know that it
    // has not been passed.
    //
    // Currently, cargo does not have a way to tell the build script whether
    // --target has been passed or not, and there is no heuristic that can
    // handle this well.
    //
    // Therefore, expose a cfg to always treat the target as host.
    if cfg!(trybuild_no_target) {
        vec![]
    } else {
        vec!["--target", TARGET]
    }
}
