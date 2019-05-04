use crate::error::Result;
use crate::manifest::Dependency;
use serde::Deserialize;
use std::collections::BTreeMap as Map;
use std::fs;
use std::path::Path;

pub fn get(manifest_dir: &Path) -> Map<String, Dependency> {
    try_get(manifest_dir).unwrap_or_default()
}

fn try_get(manifest_dir: &Path) -> Result<Map<String, Dependency>> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let manifest: Manifest = toml::from_str(&manifest_str)?;

    let mut dependencies = manifest.dependencies;
    dependencies.extend(manifest.dev_dependencies);

    for dep in dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| manifest_dir.join(path));
    }

    Ok(dependencies)
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    dependencies: Map<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: Map<String, Dependency>,
}
