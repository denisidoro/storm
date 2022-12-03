use crate::env_var;

pub struct EnvConfig {
    pub config_path: Option<String>,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self {
            config_path: env_var::get(env_var::CONFIG_PATH).ok(),
        }
    }
}
