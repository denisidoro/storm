use crate::fs::{self};

use walkdir::DirEntry;

#[derive(Debug)]
pub(super) enum SkipReason {
    NoSkip,
    InvalidPath,
    NoMetadata,
    Directory,
    OsFile,
}

pub(super) fn should_process(entry: &DirEntry) -> SkipReason {
    if entry.path().to_str().is_none() {
        return SkipReason::InvalidPath;
    }

    if let Ok(metadata) = entry.metadata() {
        if !metadata.is_file() {
            return SkipReason::Directory;
        }
    } else {
        return SkipReason::NoMetadata;
    }

    if fs::is_os_file(entry) {
        return SkipReason::OsFile;
    }

    SkipReason::NoSkip
}
