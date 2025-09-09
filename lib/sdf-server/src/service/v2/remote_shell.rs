use axum::{
    Json,
    Router,
    extract::{Host, OriginalUri, Path, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{post, get},
};
use dal::{
    TransactionsError,
};
use si_db;
use sdf_core::api_error::ApiError;
use sdf_extract::{
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use chrono;
use serde_json;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use std::process::Stdio;
use ulid::Ulid;
use veritech_client::{
    ClientError,
    FunctionResult,
    OutputStream,
    RemoteShellConnectionInfo,
    RemoteShellRequest,
    RemoteShellResultSuccess,
    RemoteShellStatus,
};

use crate::{
    AppState,
    extract::PosthogClient,
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

pub type RemoteShellApiResult<T> = Result<T, RemoteShellApiError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum RemoteShellApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("failed to receive output stream")]
    OutputReceiver,
    #[error("serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] ClientError),
}

impl IntoResponse for RemoteShellApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            RemoteShellApiError::VeritechClient(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        ApiError::new(status_code, error_message).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRemoteShellSessionRequest {
    pub image: Option<String>,
    pub working_dir: Option<String>,
    pub env_vars: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRemoteShellSessionResponse {
    pub execution_id: String,
    pub session_id: String,
    pub container_id: String,
    pub connection_info: RemoteShellConnectionInfo,
    pub status: RemoteShellStatus,
    pub message: Option<String>,
}

impl From<RemoteShellResultSuccess> for CreateRemoteShellSessionResponse {
    fn from(result: RemoteShellResultSuccess) -> Self {
        Self {
            execution_id: result.execution_id,
            session_id: result.session_id,
            container_id: result.container_id,
            connection_info: result.connection_info,
            status: result.status,
            message: result.message,
        }
    }
}

#[instrument(
    name = "remote_shell.create_session",
    level = "info",
    skip_all,
)]
pub async fn create_session(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateRemoteShellSessionRequest>,
) -> RemoteShellApiResult<ForceChangeSetResponse<CreateRemoteShellSessionResponse>> {
    let force_change_set_id = dal::ChangeSet::force_new(ctx).await?;
    
    // Generate execution ID
    let execution_id = Ulid::new().to_string();

    // Create the remote shell request
    let remote_shell_request = RemoteShellRequest {
        execution_id: execution_id.clone(),
        image: request.image,
        env_vars: request.env_vars.unwrap_or_default(),
        working_dir: request.working_dir,
    };

    // Create output channel (required by veritech client but we don't need the stream for this API)
    let (output_tx, mut _output_rx) = mpsc::channel::<OutputStream>(32);

    // Execute the remote shell request via veritech
    let workspace_id = ctx.tenancy().workspace_pk()?.to_string();
    let change_set_id = ctx.change_set_id().to_string();

    let function_result = ctx
        .veritech()
        .execute_remote_shell(
            output_tx,
            &remote_shell_request,
            &workspace_id,
            &change_set_id,
        )
        .await?;

    // Process the result
    let response = match function_result {
        FunctionResult::Success(result) => {
            info!(
                execution_id = %execution_id,
                session_id = %result.session_id,
                "remote shell session created successfully"
            );
            
            CreateRemoteShellSessionResponse::from(result)
        }
        FunctionResult::Failure(failure) => {
            let error_message = failure.error().message.clone();
            warn!(
                execution_id = %execution_id,
                error = %error_message,
                "remote shell session creation failed"
            );
            
            CreateRemoteShellSessionResponse {
                execution_id: execution_id.clone(),
                session_id: format!("failed_{}", execution_id),
                container_id: String::new(),
                connection_info: RemoteShellConnectionInfo {
                    nats_subject: String::new(),
                    stdin_subject: String::new(),
                    stdout_subject: String::new(),
                    stderr_subject: String::new(),
                    control_subject: String::new(),
                },
                status: RemoteShellStatus::Error,
                message: Some(error_message),
            }
        }
    };

    // Track the event
    track(
        &posthog_client,
        ctx,
        &original_uri,
        &host_name,
        "create_remote_shell_session",
        serde_json::json!({
            "how": "/remote-shell/create",
            "execution_id": execution_id,
            "status": response.status,
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, response))
}

pub async fn connect_shell_websocket(
    ws: WebSocketUpgrade,
    Path((workspace_id, change_set_id, session_id)): Path<(String, String, String)>,
) -> Response {
    info!(
        workspace_id = %workspace_id,
        change_set_id = %change_set_id, 
        session_id = %session_id, 
        "Remote shell WebSocket connection requested"
    );
    
    ws.on_upgrade(move |socket| handle_shell_websocket_wrapper(socket, session_id))
}

async fn handle_shell_websocket_wrapper(socket: WebSocket, session_id: String) {
    info!(session_id = %session_id, "WebSocket wrapper called");
    if let Err(err) = handle_shell_websocket(socket, session_id.clone()).await {
        error!(session_id = %session_id, error = %err, "WebSocket handler failed");
    }
}

enum ShellMode {
    Interactive,  // For commands like claude, vim, etc.
    OneShot,     // For commands like ls, ps, etc.
}

fn detect_shell_mode(command: &str) -> ShellMode {
    let cmd_parts: Vec<&str> = command.trim().split_whitespace().collect();
    if cmd_parts.is_empty() {
        return ShellMode::OneShot;
    }
    
    let interactive_commands = [
        "claude", "vim", "nvim", "nano", "emacs", 
        "less", "more", "top", "htop", "python", "node", 
        "irb", "rails", "ssh", "telnet", "mysql", "psql"
    ];
    
    // Check if this is an interactive command
    if interactive_commands.iter().any(|&ic| cmd_parts[0] == ic) {
        ShellMode::Interactive
    } else {
        ShellMode::OneShot
    }
}

fn prepare_interactive_command(command: &str) -> String {
    // For now, just return the command as-is
    // The PTY provided by 'script' should be enough for most interactive programs
    command.to_string()
}

async fn handle_shell_websocket(mut socket: WebSocket, session_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(session_id = %session_id, "Remote shell WebSocket connected");
    
    // Terminal dimensions - start with defaults, will be updated by client
    let terminal_cols = 120;
    let terminal_rows = 30;
    
    // Session timeout - 10 minutes of inactivity
    let session_timeout = tokio::time::Duration::from_secs(10 * 60);
    let mut last_activity = tokio::time::Instant::now();
    

    // Track any running interactive process
    let mut interactive_process: Option<(tokio::process::Child, tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>, mpsc::Receiver<String>)> = None;
    let mut interactive_stdin: Option<tokio::process::ChildStdin> = None;

    
    // Start a normal shell (bash)
    if let Err(e) = start_interactive_command_with_dimensions(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin, terminal_cols, terminal_rows).await {
        error!(error = %e, "Failed to start shell");
        socket.send(Message::Text(format!(
            "{{\"type\":\"system\",\"content\":\"Failed to start shell: {}\\n\",\"timestamp\":\"{}\"}}",
            e,
            chrono::Utc::now().to_rfc3339()
        ))).await?;
    }

    // Handle WebSocket messages and interactive output
    loop {
        tokio::select! {
            // Handle WebSocket messages
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        // Parse input from frontend
                        if let Ok(cmd_data) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Handle raw character input (from xterm.js onData)
                            if let Some(input) = cmd_data.get("input").and_then(|i| i.as_str()) {
                                // Update activity timestamp
                                last_activity = tokio::time::Instant::now();
                                
                                if let Some(ref mut stdin) = interactive_stdin {
                                    info!("Sending raw input to shell: {:?}", input);
                                    
                                    // Send raw input directly - no processing needed
                                    let input_bytes = input.as_bytes().to_vec();
                                    
                                    if let Err(e) = stdin.write_all(&input_bytes).await {
                                        error!(error = %e, "Failed to write to shell");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart shell
                                        if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart shell");
                                        }
                                    } else if let Err(e) = stdin.flush().await {
                                        error!(error = %e, "Failed to flush shell input");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart shell
                                        if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart shell");
                                        }
                                    } else {
                                        info!("Successfully sent raw input to shell");
                                    }
                                } else {
                                    // No active shell process - restart it
                                    if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                        error!(error = %e, "Failed to start shell");
                                    }
                                }
                            }
                            // Handle terminal resize messages
                            else if cmd_data.get("type").and_then(|t| t.as_str()) == Some("resize") {
                                if let (Some(cols), Some(rows)) = (
                                    cmd_data.get("cols").and_then(|c| c.as_u64()),
                                    cmd_data.get("rows").and_then(|r| r.as_u64())
                                ) {
                                    info!("Terminal resize requested: {}x{}", cols, rows);
                                    
                                    // Note: For most shells, we don't need to restart on resize.
                                    // The COLUMNS/LINES environment variables are set at startup.
                                    // In a full PTY implementation, you'd send SIGWINCH to the child process.
                                }
                            }
                            // Handle full commands (legacy support)
                            else if let Some(command) = cmd_data.get("command").and_then(|c| c.as_str()) {
                                let cmd = command.trim();
                                
                                // All input goes to shell (we always have an interactive process)
                                if let Some(ref mut stdin) = interactive_stdin {
                                    info!("Sending command to shell: {}", cmd);
                                    
                                    // Handle special control characters
                                    let input_bytes = if cmd == "\x03" {
                                        // Ctrl+C - send as-is without newline
                                        cmd.as_bytes().to_vec()
                                    } else {
                                        // Regular input - add newline
                                        format!("{}\n", cmd).into_bytes()
                                    };
                                    
                                    if let Err(e) = stdin.write_all(&input_bytes).await {
                                        error!(error = %e, "Failed to write to shell");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart shell
                                        if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart shell");
                                        }
                                    } else if let Err(e) = stdin.flush().await {
                                        error!(error = %e, "Failed to flush shell input");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart shell
                                        if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart shell");
                                        }
                                    } else {
                                        info!("Successfully sent input to shell");
                                    }
                                } else {
                                    // No active shell process - restart it
                                    if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                        error!(error = %e, "Failed to start shell");
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!(session_id = %session_id, "Remote shell WebSocket closed by client");
                        break;
                    }
                    Some(Err(err)) => {
                        warn!(error = %err, "WebSocket error");
                        break;
                    }
                    None => break,
                    _ => {} // Ignore other message types
                }
            }

            // Handle interactive process output
            interactive_output = async {
                if let Some((_, _, _, rx)) = interactive_process.as_mut() {
                    rx.recv().await
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    None
                }
            } => {
                if let Some(output_msg) = interactive_output {
                    if let Err(e) = socket.send(Message::Text(output_msg)).await {
                        error!(error = %e, "Failed to send interactive output to WebSocket");
                        break;
                    }
                } else if interactive_process.is_some() {
                    // Check if interactive process has exited
                    if let Some((child, _, _, _)) = interactive_process.as_mut() {
                        match child.try_wait() {
                            Ok(Some(_status)) => {
                                // shell has exited - restart it
                                info!("Shell has exited, restarting...");
                                let restart_msg = format!(
                                    "{{\"type\":\"system\",\"content\":\"Shell exited, restarting...\\n\",\"timestamp\":\"{}\"}}",
                                    chrono::Utc::now().to_rfc3339()
                                );
                                let _ = socket.send(Message::Text(restart_msg)).await;
                                
                                // Clean up
                                if let Some((mut child, stdout_task, stderr_task, _rx)) = interactive_process.take() {
                                    stdout_task.abort();
                                    stderr_task.abort();
                                    let _ = child.kill().await;
                                }
                                interactive_stdin = None;
                                
                                // Restart shell
                                if let Err(e) = start_interactive_command(&mut socket, "bash", &mut interactive_process, &mut interactive_stdin).await {
                                    error!(error = %e, "Failed to restart shell");
                                    let error_msg = format!(
                                        "{{\"type\":\"system\",\"content\":\"Failed to restart shell: {}\\n\",\"timestamp\":\"{}\"}}",
                                        e,
                                        chrono::Utc::now().to_rfc3339()
                                    );
                                    let _ = socket.send(Message::Text(error_msg)).await;
                                }
                            }
                            _ => {} // Still running or error checking
                        }
                    }
                }
            }
            
            // Check for session timeout (10 minutes of inactivity)
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                if last_activity.elapsed() >= session_timeout {
                    info!(session_id = %session_id, "Session timed out after 10 minutes of inactivity");
                    let timeout_msg = format!(
                        "{{\"type\":\"system\",\"content\":\"Session timed out after 10 minutes of inactivity. Closing connection.\\n\",\"timestamp\":\"{}\"}}",
                        chrono::Utc::now().to_rfc3339()
                    );
                    let _ = socket.send(Message::Text(timeout_msg)).await;
                    break;
                }
            }
        }
    }

    // Clean up any running interactive process
    if let Some((mut child, stdout_task, stderr_task, _rx)) = interactive_process {
        stdout_task.abort();
        stderr_task.abort();
        let _ = child.kill().await;
    }
    
    info!(session_id = %session_id, "Remote shell WebSocket disconnected");
    Ok(())
}

