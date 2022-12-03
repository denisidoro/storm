use anyhow::Result;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct Header {
    pub(super) tags: HashMap<String, String>,
}

impl Header {
    pub(super) fn new() -> Self {
        Self { tags: HashMap::new() }
    }

    pub fn from_lines(lines: &mut dyn Iterator<Item = String>) -> Result<Self> {
        let mut header = Header::new();

        for line in lines {
            if line.starts_with('=') {
                break;
            }

            if line.trim().is_empty() {
                continue;
            }

            if !line.contains(':') {
                return Err(anyhow!("Invalid header line"));
            }

            let mut parts = line.split(':');
            let key = parts.next().expect("no key found").trim();
            let value = parts.next().expect("no value found").trim();
            header.tags.insert(key.into(), value.into());
        }

        Ok(header)
    }
}

impl Display for Header {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut keys = self.tags.keys().collect::<Vec<_>>();
        keys.sort();

        for k in keys {
            fmt.write_str(&format!("{}: {}\n", k, self.tags.get(k).expect("invalid tag")))?;
        }

        Ok(())
    }
}
