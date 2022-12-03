use crate::shell::{self, ShellCmd};
use anyhow::Result;

pub fn open(uri: &str) -> Result<ShellCmd> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    shell::out(program, &[uri]).map_err(|e| e.into())
}
