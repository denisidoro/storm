use crate::config;
use crate::config::yaml::Provider;
use anyhow::Result;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub type ProviderId = str;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider not defined. Tried: {tried:?}, available: {available:?}")]
    Undefined { tried: String, available: Vec<String> },
}

fn multiple_folder_path(provider: &Provider, relative: &Path) -> PathBuf {
    provider.buffer.join(relative)
}

fn single_folder_path(provider: &Provider, relative: &Path) -> PathBuf {
    let filename = relative
        .iter()
        .filter_map(|s| s.to_str())
        .collect::<Vec<&str>>()
        .join("_");
    provider.buffer.join(filename)
}

pub fn get(provider_id: &ProviderId) -> Result<&'static Provider> {
    config::get()
        .yaml
        .cloud
        .providers
        .get(provider_id)
        .ok_or_else(|| {
            let tried = provider_id.into();
            let available = config::get()
                .yaml
                .cloud
                .providers
                .keys()
                .map(|x| x.to_owned())
                .collect();
            ProviderError::Undefined { tried, available }
        })
        .map_err(|e| e.into())
}

pub fn path(provider_id: &ProviderId, relative: &Path) -> Result<PathBuf> {
    let provider = get(provider_id)?;

    let path = if provider.single_folder {
        single_folder_path(provider, relative)
    } else {
        multiple_folder_path(provider, relative)
    };

    Ok(path)
}
