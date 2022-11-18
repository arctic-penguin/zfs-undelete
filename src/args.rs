use std::env::args;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::mode::Mode;

#[derive(Debug)]
pub(crate) struct Arguments {
    pub(crate) mode: Mode,
    pub(crate) filename: PathBuf,
}

impl Arguments {
    pub(crate) fn get_args() -> Result<Self> {
        let mut raw_args: Vec<_> = args().skip(1).rev().collect();
        if raw_args.is_empty() {
            bail!("Missing arguments");
        }
        let mode = Mode::get_from_args(&mut raw_args);
        let filename = raw_args.pop().context("filename missing")?.into();
        Ok(Self { mode, filename })
    }
}
