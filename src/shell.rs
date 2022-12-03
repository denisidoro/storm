use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use thiserror::Error;
use which::which;

type V = Mutex<HashSet<String>>;

static VERIFIED: OnceCell<V> = OnceCell::new();

fn get_verified() -> &'static V {
    VERIFIED.get().expect("config not initialized")
}

#[derive(Debug, Clone)]
pub struct ShellRes {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct ShellCmd {
    pub program: String,
    pub args: Vec<String>,
    pub res: Option<ShellRes>,
}

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("Non-zero code: {cmd:?}")]
    NonZero { cmd: ShellCmd },
    #[error("Unable to get code")]
    Code,
    #[error("Unable to unwrap lock result")]
    Lock,
    #[error("Unable to get output message")]
    Stdout,
    #[error("Unable to get error message")]
    Stderr,
    #[error("Unable to wait")]
    Wait,
    #[error("Program not available: {program:?}, {source:?}")]
    ProgramNotAvailable {
        program: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("Unable to spawn: {cmd:?}, {source:?}")]
    Spawn {
        cmd: ShellCmd,
        #[source]
        source: io::Error,
    },
}

impl ShellRes {
    fn is_success(&self) -> bool {
        self.code == 0
    }
}

pub fn verify(program: &str) -> anyhow::Result<()> {
    let mut h = get_verified().lock().map_err(|_| ShellError::Lock)?;

    if h.contains(program) {
        return Ok(());
    }

    eprintln!("which {}", program);
    which(program)?;

    h.insert(program.into());

    Ok(())
}

pub fn setup() -> anyhow::Result<()> {
    let m = Mutex::new(HashSet::new());
    VERIFIED
        .set(m)
        .map_err(|_| anyhow!("unable to set verified map"))?;
    Ok(())
}

pub fn out_inherited(program: &str, args: &[&str]) -> Result<ShellCmd, ShellError> {
    out_extra(program, args, false)
}

pub fn out(program: &str, args: &[&str]) -> Result<ShellCmd, ShellError> {
    out_extra(program, args, true)
}

fn out_extra(program: &str, args: &[&str], piped: bool) -> Result<ShellCmd, ShellError> {
    verify(program).map_err(|source| ShellError::ProgramNotAvailable {
        program: program.into(),
        source,
    })?;

    let mut cmd = ShellCmd {
        program: program.into(),
        args: args.iter().map(|x| x.to_string()).collect(),
        res: None,
    };

    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stderr(if piped { Stdio::piped() } else { Stdio::inherit() })
        .stdout(if piped { Stdio::piped() } else { Stdio::inherit() })
        .spawn()
        .map_err(|e| ShellError::Spawn {
            source: e,
            cmd: cmd.clone(),
        })?
        .wait_with_output()
        .map_err(|_| ShellError::Wait)?;

    let stdout = String::from_utf8(out.stdout).map_err(|_| ShellError::Stdout)?;
    let stderr = String::from_utf8(out.stderr).map_err(|_| ShellError::Stderr)?;
    let code = out.status.code().ok_or(ShellError::Code)?;

    cmd.res = Some(ShellRes { code, stdout, stderr });

    if cmd.res.as_ref().expect("cmd.res is none").is_success() {
        Ok(cmd)
    } else {
        Err(ShellError::NonZero { cmd })
    }
}
