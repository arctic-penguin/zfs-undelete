mod args;
mod misc;
mod path;
mod ui;
mod undelete;
mod zfs;

use anyhow::{bail, Result};
use path_absolutize::Absolutize;

fn main() -> Result<()> {
    let arguments = args::Arguments::get_args()?;
    if arguments.filename.exists() {
        bail!("Cannot restore already existing file.");
    }

    let to_recover_absolute = arguments.filename.absolutize()?;

    let dataset = zfs::Dataset::find(&to_recover_absolute)?;
    let to_recover_relative_to_mountpoint = dataset.get_relative_path(&to_recover_absolute);

    match arguments.mode {
        args::Mode::MostRecentVersion => {
            undelete::restore_most_recent_version(dataset, to_recover_relative_to_mountpoint)
        }
        args::Mode::AllVersions => {
            let to_restore =
                undelete::choose_version_to_restore(&dataset, &to_recover_relative_to_mountpoint)?;
            undelete::restore_specific_version(
                dataset,
                to_recover_relative_to_mountpoint,
                to_restore,
            )
        }
    }
}
