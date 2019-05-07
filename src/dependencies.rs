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

pub fn get(manifest_dir: &Path) -> Manifest {
    try_get(manifest_dir).unwrap_or_default()
}

fn try_get(manifest_dir: &Path) -> Result<Manifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: Manifest = toml::from_str(&manifest_str)?;

    manifest.dev_dependencies.remove("trybuild");

    for dep in manifest.dev_dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| manifest_dir.join(path));
    }

    Ok(manifest)
}

#[derive(Deserialize, Default)]
pub struct Manifest {
    #[serde(default)]
    pub package: Package,
    #[serde(default)]
    pub features: Map<String, Vec<String>>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: Map<String, Dependency>,
}

#[derive(Deserialize, Default)]
pub struct Package {
    #[serde(default)]
    pub edition: Edition,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(remote = "Self")]
pub struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(rename = "default-features", default = "get_true", skip_serializing_if = "is_true")]
    pub default_features: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

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
