use std::path::PathBuf;

use crate::fs;
use crate::provider;
use crate::provider::ProviderId;
use crate::rclone;
use crate::telegram;
use anyhow::Result;
use walkdir::WalkDir;

fn push_and_rm_rclone(provider_id: &ProviderId, remote_path: Option<PathBuf>) -> Result<()> {
    let from = &provider::get(provider_id)?.buffer;

    let mut found_file = false;
    for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
        let is_file = entry.metadata().map(|m| m.is_file()).unwrap_or(false);
        if is_file {
            found_file = true;
            break;
        }
    }

    if !found_file {
        return Ok(());
    }

    let cmd = rclone::push(from, provider_id, remote_path)?;
    dbg!(&cmd);

    fs::remove_dir_all(from)?;

    Ok(())
}

pub fn push_and_rm(provider_id: &ProviderId, remote_path: Option<PathBuf>) -> Result<()> {
    if provider_id == telegram::PROVIDER_ID {
        if remote_path.is_some() {
            return Err(anyhow!("No support for remote_path with Telegram"));
        }
        telegram::upload_buffer()
    } else {
        push_and_rm_rclone(provider_id, remote_path)
    }
}
