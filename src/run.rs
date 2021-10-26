use crate::cargo::{self, Metadata};
use crate::dependencies::{self, Dependency};
use crate::directory::Directory;
use crate::env::Update;
use crate::error::{Error, Result};
use crate::flock::Lock;
use crate::manifest::{Bin, Build, Config, Manifest, Name, Package, Workspace};
use crate::message::{self, Fail, Warn};
use crate::normalize::{self, Context, Variations};
use crate::{features, rustflags, Expected, Runner, Test};
use std::collections::BTreeMap as Map;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::mem;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Project {
    pub dir: Directory,
    source_dir: Directory,
    pub target_dir: Directory,
    pub name: String,
    update: Update,
    pub has_pass: bool,
    has_compile_fail: bool,
    pub features: Option<Vec<String>>,
    pub workspace: Directory,
    pub path_dependencies: Vec<PathDependency>,
    manifest: Manifest,
}

#[derive(Debug)]
pub struct PathDependency {
    pub name: String,
    pub normalized_path: Directory,
}

impl Runner {
    pub fn run(&mut self) {
        let mut tests = expand_globs(&self.tests);
        filter(&mut tests);

        let (project, _lock) = (|| {
            let project = self.prepare(&tests)?;
            let lock = Lock::acquire(path!(project.dir / ".lock"));
            self.write(&project)?;
            Ok((project, lock))
        })()
        .unwrap_or_else(|err| {
            message::prepare_fail(err);
            panic!("tests failed");
        });

        print!("\n\n");

        let len = tests.len();
        let mut failures = 0;

        if tests.is_empty() {
            message::no_tests_enabled();
        } else {
            for test in tests {
                if let Err(err) = test.run(&project) {
                    failures += 1;
                    message::test_fail(err);
                }
            }
        }

        print!("\n\n");

        if failures > 0 && project.name != "trybuild-tests" {
            panic!("{} of {} tests failed", failures, len);
        }
    }

    fn prepare(&self, tests: &[ExpandedTest]) -> Result<Project> {
        let Metadata {
            target_directory: target_dir,
            workspace_root: workspace,
            packages,
        } = cargo::metadata()?;

        let crate_name = env::var("CARGO_PKG_NAME").map_err(Error::PkgName)?;

        let mut has_pass = false;
        let mut has_compile_fail = false;
        for e in tests {
            match e.test.expected {
                Expected::Pass => has_pass = true,
                Expected::CompileFail => has_compile_fail = true,
            }
        }

        let source_dir = env::var_os("CARGO_MANIFEST_DIR")
            .map(Directory::from)
            .ok_or(Error::ProjectDir)?;
        let source_manifest = dependencies::get_manifest(&source_dir);

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

        let project_dir = path!(target_dir / "tests" / crate_name /);
        fs::create_dir_all(&project_dir)?;

        let project_name = format!("{}-tests", crate_name);
        let manifest = self.make_manifest(
            crate_name,
            &workspace,
            &project_name,
            &source_dir,
            tests,
            source_manifest,
        );

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
            features,
            workspace,
            path_dependencies,
            manifest,
        })
    }

    fn write(&self, project: &Project) -> Result<()> {
        let manifest_toml = toml::to_string(&project.manifest)?;

        let config = self.make_config();
        let config_toml = toml::to_string(&config)?;

        fs::create_dir_all(path!(project.dir / ".cargo"))?;
        fs::write(path!(project.dir / ".cargo" / "config"), config_toml)?;
        fs::write(path!(project.dir / "Cargo.toml"), manifest_toml)?;
        fs::write(path!(project.dir / "main.rs"), b"fn main() {}\n")?;

        cargo::build_dependencies(project)?;

        Ok(())
    }

    fn make_manifest(
        &self,
        crate_name: String,
        workspace: &Directory,
        project_name: &str,
        source_dir: &Directory,
        tests: &[ExpandedTest],
        source_manifest: dependencies::Manifest,
    ) -> Manifest {
        let workspace_manifest = dependencies::get_workspace_manifest(workspace);

        let features = source_manifest
            .features
            .keys()
            .map(|feature| {
                let enable = format!("{}/{}", crate_name, feature);
                (feature.clone(), vec![enable])
            })
            .collect();

        let mut manifest = Manifest {
            package: Package {
                name: project_name.to_owned(),
                version: "0.0.0".to_owned(),
                edition: source_manifest.package.edition,
                publish: false,
            },
            features,
            dependencies: Map::new(),
            target: source_manifest.target,
            bins: Vec::new(),
            workspace: Some(Workspace {}),
            // Within a workspace, only the [patch] and [replace] sections in
            // the workspace root's Cargo.toml are applied by Cargo.
            patch: workspace_manifest.patch,
            replace: workspace_manifest.replace,
        };

        manifest.dependencies.extend(source_manifest.dependencies);
        manifest
            .dependencies
            .extend(source_manifest.dev_dependencies);
        for target in manifest.target.values_mut() {
            let dev_dependencies = mem::replace(&mut target.dev_dependencies, Map::new());
            target.dependencies.extend(dev_dependencies);
        }
        manifest.dependencies.insert(
            crate_name,
            Dependency {
                version: None,
                path: Some(source_dir.clone()),
                default_features: false,
                features: Vec::new(),
                git: None,
                branch: None,
                tag: None,
                rev: None,
                rest: Map::new(),
            },
        );

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

        manifest
    }

    fn make_config(&self) -> Config {
        Config {
            build: Build {
                rustflags: rustflags::make_vec(),
            },
        }
    }
}

