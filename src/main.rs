mod undelete;
mod zfs;

use std::env::args;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use path_absolutize::*;

fn main() -> Result<()> {
    let mut filename: PathBuf = args().nth(1).with_context(|| "filename expected")?.into();
    filename = filename
        .absolutize()
        .with_context(|| "could not resolve absolute path of file")?
        .to_path_buf();

    if filename.exists() {
        bail!("currently refusing to work on existing files. delete an existing file if you want to restore it from a snapshot");
    }

    let mountpoint = zfs::Mountpoint::find(&filename)?;

    let snapshots = mountpoint.get_snapshots()?;
    let relative_filename = mountpoint.get_relative_path(&filename);

    // reverse order means newest to oldest
    let full_path_in_snapshot = snapshots
        .iter()
        .rev()
        .find_map(|snap| snap.contains_file(&relative_filename))
        .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;

    println!("found file here:\n{full_path_in_snapshot:?}");
    if undelete::ask_user_confirmation()? {
        undelete::restore_file_from_snapshot(&full_path_in_snapshot, &filename)?;
    }
    Ok(())
}
