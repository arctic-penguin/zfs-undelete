use crate::misc::ToStr;
use std::fmt::Display;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use path_absolutize::Absolutize;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub(crate) struct Snapshot {
    path: PathBuf,
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

#[derive(Debug)]
pub(crate) struct Dataset {
    pub(crate) path: PathBuf,
    snapshots: Vec<Snapshot>,
}

#[derive(Debug)]
pub(crate) struct FileInfo {
    pub(crate) mtime: SystemTime,
    pub(crate) size: FileSize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct FileSize {
    value: usize,
}

impl From<u64> for FileSize {
    fn from(value: u64) -> Self {
        Self {
            value: value as usize,
        }
    }
}

impl FileSize {
    fn show(&self) -> String {
        if self.value < 1_000 {
            format!("{} B", self.value)
        } else if self.value < 1_000_000 {
            format!("{} kB", self.value)
        } else if self.value < 1_000_000_000 {
            format!("{} MB", self.value)
        } else if self.value < 1_000_000_000_000 {
            format!("{} GB", self.value)
        } else if self.value < 1_000_000_000_000_000 {
            format!("{} TB", self.value)
        } else {
            panic!("files larger than 1 Petabyte? Really?")
        }
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.show())
    }
}

impl From<Metadata> for FileInfo {
    fn from(m: Metadata) -> Self {
        Self {
            mtime: m.modified().expect("should work on Linux"),
            size: m.len().into(),
        }
    }
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

    fn get_file_information(&self, file: &Path) -> Option<FileInfo> {
        let file_absolute = self.path.join(file);
        let result = file_absolute
            .parent()
            .expect("must have a parent")
            .read_dir()
            .ok()?
            .find(|f| {
                f.as_ref()
                    .expect("we have permission to read the file")
                    .file_name()
                    == file_absolute
                        .file_name()
                        .expect("path ends in proper name, not '..'")
            })?
            .ok()?
            .metadata()
            .ok()?
            .into();

        Some(result)
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

    pub(crate) fn find_newest_snapshot_containing_the_file(&self, file: &Path) -> Result<PathBuf> {
        if file.is_absolute() {
            bail!("path must be relative, not absolute")
        }

        let full_path_in_snapshot = self
            .snapshots()
            .iter()
            .rev()
            .find_map(|snap| snap.contains_file(file))
            .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;
        Ok(full_path_in_snapshot)
    }

    pub(crate) fn get_absolute_path(&self, path: &PathBuf) -> PathBuf {
        self.path.join(path)
    }

    /// Get unique versions of the file using st_mtime and st_size. Output is sorted in reverse
    /// alphabetical order.
    pub(crate) fn get_unique_versions(
        &self,
        to_recover: &Path,
    ) -> Result<Vec<(&Snapshot, FileInfo)>> {
        let result: Vec<_> = self
            .snapshots
            .iter()
            .filter_map(|s| s.get_file_information(to_recover).map(|info| (s, info)))
            .unique_by(|(_, f)| (f.mtime, f.size))
            .rev()
            .collect();
        if result.is_empty() {
            bail!("file does not exist in any snapshot")
        }
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
