use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub(crate) fn restore_file_from_snapshot(source: &Path, target: &Path) -> Result<()> {
    let mut command = Command::new("cp");
    command.args([
        "-a",
        source
            .to_str()
            .with_context(|| format!("could not convert path to string: {source:?}"))?,
        target
            .to_str()
            .with_context(|| format!("could not convert path to string: {target:?}"))?,
    ]);
    dbg!(&command);
    if !command
        .status()
        .with_context(|| "error running `cp`")?
        .success()
    {
        bail!("error while during execution of `cp`")
    }
    Ok(())
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
