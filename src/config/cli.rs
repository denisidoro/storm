use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    SetCameraBuffers,
    SetBackupBuffer,
    Debug {
        query: String,
    },
    CreateArchiveZips,
    UploadBuffer {
        #[clap(ignore_case = true)]
        provider: String,
        remote_path: Option<PathBuf>,
    },
    Move {
        #[clap(ignore_case = true)]
        provider: String,
        remote_path: PathBuf,
        local_path: PathBuf,
    },
    Push {
        from: PathBuf,
        #[clap(ignore_case = true)]
        provider: String,
        remote_path: Option<PathBuf>,
    },
    UploadTelegramFile {
        from: PathBuf,
    },
    SendTelegramMessage {
        txt: String,
    },
    UploadTelegramBuffer,
    Password {
        filename: PathBuf,
    },
    DbFromFilepaths {
        path_to_list: PathBuf,
        path: PathBuf,
        hash: String,
        date: String,
    },
    AddGeo {
        dir: PathBuf,
    },
    AddTime {
        path: PathBuf,
        minutes: u32,
    },
    DownloadAlumni {
        list: PathBuf,
        to: PathBuf,
        start: usize,
        files: usize,
    },
    SetBackupCameraBuffer,
    RemoveBackedPictures,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct ClapConfig {
    #[clap(subcommand)]
    pub cmd: Command,

    #[clap(short, long)]
    pub config: Option<PathBuf>,

    #[clap(short, long)]
    pub ignore_rotation: bool,
}

impl ClapConfig {
    pub fn new() -> Self {
        Self::parse()
    }
}
