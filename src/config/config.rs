use anyhow::{bail, Context, Result};

use super::configparser::ConfigParser;
use crate::config::misc::get_config_file;

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub(crate) ls_command: String,
    // pub(crate) ls_args: Vec<String>,
}

impl Config {
    /// Load the config file from disk.
    pub(crate) fn load() -> Result<Self> {
        let config_file = get_config_file().context("getting config file")?;
        let parser: ConfigParser = config_file.try_into()?;
        let mut result = Self::default();

        result.ls_command = parser.get_value_or("LsCommand", "ls");

        result.sanity_checked()
    }

    /// Takes ownership of the instance and returns itself, but sanity-checked and wrapped in
    /// `anyhow::Result<_>`.
    fn sanity_checked(self) -> Result<Self> {
        if self.ls_command.is_empty() {
            bail!("missing value for LsCommand");
        }
        Ok(self)
    }
}
