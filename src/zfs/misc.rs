use std::process::Command;

use anyhow::{bail, Context, Result};

/// ask zfs for name, mountpoint and mount-status of all datasets.
pub(in crate::zfs) fn get_zfs_list_output() -> Result<String> {
    match Command::new("zfs")
        .args([
            "list",
            "-t", // only datasets
            "filesystem",
            "-H", // no header
            "-o", // only specific columns
            "name,mountpoint,mounted",
        ])
        .output()
        .context("failed to run `zfs list`")
    {
        Ok(output) => {
            Ok(String::from_utf8(output.stdout).context("could not parse output of `zfs list`")?)
        }
        _ => bail!("something went wrong when running `zfs list`"),
    }
}
