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

pub(crate) fn ls(file: &Path, workdir: &Path) -> Result<()> {
    let file = file.to_str_anyhow()?;
    let dir = workdir.to_str_anyhow()?;

    if Command::new("lsd")
        .args(["-dhl", file])
        .current_dir(dir)
        .status()
        .context("error running `lsd`")?
        .success()
    {
        Ok(())
    } else {
        bail!("error during execution of `lsd`")
    }
}
