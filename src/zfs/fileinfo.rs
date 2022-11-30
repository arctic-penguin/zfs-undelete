use std::fs::Metadata;
use std::time::SystemTime;

#[derive(Debug)]
pub(in crate::zfs) struct FileInfo {
    pub mtime: SystemTime,
    pub size: usize,
}

impl From<Metadata> for FileInfo {
    fn from(m: Metadata) -> Self {
        Self {
            mtime: m.modified().expect("should work on Linux"),
            size: m.len() as usize,
        }
    }
}
