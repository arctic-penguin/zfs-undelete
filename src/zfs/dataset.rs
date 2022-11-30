use std::path::Path;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use itertools::Itertools;
use path_absolutize::Absolutize;

use super::misc::get_zfs_list_output;
use super::snapshot::Snapshot;

#[derive(Debug)]
pub(crate) struct Dataset {
    pub(crate) path: PathBuf,
    snapshots: Vec<Snapshot>,
}

impl Dataset {
    fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            snapshots: Self::get_snapshots(path.clone())
                .with_context(|| format!("could not get snapshots for dataset under {path:?}"))?,
            path,
        })
    }

    /// Traverse the absolute path from the child to root, return the first zfs mountpoint and path
    /// relative to the dataset.
    pub(crate) fn find(path: &Path) -> Result<(Self, PathBuf)> {
        let instance = Self::find_dataset(path)?;
        let path = instance.get_relative_path(path)?;
        Ok((instance, path))
    }

    fn find_dataset(path: &Path) -> Result<Self> {
        let filepath = path
            .absolutize()
            .with_context(|| format!("could not resolve filepath {path:?}"))?
            .to_path_buf();

        let mounted_datasets = get_mounted_datasets(&get_zfs_list_output()?);

        for parent in filepath.ancestors() {
            if is_zfs_dataset(parent, &mounted_datasets) {
                return parent.to_owned().try_into();
            }
        }

        bail!("file does not reside under any ZFS dataset")
    }

    pub(crate) fn get_relative_path(&self, path: &Path) -> Result<PathBuf> {
        let mut iterator = path.iter();

        // order is important! the iterator that is exhausted earlier must be the first iterator to
        // be zipped
        for (e1, e2) in self.path.iter().zip(&mut iterator) {
            if e1 != e2 {
                bail!("paths are not related")
            }
        }

        Ok(iterator.collect())
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
        assert!(!file.is_absolute(), "path must be relative, not absolute");

        let full_path_in_snapshot = self
            .snapshots()
            .iter()
            .rev()
            .find_map(|snap| snap.contains_file(file))
            .ok_or_else(|| anyhow!("file does not exist in any snapshot"))?;
        Ok(full_path_in_snapshot)
    }

    pub(crate) fn get_absolute_path(&self, path: &Path) -> PathBuf {
        self.path.join(path)
    }

    /// Get unique versions of the file using `st_mtime` and `st_size`. Output is sorted in reverse
    /// alphabetical order.
    pub(crate) fn get_unique_versions(&self, to_recover: &Path) -> Result<Vec<&Snapshot>> {
        let result: Vec<_> = self
            .snapshots
            .iter()
            .filter_map(|s| {
                s.get_file_information(to_recover)
                    .ok()
                    .map(|info| (s, info))
            })
            .unique_by(|(_, f)| (f.mtime, f.size))
            .map(|(s, _)| s)
            .rev()
            .collect();

        if result.is_empty() {
            bail!("file does not exist in any snapshot")
        }

        Ok(result)
    }
}

/// check if a path is a zfs mountpoint
fn is_zfs_dataset(path: &Path, datasets: &[PathBuf]) -> bool {
    datasets.iter().any(|d| d == path)
}

/// Get a Vec of paths of all currently mounted zfs datasets. The argument must match the structure
/// laid out in `get_zfs_list_output`.
fn get_mounted_datasets(output: &str) -> Vec<PathBuf> {
    let result = output
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
    result
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
        assert_eq!(dataset.get_relative_path(&all).unwrap(), result);

        let all = PathBuf::from("/a/b/c");
        let dataset: Dataset = PathBuf::from("/").try_into().unwrap();
        let result = PathBuf::from("a/b/c");
        assert_eq!(dataset.get_relative_path(&all).unwrap(), result);

        let all = PathBuf::from("/a/b/c");
        let dataset: Dataset = PathBuf::from("/a/b").try_into().unwrap();
        let result = PathBuf::from("c");
        assert_eq!(dataset.get_relative_path(&all).unwrap(), result);
    }
}
