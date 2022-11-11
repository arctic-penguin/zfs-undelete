mod undelete;
mod zfs;

use std::env::args;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
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

    let mut full_path_in_snapshot = None;
    // reverse alphabetical order means newest to oldest
    for snap in snapshots.iter().rev() {
        if let Some(path) = snap.contains_file(&relative_filename) {
            full_path_in_snapshot = Some(path);
            break;
        }
    }
    let to_copy = match full_path_in_snapshot {
        Some(path) => path,
        _ => bail!("file does not exist in any snapshot"),
    };

    println!("found file here:\n{to_copy:?}");
    if undelete::ask_user_confirmation()? {
        undelete::restore_file_from_snapshot(&to_copy, &filename)?;
    }
    Ok(())
}
