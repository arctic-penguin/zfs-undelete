use std::path::PathBuf;

use anyhow::{Context, Result};
use path_absolutize::Absolutize;

pub(crate) trait Absolute {
    /// Turn a relative into an absolute path, providing `anyhow::Result<PathBuf>`.
    fn make_absolute(&self) -> Result<PathBuf>;
}

impl Absolute for PathBuf {
    fn make_absolute(&self) -> Result<PathBuf> {
        Ok(self
            .absolutize()
            .context("could not resolve absolute path of file")?
            .to_path_buf())
    }
}
