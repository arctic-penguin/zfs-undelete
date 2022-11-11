use std::env::args;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

pub(crate) fn get_filename_from_args() -> Result<PathBuf> {
    Ok(args().nth(1).with_context(|| "filename expected")?.into())
}
