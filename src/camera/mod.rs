mod backup;
mod skip;
mod work;

use crate::format::{self, Format};
use crate::fs::IPathBuf;
use crate::log;
use crate::{config, normalize, smalldate};
use anyhow::Context;
use anyhow::Result;
use skip::SkipReason;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use walkdir::WalkDir;
use work::{process, WorkerResult};
use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

pub use backup::remove_backed_pictures;
pub use backup::set_camera_buffer;

fn normalize(froms: &[PathBuf], folder_id: &str) -> Result<()> {
    for from in froms {
        normalize::pictures(from, folder_id)?;
    }
    Ok(())
}

fn get_filepaths(froms: &[PathBuf]) -> Result<HashMap<usize, Vec<PathBuf>>> {
    let mut filepaths_map = HashMap::new();

    for (from_index, path) in froms.iter().enumerate() {
        let mut filepaths: Vec<PathBuf> = vec![];
        let mut video_filepaths: Vec<PathBuf> = vec![];

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let format = format::get_format(entry.path());
            let reason = skip::should_process(&entry, &format);
            match reason {
                SkipReason::Directory => (),
                SkipReason::NoSkip => {
                    let filepath = if path.is_file() {
                        PathBuf::from(".")
                    } else {
                        entry.path().strip_prefix(path.clone())?.to_owned()
                    };

                    if let Format::Video = format {
                        video_filepaths.push(filepath);
                    } else {
                        filepaths.push(filepath);
                    }
                }
                _ => log::debug(&format!(
                    "File skipped: {:?}, {}",
                    reason,
                    entry.path().to_string()
                )),
            }
        }

        filepaths.sort();
        video_filepaths.sort();
        filepaths.append(&mut video_filepaths);
        filepaths_map.insert(from_index, filepaths);
    }

    Ok(filepaths_map)
}

pub fn set_buffers() -> Result<()> {
    let froms = &config::get()
        .yaml
        .camera
        .paths
        .iter()
        .map(|x| x.from.clone())
        .collect::<Vec<_>>();

    let folder_id = smalldate::now_hours_base36()?;
    normalize(froms, &folder_id)?;

    let filepaths = get_filepaths(froms)?;
    let n = filepaths.values().map(|vs| vs.len()).sum();
    log::setup("cs".into(), n)?;

    dbg!(&froms);
    dbg!(&filepaths);

    let n_workers = config::get().yaml.parallelism.workers;
    let pool: Pool<ThunkWorker<WorkerResult>> = Pool::new(n_workers as usize);

    let (tx, rx) = channel();

    let mut i = 0;
    for (from_index, relatives) in filepaths {
        let backup = config::get()
            .yaml
            .camera
            .paths
            .get(from_index)
            .context("Invalid backup index")?;
        for relative in relatives {
            pool.execute_to(
                tx.clone(),
                Thunk::of(move || {
                    log::start(i);
                    let result = process(backup, &relative);
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
