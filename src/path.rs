use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use path_absolutize::Absolutize;

use crate::zfs;

pub(crate) trait Absolute {
    /// Turn a relative into an absolute path, providing anyhow::Result<PathBuf>.
    fn make_absolute(&self) -> Result<PathBuf>;
}

impl Absolute for PathBuf {
    fn make_absolute(&self) -> Result<PathBuf> {
        Ok(self
            .absolutize()
            .with_context(|| "could not resolve absolute path of file")?
            .to_path_buf())
    }
}

pub(crate) fn find_newest_snapshot_containing_the_file(
    mountpoint: zfs::Mountpoint,
    to_recover_relative_to_mountpoint: std::path::PathBuf,
) -> Result<std::path::PathBuf> {
    let full_path_in_snapshot = mountpoint
        .get_snapshots()?
        .iter()
        .rev()
        .find_map(|snap| snap.contains_file(&to_recover_relative_to_mountpoint))
        .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;
    Ok(full_path_in_snapshot)
}
