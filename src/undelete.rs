use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use path_absolutize::*;

pub(crate) fn restore_file_from_snapshot(
    relative_filename: &Path,
    absolute_file_in_snapshot: &Path,
) {
    let mut command = Command::new("cp");
    command.args([
        "-a",
        absolute_file_in_snapshot.to_str().unwrap(),
        relative_filename.to_str().unwrap(),
    ]);
    dbg!(&command);
    command.output().unwrap();
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
pub(crate) fn find_mountpoint(path: &Path) -> PathBuf {
    let filepath = path.absolutize().unwrap().to_path_buf();
    for parent in filepath.ancestors() {
        if is_zfs_dataset(parent) {
            return parent.to_owned();
        }
    }
    panic!()
}

/// check if a path is a zfs mountpoint using findmnt
fn is_zfs_dataset(path: &Path) -> bool {
    match Command::new("findmnt")
        .args(["--noheadings", path.to_str().unwrap()])
        .output()
    {
        Ok(output) => {
            String::from_utf8(output.stdout).unwrap().contains("zfs") && output.status.success()
        }
        _ => panic!(),
    }
}

pub(crate) fn ask_user_confirmation() -> bool {
    print!("Restore file? [y/N] ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf = buf.to_lowercase();
    buf.contains('y')
}

/// get all snapshots for a zfs mountpoint
pub(crate) fn get_snapshots(path: &Path) -> Vec<PathBuf> {
    let mut path: PathBuf = path.into();
    path.push(".zfs");
    path.push("snapshot");
    let mut result: Vec<_> = path
        .read_dir()
        .unwrap()
        .into_iter()
        .map(|i| i.unwrap().path())
        .collect();
    result.sort_unstable();
    result
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
