use std::path::Path;

use anyhow::{Context, Result};

pub(crate) trait ToStr {
    fn to_str_anyhow(&self) -> Result<&str>;
}

impl ToStr for &Path {
    fn to_str_anyhow(&self) -> Result<&str> {
        self.to_str().context("could not convert path to string")
    }
}
