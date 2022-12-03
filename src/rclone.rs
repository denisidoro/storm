use crate::fs;
use crate::fs::IPathBuf;
use crate::provider;
use crate::provider::ProviderId;
use crate::shell::{self, ShellCmd};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn get_rclone_id(provider_id: &ProviderId) -> Result<&String> {
    provider::get(provider_id)?
        .rclone
        .as_ref()
        .context("rclone not supported")
}

pub fn pull_many(provider_id: &ProviderId, remote_paths: &[PathBuf], local: &Path) -> Result<ShellCmd> {
    let rclone_id = get_rclone_id(provider_id)?;

    let remote_str = format!("{}:/", rclone_id);

    let list = remote_paths
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let dir = tempdir()?;
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S");
    let tmp_path = dir.path().join(format!("rclone_includefrom_{}.txt", timestamp,));
    fs::write(&tmp_path, &list)?;

    let args = &[
        "-vv",
        "copy",
        &remote_str,
        &local.to_string(),
        "--include-from",
        &tmp_path.to_string(),
    ];

    shell::out_inherited("rclone", args).map_err(|e| e.into())
}

pub fn push(local: &Path, provider_id: &ProviderId, remote_path: Option<PathBuf>) -> Result<ShellCmd> {
    let local_str = local.to_string();

    let provider = provider::get(provider_id)?;

    let rclone_id = provider.rclone.as_ref().context("rclone not supported")?;

    let remote_str = format!(
        "{}:{}",
        rclone_id,
        remote_path
            .or_else(|| provider.remote_path_fallback.clone())
            .map(|p| p.to_string())
            .unwrap_or_else(|| "/".into())
    );

    let extra = provider.extra_rclone_push_args.clone().unwrap_or_default();
    let mut args = vec!["--exclude", ".DS_Store", "-vv", "copy", &local_str, &remote_str];
    args.append(&mut extra.iter().map(String::as_str).collect());

    shell::out_inherited("rclone", &args).map_err(|e| e.into())
}

pub fn ls(provider_id: &ProviderId, remote_path: Option<PathBuf>) -> Result<ShellCmd> {
    let rclone_id = get_rclone_id(provider_id)?;

    let remote_str = format!(
        "{}:{}",
        rclone_id,
        remote_path.map(|p| p.to_string()).unwrap_or_else(|| "/".into())
    );

    let args = &["ls", &remote_str];
    shell::out("rclone", args).map_err(|e| e.into())
}

pub fn has_prefixed_files(
    provider_id: &ProviderId,
    remote_path: Option<PathBuf>,
    prefix: &str,
) -> Result<bool> {
    let provider = provider::get(provider_id)?;
    let rclone_id = provider.rclone.as_ref().context("rclone not supported")?;

    let remote_str = format!(
        "{}:{}",
        rclone_id,
        remote_path
            .or_else(|| provider.remote_path_fallback.clone())
            .map(|p| p.to_string())
            .unwrap_or_else(|| "/".into())
    );

    let include_str = format!("{}_*", prefix);

    let args = &["lsf", &remote_str, "--include", &include_str];
    let out = shell::out("rclone", args)?;
    let stdout = out.res.context("no res")?.stdout;
    let lines = stdout.lines().count();

    Ok(lines > 0)
}

pub fn mv(provider_id: &ProviderId, remote_path: &Path, local_path: &Path) -> Result<ShellCmd> {
    let rclone_id = get_rclone_id(provider_id)?;
    let remote_str = format!("{}:{}", rclone_id, remote_path.to_string());

    let args = &[
        "move",
        "-vv",
        "--delete-empty-src-dirs",
        &remote_str,
        &local_path.to_string(),
    ];
    shell::out_inherited("rclone", args).map_err(|e| e.into())
}

pub fn rmdirs(provider_id: &ProviderId, remote_path: &Path) -> Result<ShellCmd> {
    let rclone_id = get_rclone_id(provider_id)?;
    let remote_str = format!("{}:{}", rclone_id, remote_path.to_string());

    let args = &["rmdirs", &remote_str];
    shell::out_inherited("rclone", args).map_err(|e| e.into())
}
