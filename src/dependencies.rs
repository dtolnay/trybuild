use crate::directory::Directory;
use crate::error::Error;
use crate::inherit::InheritEdition;
use crate::manifest::Edition;
use serde::de::value::MapAccessDeserializer;
use serde::de::value::StrDeserializer;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;
use std::fmt;
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_manifest(manifest_dir: &Directory) -> Result<Manifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let mut manifest = (|| {
        let manifest_str = fs::read_to_string(&cargo_toml_path)?;
        let manifest: Manifest = toml::from_str(&manifest_str)?;
        Ok(manifest)
    })()
    .map_err(|err| Error::GetManifest(cargo_toml_path, Box::new(err)))?;

    fix_dependencies(&mut manifest.dependencies, manifest_dir);
    fix_dependencies(&mut manifest.dev_dependencies, manifest_dir);
    for target in manifest.target.values_mut() {
        fix_dependencies(&mut target.dependencies, manifest_dir);
        fix_dependencies(&mut target.dev_dependencies, manifest_dir);
    }

    Ok(manifest)
}

pub(crate) fn get_workspace_manifest(manifest_dir: &Directory) -> WorkspaceManifest {
    try_get_workspace_manifest(manifest_dir).unwrap_or_default()
}

pub(crate) fn try_get_workspace_manifest(
    manifest_dir: &Directory,
) -> Result<WorkspaceManifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: WorkspaceManifest = toml::from_str(&manifest_str)?;

    fix_dependencies(&mut manifest.workspace.dependencies, manifest_dir);
    fix_patches(&mut manifest.patch, manifest_dir);
    fix_replacements(&mut manifest.replace, manifest_dir);

    Ok(manifest)
}

fn fix_dependencies(dependencies: &mut Map<String, Dependency>, dir: &Directory) {
    dependencies.remove("trybuild");
    for dep in dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| Directory::new(dir.join(path)));
    }
}

fn fix_patches(patches: &mut Map<String, RegistryPatch>, dir: &Directory) {
    for registry in patches.values_mut() {
        registry.crates.remove("trybuild");
        for patch in registry.crates.values_mut() {
            patch.path = patch.path.as_ref().map(|path| dir.join(path));
        }
    }
}

fn fix_replacements(replacements: &mut Map<String, Patch>, dir: &Directory) {
    replacements.remove("trybuild");
    for replacement in replacements.values_mut() {
        replacement.path = replacement.path.as_ref().map(|path| dir.join(path));
    }
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct WorkspaceManifest {
    #[serde(default)]
    pub workspace: WorkspaceWorkspace,
    #[serde(default)]
    pub patch: Map<String, RegistryPatch>,
    #[serde(default)]
    pub replace: Map<String, Patch>,
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct WorkspaceWorkspace {
    #[serde(default)]
    pub package: WorkspacePackage,
    #[serde(default)]
    pub dependencies: Map<String, Dependency>,
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct WorkspacePackage {
    pub edition: Option<Edition>,
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct Manifest {
    #[serde(rename = "cargo-features", default)]
    pub cargo_features: Vec<String>,
    #[serde(default)]
    pub package: Package,
    #[serde(default)]
    pub features: Map<String, Vec<String>>,
    #[serde(default)]
    pub dependencies: Map<String, Dependency>,
    #[serde(default, alias = "dev-dependencies")]
    pub dev_dependencies: Map<String, Dependency>,
    #[serde(default)]
    pub target: Map<String, TargetDependencies>,
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct Package {
    pub name: String,
    #[serde(default)]
    pub edition: EditionOrInherit,
    pub resolver: Option<String>,
}

#[derive(Debug)]
pub(crate) enum EditionOrInherit {
    Edition(Edition),
    Inherit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "Self")]
pub(crate) struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Directory>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(
        rename = "default-features",
        default = "get_true",
        skip_serializing_if = "is_true"
    )]
    pub default_features: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub workspace: bool,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TargetDependencies {
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub dependencies: Map<String, Dependency>,
    #[serde(
        default,
        alias = "dev-dependencies",
        skip_serializing_if = "Map::is_empty"
    )]
    pub dev_dependencies: Map<String, Dependency>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub(crate) struct RegistryPatch {
    pub crates: Map<String, Patch>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Patch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

fn get_true() -> bool {
    true
}

fn is_true(boolean: &bool) -> bool {
    *boolean
}

fn is_false(boolean: &bool) -> bool {
    !*boolean
}

impl Default for EditionOrInherit {
    fn default() -> Self {
        EditionOrInherit::Edition(Edition::default())
    }
}

impl<'de> Deserialize<'de> for EditionOrInherit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EditionOrInheritVisitor;

        impl<'de> Visitor<'de> for EditionOrInheritVisitor {
            type Value = EditionOrInherit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("edition")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Edition::deserialize(StrDeserializer::new(s)).map(EditionOrInherit::Edition)
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                InheritEdition::deserialize(MapAccessDeserializer::new(map))?;
                Ok(EditionOrInherit::Inherit)
            }
        }

        deserializer.deserialize_any(EditionOrInheritVisitor)
    }
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
                    optional: false,
                    default_features: true,
                    features: Vec::new(),
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    workspace: false,
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
