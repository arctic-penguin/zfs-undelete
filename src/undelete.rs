use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub(crate) fn restore_file_from_snapshot(source: &Path, target: &Path) -> Result<()> {
    let source_str = path_to_str(source)?;
    let target_str = path_to_str(target)?;

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

fn path_to_str(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("could not convert path to str: {path:?}"))
}

pub(crate) fn ask_user_confirmation() -> Result<bool> {
    print!("Restore file? [y/N] ");
    io::stdout()
        .flush()
        .with_context(|| "could not flush stdout")?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .with_context(|| "could not read line from stdin")?;
    buf = buf.to_lowercase();
    Ok(buf.contains('y'))
}
