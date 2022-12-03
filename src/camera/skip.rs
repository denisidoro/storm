use crate::format::Format;
use walkdir::DirEntry;

#[derive(Debug)]
pub(super) enum SkipReason {
    NoSkip,
    Unsupported,
    InvalidPath,
    NoMetadata,
    Directory,
}

pub(super) fn should_process(entry: &DirEntry, format: &Format) -> SkipReason {
    if entry.path().to_str().is_none() {
        return SkipReason::InvalidPath;
    }

    if let Ok(metadata) = entry.metadata() {
        if !metadata.is_file() {
            return SkipReason::Directory;
        }
        if let Format::Unsupported = format {
            return SkipReason::Unsupported;
        }
    } else {
        return SkipReason::NoMetadata;
    }

    SkipReason::NoSkip
}
