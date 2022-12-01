use anyhow::{bail, Context, Result};
use smart_default::SmartDefault;

use super::configparser::ConfigParser;
use super::misc::get_config_file;

#[derive(Debug, SmartDefault)]
pub(crate) struct Config {
    #[default("ls".to_string())]
    pub(crate) ls_command: String,

    #[default(vec!["-hl".to_string()])]
    pub(crate) ls_args: Vec<String>,
}

impl Config {
    /// Load the config file from disk.
    pub(crate) fn load() -> Result<Self> {
        let config_file = get_config_file().context("getting config file")?;
        let result = Self::default();

        match TryInto::<ConfigParser>::try_into(config_file) {
            Ok(parser) => result.fill_from_parser(parser),
            _ => Ok(result),
        }
    }

    /// Takes ownership of the instance and returns itself, but sanity-checked and wrapped in
    /// `anyhow::Result<_>`.
    fn sanity_checked(self) -> Result<Self> {
        if self.ls_command.is_empty() {
            bail!("missing value for LsCommand");
        }
        Ok(self)
    }

    /// populate all fields with the data from the parser. Return itself, sanity-checked.
    fn fill_from_parser(mut self, parser: ConfigParser) -> Result<Config> {
        parser.get_value_into("LsCommand", &mut self.ls_command);
        parser.get_values_into("LsArgs", &mut self.ls_args);

        self.sanity_checked()
    }
}
