mod work;

use crate::config;
use crate::db;
use crate::db::Db;
use crate::fs::{self, IPathBuf};
use crate::log;
use crate::provider;
use crate::rclone;
use crate::smalldate::SmallDate;
use crate::zip;
use anyhow::{Context, Result};
use regex::Regex;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;
use work::{should_process, SkipReason};

fn get_db_folder() -> Result<&'static PathBuf> {
    config::get()
        .yaml
        .archive
        .db_folder
        .as_ref()
        .context("empty archive.db_folder")
}

fn get_tmp_buffer() -> Result<&'static PathBuf> {
    config::get()
        .yaml
        .archive
        .tmp_buffer
        .as_ref()
        .context("empty archive.tmp_buffer")
}

fn files_to_zip(ls_lines: &mut dyn Iterator<Item = String>) -> Result<Vec<PathBuf>> {
    let max_zip_kb = config::get().yaml.archive.max_zip_kb;

    let db_folder = get_db_folder()?;

    let denylist = config::get()
        .yaml
        .archive
        .denylist
        .iter()
        .map(|x| Regex::new(x).expect("invalid regex"))
        .collect::<Vec<_>>();

    let zip_kb_threshold = (0.9 * max_zip_kb as f32) as u32;
    let mut zip_kb = 0;

    let mut files = vec![];

    let hashes = db::all_hashes(db_folder);

    for line in ls_lines {
        let (bytes_str, path_str) = line.trim().split_once(' ').context("unable to split")?;
        let kb = (bytes_str.trim().parse::<u64>()? / 1024) as u32;
        let path = PathBuf::from(path_str.trim());
        let reason = should_process(&path, kb, &denylist, zip_kb, &hashes);

        if let SkipReason::NoSkip = reason {
            zip_kb += kb;
            log::debug(&format!(
                "File included: {}, cumm_mb: {}",
                &path.to_string(),
                zip_kb / 1024
            ));
            files.push(path);
        } else {
            log::warn(&format!("File skipped: {:?}, {}", &reason, &path.to_string()));
        }

        if zip_kb >= zip_kb_threshold {
            break;
        }
    }

    Ok(files)
}

fn download_files(files: Vec<PathBuf>) -> Result<()> {
    dbg!("download_files");

    let provider_id = &config::get().yaml.archive.source_provider;
    let to = get_tmp_buffer()?;

    rclone::pull_many(provider_id, &files, to)?;

    Ok(())
}

pub fn unzip_files(folder: &Path) -> Result<()> {
    dbg!("unzip_files");

    let password = config::get().crypto_password()?;

    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let is_file = entry.metadata().map(|m| m.is_file()).unwrap_or(false);
        let is_7z = is_file && path.extension().map(|e| e == "7z").unwrap_or(false);
        if !is_7z {
            continue;
        }

        let to = path.parent().context("no parent")?;
        let result = zip::extract(password, path, to);
        if result.is_ok() {
            log::info(&format!("Extracting {} succeeded!", path.to_string()));
            fs::remove_file(path)?;
        } else {
            log::error(&format!("Extracting {} failed!", path.to_string()));
        }
    }

    Ok(())
}

fn create_zip() -> Result<(PathBuf, String)> {
    dbg!("create_zip");

    let buffer = get_tmp_buffer()?;
    let provider_id = &config::get().yaml.archive.provider;

    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("{}.7z", timestamp);

    let relative_to = PathBuf::from("ByTimestamp").join(&filename);
    let to = provider::get(provider_id)?.buffer.join(&relative_to);

    let password = config::get().crypto_password()?;

    zip::create(password, buffer, &to)?;

    let file = fs::open(&to)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let hash = format!("{:x}", md5::compute(buffer));

    Ok((relative_to, hash))
}

fn save_db(zip_path: &Path, hash: String) -> Result<()> {
    dbg!("save_db");

    let mut db = Db::new();

    let buffer = get_tmp_buffer()?;

    for entry in WalkDir::new(buffer).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let is_file = entry.metadata().map(|m| m.is_file()).unwrap_or(false);
        if !is_file {
            continue;
        }

        db.add_file(path, buffer)?;
    }

    let azure_path = zip_path.to_string();
    db.add_tag("azure_path".into(), azure_path);
    db.add_tag("hash".into(), hash);
    db.add_tag("date".into(), SmallDate::now()?.to_string());

    let to = {
        let mut p = get_db_folder()?.clone();
        p.push(zip_path);
        p.set_extension("storm.txt");
        p
    };

    fs::write(&to, &db.to_string())?;

    Ok(())
}

fn remove_unwanted_files() -> Result<()> {
    let buffer = get_tmp_buffer()?;
    fs::remove_os_files(buffer)
}

fn cleanup() -> Result<()> {
    let buffer = get_tmp_buffer()?;
    fs::remove_dir_all(buffer)?;
    Ok(())
}

pub fn create_zips() -> Result<()> {
    let provider_id = &config::get().yaml.archive.source_provider;
    let ls_res = rclone::ls(provider_id, None)?.res.context("no source output")?;
    let ls_lines = ls_res.stdout.lines().map(|x| x.to_string());

    let mut zip_id = 0;
    loop {
        let mut lines = ls_lines.clone();
        let files = files_to_zip(&mut lines)?;
        dbg!((zip_id, &files));

        if files.is_empty() {
            break;
        }

        cleanup()?;
        download_files(files)?;

        let tmp_buffer = get_tmp_buffer()?;
        unzip_files(tmp_buffer)?;

        remove_unwanted_files()?;
        let (zip_path, hash) = create_zip()?;
        remove_unwanted_files()?;
        save_db(&zip_path, hash)?;
        cleanup()?;
        zip_id += 1;
    }

    Ok(())
}
