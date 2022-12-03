mod cli;
mod env;
pub mod yaml;

use anyhow::{Context, Result};
pub use cli::{ClapConfig, Command};
use env::EnvConfig;
use once_cell::sync::OnceCell;
use yaml::YamlConfig;

use crate::log;

static INSTANCE: OnceCell<Config> = OnceCell::new();

pub struct Config {
    pub yaml: YamlConfig,
    pub clap: ClapConfig,
}

impl Config {
    pub fn new() -> Self {
        let clap = ClapConfig::new();
        let env = EnvConfig::new();
        let yaml = YamlConfig::get(&env, &clap).unwrap_or_else(|e| {
            log::error(&format!(
                "Error parsing config file: {}\nFallbacking to default one...",
                e
            ));
            YamlConfig::default()
        });
        Self { yaml, clap }
    }

    pub fn cmd(&self) -> &Command {
        &self.clap.cmd
    }

    pub fn crypto_password(&self) -> Result<&String> {
        self.yaml.crypto.password.as_ref().context("password not set")
    }
}

pub fn get() -> &'static Config {
    INSTANCE.get().expect("config not initialized")
}

pub fn setup() -> Result<()> {
    let config = Config::new();
    INSTANCE
        .set(config)
        .map_err(|_| anyhow!("unable to set config"))?;
    Ok(())
}
