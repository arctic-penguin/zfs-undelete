use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use path_absolutize::*;

pub(crate) fn restore_file_from_snapshot(
    relative_filename: &Path,
    absolute_file_in_snapshot: &Path,
) -> Result<()> {
    let mut command = Command::new("cp");
    command.args([
        "-a",
        absolute_file_in_snapshot.to_str().with_context(|| {
            format!("could not convert path to string: {absolute_file_in_snapshot:?}")
        })?,
        relative_filename
            .to_str()
            .with_context(|| format!("could not convert path to string: {relative_filename:?}"))?,
    ]);
    dbg!(&command);
    if !command
        .status()
        .with_context(|| "error running `cp`")?
        .success()
    {
        bail!("error while during execution of `cp`")
    }
    Ok(())
}

pub(crate) fn mountpoint_contains_file(mountpoint: &Path, filename: &Path) -> bool {
    let buf = PathBuf::from(mountpoint);
    let actual = buf.join(filename);
    actual.exists()
}

pub(crate) fn get_path_relative_to_mountpoint(path: &Path, mountpoint: &Path) -> PathBuf {
    path.iter()
        .skip(dbg!(mountpoint.ancestors().count()))
        .collect()
}

/// iterate the path from the child to root, return the first zfs mountpoint
pub(crate) fn find_mountpoint(path: &Path) -> Result<PathBuf> {
    let filepath = path
        .absolutize()
        .with_context(|| format!("could not resolve filepath {path:?}"))?
        .to_path_buf();
    for parent in filepath.ancestors() {
        if is_zfs_dataset(parent)? {
            return Ok(parent.to_owned());
        }
    }
    bail!("file does not reside under any ZFS dataset")
}

/// check if a path is a zfs mountpoint using findmnt
fn is_zfs_dataset(path: &Path) -> Result<bool> {
    match Command::new("findmnt")
        .args([
            "--noheadings",
            path.to_str()
                .with_context(|| "could not convert path to string")?,
        ])
        .output()
    {
        Ok(output) => Ok(String::from_utf8(output.stdout)
            .with_context(|| "zfs dataset name contains invalid UTF8")?
            .contains("zfs")
            && output.status.success()),
        _ => bail!("could not determine if {path:?} is a zfs dataset"),
    }
}

pub(crate) fn ask_user_confirmation() -> Result<bool> {
    print!("Restore file? [y/N] ");
    io::stdout()
        .flush()
        .with_context(|| "could not flush stdout")?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .with_context(|| "could not read line from stdin")?;
    buf = buf.to_lowercase();
    Ok(buf.contains('y'))
}

/// get all snapshots for a zfs mountpoint
pub(crate) fn get_snapshots(path: &Path) -> Result<Vec<PathBuf>> {
    let mut path: PathBuf = path.into();
    path.push(".zfs");
    path.push("snapshot");
    let mut errors = vec![];
    let mut result: Vec<_> = path
        .read_dir()
        .with_context(|| format!("could not read zfs snapshot dir `{path:?}`"))?
        .into_iter()
        .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
        .map(|i| i.path())
        .collect();
    result.sort_unstable();
    if !errors.is_empty() {
        bail!("aggregation of snapshots failed, {:?}", errors);
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::get_path_relative_to_mountpoint;

    #[test]
    fn make_path_relative() {
        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/a");
        let result = PathBuf::from("b/c");

        assert_eq!(get_path_relative_to_mountpoint(&all, &mountpoint), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/");
        let result = PathBuf::from("a/b/c");

        assert_eq!(get_path_relative_to_mountpoint(&all, &mountpoint), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/a/b");
        let result = PathBuf::from("c");

        assert_eq!(get_path_relative_to_mountpoint(&all, &mountpoint), result);
    }
}
