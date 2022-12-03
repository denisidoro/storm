use crate::smalldate::SmallDate;
use anyhow::{Context, Result};
use std::fmt::{self, Display};

pub type Kb = u32;

#[derive(Clone, Debug)]
pub struct File {
    pub filename: String,
    pub kb: Option<Kb>,
    pub date: Option<SmallDate>,
}

impl File {
    pub fn from_line(line: &str) -> Result<Self> {
        let mut parts = line.split(';');

        let filename = parts.next().context("no filename")?.into();

        let kb_str = parts.next().context("no kb")?;
        let kb = if kb_str.is_empty() {
            None
        } else {
            Some(kb_str.parse()?)
        };

        let date_str = parts.next().context("no date")?;
        let date = if date_str.is_empty() {
            None
        } else {
            Some(SmallDate::from_str(date_str)?)
        };

        let file = File { filename, kb, date };
        Ok(file)
    }
}

impl Display for File {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let txt = format!(
            "{};{};{}",
            self.filename,
            self.kb.map(|x| x.to_string()).unwrap_or_else(|| "".into()),
            self.date.map(|x| x.to_string()).unwrap_or_else(|| "".into())
        );
        fmt.write_str(&txt)
    }
}
