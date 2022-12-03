use crate::fs;
use crate::fs::IPathBuf;
use regex::Regex;
use walkdir::DirEntry;

#[derive(Debug)]
pub(super) enum SkipReason {
    NoSkip,
    TooBig(u64),
    InvalidPath,
    Denylist(String),
    Directory,
    NoMetadata,
    OsFile,
}

pub(super) fn should_process(entry: &DirEntry, max_kb: u64, denylist: &[Regex]) -> SkipReason {
    let path = entry.path();

    if path.to_str().is_none() {
        return SkipReason::InvalidPath;
    }

    if let Ok(metadata) = entry.metadata() {
        if !metadata.is_file() {
            return SkipReason::Directory;
        }
        let kb = metadata.len() / 1024;
        if kb > max_kb {
            return SkipReason::TooBig(kb);
        }
        for regex in denylist {
            if regex.is_match(&path.to_string()) {
                return SkipReason::Denylist(regex.to_string());
            }
        }
    } else {
        return SkipReason::NoMetadata;
    }

    if fs::is_os_file(entry) {
        return SkipReason::OsFile;
    }

    SkipReason::NoSkip
}
