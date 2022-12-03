mod skip;
mod work;

use crate::config;
use crate::fs::IPathBuf;
use crate::log;
use anyhow::{Context, Result};
use regex::Regex;
use skip::{should_process, SkipReason};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use walkdir::WalkDir;
use work::{process, WorkerResult};
use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

fn get_filepaths(froms: &[PathBuf]) -> Result<HashMap<usize, Vec<PathBuf>>> {
    let mut filepaths_map = HashMap::new();

    let max_kb = config::get().yaml.backup.max_kb as u64;
    let denylist = config::get()
        .yaml
        .backup
        .denylist
        .iter()
        .map(|x| Regex::new(x).expect("invalid regex"))
        .collect::<Vec<_>>();

    for (from_index, local) in froms.iter().enumerate() {
        let mut filepaths: Vec<PathBuf> = vec![];

        for entry in WalkDir::new(local).into_iter().filter_map(|e| e.ok()) {
            let reason = should_process(&entry, max_kb, &denylist);
            match reason {
                SkipReason::Directory => (),
                SkipReason::NoSkip => {
                    log::debug(&entry.path().to_string());
                    let filepath = entry.path().strip_prefix(local.clone())?.to_owned();
                    filepaths.push(filepath);
                }
                _ => log::debug(&format!(
                    "File skipped: {:?}, {}",
                    reason,
                    entry.path().to_string()
                )),
            }
        }

        filepaths.sort();
        filepaths_map.insert(from_index, filepaths);
    }

    Ok(filepaths_map)
}

pub fn set_buffer() -> Result<()> {
    let froms = config::get()
        .yaml
        .backup
        .paths
        .iter()
        .map(|x| x.from.clone())
        .collect::<Vec<_>>();
    let filepaths = get_filepaths(&froms)?;
    let n = filepaths.values().map(|vs| vs.len()).sum();
    log::setup("bs".into(), n)?;

    let n_workers = config::get().yaml.parallelism.workers;
    let pool: Pool<ThunkWorker<WorkerResult>> = Pool::new(n_workers as usize);

    let (tx, rx) = channel();

    let mut i = 0;
    for (from_index, relatives) in filepaths {
        let backup = config::get()
            .yaml
            .backup
            .paths
            .get(from_index)
            .context("Invalid backup index")?;
        for relative in relatives {
            pool.execute_to(
                tx.clone(),
                Thunk::of(move || {
                    log::start(i);
                    let result = process(backup, relative);
                    let error = if let Err(e) = result { Some(e) } else { None };
                    WorkerResult(i, error)
                }),
            );
            i += 1;
        }
    }

    let mut errors = 0;
    for WorkerResult(i, error) in rx.iter().take(n) {
        if let Some(e) = error {
            log::failure(i, e);
            errors += 1;
        } else {
            log::success(i);
        }
    }

    if errors > 0 {
        Err(anyhow!("{} files failed", errors))
    } else {
        Ok(())
    }
}
