use std::{
    io,
    process::{ExitStatus, Stdio},
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};

/// Spawns a `Command`, manages the output streams, and returns its `CommandResult`.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The command failed to spawn
/// * One of the I/O streams failed to be properly captured
/// * One of the output-reading threads panics
/// * The command wasn't running
pub async fn spawn_command(
    cmd: Command,
    capture_output: CaptureOutput,
    tx: mpsc::Sender<OutputLine>,
) -> Result<CommandResult, Error> {
    spawn_command_with_stdin(cmd, capture_output, None, tx).await
}

/// Spawns a `Command` with data for the standard input stream, manages the output streams, and
/// returns its `CommandResult`.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The command failed to spawn
/// * One of the I/O streams failed to be properly captured
/// * One of the output-reading threads panics
/// * The command wasn't running
pub async fn spawn_command_with_stdin(
    mut cmd: Command,
    capture_output: CaptureOutput,
    stdin_bytes: Option<Vec<u8>>,
    tx: mpsc::Sender<OutputLine>,
) -> Result<CommandResult, Error> {
    let mut w = StreamWriter { tx };

    w.stdout("---- Running Command ----").await?;
    w.stdout(format!("{:?}", cmd)).await?;
    w.stdout("---- Output ----").await?;
    w.stderr("---- Running Command ----").await?;
    w.stderr(format!("{:?}", cmd)).await?;
    w.stderr("---- Error Output ----").await?;

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.take().ok_or(Error::NoIoPipe)?;
    if let Some(stdin_bytes) = stdin_bytes {
        stdin.write_all(stdin_bytes.as_ref()).await?;
    }
    drop(stdin);

    let (tx, mut rx) = mpsc::channel(100000);

    let stdout = child.stdout.take().ok_or(Error::NoIoPipe)?;
    let stdout_tx = tx.clone();
    let stdout_handle = tokio::spawn(read_stdout(stdout_tx, stdout));
    let stderr = child.stderr.take().ok_or(Error::NoIoPipe)?;
    let stderr_handle = tokio::spawn(read_stderr(tx, stderr));

    let mut stdout_string = if capture_output.stdout() {
        Some(String::new())
    } else {
        None
    };
    let mut stderr_string = if capture_output.stderr() {
        Some(String::new())
    } else {
        None
    };
    let mut finished_count: isize = 0;

    while let Some(output) = rx.recv().await {
        match output {
            OutputLine::Stdout(line) => {
                if capture_output.stdout() {
                    stdout_string
                        .as_mut()
                        .expect("mutable reference exists")
                        .push_str(line.as_ref());
                }
                w.stdout(line).await?;
            }
            OutputLine::Stderr(line) => {
                if capture_output.stderr() {
                    stderr_string
                        .as_mut()
                        .expect("mutable reference exists")
                        .push_str(line.as_ref());
                }
                w.stderr(line).await?;
            }
            OutputLine::Finished => {
                finished_count = finished_count + 1;
                if finished_count >= 2 {
                    break;
                }
            }
        }
    }

    let (_stdout_result, _stderr_result, child_result) =
        tokio::join!(stdout_handle, stderr_handle, child);

    let child_status = child_result.map_err(Error::IO)?;
    w.stdout("---- Finished Command ----").await?;
    w.stderr("---- Finished Command ----").await?;

    Ok(CommandResult::new(
        child_status,
        stdout_string,
        stderr_string,
    ))
}

pub enum CaptureOutput {
    None,
    Stdout,
    Stderr,
    Both,
}

impl CaptureOutput {
    pub fn stdout(&self) -> bool {
        match self {
            CaptureOutput::Stdout | CaptureOutput::Both => true,
            _ => false,
        }
    }

    pub fn stderr(&self) -> bool {
        match self {
            CaptureOutput::Stderr | CaptureOutput::Both => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    exit_status: ExitStatus,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl CommandResult {
    pub fn new(
        exit_status: ExitStatus,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> CommandResult {
        CommandResult {
            exit_status,
            stdout,
            stderr,
        }
    }

    pub fn success(self) -> Result<CommandResult, Error> {
        if self.exit_status.success() {
            Ok(self)
        } else {
            Err(Error::Command(self))
        }
    }

    pub fn try_stdout(&mut self) -> Result<String, Error> {
        self.stdout.take().ok_or(Error::ExpectedOutput)
    }

    pub fn stdout(&self) -> Option<&String> {
        self.stdout.as_ref()
    }

    pub fn try_stderr(&mut self) -> Result<String, Error> {
        self.stderr.take().ok_or(Error::ExpectedOutput)
    }

    pub fn stderr(&self) -> Option<&String> {
        self.stderr.as_ref()
    }

    pub fn into_outputs(self) -> (Option<String>, Option<String>) {
        (self.stdout, self.stderr)
    }
}

impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command Result Code: {}\n", self.exit_status)
    }
}

#[derive(Debug)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
    Finished,
}

impl OutputLine {
    fn stdout(line: impl Into<String>) -> Self {
        Self::Stdout(line.into())
    }

    fn stderr(line: impl Into<String>) -> Self {
        Self::Stderr(line.into())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("command expected output, and has none")]
    ExpectedOutput,
    #[error("command failed: {0}")]
    Command(CommandResult),
    #[error("i/o error: {0}")]
    IO(#[from] io::Error),
    #[error("no i/o pipe during command call")]
    NoIoPipe,
    #[error("output send error: {0}")]
    OutputSend(#[from] mpsc::error::SendError<OutputLine>),
}

struct StreamWriter {
    tx: mpsc::Sender<OutputLine>,
}

impl StreamWriter {
    async fn stdout(&mut self, line: impl Into<String>) -> Result<(), Error> {
        self.tx
            .send(OutputLine::stdout(line))
            .await
            .map_err(From::from)
    }

    async fn stderr(&mut self, line: impl Into<String>) -> Result<(), Error> {
        self.tx
            .send(OutputLine::stderr(line))
            .await
            .map_err(From::from)
    }
}

async fn read_stdout(
    mut tx: mpsc::Sender<OutputLine>,
    io_pipe: tokio::process::ChildStdout,
) -> Result<(), Error> {
    let buffer = BufReader::new(io_pipe);
    let mut lines = buffer.lines();
    while let Some(line) = lines.next_line().await? {
        tx.send(OutputLine::Stdout(line)).await?;
    }
    tx.send(OutputLine::Finished).await?;
    Ok(())
}

async fn read_stderr(
    mut tx: mpsc::Sender<OutputLine>,
    io_pipe: tokio::process::ChildStderr,
) -> Result<(), Error> {
    let buffer = BufReader::new(io_pipe);
    let mut lines = buffer.lines();
    while let Some(line) = lines.next_line().await? {
        tx.send(OutputLine::Stderr(line)).await?;
    }
    tx.send(OutputLine::Finished).await?;
    Ok(())
}