async fn start_interactive_command_with_dimensions(
    socket: &mut WebSocket,
    command: &str,
    interactive_process: &mut Option<(tokio::process::Child, tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>, mpsc::Receiver<String>)>,
    interactive_stdin: &mut Option<tokio::process::ChildStdin>,
    cols: u64,
    rows: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if we already have a running process - don't start duplicates
    if let Some((child, _, _, _)) = interactive_process.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                info!("Shell process {} already running, not starting duplicate", child.id().unwrap_or(0));
                return Ok(());
            }
            Ok(Some(status)) => {
                info!("Previous shell process exited with status: {:?}", status);
            }
            Err(e) => {
                info!("Error checking shell process status: {}, will restart", e);
            }
        }
    }

    // Clean up any existing interactive process
    if let Some((mut child, stdout_task, stderr_task, _rx)) = interactive_process.take() {
        stdout_task.abort();
        stderr_task.abort();
        let _ = child.kill().await;
    }
    *interactive_stdin = None;

    // Prepare the command with any special handling
    let _prepared_command = prepare_interactive_command(command);
    
    // Interactive commands need a proper terminal environment - try different approaches
    info!("Starting {} with proper terminal emulation", command);
    
    // Use script command to create proper PTY for interactive shell
    let child = if command == "bash" {
        // Use script with immediate flush to create proper PTY
        Command::new("script")
            .arg("-qfe")   // -q quiet, -f flush output immediately, -e return exit code
            .arg("-c")     // -c command to run
            .arg("bash --norc --noprofile")
            .arg("/dev/null")  // Don't create log file
            .env("TERM", "xterm-256color")
            .env("COLUMNS", &cols.to_string()) 
            .env("LINES", &rows.to_string())
            .env("LANG", "en_US.UTF-8")
            .env("PS1", r"\u@\h:\w$ ")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) 
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new(command)
            .env("TERM", "xterm-256color")
            .env("COLUMNS", &cols.to_string()) 
            .env("LINES", &rows.to_string())
            .env("LANG", "en_US.UTF-8")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) 
            .stderr(Stdio::piped())
            .spawn()
    };

    let mut child = match child {
        Ok(c) => {
            info!("{} started directly, PID: {:?}", command, c.id());
            c
        }
        Err(e) => {
            error!("Failed to start {} directly: {}, trying script", command, e);
            // Fallback - already using script as primary method, so this shouldn't happen
            // But keep it as backup
            let script_cmd = if command == "bash" {
                "script -qfe -c 'bash --norc --noprofile' /dev/null".to_string()
            } else {
                format!("script -qfe -c '{}' /dev/null", command)
            };
            let script_child = Command::new("bash")
                .arg("-c")
                .arg(&script_cmd)
                .env("TERM", "xterm-256color")
                .env("COLUMNS", &cols.to_string())
                .env("LINES", &rows.to_string())
                .env("LANG", "en_US.UTF-8")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
                
            match script_child {
                Ok(c) => {
                    info!("{} started with script, PID: {:?}", command, c.id());
                    c
                }
                Err(e) => {
                    error!("Failed to start {} with script: {}, trying direct", command, e);
                    // Last resort - direct bash without PTY (will be limited)
                    if command == "bash" {
                        Command::new("bash")
                            .arg("--norc")
                            .arg("--noprofile") 
                            .arg("-i")
                            .env("TERM", "xterm-256color")
                            .env("COLUMNS", &cols.to_string())
                            .env("LINES", &rows.to_string()) 
                            .env("LANG", "en_US.UTF-8")
                            .env("PS1", "$ ")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?
                    } else {
                        Command::new(command)
                            .env("TERM", "xterm-256color")
                            .env("COLUMNS", &cols.to_string())
                            .env("LINES", &rows.to_string()) 
                            .env("LANG", "en_US.UTF-8")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?
                    }
                }
            }
        }
    };

    let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    
    // Create channels for communication
    let (tx, rx) = mpsc::channel::<String>(32);

    // Spawn task to handle stdout with better buffering
    let socket_tx1 = tx.clone();
    let stdout_task = tokio::spawn(async move {
        info!("Started stdout reader task for shell");
        let mut stdout_reader = BufReader::new(stdout);
        let mut buffer = [0u8; 1024]; // Larger buffer but process more responsively
        
        loop {
            match stdout_reader.read(&mut buffer).await {
                Ok(0) => {
                    info!("shell stdout EOF reached");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    info!("Received stdout chunk ({} bytes): {:?}", n, chunk);
                    
                    // Send raw chunks immediately - use serde_json for proper escaping
                    let message = serde_json::json!({
                        "type": "stdout",
                        "content": chunk.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    let msg = message.to_string();
                    if socket_tx1.send(msg).await.is_err() {
                        error!("Failed to send stdout message to WebSocket");
                        return;
                    }
                }
                Err(e) => {
                    error!("Error reading shell stdout: {}", e);
                    break;
                }
            }
        }
        info!("shell stdout reader task ended");
    });

    // Spawn task to handle stderr
    let socket_tx2 = tx.clone();
    let stderr_task = tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        info!("Started stderr reader task for shell");
        let mut stderr_reader = BufReader::new(stderr);
        let mut line = String::new();
        
        loop {
            line.clear();
            match stderr_reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("shell stderr EOF reached");
                    break; // EOF
                }
                Ok(_) => {
                    let line_content = line.trim_end();
                    
                    // Filter out common bash startup warnings/errors that are harmless but noisy
                    let should_suppress = line_content.contains("cannot set terminal process group") ||
                                        line_content.contains("no job control in this shell") ||
                                        line_content.contains("bash: cannot set terminal process group") ||
                                        line_content.contains("bash: no job control") ||
                                        line_content.contains("_completion_loader:") ||
                                        line_content.contains("bash: complete: command not found") ||
                                        line_content.contains("bash: shopt:") ||
                                        line_content.contains("invalid shell option name") ||
                                        line_content.contains("progcomp") ||
                                        line_content.contains("hostcomplete");
                    
                    if !should_suppress && !line_content.is_empty() {
                        info!("Received stderr line: {:?}", line_content);
                        let message = serde_json::json!({
                            "type": "stderr", 
                            "content": line_content,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });
                        let msg = message.to_string();
                        if socket_tx2.send(msg).await.is_err() {
                            error!("Failed to send stderr message to WebSocket");
                            break;
                        }
                    } else {
                        // Log suppressed warnings for debugging but don't send to client
                        if should_suppress {
                            info!("Suppressed bash warning: {:?}", line_content);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading shell stderr: {}", e);
                    break;
                }
            }
        }
        info!("shell stderr reader task ended");
    });


    // Give Claude a moment to initialize and send initial output
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Check if the process is still running
    match child.try_wait() {
        Ok(Some(status)) => {
            error!("Shell exited immediately with status: {:?}", status);
            socket.send(Message::Text(format!(
                "{{\"type\":\"system\",\"content\":\"Shell exited immediately: {:?}\\n\",\"timestamp\":\"{}\"}}",
                status,
                chrono::Utc::now().to_rfc3339()
            ))).await?;
        }
        Ok(None) => {
            info!("Shell is running");
            
            // Send terminal banner and initial prompt
            if let Some(ref mut stdin) = *interactive_stdin {
                // Wait a moment for bash to be ready
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // Send the welcome banner directly to WebSocket first to test
                let banner_msg = serde_json::json!({
                    "type": "stdout",
                    "content": "\n╭─────────────────────────────────────────────────────────────╮\n│  ____            _                   ___       _ _   _       │\n│ / ___| _   _ ___| |_ ___ _ __ ___    |_ _|_ __ (_) |_(_) __ _ │\n│ \\___ \\| | | / __| __/ _ \\ '_ ` _ \\    | || '_ \\| | __| |/ _` |│\n│  ___) | |_| \\__ \\ ||  __/ | | | | |  | || | | | | |_| | (_| |│\n│ |____/ \\__, |___/\\__\\___|_| |_| |_| |___|_| |_|_|\\__|_|\\__,_|│\n│        |___/                                                 │\n│                                                             │\n│  🚀 System Initiative Managed Terminal                     │\n│                                                             │\n│  📋 Available Tools:                                        │\n│     • claude      - Claude Code CLI assistant              │\n│     • aws         - AWS CLI tools                          │\n│     • docker      - Container management                   │\n│     • git         - Version control                        │\n│                                                             │\n│  ⚠️  Notice:                                                │\n│     • No persistence between sessions                      │\n│     • Files are temporary and will be lost                 │\n│     • Use for development and testing only                 │\n│                                                             │\n│  💡 Get started: try 'echo hello world' or 'ls'            │\n╰─────────────────────────────────────────────────────────────╯\n\n",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                if let Err(e) = socket.send(Message::Text(banner_msg.to_string())).await {
                    error!("Failed to send banner to WebSocket: {}", e);
                } else {
                    info!("Sent banner directly to WebSocket");
                }
                
                // Send a test command to verify bash is working
                let test_cmd = "echo 'Shell is ready - type commands here:'\n";
                if let Err(e) = stdin.write_all(test_cmd.as_bytes()).await {
                    error!("Failed to send test command to shell: {}", e);
                } else if let Err(e) = stdin.flush().await {
                    error!("Failed to flush test command: {}", e);
                } else {
                    info!("Sent test command to bash");
                }
            }
        }
        Err(e) => {
            error!("Error checking shell status: {}", e);
        }
    }

    *interactive_process = Some((child, stdout_task, stderr_task, rx));
    *interactive_stdin = Some(stdin);

    Ok(())
}

async fn send_terminal_banner(stdin: &mut tokio::process::ChildStdin) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let banner_lines = vec![
        "",
        "╭─────────────────────────────────────────────────────────────╮",
        "│  ____            _                   ___       _ _   _       │",
        "│ / ___| _   _ ___| |_ ___ _ __ ___    |_ _|_ __ (_) |_(_) __ _ │",
        "│ \\___ \\| | | / __| __/ _ \\ '_ ` _ \\    | || '_ \\| | __| |/ _` |│",
        "│  ___) | |_| \\__ \\ ||  __/ | | | | |  | || | | | | |_| | (_| |│",
        "│ |____/ \\__, |___/\\__\\___|_| |_| |_| |___|_| |_|_|\\__|_|\\__,_|│",
        "│        |___/                                                 │",
        "│                                                             │",
        "│  🚀 System Initiative Managed Terminal                     │",
        "│                                                             │",
        "│  📋 Available Tools:                                        │",
        "│     • claude      - Claude Code CLI assistant              │",
        "│     • aws         - AWS CLI tools                          │",
        "│     • docker      - Container management                   │",
        "│     • git         - Version control                        │",
        "│                                                             │",
        "│  ⚠️  Notice:                                                │",
        "│     • No persistence between sessions                      │",
        "│     • Files are temporary and will be lost                 │",
        "│     • Use for development and testing only                 │",
        "│                                                             │",
        "│  💡 Get started: type 'claude' to launch Claude Code CLI   │",
        "╰─────────────────────────────────────────────────────────────╯",
        "",
    ];

    for line in banner_lines {
        stdin.write_all(format!("echo '{}'\n", line).as_bytes()).await?;
    }
    stdin.flush().await?;
    
    Ok(())
}

