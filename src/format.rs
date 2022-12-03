use std::path::Path;

pub enum Format {
    Image,
    Video,
    Unsupported,
}

pub fn get_format(path: &Path) -> Format {
    if path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .starts_with(".pending")
    {
        return Format::Unsupported;
    }

    match path.extension().and_then(|e| e.to_str()) {
        None => Format::Unsupported,
        Some(extension) => match extension.to_lowercase().as_ref() {
            "jpeg" | "jpg" | "png" | "tiff" => Format::Image,
            "mp4" | "avi" | "mkv" | "mpg" | "mpeg" | "mov" | "flv" | "gif" | "m4v" => Format::Video,
            _ => Format::Unsupported,
        },
    }
}
