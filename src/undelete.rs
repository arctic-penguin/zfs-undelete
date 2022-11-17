use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::ui;
use crate::{misc::ToStr, zfs::Dataset};

fn restore_file_from_snapshot(source: &Path, target: &Path) -> Result<()> {
    let source_str = source.to_str_anyhow()?;
    let target_str = target.to_str_anyhow()?;

    match Command::new("cp")
        .args(["-a", source_str, target_str])
        .status()
        .context("error running `cp`")?
        .success()
    {
        true => Ok(()),
        false => bail!("error during execution of `cp`"),
    }
}

pub(crate) fn restore_most_recent_version(dataset: Dataset, to_recover: PathBuf) -> Result<()> {
    let full_path_in_snapshot = dataset.find_newest_snapshot_containing_the_file(&to_recover)?;

    println!("found file:\n{full_path_in_snapshot:?}");

    if ui::user_wants_to_continue()? {
        let full_path_in_dataset = dataset.get_absolute_path(&to_recover);
        restore_file_from_snapshot(&full_path_in_snapshot, &full_path_in_dataset)?;
    }
    Ok(())
}

pub(crate) fn choose_version_to_restore(dataset: &Dataset, to_recover: &Path) -> Result<PathBuf> {
    let unique_versions = dataset.get_unique_versions(to_recover)?;
    for (i, v) in unique_versions.iter().enumerate() {
        println!("{i}: {}, {}", v.0, v.1.size);
    }
    print!("choose [0-{}]: ", unique_versions.len() - 1);
    stdout().lock().flush()?;
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    let input = buf.trim();

    let choice: usize = input.parse()?;
    let version = unique_versions
        .get(choice)
        .ok_or_else(|| anyhow!("invalid answer"))?;

    version
        .0
        .contains_file(to_recover)
        .ok_or_else(|| anyhow!("cannot happen"))
}

pub(crate) fn restore_specific_version(
    dataset: Dataset,
    to_recover_relative_to_mountpoint: PathBuf,
    to_restore: PathBuf,
) -> Result<()> {
    let full_path_in_dataset = dataset.path.join(to_recover_relative_to_mountpoint);
    restore_file_from_snapshot(&to_restore, &full_path_in_dataset)
}
