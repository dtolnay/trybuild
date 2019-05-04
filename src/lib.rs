#[macro_use]
mod term;

#[macro_use]
mod path;

mod banner;
mod cargo;
mod error;
mod manifest;
mod message;
mod normalize;
mod run;

use crate::manifest::Dependency;
use std::cell::RefCell;
use std::collections::BTreeMap as Map;
use std::path::{Path, PathBuf};
use std::thread;

pub struct TestCases {
    runner: RefCell<Runner>,
}

struct Runner {
    deps: Map<String, Dependency>,
    tests: Vec<Test>,
}

#[derive(Clone)]
struct Test {
    path: PathBuf,
    expected: Expected,
}

#[derive(Copy, Clone)]
enum Expected {
    Pass,
    CompileFail,
}

impl TestCases {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TestCases {
            runner: RefCell::new(Runner {
                deps: Map::new(),
                tests: Vec::new(),
            }),
        }
    }

    pub fn dependencies(&self, dependencies: &str) {
        match toml::from_str::<Map<String, Dependency>>(dependencies) {
            Ok(deps) => self.runner.borrow_mut().deps.extend(deps),
            Err(error) => panic!("{}", error),
        }
    }

    pub fn pass<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().tests.push(Test {
            path: path.as_ref().to_owned(),
            expected: Expected::Pass,
        });
    }

    pub fn compile_fail<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().tests.push(Test {
            path: path.as_ref().to_owned(),
            expected: Expected::CompileFail,
        });
    }
}

impl Drop for TestCases {
    fn drop(&mut self) {
        if !thread::panicking() {
            self.runner.borrow_mut().run();
        }
    }
}
