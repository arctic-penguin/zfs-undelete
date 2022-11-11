mod undelete;
mod zfs;

use std::env::args;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use path_absolutize::*;

const MSG: &str = "Currently refusing to work on existing files.
Delete an existing file if you want to restore it from a snapshot";

fn main() -> Result<()> {
    let to_recover: PathBuf = args().nth(1).with_context(|| "filename expected")?.into();
    let to_recover_absolute = to_recover
        .absolutize()
        .with_context(|| "could not resolve absolute path of file")?
        .to_path_buf();

    if to_recover_absolute.exists() {
        bail!(MSG);
    }

    let mountpoint = zfs::Mountpoint::find(&to_recover_absolute)?;

    let snapshots = mountpoint.get_snapshots()?;
    let to_recover_relative_to_mountpoint = mountpoint.get_relative_path(&to_recover_absolute);

    // reverse order means newest to oldest
    let full_path_in_snapshot = snapshots
        .iter()
        .rev()
        .find_map(|snap| snap.contains_file(&to_recover_relative_to_mountpoint))
        .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;

    println!("found file here:\n{full_path_in_snapshot:?}");
    if undelete::ask_user_confirmation()? {
        undelete::restore_file_from_snapshot(&full_path_in_snapshot, &to_recover_absolute)?;
    }

    Ok(())
}
