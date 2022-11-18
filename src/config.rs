use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result};

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) ls_command: String,
}

impl Config {
    pub(crate) fn load() -> Result<Self> {
        let conf_file = Self::get_config_file().context("could not find config file")?;
        let content = fs::read_to_string(&conf_file)
            .with_context(|| format!("reading config file {conf_file:?}"))?;
        let ls_command = content.trim().to_owned();
        Ok(Self { ls_command })
    }

    fn get_config_file() -> Result<PathBuf> {
        let mut conf_dir = get_xdg_config_home().context("could not get XDG_CONFIG_HOME")?;
        conf_dir.push("zfs-undelete.conf");
        Ok(conf_dir)
    }
}

fn get_xdg_config_home() -> Result<PathBuf> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(s) => Ok(s.into()),
        _ => {
            let mut home = get_home_dir()?;
            home.push(".config");
            Ok(home)
        }
    }
}

fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("$HOME not declared")?.into())
}

impl Default for Config {
    fn default() -> Self {
        println!("warning: no config file found, falling back to default config");
        Self {
            ls_command: "ls".to_owned(),
        }
    }
}
