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
    pub(crate) fn load() -> Result<Self> {
        let conf_file = Self::get_config_file().context("could not find config file")?;
        let content = fs::read_to_string(&conf_file)
            .with_context(|| format!("reading config file {conf_file:?}"))?;

        let mut this = Self::default();

        for line in content.lines() {
            let mut split = line.split('=');
            if split.next().unwrap().trim() == "LsCommand" {
                this.ls_command = split.next().unwrap().trim().to_owned();
            }
        }

        this.sanity_checked()
    }

    fn get_config_file() -> Result<PathBuf> {
        let mut conf_dir = get_xdg_config_home().context("could not get XDG_CONFIG_HOME")?;
        conf_dir.push("zfs-undelete.conf");
        Ok(conf_dir)
    }

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

fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("$HOME not declared")?.into())
}
