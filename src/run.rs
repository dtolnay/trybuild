use crate::cargo::{self, Metadata, PackageMetadata};
use crate::dependencies::{self, Dependency, EditionOrInherit};
use crate::directory::Directory;
use crate::env::Update;
use crate::error::{Error, Result};
use crate::expand::{expand_globs, ExpandedTest};
use crate::flock::Lock;
use crate::manifest::{Bin, Build, Config, Manifest, Name, Package, Workspace};
use crate::message::{self, Fail, Warn};
use crate::normalize::{self, Context, Variations};
use crate::{features, rustflags, Expected, Runner, Test};
use serde_derive::Deserialize;
use std::collections::{BTreeMap as Map, BTreeSet as Set};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::mem;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug)]
pub(crate) struct Project {
    pub dir: Directory,
    source_dir: Directory,
    pub target_dir: Directory,
    pub name: String,
    update: Update,
    pub has_pass: bool,
    has_compile_fail: bool,
    pub has_pass_test: bool,
    pub features: Option<Vec<String>>,
    pub workspace: Directory,
    pub path_dependencies: Vec<PathDependency>,
    manifest: Manifest,
    pub keep_going: bool,
}

#[derive(Debug)]
pub(crate) struct PathDependency {
    pub name: String,
    pub normalized_path: Directory,
}

struct Report {
    failures: usize,
    created_wip: usize,
}

impl Runner {
    pub(crate) fn run(&mut self) {
        let mut tests = expand_globs(&self.tests);
        filter(&mut tests);

        let (project, _lock) = (|| {
            let mut project = self.prepare(&tests)?;
            let lock = Lock::acquire(path!(project.dir / ".lock"))?;
            self.write(&mut project)?;
            Ok((project, lock))
        })()
        .unwrap_or_else(|err| {
            message::prepare_fail(err);
            panic!("tests failed");
        });

        print!("\n\n");

        let len = tests.len();
        let mut report = Report {
            failures: 0,
            created_wip: 0,
        };

        if tests.is_empty() {
            message::no_tests_enabled();
        } else if project.keep_going && !project.has_pass {
            report = match self.run_all(&project, tests) {
                Ok(failures) => failures,
                Err(err) => {
                    message::test_fail(err);
                    Report {
                        failures: len,
                        created_wip: 0,
                    }
                }
            }
        } else {
            for test in tests {
                match test.run(&project) {
                    Ok(Outcome::Passed) => {}
                    Ok(Outcome::CreatedWip) => report.created_wip += 1,
                    Err(err) => {
                        report.failures += 1;
                        message::test_fail(err);
                    }
                }
            }
        }

        print!("\n\n");

        if report.failures > 0 && project.name != "trybuild-tests" {
            panic!("{} of {} tests failed", report.failures, len);
        }
        if report.created_wip > 0 && project.name != "trybuild-tests" {
            panic!(
                "successfully created new stderr files for {} test cases",
                report.created_wip,
            );
        }
    }

    fn prepare(&self, tests: &[ExpandedTest]) -> Result<Project> {
        let Metadata {
            target_directory: target_dir,
            workspace_root: workspace,
            packages,
        } = cargo::metadata()?;

        let mut has_pass = false;
        let mut has_compile_fail = false;
        let mut has_pass_test = false;
        for e in tests {
            match e.test.expected {
                Expected::Pass => has_pass = true,
                Expected::CompileFail => has_compile_fail = true,
                Expected::PassTest => has_pass_test = true,
            }
        }

        let source_dir = cargo::manifest_dir()?;
        let source_manifest = dependencies::get_manifest(&source_dir)?;

        let mut features = features::find();

        let path_dependencies = source_manifest
            .dependencies
            .iter()
            .filter_map(|(name, dep)| {
                let path = dep.path.as_ref()?;
                if packages.iter().any(|p| &p.name == name) {
                    // Skip path dependencies coming from the workspace itself
                    None
                } else {
                    Some(PathDependency {
                        name: name.clone(),
                        normalized_path: path.canonicalize().ok()?,
                    })
                }
            })
            .collect();

        let crate_name = &source_manifest.package.name;
        let project_dir = path!(target_dir / "tests" / "trybuild" / crate_name /);
        fs::create_dir_all(&project_dir)?;

        let project_name = format!("{}-tests", crate_name);
        let manifest = self.make_manifest(
            &workspace,
            &project_name,
            &source_dir,
            &packages,
            tests,
            source_manifest,
        )?;

        if let Some(enabled_features) = &mut features {
            enabled_features.retain(|feature| manifest.features.contains_key(feature));
        }

        Ok(Project {
            dir: project_dir,
            source_dir,
            target_dir,
            name: project_name,
            update: Update::env()?,
            has_pass,
            has_compile_fail,
            has_pass_test,
            features,
            workspace,
            path_dependencies,
            manifest,
            keep_going: false,
        })
    }

