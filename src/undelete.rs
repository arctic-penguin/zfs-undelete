use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::cmd::{copy, ls};
use crate::config::Config;
use crate::mode::Mode;
use crate::ui::{self, user_wants_to_continue};
use crate::zfs::{Dataset, Snapshot};

#[derive(Debug)]
pub(crate) struct Undelete {
    dataset: Dataset,
    to_recover_relative_to_mountpoint: PathBuf,
    conf: Config,
    mode: Mode,
}

impl Undelete {
    pub(crate) fn new(
        dataset: Dataset,
        to_recover_relative_to_mountpoint: PathBuf,
        conf: Config,
        mode: Mode,
    ) -> Self {
        Self {
            dataset,
            to_recover_relative_to_mountpoint,
            conf,
            mode,
        }
    }

    pub(crate) fn run(&self) -> Result<()> {
        match self.mode {
            Mode::MostRecentVersion => self.restore_most_recent_version(),
            Mode::ChooseVersionInteractively => self.restore_interactively(),
        }
    }

    fn restore_most_recent_version(&self) -> Result<()> {
        let full_path_in_snapshot = self
            .dataset
            .find_newest_snapshot_containing_the_file(&self.to_recover_relative_to_mountpoint)?;

        println!("found file:\n  {}", full_path_in_snapshot.display());

        if ui::user_wants_to_continue()? {
            let full_path_in_dataset = self
                .dataset
                .get_absolute_path(&self.to_recover_relative_to_mountpoint);
            copy(&full_path_in_snapshot, &full_path_in_dataset)?;
        }
        Ok(())
    }

    fn restore_interactively(&self) -> Result<()> {
        let unique_versions = self
            .dataset
            .get_unique_versions(&self.to_recover_relative_to_mountpoint)?;
        self.show_enumerated_snapshots(&unique_versions)?;

        let snapshot;
        loop {
            match self.choose_version(&unique_versions) {
                Ok(snap) => {
                    snapshot = snap;
                    break;
                }
                Err(e) => println!("{e}"),
            }
        }

        let to_restore = snapshot.join(&self.to_recover_relative_to_mountpoint);

        self.restore_specific_version(&to_restore)
    }

    fn choose_version<'a>(&self, unique_versions: &Vec<&'a Snapshot>) -> Result<&'a Snapshot> {
        let length = unique_versions.len();

        let choice = if length == 1 {
            self.ask_restore_only_snapshot(unique_versions)?
        } else {
            let choice = self.ask_restore_snapshot_version(unique_versions)?;
            if choice >= length {
                bail!("invalid answer")
            }
            choice
        };

        let version = unique_versions
            .get(choice)
            .ok_or_else(|| anyhow!("this should not happen"))?;

        Ok(version)
    }

    fn ask_restore_snapshot_version(
        &self,
        unique_versions: &Vec<&Snapshot>,
    ) -> Result<usize, anyhow::Error> {
        ui::ask_user_for_version(unique_versions.len())
    }

    fn show_enumerated_snapshots(
        &self,
        unique_versions: &[&Snapshot],
    ) -> Result<(), anyhow::Error> {
        for (i, snap) in unique_versions.iter().enumerate() {
            print!("{i}: ");
            stdout().lock().flush()?;
            ls(
                &self.to_recover_relative_to_mountpoint,
                snap.path(),
                &self.conf.ls_command,
            )?;
        }
        Ok(())
    }

    fn ask_restore_only_snapshot(
        &self,
        unique_versions: &[&Snapshot],
    ) -> Result<usize, anyhow::Error> {
        let snapshot = &unique_versions.get(0).expect("contains one value");
        ls(
            &self.to_recover_relative_to_mountpoint,
            snapshot.path(),
            &self.conf.ls_command,
        )?;

        let result = if user_wants_to_continue()? {
            0
        } else {
            bail!("user does not want to restore");
        };

        Ok(result)
    }

    pub(crate) fn restore_specific_version(&self, to_restore: &Path) -> Result<()> {
        let full_path_in_dataset = self
            .dataset
            .path
            .join(&self.to_recover_relative_to_mountpoint);
        copy(to_restore, &full_path_in_dataset)
    }
}
