mod args;
mod cmd;
mod config;
mod misc;
mod mode;
mod path;
mod ui;
mod undelete;
mod zfs;

use anyhow::{bail, Context, Result};
use path_absolutize::Absolutize;
use undelete::Undelete;

fn main() -> Result<()> {
    let conf = config::Config::load().context("reading config file")?;
    let arguments = args::Arguments::get_args().context("processing arguments")?;
    if arguments.filename.exists() {
        bail!("Cannot restore already existing file.");
    }

    let to_recover_absolute = arguments.filename.absolutize()?;
    let (dataset, to_recover_relative_to_mountpoint) = zfs::Dataset::find(&to_recover_absolute)?;

    let undelete = Undelete::new(
        dataset,
        to_recover_relative_to_mountpoint,
        conf,
        arguments.mode,
    );

    undelete.run()
}
