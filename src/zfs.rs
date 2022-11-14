use crate::misc::ToStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use path_absolutize::Absolutize;

trait Zfs {
    fn to_dataset(self) -> Result<Dataset>;
}

impl Zfs for PathBuf {
    fn to_dataset(self) -> Result<Dataset> {
        Dataset::new(self)
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub(crate) struct Snapshot {
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Dataset {
    path: PathBuf,
    snapshots: Vec<Snapshot>,
}

impl Snapshot {
    /// check if the file is contained in the snapshot. Return its full path if found.
    pub(crate) fn contains_file(&self, path: &Path) -> Option<PathBuf> {
        let buf = self.path.clone();
        let actual = buf.join(path);
        if actual.exists() {
            Some(actual)
        } else {
            None
        }
    }
}

impl Dataset {
    fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            snapshots: Self::get_snapshots(path.clone())
                .with_context(|| format!("could not get snapshots for dataset under {path:?}"))?,
            path,
        })
    }

    /// iterate the path from the child to root, return the first zfs mountpoint
    pub(crate) fn find(path: &Path) -> Result<Self> {
        let filepath = path
            .absolutize()
            .with_context(|| format!("could not resolve filepath {path:?}"))?
            .to_path_buf();
        for parent in filepath.ancestors() {
            if is_zfs_dataset(parent)? {
                return parent.to_owned().to_dataset();
            }
        }
        bail!("file does not reside under any ZFS dataset")
    }

    pub(crate) fn get_relative_path(&self, path: &Path) -> PathBuf {
        path.iter().skip(self.path.ancestors().count()).collect()
    }

    pub(crate) fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    /// get snapshots in alphabetically ascending order
    fn get_snapshots(mut path: PathBuf) -> Result<Vec<Snapshot>> {
        let subdir = PathBuf::from(".zfs/snapshot");
        path.push(subdir);
        let mut errors = vec![];
        let mut result: Vec<_> = path
            .read_dir()
            .with_context(|| format!("could not read zfs snapshot dir `{path:?}`"))?
            .into_iter()
            .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
            .map(|i| i.path())
            .map(Snapshot::from)
            .collect();

        if !errors.is_empty() {
            bail!("aggregation of snapshots failed, {:?}", errors);
        }

        result.sort_unstable();
        Ok(result)
    }
}

impl From<PathBuf> for Snapshot {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

/// check if a path is a zfs mountpoint using findmnt
fn is_zfs_dataset(path: &Path) -> Result<bool> {
    match Command::new("findmnt")
        .args(["--noheadings", path.to_str_anyhow()?])
        .output()
        .with_context(|| "failed to run `findmnt`")
    {
        Ok(output) => Ok(String::from_utf8(output.stdout)
            .with_context(|| "zfs dataset name contains invalid UTF8")?
            .contains("zfs")
            && output.status.success()),
        Err(e) => bail!("could not determine if {path:?} is a zfs dataset\n{e:?}"),
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::Zfs;

    #[test]
    fn make_path_relative() {
        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/a").to_dataset().unwrap();
        let result = PathBuf::from("b/c");
        assert_eq!(mountpoint.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/").to_dataset().unwrap();
        let result = PathBuf::from("a/b/c");
        assert_eq!(mountpoint.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = PathBuf::from("/a/b").to_dataset().unwrap();
        let result = PathBuf::from("c");
        assert_eq!(mountpoint.get_relative_path(&all), result);
    }
}
