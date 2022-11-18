use std::path::PathBuf;
use std::{env, fs};

use anyhow::Result;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) ls_command: String,
}

impl Config {
    pub(crate) fn load() -> Result<Self> {
        let conf_file = Self::get_config_file()?;
        let ls_command = fs::read_to_string(conf_file)?;
        Ok(Self { ls_command })
    }

    fn get_config_file() -> Result<PathBuf> {
        let mut conf_dir = get_xdg_config_home()?;
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
    Ok(env::var("HOME")?.into())
}