async fn start_interactive_command(
    socket: &mut WebSocket,
    command: &str,
    interactive_process: &mut Option<(tokio::process::Child, tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>, mpsc::Receiver<String>)>,
    interactive_stdin: &mut Option<tokio::process::ChildStdin>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use default dimensions
    start_interactive_command_with_dimensions(socket, command, interactive_process, interactive_stdin, 120, 30).await
}

async fn execute_test_command(socket: &mut WebSocket) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing test command: echo");
    let mut child = Command::new("echo")
        .arg("Test message from server")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    // Read all output
    AsyncReadExt::read_to_end(&mut stdout_reader, &mut stdout_buf).await?;
    AsyncReadExt::read_to_end(&mut stderr_reader, &mut stderr_buf).await?;

    // Wait for process to complete
    let status = child.wait().await?;
    info!("Test command completed with status: {:?}", status);

    // Send stdout if any
    if !stdout_buf.is_empty() {
        let stdout_text = String::from_utf8_lossy(&stdout_buf);
        info!("Test command stdout: {}", stdout_text);
        let stdout_msg = format!(
            "{{\"type\":\"stdout\",\"content\":\"{}\",\"timestamp\":\"{}\"}}",
            stdout_text.replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r"),
            chrono::Utc::now().to_rfc3339()
        );
        socket.send(Message::Text(stdout_msg)).await?;
    }

    Ok(())
}

