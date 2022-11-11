use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use path_absolutize::Absolutize;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub(crate) struct Snapshot {
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Mountpoint {
    path: PathBuf,
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

impl Mountpoint {
    /// iterate the path from the child to root, return the first zfs mountpoint
    pub(crate) fn find(path: &Path) -> Result<Self> {
        let filepath = path
            .absolutize()
            .with_context(|| format!("could not resolve filepath {path:?}"))?
            .to_path_buf();
        for parent in filepath.ancestors() {
            if is_zfs_dataset(parent)? {
                return Ok(parent.to_owned().into());
            }
        }
        bail!("file does not reside under any ZFS dataset")
    }

    pub(crate) fn get_relative_path(&self, path: &Path) -> PathBuf {
        path.iter().skip(self.path.ancestors().count()).collect()
    }

    fn get_pathbuf(&self) -> PathBuf {
        self.path.to_owned()
    }

    /// get snapshots in alphabetically ascending order
    pub(crate) fn get_snapshots(&self) -> Result<Vec<Snapshot>> {
        let mut path: PathBuf = self.get_pathbuf();
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

impl From<PathBuf> for Mountpoint {
    fn from(path: PathBuf) -> Self {
        Self { path }
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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::Mountpoint;

    #[test]
    fn make_path_relative() {
        let all = PathBuf::from("/a/b/c");
        let mountpoint = Mountpoint::from(PathBuf::from("/a"));
        let result = PathBuf::from("b/c");
        assert_eq!(mountpoint.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = Mountpoint::from(PathBuf::from("/"));
        let result = PathBuf::from("a/b/c");
        assert_eq!(mountpoint.get_relative_path(&all), result);

        let all = PathBuf::from("/a/b/c");
        let mountpoint = Mountpoint::from(PathBuf::from("/a/b"));
        let result = PathBuf::from("c");
        assert_eq!(mountpoint.get_relative_path(&all), result);
    }
}
