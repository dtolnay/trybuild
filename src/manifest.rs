use crate::dependencies::Dependency;
use serde::Serialize;
use std::collections::BTreeMap as Map;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: Map<String, Dependency>,
    #[serde(rename = "bin")]
    pub bins: Vec<Bin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
}

#[derive(Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Edition,
    pub publish: bool,
}

#[derive(Serialize)]
pub enum Edition {
    #[serde(rename = "2018")]
    E2018,
}

#[derive(Serialize)]
pub struct Bin {
    pub name: Name,
    pub path: PathBuf,
}

#[derive(Serialize, Clone)]
pub struct Name(pub String);

#[derive(Serialize)]
pub struct Config {
    pub build: Build,
}

#[derive(Serialize)]
pub struct Build {
    pub rustflags: Vec<String>,
}

#[derive(Serialize)]
pub struct Workspace {}

impl AsRef<OsStr> for Name {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}
