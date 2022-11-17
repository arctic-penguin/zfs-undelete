use std::path::Path;
use std::process::Command;

use crate::misc::ToStr;
use anyhow::{bail, Context, Result};

pub(crate) fn copy(source: &Path, target: &Path) -> Result<()> {
    let source_str = source.to_str_anyhow()?;
    let target_str = target.to_str_anyhow()?;

    match Command::new("cp")
        .args(["-a", source_str, target_str])
        .status()
        .context("error running `cp`")?
        .success()
    {
        true => Ok(()),
        false => bail!("error during execution of `cp`"),
    }
}
