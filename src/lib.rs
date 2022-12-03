#[macro_use]
extern crate anyhow;

mod alumni;
mod archive;
mod backup;
mod camera;
mod compress;
mod config;
mod crypto;
mod db;
mod env_var;
mod exif;
mod format;
mod fs;
mod geo;
mod log;
mod normalize;
mod provider;
mod rclone;
mod shell;
mod smalldate;
mod tasker;
mod telegram;
mod upload;
mod url;
mod zip;

use std::path::PathBuf;

use crate::{config::Command, fs::IPathBuf};
use anyhow::Result;

pub fn handle() -> Result<()> {
    use Command::*;
    config::setup()?;
    shell::setup()?;

    match config::get().cmd() {
        SetCameraBuffers => {
            camera::set_buffers()?;
            Ok(())
        }
        SetBackupBuffer => {
            backup::set_buffer()?;
            Ok(())
        }
        SetBackupCameraBuffer => {
            camera::set_camera_buffer()?;
            Ok(())
        }
        RemoveBackedPictures => {
            camera::remove_backed_pictures()?;
            Ok(())
        }
        UploadBuffer {
            provider,
            remote_path,
        } => {
            upload::push_and_rm(provider, remote_path.clone())?;
            Ok(())
        }
        UploadTelegramBuffer => {
            telegram::upload_buffer()?;
            Ok(())
        }
        SendTelegramMessage { txt } => {
            let cmd = telegram::send_message(txt)?;
            dbg!(&cmd);
            Ok(())
        }
        Password { filename } => {
            let password = config::get().crypto_password()?;
            let out = zip::gen_password(password, filename)?;
            println!("{}", out);
            Ok(())
        }
        UploadTelegramFile { from } => {
            let cmd = telegram::upload(from)?;
            dbg!(&cmd);
            Ok(())
        }
        Move {
            provider,
            remote_path,
            local_path,
        } => {
            rclone::mv(provider, remote_path, local_path)?;
            match rclone::rmdirs(provider, remote_path) {
                Ok(_) => {}
                Err(e) => {
                    dbg!(e);
                }
            }
            Ok(())
        }
        Debug { query: _ } => {
            log::debug("debug!");
            let path = PathBuf::from("/Users/denis.isidoro/dev/storm/Cargo.toml")
                .join(PathBuf::from("."))
                .to_string();
            dbg!(&path);
            Ok(())
        }
        CreateArchiveZips => archive::create_zips(),
        AddTime { path, minutes } => exif::add_time(path, *minutes),
        AddGeo { dir } => geo::add(dir),
        DbFromFilepaths {
            path_to_list,
            path,
            hash,
            date,
        } => {
            let db = db::read_list(path_to_list)?;
            println!("azure_path: {}", path.to_string());
            println!("hash: {}", hash);
            println!("uploaded: {}", date);
            println!("=");
            println!("{}", db);
            Ok(())
        }
        DownloadAlumni {
            list,
            to,
            start,
            files,
        } => {
            alumni::download(list, to, *start, *files)?;
            Ok(())
        }
        Push {
            from,
            provider,
            remote_path,
        } => {
            let cmd = rclone::push(from, provider, remote_path.clone())?;
            dbg!(&cmd);
            Ok(())
        }
    }
}
