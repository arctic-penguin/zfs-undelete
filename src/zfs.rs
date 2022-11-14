use crate::misc::ToStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use path_absolutize::Absolutize;

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
    /// Check if the file is contained in the snapshot. Return its full path if found.
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
                return parent.to_owned().try_into();
            }
        }
        bail!("file does not reside under any ZFS dataset")
    }

    pub(crate) fn get_relative_path(&self, path: &Path) -> PathBuf {
        path.iter().skip(self.path.ancestors().count()).collect()
    }

    /// Get a slice with references to all Snapshots under the Dataset.
    pub(crate) fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    /// Get snapshots in alphabetically ascending order.
    #[cfg(not(test))]
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

    /// Mock the function to enable tests on Dataset.
    #[cfg(test)]
    fn get_snapshots(_path: PathBuf) -> Result<Vec<Snapshot>> {
        Ok(vec![])
    }

    pub(crate) fn find_newest_snapshot_containing_the_file(
        &self,
        file: std::path::PathBuf,
    ) -> Result<std::path::PathBuf> {
        if file.is_absolute() {
            bail!("path must be relative, not absolute")
        }

        let full_path_in_snapshot = self
            .snapshots()
            .iter()
            .rev()
            .find_map(|snap| snap.contains_file(&file))
            .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;
        Ok(full_path_in_snapshot)
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

impl TryFrom<PathBuf> for Dataset {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        Self::new(path)
    }
}

#[cfg(test)]
mod test {
    use super::Dataset;
    use std::path::PathBuf;

    #[test]
    fn make_path_relative() {
        let all = PathBuf::from("/a/b/c");
        let dataset: Dataset = PathBuf::from("/a").try_into().unwrap();
        let result = PathBuf::from("b/c");
        assert_eq!(dataset.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let dataset: Dataset = PathBuf::from("/").try_into().unwrap();
        let result = PathBuf::from("a/b/c");
        assert_eq!(dataset.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let dataset: Dataset = PathBuf::from("/a/b").try_into().unwrap();
        let result = PathBuf::from("c");
        assert_eq!(dataset.get_relative_path(&all), result);
    }
}
