use crate::config;
use crate::url;
use anyhow::Result;
use std::fmt;
use std::thread;

enum Task {
    Log,
}

fn log_task() -> Option<&'static String> {
    config::get().yaml.tasker.log_task.as_ref()
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Log => log_task().expect("Invalid task"),
        };
        write!(f, "{}", s)
    }
}

pub fn maybe_log(data: &str) -> Result<()> {
    if log_task().is_some() {
        execute(Task::Log, Some(data), None)?;
    }
    Ok(())
}

fn execute(task: Task, par1: Option<&str>, par2: Option<&str>) -> Result<()> {
    let uri = format!(
        "tasker://assistantactions?task={}&par1={}&par2={}",
        task,
        par1.unwrap_or(""),
        par2.unwrap_or("")
    );

    thread::spawn(move || url::open(&uri));

    Ok(())
}
