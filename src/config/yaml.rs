use super::env::EnvConfig;
use super::ClapConfig;
use crate::fs;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraBackupDef {
    pub from: PathBuf,
    pub to: PathBuf,
    #[serde(default = "default_low_unzipped")]
    pub low_unzipped: String,
    #[serde(default = "default_low_zipped")]
    pub low_zipped: String,
    #[serde(default = "default_high_unzipped")]
    pub high_unzipped: String,
    #[serde(default = "default_high_zipped")]
    pub high_zipped: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub paths: Vec<CameraBackupDef>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CameraBackup {
    pub local_source: PathBuf,
    pub local_intermediate: PathBuf,
    pub intermediate_provider: String,
    pub intermediate_relative: PathBuf,
    pub ref_provider: String,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Telegram {
    pub chat_id: String,
    pub token: String,
    pub db_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BackupPath {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backup {
    #[serde(default = "default_backup_max_kb")]
    pub max_kb: u32,
    #[serde(default = "default_backup_denylist")]
    pub denylist: Vec<String>,
    #[serde(default = "default_provider")]
    pub provider: String,
    pub paths: Vec<BackupPath>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub buffer: PathBuf,
    pub rclone: Option<String>,
    #[serde(default = "default_single_folder")]
    pub single_folder: bool,
    pub extra_rclone_push_args: Option<Vec<String>>,
    pub remote_path_fallback: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Crypto {
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Parallelism {
    pub workers: u8,
}

#[derive(Default, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Storm {
    pub providers: HashMap<String, Provider>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Tasker {
    pub log_task: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    #[serde(default = "default_archive_max_file_kb")]
    pub max_file_kb: u32,
    #[serde(default = "default_archive_max_zip_kb")]
    pub max_zip_kb: u32,
    pub denylist: Vec<String>,
    #[serde(default = "default_archive_source_provider")]
    pub source_provider: String,
    #[serde(default = "default_archive_provider")]
    pub provider: String,
    pub db_folder: Option<PathBuf>,
    pub tmp_buffer: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct YamlConfig {
    pub archive: Archive,
    pub camera: Camera,
    pub camera_backup: CameraBackup,
    pub crypto: Crypto,
    pub cloud: Storm,
    pub tasker: Tasker,
    pub parallelism: Parallelism,
    pub telegram: Telegram,
    pub backup: Backup,
}

impl YamlConfig {
    fn _from_str(text: &str) -> Result<Self> {
        serde_yaml::from_str(text).map_err(|e| e.into())
    }

    fn from_path(path: &Path) -> Result<Self> {
        let file = fs::open(path)?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).map_err(|e| e.into())
    }

    pub fn get(env: &EnvConfig, clap: &ClapConfig) -> Result<Self> {
        if let Some(p) = clap.config.clone() {
            return YamlConfig::from_path(&p);
        }
        if let Some(path_str) = env.config_path.as_ref() {
            let p = PathBuf::from(path_str);
            return YamlConfig::from_path(&p);
        }
        if let Ok(p) = fs::default_config_pathbuf() {
            if p.exists() {
                return YamlConfig::from_path(&p);
            }
        }
        Ok(YamlConfig::default())
    }
}

impl Default for Parallelism {
    fn default() -> Self {
        Self { workers: 4 }
    }
}

fn default_low_unzipped() -> String {
    "gphotos".into()
}

fn default_low_zipped() -> String {
    "pcloud".into()
}

fn default_high_unzipped() -> String {
    "telegram".into()
}

fn default_high_zipped() -> String {
    "alumni".into()
}

fn default_backup_max_kb() -> u32 {
    1024
}

fn default_backup_denylist() -> Vec<String> {
    vec![r#".*\.app"#.into()]
}

fn default_provider() -> String {
    "box".into()
}

fn default_single_folder() -> bool {
    false
}

fn default_archive_max_file_kb() -> u32 {
    300 * 1024
}

fn default_archive_max_zip_kb() -> u32 {
    500 * 1024
}

fn default_archive_source_provider() -> String {
    "alumni".into()
}

fn default_archive_provider() -> String {
    "azure".into()
}
