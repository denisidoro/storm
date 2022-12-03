use crate::fs::{self, IPathBuf};
use crate::shell::{self, ShellCmd};
use crate::{crypto, log};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn gen_password(password: &str, to: &Path) -> Result<String> {
    let filename = not_zipped_filename(to);
    crypto::gen_password(password, &filename)
}

pub fn create(password: &str, from: &Path, to: &Path) -> Result<()> {
    if to.exists() {
        fs::remove_file(to)?;
    }

    let from_file = from.metadata().map(|m| m.is_file()).unwrap_or(false);

    let to_str = to.to_string();
    let from_str = if from_file {
        from.to_string()
    } else {
        format!("{}/*", from.to_string())
    };

    let full_password = gen_password(password, to)?;
    let pass = format!("-p{}", full_password);

    log::info(&format!("zip {}", &from_str));

    fs::create_parent_all(to)?;

    shell::out("7z", &["a", &pass, &to_str, &from_str])?;

    if from_file {
        shell::out("touch", &["-r", &from_str, &to_str])?;
    }

    Ok(())
}

pub fn extract(password: &str, from: &Path, to_folder: &Path) -> Result<ShellCmd> {
    let from_str = from.to_string();
    let to_str = format!("-o{}", to_folder.to_string());

    let full_password = gen_password(password, from)?;
    let pass = format!("-p{}", full_password);

    shell::out("7z", &["x", &from_str, &pass, &to_str, "-aos"]).map_err(|e| e.into())
}

pub fn zipped_name(p: &Path) -> PathBuf {
    let s = p.to_str().expect("invalid path");
    format!("{}.7z", s).into()
}

fn not_zipped_filename(p: &Path) -> String {
    let filename = p.file_name().expect("invalid filename").to_string_lossy();
    filename.as_ref().trim_end_matches(".7z").to_owned()
}
