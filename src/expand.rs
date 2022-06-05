use crate::error::{Error, Result};
use crate::manifest::Name;
use crate::Test;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct ExpandedTest {
    pub name: Name,
    pub test: Test,
    pub error: Option<Error>,
}

pub(crate) fn expand_globs(tests: &[Test]) -> Vec<ExpandedTest> {
    let mut set = ExpandedTestSet::new();

    for test in tests {
        if let Some(utf8) = test.path.to_str() {
            if utf8.contains('*') {
                match glob(utf8) {
                    Ok(paths) => {
                        let expected = test.expected;
                        for path in paths {
                            set.insert(Test { path, expected }, None);
                        }
                    }
                    Err(error) => set.insert(test.clone(), Some(error)),
                }
                continue;
            }
        }
        set.insert(test.clone(), None);
    }

    set.vec
}

struct ExpandedTestSet {
    vec: Vec<ExpandedTest>,
}

impl ExpandedTestSet {
    fn new() -> Self {
        ExpandedTestSet { vec: Vec::new() }
    }

    fn insert(&mut self, test: Test, error: Option<Error>) {
        let name = Name(format!("trybuild{:03}", self.vec.len()));
        self.vec.push(ExpandedTest { name, test, error });
    }
}

fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut paths = glob::glob(pattern)?
        .map(|entry| entry.map_err(Error::from))
        .collect::<Result<Vec<PathBuf>>>()?;
    paths.sort();
    Ok(paths)
}
