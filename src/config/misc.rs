use std::{env, path::PathBuf};

use anyhow::{Context, Result};

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
pub fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("$HOME not declared")?.into())
}

/// Get the path of the config file like. This is `$XDG_CONFIG_HOME/zfs-undelete.conf`.
pub(super) fn get_config_file() -> Result<PathBuf> {
    let mut conf_dir = get_xdg_config_home().context("could not get XDG_CONFIG_HOME")?;
    conf_dir.push("zfs-undelete.conf");
    Ok(conf_dir)
}
