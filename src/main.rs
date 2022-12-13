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
use undelete::Undelete;

fn main() -> Result<()> {
    let conf = config::Config::load().context("loading config")?;
    let arguments = args::Arguments::get_args().context("processing arguments")?;
    if arguments.filename.exists() {
        bail!("Cannot restore already existing file.");
    }

    let (dataset, to_recover_relative_to_mountpoint) = zfs::Dataset::find(&arguments.filename)?;

    let undelete = Undelete::new(
        dataset,
        to_recover_relative_to_mountpoint,
        conf,
        arguments.mode,
    );

    undelete.run()
}
