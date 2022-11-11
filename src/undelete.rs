use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::misc::ToStr;

pub(crate) fn restore_file_from_snapshot(source: &Path, target: &Path) -> Result<()> {
    let source_str = source.to_str_anyhow()?;
    let target_str = target.to_str_anyhow()?;

    match Command::new("cp")
        .args(["-a", source_str, target_str])
        .status()
        .with_context(|| "error running `cp`")?
        .success()
    {
        true => Ok(()),
        false => bail!("error during execution of `cp`"),
    }
}
