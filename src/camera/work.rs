use crate::compress::{self, Quality};
use crate::config::yaml::CameraBackupDef;
use crate::provider;
use crate::zip;
use crate::{config, fs};
use anyhow::{Error, Result};
use std::path::Path;

pub(super) struct WorkerResult(pub usize, pub Option<Error>);

pub(super) fn process(backup: &CameraBackupDef, relative: &Path) -> Result<()> {
    let password = config::get().crypto_password()?;

    let entry_path = backup.from.join(relative);
    let relative_with_base = &backup.to.join(relative);
    let relative_with_base_zipped = &zip::zipped_name(relative_with_base);

    let low_unzipped_path = provider::path(&backup.low_unzipped, relative)?; // TODO: make this configurable
    let low_zipped_path = provider::path(&backup.low_zipped, relative_with_base_zipped)?;
    let high_unzipped_path = provider::path(&backup.high_unzipped, relative_with_base)?;
    let high_zipped_path = provider::path(&backup.high_zipped, relative_with_base_zipped)?;

    compress::compress(&entry_path, &low_unzipped_path, Quality::Low)?;
    compress::compress(&entry_path, &high_unzipped_path, Quality::High)?;

    zip::create(password, &low_unzipped_path, &low_zipped_path)?;
    zip::create(password, &high_unzipped_path, &high_zipped_path)?;

    fs::remove_file(&high_unzipped_path)?;
    fs::mv(&entry_path, &high_unzipped_path)?;

    Ok(())
}