    fn write(&self, project: &mut Project) -> Result<()> {
        let manifest_toml = toml::to_string(&project.manifest)?;

        let config = self.make_config();
        let config_toml = toml::to_string(&config)?;

        fs::create_dir_all(path!(project.dir / ".cargo"))?;
        fs::write(path!(project.dir / ".cargo" / "config.toml"), config_toml)?;
        fs::write(path!(project.dir / "Cargo.toml"), manifest_toml)?;

        let main_rs = b"\
            #![allow(unused_crate_dependencies, missing_docs)]\n\
            fn main() {}\n\
        ";
        fs::write(path!(project.dir / "main.rs"), &main_rs[..])?;

        cargo::build_dependencies(project)?;

        Ok(())
    }

    fn make_manifest(
        &self,
        workspace: &Directory,
        project_name: &str,
        source_dir: &Directory,
        packages: &[PackageMetadata],
        tests: &[ExpandedTest],
        source_manifest: dependencies::Manifest,
    ) -> Result<Manifest> {
        let crate_name = source_manifest.package.name;
        let workspace_manifest = dependencies::get_workspace_manifest(workspace);

        let edition = match source_manifest.package.edition {
            EditionOrInherit::Edition(edition) => edition,
            EditionOrInherit::Inherit => workspace_manifest
                .workspace
                .package
                .edition
                .ok_or(Error::NoWorkspaceManifest)?,
        };

        let mut dependencies = Map::new();
        dependencies.extend(source_manifest.dependencies);
        dependencies.extend(source_manifest.dev_dependencies);

        let cargo_toml_path = source_dir.join("Cargo.toml");
        let mut has_lib_target = true;
        for package_metadata in packages {
            if package_metadata.manifest_path == cargo_toml_path {
                has_lib_target = package_metadata
                    .targets
                    .iter()
                    .any(|target| target.crate_types != ["bin"]);
            }
        }
        if has_lib_target {
            dependencies.insert(
                crate_name.clone(),
                Dependency {
                    version: None,
                    path: Some(source_dir.clone()),
                    optional: false,
                    default_features: false,
                    features: Vec::new(),
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    workspace: false,
                    rest: Map::new(),
                },
            );
        }

        let mut targets = source_manifest.target;
        for target in targets.values_mut() {
            let dev_dependencies = mem::take(&mut target.dev_dependencies);
            target.dependencies.extend(dev_dependencies);
        }

        let mut features = source_manifest.features;
        for (feature, enables) in &mut features {
            enables.retain(|en| {
                let Some(dep_name) = en.strip_prefix("dep:") else {
                    return false;
                };
                if let Some(Dependency { optional: true, .. }) = dependencies.get(dep_name) {
                    return true;
                }
                for target in targets.values() {
                    if let Some(Dependency { optional: true, .. }) =
                        target.dependencies.get(dep_name)
                    {
                        return true;
                    }
                }
                false
            });
            if has_lib_target {
                enables.insert(0, format!("{}/{}", crate_name, feature));
            }
        }

        let mut manifest = Manifest {
            cargo_features: source_manifest.cargo_features,
            package: Package {
                name: project_name.to_owned(),
                version: "0.0.0".to_owned(),
                edition,
                resolver: source_manifest.package.resolver,
                publish: false,
            },
            features,
            dependencies,
            target: targets,
            bins: Vec::new(),
            workspace: Some(Workspace {
                dependencies: workspace_manifest.workspace.dependencies,
            }),
            // Within a workspace, only the [patch] and [replace] sections in
            // the workspace root's Cargo.toml are applied by Cargo.
            patch: workspace_manifest.patch,
            replace: workspace_manifest.replace,
        };

        manifest.bins.push(Bin {
            name: Name(project_name.to_owned()),
            path: Path::new("main.rs").to_owned(),
        });

        for expanded in tests {
            if expanded.error.is_none() {
                manifest.bins.push(Bin {
                    name: expanded.name.clone(),
                    path: source_dir.join(&expanded.test.path),
                });
            }
        }

        Ok(manifest)
    }

    fn make_config(&self) -> Config {
        Config {
            build: Build {
                rustflags: rustflags::make_vec(),
            },
        }
    }

