mod skip;

use crate::fs::{self, IPathBuf};
use crate::provider::ProviderId;
use crate::shell;
use crate::{config, log};
use crate::{db, provider};
use anyhow::Result;
use skip::{should_process, SkipReason};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const PROVIDER_ID: &ProviderId = "telegram";

pub fn upload(path: &Path) -> Result<shell::ShellCmd> {
    let filepath = path.to_string();
    let token = &config::get().yaml.telegram.token;
    let chat_id = &config::get().yaml.telegram.chat_id;

    let buffer = &provider::get(PROVIDER_ID)?.buffer;
    let caption = filepath
        .trim_start_matches(&buffer.to_string())
        .trim_start_matches('/')
        .replace('"', "\\\"");

    let document = format!("document=@\"{}\"", filepath);
    let url = format!(
        "https://api.telegram.org/bot{}/sendDocument?chat_id={}&caption={}",
        token, chat_id, caption
    );

    let args = &["-F", &document, &url];
    shell::out("curl", args).map_err(|e| e.into())
}

pub fn send_message(txt: &str) -> Result<shell::ShellCmd> {
    let chat_id = &config::get().yaml.telegram.chat_id;
    let token = &config::get().yaml.telegram.token;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let escaped_txt = txt.replace('"', "\\\"");
    let data = format!(r#"{{"chat_id": "{}", "text": "{}"}}"#, chat_id, escaped_txt);

    let args = &["-d", &data, "-H", "Content-Type: application/json", &url];
    shell::out("curl", args).map_err(|e| e.into())
}

fn get_filepaths(path: &Path) -> Result<Vec<PathBuf>> {
    let mut filepaths = vec![];

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let reason = should_process(&entry);
        if let SkipReason::NoSkip = reason {
            filepaths.push(entry.path().into());
        }
    }

    Ok(filepaths)
}

pub fn upload_buffer() -> Result<()> {
    let buffer = &provider::get(PROVIDER_ID)?.buffer;
    fs::remove_os_files(buffer)?;

    let filepaths = get_filepaths(buffer)?;
    let n = filepaths.len();
    log::setup("tu".into(), n)?;

    let db_path = &config::get().yaml.telegram.db_path;

    dbg!(&filepaths);

    let mut errors = 0;
    for (i, filepath) in filepaths.into_iter().enumerate() {
        match upload(&filepath) {
            Ok(cmd) => {
                dbg!(cmd);

                let mut db = db::read(db_path)?;
                db.add_file(&filepath, buffer)?;
                db::write(db, db_path)?;

                fs::remove_file(&filepath)?;

                log::success(i);
            }
            Err(e) => {
                log::failure(i, e);
                errors += 1;
            }
        }
    }

    if errors > 0 {
        Err(anyhow!("{} files failed", errors))
    } else {
        Ok(())
    }
}
