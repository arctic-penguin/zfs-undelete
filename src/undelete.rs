use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::cmd::copy;
use crate::ui;
use crate::zfs::{Dataset, FileInfo, Snapshot};

pub(crate) fn restore_most_recent_version(dataset: Dataset, to_recover: PathBuf) -> Result<()> {
    let full_path_in_snapshot = dataset.find_newest_snapshot_containing_the_file(&to_recover)?;

    println!("found file:\n{full_path_in_snapshot:?}");

    if ui::user_wants_to_continue()? {
        let full_path_in_dataset = dataset.get_absolute_path(&to_recover);
        copy(&full_path_in_snapshot, &full_path_in_dataset)?;
    }
    Ok(())
}

pub(crate) fn restore_interactively(dataset: &Dataset, to_recover: &Path) -> Result<PathBuf> {
    let unique_versions = dataset.get_unique_versions(to_recover)?;
    let version = choose_version(unique_versions)?;

    version
        .0
        .contains_file(to_recover)
        .ok_or_else(|| unreachable!("cannot happen"))
}

fn choose_version(unique_versions: Vec<(&Snapshot, FileInfo)>) -> Result<(&Snapshot, FileInfo)> {
    for (i, v) in unique_versions.iter().enumerate() {
        println!("{i}: {}, {}", v.0, v.1.size);
    }

    let choice = ui::ask_user_for_version(unique_versions.len())?;

    let version = unique_versions
        .into_iter()
        .nth(choice)
        .ok_or_else(|| anyhow!("invalid answer"))?;

    Ok(version)
}
pub(crate) fn restore_specific_version(
    dataset: Dataset,
    to_recover_relative_to_mountpoint: PathBuf,
    to_restore: PathBuf,
) -> Result<()> {
    let full_path_in_dataset = dataset.path.join(to_recover_relative_to_mountpoint);
    copy(&to_restore, &full_path_in_dataset)
}
