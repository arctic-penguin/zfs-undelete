mod undelete;

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

    let mountpoint = dbg!(undelete::find_mountpoint(&filename)?);

    let mut snapshots = undelete::get_snapshots(&mountpoint)?;
    let relative_filename = undelete::get_path_relative_to_mountpoint(dbg!(&filename), &mountpoint);

    let mut best_snapshot = None;
    // reverse alphabetical order means newest to oldest
    for snap in snapshots.iter_mut().rev() {
        if undelete::mountpoint_contains_file(snap, &relative_filename) {
            best_snapshot = Some(snap);
            break;
        }
    }
    let absolute_file_in_snapshot = best_snapshot.with_context(|| "file not found in snapshots")?;
    absolute_file_in_snapshot.push(dbg!(relative_filename));
    println!("found file here:\n{absolute_file_in_snapshot:?}");
    if undelete::ask_user_confirmation()? {
        undelete::restore_file_from_snapshot(&filename, absolute_file_in_snapshot)?;
    }
    Ok(())
}
