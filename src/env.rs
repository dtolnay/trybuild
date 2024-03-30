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
