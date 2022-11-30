use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use super::fileinfo::FileInfo;
use anyhow::{anyhow, Context, Result};

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub(crate) struct Snapshot {
    path: PathBuf,
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

impl Snapshot {
    /// Check if the file is contained in the snapshot. Return its full path if found.
    pub(crate) fn contains_file(&self, path: &Path) -> Option<PathBuf> {
        let actual = self.join(path);
        if actual.exists() {
            Some(actual)
        } else {
            None
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn join(&self, path: &Path) -> PathBuf {
        self.path.clone().join(path)
    }

    pub(super) fn get_file_information(&self, file: &Path) -> Result<FileInfo> {
        let file_absolute = self.path.join(file);
        let result = file_absolute
            .parent()
            .context("must have a parent")?
            .read_dir()
            .context("reading dir")?
            .find(|f| {
                f.as_ref()
                    .expect("we have permission to read the file")
                    .file_name()
                    == file_absolute
                        .file_name()
                        .expect("path ends in proper name, not '..'")
            })
            .ok_or_else(|| anyhow!("could not find file that should be there"))??
            .metadata()?
            .into();

        Ok(result)
    }
}

impl From<PathBuf> for Snapshot {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}
