use crate::smalldate::SmallDate;
use crate::{exif, log};
use anyhow::{Context, Error, Result};
use directories_next::BaseDirs;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Error, Debug)]
#[error("Invalid path `{0}`")]
pub struct InvalidPath(pub PathBuf);

#[derive(Error, Debug)]
#[error("Unable to read directory `{dir}`")]
pub struct UnreadableDir {
    dir: PathBuf,
    #[source]
    source: Error,
}

impl IPathBuf for Path {
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

pub trait IPathBuf {
    fn to_string(&self) -> String;
}

pub fn open(filename: &Path) -> Result<File> {
    File::open(filename).with_context(|| {
        let x = filename.to_string();
        format!("Failed to open file {}", &x)
    })
}

pub fn read_lines(filename: &Path) -> Result<impl Iterator<Item = String>> {
    let file = open(filename)?;
    let lines = BufReader::new(file).lines();
    Ok(lines.map(|x| x.expect("bad line")))
}

pub fn default_config_pathbuf() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("unable to get base dirs")?;

    let mut pathbuf = PathBuf::from(base_dirs.config_dir());
    pathbuf.push("storm");
    pathbuf.push("config.yaml");
    Ok(pathbuf)
}

pub fn mv(from: &Path, to: &Path) -> Result<()> {
    remove_file(to)?;
    create_parent_all(to)?;

    log::warn(&format!("move_file {} to {}", from.to_string(), to.to_string()));

    fs::rename(from, to)?;
    Ok(())
}

pub fn remove_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    log::warn(&format!("remove_file {}", path.to_string()));

    fs::remove_file(path).with_context(|| format!("Failed to rm {}", path.to_string()))?;
    Ok(())
}

pub fn write(to: &Path, txt: &str) -> Result<()> {
    create_parent_all(to)?;

    log::warn(&format!("write {}", to.to_string()));

    fs::write(to, txt).with_context(|| format!("Couldn't write to {}", to.to_string()))
}

pub fn copy(from: &Path, to: &Path) -> Result<u64> {
    remove_file(to)?;
    create_parent_all(to)?;

    log::warn(&format!("copy file {} to {}", from.to_string(), to.to_string()));

    fs::copy(from, to).with_context(|| format!("Failed to copy {} to {}", from.to_string(), to.to_string()))
}

pub fn metadata(filepath: &Path) -> Result<(u32, SmallDate)> {
    let metadata = std::fs::metadata(filepath)?;
    let kb = (metadata.len() / 1024) as u32;

    let date = if let Ok(d) = exif::date(filepath) {
        d
    } else {
        let modified = metadata.modified()?;
        SmallDate::from_system_time(modified)?
    };

    Ok((kb, date))
}

pub fn remove_dir_all(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    log::warn(&format!("remove_dir_all {}", dir.to_string()));

    fs::remove_dir_all(dir).with_context(|| {
        let x = dir.to_string();
        format!("Failed to remove dir {}", &x)
    })
}

pub fn create_parent_all(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Unable to create parent dir for {}",
                path.to_str().unwrap_or("invalid path")
            )
        })?;
    }
    Ok(())
}

pub fn is_os_file(entry: &DirEntry) -> bool {
    match entry.path().to_str() {
        None => false,
        Some(p) => p.to_lowercase().contains("ds_store"),
    }
}

pub fn remove_os_files(path: &Path) -> Result<()> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.to_string().contains("DS_Store") {
            remove_file(path)?;
        }
    }

    Ok(())
}
