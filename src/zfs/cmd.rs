use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

/// Use `zfs list` to get the paths of all mounted datasets.
pub(super) fn get_mountpoints_of_mounted_datasets() -> Result<Vec<PathBuf>> {
    let string = run_list_command()?;
    let result = string
        .lines()
        .map(|l| l.split_terminator('\t').collect::<Vec<_>>())
        .filter(|split| {
            split
                .get(2)
                .expect("has a 'mounted' column")
                .contains("yes")
        })
        .map(|split| split.get(1).expect("has a 'mountpoint' column").into())
        .collect();
    Ok(result)
}

/// Return zfs datasets as String in the form `<name>\t<mountpoint>\t<mounted>`.
fn run_list_command() -> Result<String> {
    let output = Command::new("zfs")
        .args([
            "list",
            "-t", // only datasets
            "filesystem",
            "-H", // no header
            "-o", // only specific columns
            "name,mountpoint,mounted",
        ])
        .output()
        .context("could not get output of `zfs list`")?;

    let result = String::from_utf8(output.stdout).context("`zfs list` returned invalid UTF8")?;

    Ok(result)
}
