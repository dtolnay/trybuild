use crate::error::{Error, Result};
use std::env;

#[derive(PartialEq, Default, Debug)]
pub(crate) enum Update {
    #[default]
    Wip,
    Overwrite,
}

impl Update {
    pub fn env() -> Result<Self> {
        let Some(var) = env::var_os("TRYBUILD") else {
            return Ok(Update::default());
        };

        match var.as_os_str().to_str() {
            Some("wip") => Ok(Update::Wip),
            Some("overwrite") => Ok(Update::Overwrite),
            _ => Err(Error::UpdateVar(var)),
        }
    }
}

/// Function that examines the `TRYBUILD_NO_OFFLINE_MODE` environment variable
/// and returns a boolean indicating whether or not the `--offline` option
/// should be passed to cargo.
pub(crate) fn use_offline_mode() -> bool {
    // if the environment variable is set (we don't care what value), then
    // offline mode should be disabled. Otherwise, it should be enabled (the
    // default behavior)
    env::var_os("TRYBUILD_NO_OFFLINE_MODE").is_none()
}