    fn run_all(&self, project: &Project, tests: Vec<ExpandedTest>) -> Result<Report> {
        let mut report = Report {
            failures: 0,
            created_wip: 0,
        };

        let mut path_map = Map::new();
        for t in &tests {
            let src_path = project.source_dir.join(&t.test.path);
            path_map.insert(src_path, (&t.name, &t.test));
        }

        let output = cargo::build_all_tests(project)?;
        let parsed = parse_cargo_json(project, &output.stdout, &path_map);
        let fallback = Stderr::default();

        for mut t in tests {
            let show_expected = false;
            message::begin_test(&t.test, show_expected);

            if t.error.is_none() {
                t.error = check_exists(&t.test.path).err();
            }

            if t.error.is_none() {
                let src_path = project.source_dir.join(&t.test.path);
                let this_test = parsed.stderrs.get(&src_path).unwrap_or(&fallback);
                match t.test.check(project, &t.name, this_test, "") {
                    Ok(Outcome::Passed) => {}
                    Ok(Outcome::CreatedWip) => report.created_wip += 1,
                    Err(error) => t.error = Some(error),
                }
            }

            if let Some(err) = t.error {
                report.failures += 1;
                message::test_fail(err);
            }
        }

        Ok(report)
    }
}

enum Outcome {
    Passed,
    CreatedWip,
}

impl Test {
    fn run(&self, project: &Project, name: &Name) -> Result<Outcome> {
        let show_expected = project.has_pass && project.has_compile_fail;
        message::begin_test(self, show_expected);
        check_exists(&self.path)?;

        let mut path_map = Map::new();
        let src_path = project.source_dir.join(&self.path);
        path_map.insert(src_path.clone(), (name, self));

        let output = cargo::build_test(project, name)?;
        let parsed = parse_cargo_json(project, &output.stdout, &path_map);
        let fallback = Stderr::default();
        let this_test = parsed.stderrs.get(&src_path).unwrap_or(&fallback);
        self.check(project, name, this_test, &parsed.stdout)
    }

    fn check(
        &self,
        project: &Project,
        name: &Name,
        result: &Stderr,
        build_stdout: &str,
    ) -> Result<Outcome> {
        let check = match self.expected {
            Expected::Pass => Test::check_pass,
            Expected::CompileFail => Test::check_compile_fail,
            Expected::PassTest => Test::check_pass_test,
        };

        check(
            self,
            project,
            name,
            result.success,
            build_stdout,
            &result.stderr,
        )
    }

    fn check_pass(
        &self,
        project: &Project,
        name: &Name,
        success: bool,
        build_stdout: &str,
        variations: &Variations,
    ) -> Result<Outcome> {
        let preferred = variations.preferred();
        if !success {
            message::failed_to_build(preferred);
            return Err(Error::CargoFail);
        }

        let mut output = cargo::run(project, name)?;
        output.stdout.splice(..0, build_stdout.bytes());
        message::output(preferred, &output);
        if output.status.success() {
            Ok(Outcome::Passed)
        } else {
            Err(Error::RunFailed)
        }
    }

    fn check_compile_fail(
        &self,
        project: &Project,
        _name: &Name,
        success: bool,
        build_stdout: &str,
        variations: &Variations,
    ) -> Result<Outcome> {
        let preferred = variations.preferred();

        if success {
            message::should_not_have_compiled();
            message::fail_output(Fail, build_stdout);
            message::warnings(preferred);
            return Err(Error::ShouldNotHaveCompiled);
        }

        let stderr_path = self.path.with_extension("stderr");

        if !stderr_path.exists() {
            let outcome = match project.update {
                Update::Wip => {
                    let wip_dir = Path::new("wip");
                    fs::create_dir_all(wip_dir)?;
                    let gitignore_path = wip_dir.join(".gitignore");
                    fs::write(gitignore_path, "*\n")?;
                    let stderr_name = stderr_path
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("test.stderr"));
                    let wip_path = wip_dir.join(stderr_name);
                    message::write_stderr_wip(&wip_path, &stderr_path, preferred);
                    fs::write(wip_path, preferred).map_err(Error::WriteStderr)?;
                    Outcome::CreatedWip
                }
                Update::Overwrite => {
                    message::overwrite_stderr(&stderr_path, preferred);
                    fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
                    Outcome::Passed
                }
            };
            message::fail_output(Warn, build_stdout);
            return Ok(outcome);
        }

        let expected = fs::read_to_string(&stderr_path)
            .map_err(Error::ReadStderr)?
            .replace("\r\n", "\n");

        if variations.any(|stderr| expected == stderr) {
            message::ok();
            return Ok(Outcome::Passed);
        }

