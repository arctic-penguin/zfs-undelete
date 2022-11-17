use std::env::args;
use std::path::PathBuf;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct Arguments {
    pub(crate) mode: Mode,
    pub(crate) filename: PathBuf,
}

#[derive(Debug)]
pub(crate) enum Mode {
    MostRecentVersion,
    ChooseVersionInteractively,
}

impl Mode {
    fn get_from_args(args: &mut Vec<String>) -> Self {
        if args.last().expect("there's at least one element") == "-V" {
            args.pop().expect("there's at least one element");
            Self::ChooseVersionInteractively
        } else {
            Self::MostRecentVersion
        }
    }
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
