pub mod bsp;
pub mod db;
pub mod google;
pub mod simple8b;

use crate::exif;
use crate::fs::IPathBuf;
use anyhow::Result;
use chrono::DateTime;
use db::GeoDB as Db;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn add_one(filepath: &Path, db: &Db) -> Result<()> {
    let has_latitude = exif::has_latitude(filepath);
    if let Ok(false) = has_latitude {
    } else {
        return Ok(());
    }

    println!("\n{}", filepath.to_string());

    let timestamp = exif::datetime(filepath)?;
    println!("{timestamp}");
    let pos = db.pos(timestamp)?;
    println!("({}, {})", pos.0, pos.1);
    exif::add_geo(filepath, pos)?;
    Ok(())
}

pub fn add(dir: &Path) -> Result<()> {
    dbg!("hi");

    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
        .map(|entry| entry.path().to_owned())
        .collect();
    files.sort();

    let path = PathBuf::from("/Users/denis.isidoro/Downloads/Takeout/Location History/Records_simple3.txt");
    let min_time = DateTime::parse_from_rfc3339("2015-01-01T12:00:00-03:00").unwrap();
    let max_time = DateTime::parse_from_rfc3339("2030-01-01T12:00:00-03:00").unwrap();
    let db = google::get_db(&path, min_time, max_time)?;

    for file in &files {
        add_one(file, &db)?;
    }

    Ok(())
}
