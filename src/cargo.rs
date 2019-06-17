use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::error::{Error, Result};
use crate::manifest::Name;
use crate::run::Project;
use crate::rustflags;

#[derive(Deserialize)]
pub struct Metadata {
    pub target_directory: PathBuf,
    pub workspace_root: Option<PathBuf>,
}

fn raw_cargo() -> Command {
    Command::new(option_env!("CARGO").unwrap_or("cargo"))
}

fn cargo(project: &Project) -> Command {
    let mut cmd = raw_cargo();
    cmd.current_dir(&project.dir);
    cmd.env("CARGO_TARGET_DIR", &project.target_dir);
    rustflags::set_env(&mut cmd);
    cmd
}

pub fn build_dependencies(project: &Project) -> Result<()> {
    let status = cargo(project)
        .arg(if project.has_pass { "build" } else { "check" })
        .arg("--bin")
        .arg(&project.name)
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
        .arg("--format-version=1")
        .output()
        .map_err(Error::Cargo)?;

    serde_json::from_slice(&output.stdout).map_err(Error::Metadata)
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
