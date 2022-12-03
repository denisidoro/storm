use crate::log;
use crate::tasker;
use anyhow::Error;
use anyhow::Result;
use chrono::Timelike;
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::{Mutex, MutexGuard};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub struct Logger {
    label: String,
    n: usize,
    processing: HashSet<usize>,
}

impl Logger {
    fn step(&mut self, started: bool, i: usize, args: &[&str]) {
        if started {
            self.processing.insert(i);
        } else {
            self.processing.remove(&i);
        }

        let processing = self
            .processing
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let msg = format!(
            "{};{};{};{};{};{}",
            self.label,
            i,
            self.n,
            processing,
            self.processing.len(),
            args.join(";")
        );

        log::debug(&format!("=> {}", &msg));

        let result = tasker::maybe_log(&msg);
        if result.is_err() {
            log::error("failed to execute log task");
        }
    }
}

fn get() -> MutexGuard<'static, Logger> {
    INSTANCE
        .get()
        .expect("logger not initialized")
        .lock()
        .expect("failed to lock logger")
}

static INSTANCE: OnceCell<Mutex<Logger>> = OnceCell::new();

pub fn setup(label: String, n: usize) -> Result<()> {
    let logger = Logger {
        label,
        n,
        processing: HashSet::new(),
    };

    INSTANCE
        .set(Mutex::new(logger))
        .map_err(|_| anyhow!("unable to set logger"))?;

    Ok(())
}

pub fn start(i: usize) {
    get().step(true, i, &["start"]);
}

pub fn success(i: usize) {
    get().step(false, i, &["success"])
}

pub fn failure(i: usize, error: Error) {
    get().step(false, i, &["error", &error.to_string()])
}

fn write_raw(color: Color, msg: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(color)))?;

    let now = chrono::offset::Local::now();
    writeln!(
        &mut buffer,
        "{:02}:{:02}:{:02} {}",
        now.hour(),
        now.minute(),
        now.second(),
        msg
    )?;

    bufwtr.print(&buffer)?;

    let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();
    buffer.reset()?;
    bufwtr.print(&buffer)
}

fn write(color: Color, msg: &str) {
    write_raw(color, msg).expect("failed to write to terminal")
}

pub fn debug(msg: &str) {
    write(Color::Green, msg)
}

pub fn info(msg: &str) {
    write(Color::Cyan, msg)
}

pub fn warn(msg: &str) {
    write(Color::Yellow, msg)
}

pub fn error(msg: &str) {
    write(Color::Red, msg)
}
