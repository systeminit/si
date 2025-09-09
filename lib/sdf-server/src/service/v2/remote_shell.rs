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
    let mut terminal_cols = 120;
    let mut terminal_rows = 30;
    
    // Send welcome message
    socket.send(Message::Text(format!(
        "{{\"type\":\"system\",\"content\":\"Connected to Claude CLI session: {}\\n\",\"timestamp\":\"{}\"}}",
        session_id,
        chrono::Utc::now().to_rfc3339()
    ))).await?;
    
    // Send startup message
    socket.send(Message::Text(format!(
        "{{\"type\":\"system\",\"content\":\"Starting Claude CLI...\\n\",\"timestamp\":\"{}\"}}",
        chrono::Utc::now().to_rfc3339()
    ))).await?;

    // Track any running interactive process
    let mut interactive_process: Option<(tokio::process::Child, tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>, mpsc::Receiver<String>)> = None;
    let mut interactive_stdin: Option<tokio::process::ChildStdin> = None;

    // Test with a simple command first to debug the issue
    socket.send(Message::Text(format!(
        "{{\"type\":\"system\",\"content\":\"Testing basic command execution...\\n\",\"timestamp\":\"{}\"}}",
        chrono::Utc::now().to_rfc3339()
    ))).await?;
    
    // Try a simple echo command first to verify our piping works
    if let Err(e) = execute_test_command(&mut socket).await {
        error!(error = %e, "Failed to run test command");
    }
    
    // Now try to start Claude CLI
    if let Err(e) = start_interactive_command_with_dimensions(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin, terminal_cols, terminal_rows).await {
        error!(error = %e, "Failed to auto-start Claude CLI");
        socket.send(Message::Text(format!(
            "{{\"type\":\"system\",\"content\":\"Failed to start Claude CLI: {}\\n\",\"timestamp\":\"{}\"}}",
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
                                if let Some(ref mut stdin) = interactive_stdin {
                                    info!("Sending raw input to Claude CLI: {:?}", input);
                                    
                                    // Send raw input directly - no processing needed
                                    let input_bytes = input.as_bytes().to_vec();
                                    
                                    if let Err(e) = stdin.write_all(&input_bytes).await {
                                        error!(error = %e, "Failed to write to Claude CLI");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart Claude
                                        if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart Claude CLI");
                                        }
                                    } else if let Err(e) = stdin.flush().await {
                                        error!(error = %e, "Failed to flush Claude CLI input");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart Claude
                                        if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart Claude CLI");
                                        }
                                    } else {
                                        info!("Successfully sent raw input to Claude CLI");
                                    }
                                } else {
                                    // No active Claude process - restart it
                                    if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                        error!(error = %e, "Failed to start Claude CLI");
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
                                    terminal_cols = cols;
                                    terminal_rows = rows;
                                    
                                    // Restart Claude CLI with new dimensions
                                    if interactive_process.is_some() {
                                        info!("Restarting Claude CLI with new terminal dimensions");
                                        if let Err(e) = start_interactive_command_with_dimensions(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin, terminal_cols, terminal_rows).await {
                                            error!(error = %e, "Failed to restart Claude CLI with new dimensions");
                                        }
                                    }
                                }
                            }
                            // Handle full commands (legacy support)
                            else if let Some(command) = cmd_data.get("command").and_then(|c| c.as_str()) {
                                let cmd = command.trim();
                                
                                // All input goes to Claude CLI (we always have an interactive process)
                                if let Some(ref mut stdin) = interactive_stdin {
                                    info!("Sending command to Claude CLI: {}", cmd);
                                    
                                    // Handle special control characters
                                    let input_bytes = if cmd == "\x03" {
                                        // Ctrl+C - send as-is without newline
                                        cmd.as_bytes().to_vec()
                                    } else {
                                        // Regular input - add newline
                                        format!("{}\n", cmd).into_bytes()
                                    };
                                    
                                    if let Err(e) = stdin.write_all(&input_bytes).await {
                                        error!(error = %e, "Failed to write to Claude CLI");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart Claude
                                        if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart Claude CLI");
                                        }
                                    } else if let Err(e) = stdin.flush().await {
                                        error!(error = %e, "Failed to flush Claude CLI input");
                                        // Clean up dead process and restart Claude
                                        interactive_process = None;
                                        interactive_stdin = None;
                                        
                                        // Attempt to restart Claude
                                        if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                            error!(error = %e, "Failed to restart Claude CLI");
                                        }
                                    } else {
                                        info!("Successfully sent input to Claude CLI");
                                    }
                                } else {
                                    // No active Claude process - restart it
                                    if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                        error!(error = %e, "Failed to start Claude CLI");
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
                                // Claude CLI has exited - restart it
                                info!("Claude CLI has exited, restarting...");
                                let restart_msg = format!(
                                    "{{\"type\":\"system\",\"content\":\"Claude CLI exited, restarting...\\n\",\"timestamp\":\"{}\"}}",
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
                                
                                // Restart Claude CLI
                                if let Err(e) = start_interactive_command(&mut socket, "claude", &mut interactive_process, &mut interactive_stdin).await {
                                    error!(error = %e, "Failed to restart Claude CLI");
                                    let error_msg = format!(
                                        "{{\"type\":\"system\",\"content\":\"Failed to restart Claude CLI: {}\\n\",\"timestamp\":\"{}\"}}",
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
    // Clean up any existing interactive process
    if let Some((mut child, stdout_task, stderr_task, _rx)) = interactive_process.take() {
        stdout_task.abort();
        stderr_task.abort();
        let _ = child.kill().await;
    }
    *interactive_stdin = None;

    // Prepare the command with any special handling
    let _prepared_command = prepare_interactive_command(command);
    
    // Claude CLI needs a proper terminal environment - try different approaches
    info!("Starting Claude CLI with proper terminal emulation");
    
    // Try using unbuffer first (if available), which creates a PTY
    let child = Command::new("unbuffer")
        .arg("claude")
        .env("TERM", "xterm-256color")
        .env("COLUMNS", &cols.to_string()) 
        .env("LINES", &rows.to_string())
        .env("LANG", "en_US.UTF-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped()) 
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => {
            info!("Claude CLI started with unbuffer, PID: {:?}", c.id());
            c
        }
        Err(e) => {
            error!("Failed to start Claude CLI with unbuffer: {}, trying script", e);
            // Try with script for PTY
            let script_cmd = "script -qfc 'claude' /dev/null";
            let script_child = Command::new("bash")
                .arg("-c")
                .arg(script_cmd)
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
                    info!("Claude CLI started with script, PID: {:?}", c.id());
                    c
                }
                Err(e) => {
                    error!("Failed to start Claude CLI with script: {}, trying direct", e);
                    // Last resort - direct execution
                    Command::new("claude")
                        .env("TERM", "xterm-256color")
                        .env("COLUMNS", "120")
                        .env("LINES", "30") 
                        .env("LANG", "en_US.UTF-8")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?
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
        info!("Started stdout reader task for Claude CLI");
        let mut stdout_reader = BufReader::new(stdout);
        let mut buffer = [0u8; 1024]; // Larger buffer but process more responsively
        
        loop {
            match stdout_reader.read(&mut buffer).await {
                Ok(0) => {
                    info!("Claude CLI stdout EOF reached");
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
                    error!("Error reading Claude CLI stdout: {}", e);
                    break;
                }
            }
        }
        info!("Claude CLI stdout reader task ended");
    });

    // Spawn task to handle stderr
    let socket_tx2 = tx.clone();
    let stderr_task = tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        info!("Started stderr reader task for Claude CLI");
        let mut stderr_reader = BufReader::new(stderr);
        let mut line = String::new();
        
        loop {
            line.clear();
            match stderr_reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Claude CLI stderr EOF reached");
                    break; // EOF
                }
                Ok(_) => {
                    info!("Received stderr line: {:?}", line);
                    let message = serde_json::json!({
                        "type": "stderr", 
                        "content": line.trim_end(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    let msg = message.to_string();
                    if socket_tx2.send(msg).await.is_err() {
                        error!("Failed to send stderr message to WebSocket");
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading Claude CLI stderr: {}", e);
                    break;
                }
            }
        }
        info!("Claude CLI stderr reader task ended");
    });

    // Send notification that interactive mode started
    socket.send(Message::Text(format!(
        "{{\"type\":\"system\",\"content\":\"Started Claude CLI (PID: {})\\n\",\"timestamp\":\"{}\"}}",
        child.id().unwrap_or(0),
        chrono::Utc::now().to_rfc3339()
    ))).await?;

    // Give Claude a moment to initialize and send initial output
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Check if the process is still running
    match child.try_wait() {
        Ok(Some(status)) => {
            error!("Claude CLI exited immediately with status: {:?}", status);
            socket.send(Message::Text(format!(
                "{{\"type\":\"system\",\"content\":\"Claude CLI exited immediately: {:?}\\n\",\"timestamp\":\"{}\"}}",
                status,
                chrono::Utc::now().to_rfc3339()
            ))).await?;
        }
        Ok(None) => {
            info!("Claude CLI is running");
            socket.send(Message::Text(format!(
                "{{\"type\":\"system\",\"content\":\"Claude CLI is running and ready\\n\",\"timestamp\":\"{}\"}}",
                chrono::Utc::now().to_rfc3339()
            ))).await?;
        }
        Err(e) => {
            error!("Error checking Claude CLI status: {}", e);
        }
    }

    *interactive_process = Some((child, stdout_task, stderr_task, rx));
    *interactive_stdin = Some(stdin);

    // Try sending an initial newline to trigger the interface
    if let Some(stdin_handle) = interactive_stdin {
        info!("Sending initial newline to trigger Claude interface");
        let _ = stdin_handle.write_all(b"\n").await;
        let _ = stdin_handle.flush().await;
    }

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