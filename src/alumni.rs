use crate::archive;
use crate::fs;
use crate::rclone;
use anyhow::{self, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn download(list: &Path, to: &Path, start: usize, files: usize) -> Result<()> {
    let lines = fs::read_lines(list)?;
    let allowlist: Vec<PathBuf> = lines
        .skip(start)
        .take(files)
        .into_iter()
        .map(|s| PathBuf::from(format!("/Pictures/Camera/{}", s)))
        .collect();
    dbg!(&allowlist);

    rclone::pull_many("alumni", &allowlist[..], to)?;

    let mut camera = to.to_owned();
    camera.push("Pictures");
    camera.push("Camera");
    dbg!(&camera);

    move_root_files(&camera)?;

    archive::unzip_files(&camera)?;

    Ok(())
}

fn move_root_files(root: &Path) -> Result<()> {
    let entries = WalkDir::new(root).max_depth(1).into_iter().filter_map(|e| e.ok());
    for entry in entries {
        let path = entry.path();
        dbg!(&path);

        let is_file = entry.metadata().map(|m| m.is_file()).unwrap_or(false);
        if !is_file {
            continue;
        }

        let filename = path.file_name().unwrap_or_default();
        let mut p = root.to_owned();
        p.push("no_folder");
        p.push(filename);

        fs::mv(path, &p)?;
    }

    Ok(())
}
