mod args;
mod cmd;
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
    let (dataset, to_recover_relative_to_mountpoint) = zfs::Dataset::find(&to_recover_absolute)?;

    match arguments.mode {
        args::Mode::MostRecentVersion => {
            undelete::restore_most_recent_version(dataset, to_recover_relative_to_mountpoint)
        }

        args::Mode::ChooseVersionInteractively => {
            undelete::restore_interactively(dataset, to_recover_relative_to_mountpoint)
        }
    }
}
