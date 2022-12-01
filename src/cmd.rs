use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::misc::ToStr;

pub(crate) fn copy(source: &Path, target: &Path) -> Result<()> {
    let source_str = source.to_str_anyhow()?;
    let target_str = target.to_str_anyhow()?;

    if Command::new("cp")
        .args(["-a", source_str, target_str])
        .status()
        .context("error running `cp`")?
        .success()
    {
        Ok(())
    } else {
        bail!("error during execution of `cp`")
    }
}

pub(crate) fn ls(file: &Path, ls_command: &str, ls_args: &[String]) -> Result<()> {
    let workdir = file
        .parent()
        .context("file must have a parent")?
        .to_str_anyhow()?
        .to_owned();

    let file = file
        .file_name()
        .context("get filename from path")?
        .to_str()
        .context("OS-string to string conversion")?;

    if Command::new(ls_command)
        .args(ls_args)
        .arg(file)
        .current_dir(workdir)
        .status()
        .with_context(|| format!("running ls command '{ls_command}'"))?
        .success()
    {
        Ok(())
    } else {
        bail!("execution of 'ls' command")
    }
}
