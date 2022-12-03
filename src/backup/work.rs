use crate::config;
use crate::config::yaml::BackupPath;
use crate::fs::{self};
use crate::provider;
use anyhow::{Error, Result};
use std::path::PathBuf;

pub(super) struct WorkerResult(pub usize, pub Option<Error>);

pub(super) fn process(backup: &BackupPath, relative: PathBuf) -> Result<()> {
    let from_is_file = backup.from.is_file();

    let (from, relative_to) = if from_is_file {
        (backup.from.clone(), backup.to.clone())
    } else {
        (backup.from.join(&relative), backup.to.join(&relative))
    };

    let provider_id = &config::get().yaml.backup.provider;
    let to = provider::path(provider_id, &relative_to)?;

    fs::copy(&from, &to)?;

    Ok(())
}
