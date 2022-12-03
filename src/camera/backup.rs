use std::path::PathBuf;

use super::skip::{self, SkipReason};
use crate::config;
use crate::format::{self};
use crate::fs::{self, IPathBuf};
use crate::log;
use crate::provider;
use crate::rclone;
use crate::smalldate;
use anyhow::{Context, Result};
use walkdir::WalkDir;

pub fn set_camera_buffer() -> Result<()> {
    let provider_id = &config::get().yaml.camera_backup.intermediate_provider;
    let remote_base = &config::get().yaml.camera_backup.intermediate_relative;
    let local_source = &config::get().yaml.camera_backup.local_source;
    let local_intermediate = &config::get().yaml.camera_backup.local_intermediate;

    let folder_id = smalldate::now_hours_base36()?;
    let target = provider::path(provider_id, remote_base)?;

    for entry in WalkDir::new(local_source).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let format = format::get_format(path);
        let reason = skip::should_process(&entry, &format);

        match reason {
            SkipReason::NoSkip => {}
            SkipReason::Directory => {
                continue;
            }
            _ => {
                log::debug(&format!("File skipped: {:?}, {}", reason, path.to_string()));
                continue;
            }
        }

        let filename = path.file_name().context("file without filename")?;
        let parent = path.parent().context("file without parent")?;
        let middle_path = if parent == local_source {
            PathBuf::from(&folder_id)
        } else {
            parent.strip_prefix(local_source)?.to_owned()
        };

        let intermediate_path = {
            let mut p = local_intermediate.clone();
            p.push(&middle_path);
            p.push(filename);
            p
        };

        let backup_path = {
            let mut p = target.clone();
            p.push(&middle_path);
            p.push(filename);
            p
        };

        fs::mv(path, &intermediate_path)?;
        fs::copy(&intermediate_path, &backup_path)?;
    }

    Ok(())
}

pub fn remove_backed_pictures() -> Result<()> {
    let ref_provider_id = &config::get().yaml.camera_backup.ref_provider;
    let local_intermediate = config::get().yaml.camera_backup.local_intermediate.clone();

    for entry in WalkDir::new(local_intermediate)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }

        let path = entry.path();
        let folder_name = path
            .file_name()
            .context("directory without name")?
            .to_string_lossy()
            .to_string();

        let already_uploaded = rclone::has_prefixed_files(ref_provider_id, None, &folder_name)?;

        if already_uploaded {
            fs::remove_dir_all(path)?;
        } else {
            log::info(&format!("{} doesn't seem to be uploaded", &folder_name));
        }
    }

    Ok(())
}