async fn execute_oneshot_command(socket: &mut WebSocket, command: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Handle built-in commands
    match command {
        "clear" => {
            let clear_msg = format!(
                "{{\"type\":\"stdout\",\"content\":\"\\u001b[2J\\u001b[H\",\"timestamp\":\"{}\"}}",
                chrono::Utc::now().to_rfc3339()
            );
            socket.send(Message::Text(clear_msg)).await?;
            return Ok(());
        },
        _ => {}
    }
    
    // Execute the command
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    // Read all output
    AsyncReadExt::read_to_end(&mut stdout_reader, &mut stdout_buf).await?;
    AsyncReadExt::read_to_end(&mut stderr_reader, &mut stderr_buf).await?;

    // Wait for process to complete
    let status = child.wait().await?;

    // Send stdout if any
    if !stdout_buf.is_empty() {
        let stdout_text = String::from_utf8_lossy(&stdout_buf);
        let stdout_msg = format!(
            "{{\"type\":\"stdout\",\"content\":\"{}\",\"timestamp\":\"{}\"}}",
            stdout_text.replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r"),
            chrono::Utc::now().to_rfc3339()
        );
        socket.send(Message::Text(stdout_msg)).await?;
    }

    // Send stderr if any
    if !stderr_buf.is_empty() {
        let stderr_text = String::from_utf8_lossy(&stderr_buf);
        let stderr_msg = format!(
            "{{\"type\":\"stderr\",\"content\":\"{}\",\"timestamp\":\"{}\"}}",
            stderr_text.replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r"),
            chrono::Utc::now().to_rfc3339()
        );
        socket.send(Message::Text(stderr_msg)).await?;
    }

    // If command failed, send exit code
    if !status.success() {
        if let Some(code) = status.code() {
            let exit_msg = format!(
                "{{\"type\":\"system\",\"content\":\"Command exited with code {}\\n\",\"timestamp\":\"{}\"}}",
                code,
                chrono::Utc::now().to_rfc3339()
            );
            socket.send(Message::Text(exit_msg)).await?;
        }
    }

    Ok(())
}



pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_session))
        .route("/test/:session_id", get(test_websocket_endpoint))
        .route("/connect/:session_id", get(connect_shell_websocket))
}

pub async fn test_websocket_endpoint(
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!(session_id = %session_id, "Test endpoint called");
    (StatusCode::OK, format!("Test endpoint for session: {}", session_id))
}