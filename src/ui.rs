use std::io::{self, Write};

use anyhow::{Context, Result};

pub(crate) fn user_wants_to_continue() -> Result<bool> {
    print!("Restore file? [y/N] ");
    io::stdout().flush().context("could not flush stdout")?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("could not read line from stdin")?;
    buf = buf.to_lowercase();
    Ok(buf.contains('y'))
}
