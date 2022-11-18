use std::io::{self, Write};

use anyhow::{Context, Result};

pub(crate) fn user_wants_to_continue() -> Result<bool> {
    print!("Restore? [y/N] ");
    io::stdout().flush().context("could not flush stdout")?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("could not read line from stdin")?;
    buf = buf.to_lowercase();
    Ok(buf.contains('y'))
}

pub(crate) fn ask_user_for_version(num_items: usize) -> Result<usize> {
    print!("choose [0-{}]: ", num_items - 1);

    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let choice = buf.trim().parse()?;
    Ok(choice)
}
