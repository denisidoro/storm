use crate::exif::{self, CompressionInvariantProps, Props};
use crate::format::{self, Format};
use crate::fs::{self, IPathBuf};
use crate::log;
use crate::shell;
use anyhow::{self, Result};
use std::path::Path;

pub enum Quality {
    High,
    Low,
}

fn get_new_dimensions(width: u32, height: u32, max_allowed_min_dim: u32) -> (u32, u32) {
    let (is_wider, max_dim, min_dim) = if width > height {
        (true, width, height)
    } else {
        (false, height, width)
    };

    let new_min_dim = {
        let d = max_allowed_min_dim;
        if d > min_dim {
            min_dim
        } else {
            d
        }
    };

    let new_max_dim = new_min_dim * (max_dim / min_dim);

    if is_wider {
        (new_max_dim, new_min_dim)
    } else {
        (new_min_dim, new_max_dim)
    }
}

fn compress_image(from_props: &Props, to: &Path, quality: Quality) -> Result<()> {
    let from = &from_props.path;
    let from_str = from.to_string();
    let to_str = to.to_string();

    let is_high = matches!(quality, Quality::High);
    let quality_number: u32 = if is_high { 88 } else { 82 };
    let quality_str = quality_number.to_string();

    let max_allowed_min_dim = 1080;
    let width = from_props.width;
    let height = from_props.height;
    let (new_width, new_height) = get_new_dimensions(width, height, max_allowed_min_dim);
    let max_dims_str = format!("{}x{}", new_width, new_height);

    fs::create_parent_all(to)?;

    let args = {
        let mut a: Vec<&str> = vec![
            &from_str,
            "-sampling-factor",
            "4:2:0",
            "-strip",
            "-quality",
            &quality_str,
            "-interlace",
            "JPEG",
        ];

        if !is_high {
            a.push("-resize");
            a.push(&max_dims_str);
        }

        a.push(&to_str);

        a
    };

    let cmd = shell::out("convert", args.as_slice())?;
    eprintln!("{:?}", &cmd);

    exif::copy_metadata(from_props, to)?;

    Ok(())
}

fn compress_video(from_props: &Props, to: &Path, quality: Quality) -> Result<()> {
    let from = &from_props.path;
    let from_str = from.to_string();
    let to_str = to.to_string();

    fs::create_parent_all(to)?;

    let codec = "libx265";
    let preset = "superfast";
    let is_high = matches!(quality, Quality::High);
    let crf: u32 = if is_high { 28 } else { 36 };

    let args = &[
        "-hide_banner",
        "-nostats",
        "-y",
        "-i",
        &from_str,
        "-vcodec",
        codec,
        "-preset",
        preset,
        "-crf",
        &crf.to_string(),
        &to_str,
    ];

    let cmd = shell::out("ffmpeg", args)?;
    let cmd_fmt = {
        let mut s = format!("{:?}", cmd);
        s.truncate(500);
        s
    };
    eprintln!("{cmd_fmt}...");

    exif::copy_metadata(from_props, to)?;

    Ok(())
}

pub fn target_created(from_props: &Props, to: &Path) -> (bool, Option<CompressionInvariantProps>) {
    if !to.exists() {
        return (false, None);
    }

    let to_props = exif::props(to);

    match to_props {
        Ok(t) => {
            let eq = from_props.compression_invariant == t.compression_invariant;
            (eq, Some(t.compression_invariant))
        }
        _ => (false, None),
    }
}

fn replace_by_origin_if_larger(from: &Path, to: &Path) -> Result<()> {
    let from_size = from.metadata()?.len();
    let to_size = to.metadata()?.len();

    if to_size > from_size {
        log::error(&format!(
            "new_size > original_size ({}: {}, {}: {})",
            from.to_string(),
            from_size,
            to.to_string(),
            to_size
        ));

        fs::remove_file(to)?;
        fs::copy(from, to)?;
    }

    Ok(())
}

pub fn compress(from: &Path, to: &Path, quality: Quality) -> Result<()> {
    let from_props = &exif::props(from)?;

    if target_created(from_props, to).0 {
        log::warn(&format!("skipped compress {}", from.to_string()));
        return Ok(());
    }

    log::info(&format!("compress {}", from.to_string()));
    fs::remove_file(to)?;

    let format = format::get_format(from);
    match format {
        Format::Image => compress_image(from_props, to, quality)?,
        Format::Video => compress_video(from_props, to, quality)?,
        _ => unreachable!(),
    };

    replace_by_origin_if_larger(from, to)?;

    let (eq, to_inv_props) = target_created(from_props, to);
    if !eq {
        dbg!(&from_props.compression_invariant);
        dbg!(&to_inv_props);
        return Err(anyhow!("failed to create destination file: {}", to.to_string()));
    };

    Ok(())
}
