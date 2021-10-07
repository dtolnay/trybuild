use crate::directory::Directory;
use crate::error::{Error, Result};
use crate::manifest::Name;
use crate::run::Project;
use crate::rustflags;
use serde::Deserialize;
use std::process::{Command, Output, Stdio};
use std::{env, fs};

#[derive(Deserialize)]
pub struct Metadata {
    pub target_directory: Directory,
    pub workspace_root: Directory,
    pub packages: Vec<Package>,
}

#[derive(Deserialize)]
pub struct Package {
    pub name: String,
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
    cmd.env(
        "CARGO_TARGET_DIR",
        path!(project.target_dir / "tests" / "target"),
    );
    cmd.arg("--offline");
    rustflags::set_env(&mut cmd);
    cmd
}

pub fn build_dependencies(project: &Project) -> Result<()> {
    let workspace_cargo_lock = path!(project.workspace / "Cargo.lock");
    if workspace_cargo_lock.exists() {
        let _ = fs::copy(workspace_cargo_lock, path!(project.dir / "Cargo.lock"));
    } else {
        let _ = cargo(project).arg("generate-lockfile").status();
    }

    let status = cargo(project)
        .arg(if project.has_pass { "build" } else { "check" })
        .args(target())
        .arg("--bin")
        .arg(&project.name)
        .args(features(project))
        .status()
        .map_err(Error::Cargo)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CargoFail)
    }
}

pub fn build_test(project: &Project, name: &Name) -> Result<Output> {
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
        .output()
        .map_err(Error::Cargo)
}

pub fn run_test(project: &Project, name: &Name) -> Result<Output> {
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

pub fn metadata() -> Result<Metadata> {
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
    const TARGET: Option<&str> = include!(concat!(env!("OUT_DIR"), "/target"));

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
    } else if let Some(target) = TARGET {
        vec!["--target", target]
    } else {
        vec![]
    }
}
