use std::io::{stdout, Write};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::cmd::{copy, ls};
use crate::ui;
use crate::zfs::{Dataset, FileInfo, Snapshot};

pub(crate) fn restore_most_recent_version(dataset: &Dataset, to_recover: &Path) -> Result<()> {
    let full_path_in_snapshot = dataset.find_newest_snapshot_containing_the_file(to_recover)?;

    println!("found file:\n  {}", full_path_in_snapshot.display());

    if ui::user_wants_to_continue()? {
        let full_path_in_dataset = dataset.get_absolute_path(to_recover);
        copy(&full_path_in_snapshot, &full_path_in_dataset)?;
    }
    Ok(())
}

pub(crate) fn restore_interactively(
    dataset: &Dataset,
    to_recover_relative_to_mountpoint: &Path,
) -> Result<()> {
    let unique_versions = dataset.get_unique_versions(to_recover_relative_to_mountpoint)?;
    let snapshot = choose_version(unique_versions, to_recover_relative_to_mountpoint)?;

    let to_restore = snapshot.join(to_recover_relative_to_mountpoint);

    restore_specific_version(dataset, to_recover_relative_to_mountpoint, &to_restore)
}

fn choose_version<'a>(
    unique_versions: Vec<(&'a Snapshot, FileInfo)>,
    to_recover_relative_to_mountpoint: &Path,
) -> Result<&'a Snapshot> {
    for (i, (snap, _)) in unique_versions.iter().enumerate() {
        print!("{i}: ");
        stdout().lock().flush()?;
        ls(to_recover_relative_to_mountpoint, snap.path())?;
    }

    let choice = ui::ask_user_for_version(unique_versions.len())?;

    let version = unique_versions
        .into_iter()
        .nth(choice)
        .map(|(snap, _)| snap)
        .ok_or_else(|| anyhow!("invalid answer"))?;

    Ok(version)
}

pub(crate) fn restore_specific_version(
    dataset: &Dataset,
    to_recover_relative_to_mountpoint: &Path,
    to_restore: &Path,
) -> Result<()> {
    let full_path_in_dataset = dataset.path.join(to_recover_relative_to_mountpoint);
    copy(to_restore, &full_path_in_dataset)
}