        match project.update {
            Update::Wip => {
                message::mismatch(&expected, preferred);
                Err(Error::Mismatch)
            }
            Update::Overwrite => {
                message::overwrite_stderr(&stderr_path, preferred);
                fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
                Ok(Outcome::Passed)
            }
        }
    }

    fn check_pass_test(
        &self,
        project: &Project,
        name: &Name,
        success: bool,
        build_stdout: &str,
        variations: &Variations,
    ) -> Result<Outcome> {
        let preferred = variations.preferred();
        if !success {
            message::failed_to_build(preferred);
            return Err(Error::CargoFail);
        }

        let mut output = cargo::test(project, name)?;
        output.stdout.splice(..0, build_stdout.bytes());
        message::output(preferred, &output);
        if output.status.success() {
            Ok(Outcome::Passed)
        } else {
            Err(Error::RunFailed)
        }
    }
}

fn check_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    match File::open(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Open(path.to_owned(), err)),
    }
}

impl ExpandedTest {
    fn run(self, project: &Project) -> Result<Outcome> {
        match self.error {
            None => self.test.run(project, &self.name),
            Some(error) => {
                let show_expected = false;
                message::begin_test(&self.test, show_expected);
                Err(error)
            }
        }
    }
}

// Filter which test cases are run by trybuild.
//
//     $ cargo test -- ui trybuild=tuple_structs.rs
//
// The first argument after `--` must be the trybuild test name i.e. the name of
// the function that has the #[test] attribute and calls trybuild. That's to get
// Cargo to run the test at all. The next argument starting with `trybuild=`
// provides a filename filter. Only test cases whose filename contains the
// filter string will be run.
#[allow(clippy::needless_collect)] // false positive https://github.com/rust-lang/rust-clippy/issues/5991
fn filter(tests: &mut Vec<ExpandedTest>) {
    let filters = env::args_os()
        .flat_map(OsString::into_string)
        .filter_map(|mut arg| {
            const PREFIX: &str = "trybuild=";
            if arg.starts_with(PREFIX) && arg != PREFIX {
                Some(arg.split_off(PREFIX.len()))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    if filters.is_empty() {
        return;
    }

    tests.retain(|t| {
        filters
            .iter()
            .any(|f| t.test.path.to_string_lossy().contains(f))
    });
}

#[derive(Deserialize)]
struct CargoMessage {
    #[allow(dead_code)]
    reason: Reason,
    target: RustcTarget,
    message: RustcMessage,
}

#[derive(Deserialize)]
enum Reason {
    #[serde(rename = "compiler-message")]
    CompilerMessage,
}

#[derive(Deserialize)]
struct RustcTarget {
    src_path: PathBuf,
}

#[derive(Deserialize)]
struct RustcMessage {
    rendered: String,
    level: String,
}

struct ParsedOutputs {
    stdout: String,
    stderrs: Map<PathBuf, Stderr>,
}

struct Stderr {
    success: bool,
    stderr: Variations,
}

impl Default for Stderr {
    fn default() -> Self {
        Stderr {
            success: true,
            stderr: Variations::default(),
        }
    }
}

fn parse_cargo_json(
    project: &Project,
    stdout: &[u8],
    path_map: &Map<PathBuf, (&Name, &Test)>,
) -> ParsedOutputs {
    let mut map = Map::new();
    let mut nonmessage_stdout = String::new();
    let mut remaining = &*String::from_utf8_lossy(stdout);
    let mut seen = Set::new();
    while !remaining.is_empty() {
        let Some(begin) = remaining.find("{\"reason\":") else {
            break;
        };
        let (nonmessage, rest) = remaining.split_at(begin);
        nonmessage_stdout.push_str(nonmessage);
        let len = match rest.find('\n') {
            Some(end) => end + 1,
            None => rest.len(),
        };
        let (message, rest) = rest.split_at(len);
        remaining = rest;
        if !seen.insert(message) {
            // Discard duplicate messages. This might no longer be necessary
            // after https://github.com/rust-lang/rust/issues/106571 is fixed.
            // Normally rustc would filter duplicates itself and I think this is
            // a short-lived bug.
            continue;
        }
        if let Ok(de) = serde_json::from_str::<CargoMessage>(message) {
            if de.message.level != "failure-note" {
                let Some((name, test)) = path_map.get(&de.target.src_path) else {
                    continue;
                };
                let entry = map
                    .entry(de.target.src_path)
                    .or_insert_with(Stderr::default);
                if de.message.level == "error" {
                    entry.success = false;
                }
                let normalized = normalize::diagnostics(
                    &de.message.rendered,
                    Context {
                        krate: &name.0,
                        source_dir: &project.source_dir,
                        workspace: &project.workspace,
                        input_file: &test.path,
                        target_dir: &project.target_dir,
                        path_dependencies: &project.path_dependencies,
                    },
                );
                entry.stderr.concat(&normalized);
            }
        }
    }
    nonmessage_stdout.push_str(remaining);
    ParsedOutputs {
        stdout: nonmessage_stdout,
        stderrs: map,
    }
}
