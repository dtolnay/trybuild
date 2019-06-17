use crate::error::Error;
use crate::manifest::Edition;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap as Map;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;

pub fn get_manifest(manifest_dir: &Path) -> Manifest {
    try_get_manifest(manifest_dir).unwrap_or_default()
}

fn try_get_manifest(manifest_dir: &Path) -> Result<Manifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: Manifest = toml::from_str(&manifest_str)?;

    fix_dependencies(&mut manifest.dependencies, manifest_dir);
    fix_dependencies(&mut manifest.dev_dependencies, manifest_dir);

    if let Some(ref mut patches) = manifest.patch {
        fix_patches(patches, manifest_dir);
    }

    if let Some(ref mut replacements) = manifest.replace {
        fix_replacements(replacements, manifest_dir);
    }

    Ok(manifest)
}

pub fn try_get_workspace_manifest(manifest_dir: &Path) -> Result<WorkspaceManifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: WorkspaceManifest = toml::from_str(&manifest_str)?;

    if let Some(ref mut patches) = manifest.patch {
        fix_patches(patches, manifest_dir);
    }

    if let Some(ref mut replacements) = manifest.replace {
        fix_replacements(replacements, manifest_dir);
    }

    Ok(manifest)
}

fn fix_dependencies(dependencies: &mut Map<String, Dependency>, dir: &Path) {
    dependencies.remove("trybuild");
    for dep in dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| dir.join(path));
    }
}

fn fix_patches(patches: &mut Map<String, RegistryPatch>, dir: &Path) {
    for registry in patches.values_mut() {
        registry.crates.remove("trybuild");
        for patch in registry.crates.values_mut() {
            patch.path = patch.path.as_ref().map(|path| dir.join(path));
        }
    }
}

fn fix_replacements(replacements: &mut Map<String, Replacement>, dir: &Path) {
    replacements.remove("trybuild");
    for replacement in replacements.values_mut() {
        replacement.path = replacement.path.as_ref().map(|path| dir.join(path));
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct WorkspaceManifest {
    #[serde(default)]
    pub members: Members,
    pub patch: Option<Map<String, RegistryPatch>>,
    pub replace: Option<Map<String, Replacement>>,
}

impl WorkspaceManifest {
    /// Within a workspace the [patch], [replace] and [profile.*] sections in Cargo.toml are only
    /// recognized in the root crate's manifest, and ignored in member crates' manifests:
    pub fn apply_to(&self, manifest: &mut Manifest) {
        manifest.patch = self.patch.clone();
        manifest.replace = self.replace.clone();
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct Members {}

#[derive(Deserialize, Default, Debug)]
pub struct Manifest {
    #[serde(default)]
    pub package: Package,
    #[serde(default)]
    pub features: Map<String, Vec<String>>,
    #[serde(default)]
    pub dependencies: Map<String, Dependency>,
    #[serde(default, alias = "dev-dependencies")]
    pub dev_dependencies: Map<String, Dependency>,
    pub patch: Option<Map<String, RegistryPatch>>,
    pub replace: Option<Map<String, Replacement>>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Package {
    #[serde(default)]
    pub edition: Edition,
    pub workspace: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "Self")]
pub struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(
        rename = "default-features",
        default = "get_true",
        skip_serializing_if = "is_true"
    )]
    pub default_features: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryPatch {
    #[serde(flatten)]
    crates: Map<String, Patch>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Patch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

pub type Replacement = Patch;

fn get_true() -> bool {
    true
}

fn is_true(boolean: &bool) -> bool {
    *boolean
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Dependency::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DependencyVisitor;

        impl<'de> Visitor<'de> for DependencyVisitor {
            type Value = Dependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a version string like \"0.9.8\" or a \
                     dependency like { version = \"0.9.8\" }",
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dependency {
                    version: Some(s.to_owned()),
                    path: None,
                    default_features: true,
                    features: Vec::new(),
                    rest: Map::new(),
                })
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                Dependency::deserialize(MapAccessDeserializer::new(map))
            }
        }

        deserializer.deserialize_any(DependencyVisitor)
    }
}
