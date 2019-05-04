use std::collections::BTreeMap as Map;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::{Expected, Runner, Test};
use crate::banner;
use crate::cargo;
use crate::env::Update;
use crate::error::{Error, Result};
use crate::manifest::{
    Bin, Build, Config, Dependency, Edition, Manifest, Name, Package, Workspace,
};
use crate::message;
use crate::normalize;

const IGNORED_LINTS: &[&str] = &["dead_code"];

pub struct Project {
    pub dir: PathBuf,
    pub target_dir: PathBuf,
    pub name: String,
    update: Update,
    has_pass: bool,
    has_compile_fail: bool,
}

impl Runner {
    pub fn run(&mut self) {
        let tests = expand_globs(&self.tests);

        let project = self.prepare(&tests).unwrap_or_else(|err| {
            message::prepare_fail(err);
            panic!("tests failed");
        });

        println!();
        banner::colorful();

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

        banner::colorful();
        println!();

        if failures > 0 {
            panic!("{} of {} tests failed", failures, len);
        }
    }

    fn prepare(&self, tests: &[ExpandedTest]) -> Result<Project> {
        let target_dir = cargo::target_dir()?;
        let crate_name = env::var("CARGO_PKG_NAME").map_err(Error::PkgName)?;

        let mut has_pass = false;
        let mut has_compile_fail = false;
        for e in tests {
            match e.test.expected {
                Expected::Pass => has_pass = true,
                Expected::CompileFail => has_compile_fail = true,
            }
        }

        let project = Project {
            dir: path!(target_dir / "tests" / crate_name),
            target_dir,
            name: format!("{}-tests", crate_name),
            update: Update::env()?,
            has_pass,
            has_compile_fail,
        };

        let manifest = self.make_manifest(crate_name, &project, tests)?;
        let manifest_toml = toml::to_string(&manifest)?;

        let config = self.make_config();
        let config_toml = toml::to_string(&config)?;

        fs::create_dir_all(path!(project.dir / ".cargo"))?;
        fs::write(path!(project.dir / ".cargo" / "config"), config_toml)?;
        fs::write(path!(project.dir / "Cargo.toml"), manifest_toml)?;
        fs::write(path!(project.dir / "main.rs"), b"fn main() {}\n")?;

        cargo::build_dependencies(&project)?;

        Ok(project)
    }

    fn make_manifest(
        &self,
        crate_name: String,
        project: &Project,
        tests: &[ExpandedTest],
    ) -> Result<Manifest> {
        let mut manifest = Manifest {
            package: Package {
                name: project.name.clone(),
                version: "0.0.0".to_owned(),
                edition: Edition::E2018,
                publish: false,
            },
            dependencies: Map::new(),
            bins: Vec::new(),
            workspace: Some(Workspace {}),
        };

        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .ok_or(Error::ProjectDir)?;

        if self.deps.is_empty() {
            manifest.dependencies.insert(
                crate_name,
                Dependency {
                    version: None,
                    path: Some(manifest_dir.clone()),
                    rest: Map::new(),
                },
            );
        }

        for (depname, dependency) in &self.deps {
            manifest.dependencies.insert(
                depname.clone(),
                Dependency {
                    path: dependency.path.as_ref().map(|path| manifest_dir.join(path)),
                    ..dependency.clone()
                },
            );
        }

        manifest.bins.push(Bin {
            name: Name(project.name.to_owned()),
            path: Path::new("main.rs").to_owned(),
        });

        for expanded in tests {
            if expanded.error.is_none() {
                manifest.bins.push(Bin {
                    name: expanded.name.clone(),
                    path: manifest_dir.join(&expanded.test.path),
                });
            }
        }

        Ok(manifest)
    }

    fn make_config(&self) -> Config {
        let mut rustflags = Vec::new();

        for &lint in IGNORED_LINTS {
            rustflags.push("-A".to_owned());
            rustflags.push(lint.to_owned());
        }

        Config {
            build: Build { rustflags },
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
        let stderr = normalize::diagnostics(output.stderr);

        let check = match self.expected {
            Expected::Pass => Test::check_pass,
            Expected::CompileFail => Test::check_compile_fail,
        };

        check(self, project, name, success, stderr)
    }

    fn check_pass(
        &self,
        project: &Project,
        name: &Name,
        success: bool,
        stderr: String,
    ) -> Result<()> {
        if !success {
            message::failed_to_build(stderr);
            return Err(Error::CargoFail);
        }

        let output = cargo::run_test(project, name)?;
        message::output(stderr, &output);

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
        stderr: String,
    ) -> Result<()> {
        if success {
            message::should_not_have_compiled();
            message::warnings(stderr);
            return Err(Error::ShouldNotHaveCompiled);
        }

        let stderr_path = self.path.with_extension("stderr");

        if !stderr_path.exists() {
            match project.update {
                Update::Wip => {
                    let wip_dir = Path::new("wip");
                    fs::create_dir_all(wip_dir)?;
                    let stderr_name = stderr_path
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("test.stderr"));
                    let wip_path = wip_dir.join(stderr_name);
                    message::write_stderr_wip(&wip_path, &stderr_path, &stderr);
                    fs::write(wip_path, stderr).map_err(Error::WriteStderr)?;
                }
                Update::Overwrite => {
                    message::overwrite_stderr(&stderr_path, &stderr);
                    fs::write(stderr_path, stderr).map_err(Error::WriteStderr)?;
                }
            }
            return Ok(());
        }

        let expected = fs::read_to_string(&stderr_path)
            .map_err(Error::ReadStderr)?
            .replace("\r\n", "\n");

        if expected == stderr {
            message::nice();
            return Ok(());
        }

        match project.update {
            Update::Wip => {
                message::mismatch(&expected, &stderr);
                Err(Error::Mismatch)
            }
            Update::Overwrite => {
                message::overwrite_stderr(&stderr_path, &stderr);
                fs::write(stderr_path, stderr).map_err(Error::WriteStderr)?;
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
        Name(format!("test{:03}", i))
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
