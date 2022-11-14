mod args;
mod misc;
mod path;
mod ui;
mod undelete;
mod zfs;

use anyhow::{bail, Result};

use crate::path::Absolute;

fn main() -> Result<()> {
    let to_recover = args::get_filename_from_args()?;
    if to_recover.exists() {
        bail!("Cannot restore already existing file.");
    }

    let to_recover_absolute = to_recover.make_absolute()?;

    let dataset = zfs::Dataset::find(&to_recover_absolute)?;
    let to_recover_relative_to_mountpoint = dataset.get_relative_path(&to_recover_absolute);

    let full_path_in_snapshot =
        path::find_newest_snapshot_containing_the_file(dataset, to_recover_relative_to_mountpoint)?;

    println!("found file:\n{full_path_in_snapshot:?}");

    if ui::user_wants_to_continue()? {
        undelete::restore_file_from_snapshot(&full_path_in_snapshot, &to_recover_absolute)?;
    }

    Ok(())
}
