use crate::config;
use crate::db;

use crate::fs::IPathBuf;

use regex::Regex;

use std::path::Path;

#[derive(Debug)]
pub(super) enum SkipReason {
    NoSkip,
    ExceedsFileSize(u32),
    ExceedsZipSize(u32),
    Denylist(String),
    AlreadyUploaded,
}

pub(super) fn should_process(
    path: &Path,
    kb: u32,
    denylist: &[Regex],
    zip_kb: u32,
    hashes: &[u64],
) -> SkipReason {
    let max_file_kb = config::get().yaml.archive.max_file_kb;
    if kb > max_file_kb {
        return SkipReason::ExceedsFileSize(kb);
    }

    let max_zip_kb = config::get().yaml.archive.max_zip_kb;
    let new_zip_kb = zip_kb + kb;
    if new_zip_kb > max_zip_kb {
        return SkipReason::ExceedsZipSize(new_zip_kb);
    }

    for regex in denylist {
        if regex.is_match(&path.to_string()) {
            return SkipReason::Denylist(regex.to_string());
        }
    }

    if db::has(path, hashes) {
        return SkipReason::AlreadyUploaded;
    }

    SkipReason::NoSkip
}
