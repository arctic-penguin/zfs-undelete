use std::path::PathBuf;
use std::{env, fs};

use anyhow::{bail, Context, Result};
use smart_default::SmartDefault;

#[derive(Debug, SmartDefault)]
pub(crate) struct Config {
    #[default("ls")]
    pub(crate) ls_command: String,
}

impl Config {
    /// Load the program config from file. Apply defaults if no value provided for a config key.
    pub(crate) fn load() -> Result<Self> {
        let mut this = Self::default();

        let conf_file = Self::get_config_file().context("could not find config file")?;
        if !conf_file.exists() {
            return Ok(this);
        }

        let content = fs::read_to_string(&conf_file)
            .with_context(|| format!("reading config file {conf_file:?}"))?;

        for line in content.lines() {
            let mut split = line.split('=');
            if split.next().unwrap().trim() == "LsCommand" {
                this.ls_command = split.next().unwrap().trim().to_owned();
            }
        }

        this.sanity_checked()
    }

    /// Get the path of the config file like. This is `$XDG_CONFIG_HOME/zfs-undelete.conf`.
    fn get_config_file() -> Result<PathBuf> {
        let mut conf_dir = get_xdg_config_home().context("could not get XDG_CONFIG_HOME")?;
        conf_dir.push("zfs-undelete.conf");
        Ok(conf_dir)
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

fn get_xdg_config_home() -> Result<PathBuf> {
    if let Ok(s) = env::var("XDG_CONFIG_HOME") {
        Ok(s.into())
    } else {
        let mut home = get_home_dir()?;
        home.push(".config");
        Ok(home)
    }
}

/// Get the home-directory of the current user by looking up the `$HOME` environment variable.
fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("$HOME not declared")?.into())
}
