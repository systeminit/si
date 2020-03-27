use crate::error::{CeaError, CeaResult};
use crate::{EntityEvent, MqttAsyncClientInternal};
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

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

    pub fn success(self) -> CeaResult<CommandResult> {
        if self.exit_status.success() {
            Ok(self)
        } else {
            Err(CeaError::CommandFailed(self))
        }
    }

    pub fn try_stdout(&mut self) -> CeaResult<String> {
        self.stdout.take().ok_or(CeaError::CommandExpectedOutput)
    }

    pub fn stdout(&self) -> Option<&String> {
        self.stdout.as_ref()
    }

    pub fn try_stderr(&mut self) -> CeaResult<String> {
        self.stderr.take().ok_or(CeaError::CommandExpectedOutput)
    }

    pub fn stderr(&self) -> Option<&String> {
        self.stderr.as_ref()
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

async fn read_stdout(
    mut tx: mpsc::Sender<OutputLine>,
    io_pipe: tokio::process::ChildStdout,
) -> CeaResult<()> {
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
) -> CeaResult<()> {
    let buffer = BufReader::new(io_pipe);
    let mut lines = buffer.lines();
    while let Some(line) = lines.next_line().await? {
        tx.send(OutputLine::Stderr(line)).await?;
    }
    tx.send(OutputLine::Finished).await?;
    Ok(())
}

/// Spawns a `Command` with data for the standard input stream, indents the output stream contents,
/// and returns its `CommandResult`.
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
    mqtt_client: &MqttAsyncClientInternal,
    mut cmd: Command,
    entity_event: &mut impl EntityEvent,
    capture_output: CaptureOutput,
) -> CeaResult<CommandResult> {
    entity_event.log(format!("---- Running Command ----"));
    entity_event.log(format!("{:?}", cmd));
    entity_event.log(format!("---- Output ----"));
    entity_event.error_log(format!("---- Running Command ----"));
    entity_event.error_log(format!("{:?}", cmd));
    entity_event.error_log(format!("---- Error Output ----"));
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Close STDIN by dropping it
    let mut child = cmd.spawn()?;
    let stdin = child.stdin.take().ok_or(CeaError::NoIoPipe)?;
    drop(stdin);

    let (tx, mut rx) = mpsc::channel(100000);

    let stdout = child.stdout.take().ok_or(CeaError::NoIoPipe)?;
    let stdout_tx = tx.clone();
    let stdout_handle = tokio::spawn(read_stdout(stdout_tx, stdout));
    let stderr = child.stderr.take().ok_or(CeaError::NoIoPipe)?;
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
                    stdout_string.as_mut().unwrap().push_str(line.as_ref());
                }
                entity_event.log(line);
                entity_event.send_via_mqtt(mqtt_client).await?;
            }
            OutputLine::Stderr(line) => {
                if capture_output.stderr() {
                    stderr_string.as_mut().unwrap().push_str(line.as_ref());
                }
                entity_event.error_log(line);
                entity_event.send_via_mqtt(mqtt_client).await?;
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

    let child_status = child_result.map_err(CeaError::IO)?;
    entity_event.log(format!("---- Finished Command ----"));
    entity_event.error_log(format!("---- Finished Command ----"));

    Ok(CommandResult::new(
        child_status,
        stdout_string,
        stderr_string,
    ))
}