impl Test {
    fn run(&self, project: &Project, name: &Name) -> Result<()> {
        let show_expected = project.has_pass && project.has_compile_fail;
        message::begin_test(self, show_expected);
        check_exists(&self.path)?;

        let output = cargo::build_test(project, name)?;
        let success = output.status.success();
        let stdout = output.stdout;
        let stderr = normalize::diagnostics(
            output.stderr,
            Context {
                krate: &name.0,
                source_dir: &project.source_dir,
                workspace: &project.workspace,
                input_file: &self.path,
                path_dependencies: &project.path_dependencies,
            },
        );

        let check = match self.expected {
            Expected::Pass => Test::check_pass,
            Expected::CompileFail => Test::check_compile_fail,
        };

        check(self, project, name, success, stdout, stderr)
    }

    fn check_pass(
        &self,
        project: &Project,
        name: &Name,
        success: bool,
        build_stdout: Vec<u8>,
        variations: Variations,
    ) -> Result<()> {
        let preferred = variations.preferred();
        if !success {
            message::failed_to_build(preferred);
            return Err(Error::CargoFail);
        }

        let mut output = cargo::run_test(project, name)?;
        output.stdout.splice(..0, build_stdout);
        message::output(preferred, &output);
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::RunFailed)
        }
    }

    fn check_compile_fail(
        &self,
        project: &Project,
        _name: &Name,
        success: bool,
        build_stdout: Vec<u8>,
        variations: Variations,
    ) -> Result<()> {
        let preferred = variations.preferred();

        if success {
            message::should_not_have_compiled();
            message::fail_output(Fail, &build_stdout);
            message::warnings(preferred);
            return Err(Error::ShouldNotHaveCompiled);
        }

        let stderr_path = self.path.with_extension("stderr");

        if !stderr_path.exists() {
            match project.update {
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
                }
                Update::Overwrite => {
                    message::overwrite_stderr(&stderr_path, preferred);
                    fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
                }
            }
            message::fail_output(Warn, &build_stdout);
            return Ok(());
        }

        let expected = fs::read_to_string(&stderr_path)
            .map_err(Error::ReadStderr)?
            .replace("\r\n", "\n");

        if variations.any(|stderr| expected == stderr) {
            message::ok();
            return Ok(());
        }

        match project.update {
            Update::Wip => {
                message::mismatch(&expected, preferred);
                Err(Error::Mismatch)
            }
            Update::Overwrite => {
                message::overwrite_stderr(&stderr_path, preferred);
                fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
                Ok(())
            }
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

#[derive(Debug)]
struct ExpandedTest {
    name: Name,
    test: Test,
    error: Option<Error>,
}

fn expand_globs(tests: &[Test]) -> Vec<ExpandedTest> {
    fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = glob::glob(pattern)?
            .map(|entry| entry.map_err(Error::from))
            .collect::<Result<Vec<PathBuf>>>()?;
        paths.sort();
        Ok(paths)
    }

    fn bin_name(i: usize) -> Name {
        Name(format!("trybuild{:03}", i))
    }

    let mut vec = Vec::new();

    for test in tests {
        let mut expanded = ExpandedTest {
            name: bin_name(vec.len()),
            test: test.clone(),
            error: None,
        };
        if let Some(utf8) = test.path.to_str() {
            if utf8.contains('*') {
                match glob(utf8) {
                    Ok(paths) => {
                        for path in paths {
                            vec.push(ExpandedTest {
                                name: bin_name(vec.len()),
                                test: Test {
                                    path,
                                    expected: expanded.test.expected,
                                },
                                error: None,
                            });
                        }
                        continue;
                    }
                    Err(error) => expanded.error = Some(error),
                }
            }
        }
        vec.push(expanded);
    }

    vec
}

impl ExpandedTest {
    fn run(self, project: &Project) -> Result<()> {
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
