use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use paho_mqtt as mqtt;
use prost::Message;
use tempfile::TempDir;
use tokio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{oneshot, Mutex};

use tracing::{debug, debug_span, error, warn};
use tracing_futures::Instrument;
use uuid::Uuid;

use si_data::Db;
use si_external_api_gateway::aws::ec2;

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::os::unix::fs::PermissionsExt;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;

use crate::error::{Result, SshKeyError};
use crate::model::entity::{EntityEvent, KeyFormat, KeyType};

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

    pub fn success(self) -> Result<CommandResult> {
        if self.exit_status.success() {
            Ok(self)
        } else {
            Err(SshKeyError::CommandFailed(self))
        }
    }

    pub fn try_stdout(&mut self) -> Result<String> {
        self.stdout.take().ok_or(SshKeyError::CommandExpectedOutput)
    }

    pub fn stdout(&self) -> Option<&String> {
        self.stdout.as_ref()
    }

    pub fn try_stderr(&mut self) -> Result<String> {
        self.stderr.take().ok_or(SshKeyError::CommandExpectedOutput)
    }

    pub fn stderr(&self) -> Option<&String> {
        self.stderr.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct AgentFinalizer {
    mqtt: MqttAsyncClientInternal,
    db: Db,
}

impl AgentFinalizer {
    pub fn new(db: Db) -> AgentFinalizer {
        let client_id = format!("agent_finalizer:{}", Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        AgentFinalizer {
            mqtt: MqttAsyncClientInternal { mqtt: cli },
            db,
        }
    }

    fn subscribe_topics(&self) -> (Vec<String>, Vec<i32>) {
        let finalized_topic = format!("+/+/+/+/+/+/action/+/+/finalized",);
        (vec![finalized_topic], vec![2])
    }

    #[tracing::instrument]
    pub async fn dispatch(&mut self, entity_event: EntityEvent) -> Result<()> {
        debug!("updating_entity_event");
        self.db.upsert(&entity_event).await?;
        let entity = entity_event
            .output_entity
            .ok_or(SshKeyError::MissingOutputEntity)?;
        debug!("updating_entity");
        self.db.upsert(&entity).await?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Whats the right value? Who knows? God only knows. Ask the Beach Boys.
        let mut rx = self.mqtt.get_stream(1000).compat();
        println!("Finalizer connecting to the MQTT server...");
        let (server_uri, ver, session_present) = self
            .mqtt
            .connect(mqtt::ConnectOptions::new())
            .compat()
            .await?;
        // Make the connection to the broker
        println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
        if !session_present {
            let (subscribe_channels, subscribe_qos) = self.subscribe_topics();
            // Subscribe to multiple topics
            self.mqtt
                .subscribe(&subscribe_channels[0], subscribe_qos[0])
                //.subscribe_many(&subscribe_channels, &subscribe_qos)
                .compat()
                .await?;
        }

        // Just wait for incoming messages by running the receiver stream
        // in this thread.
        println!("Waiting for messages...");
        while let Some(stream_msg) = rx.next().await {
            let msg = match stream_msg {
                Ok(maybe_msg) => match maybe_msg {
                    Some(msg) => msg,
                    None => {
                        debug!("you don't have a message, eh?");
                        continue;
                    }
                },
                Err(_) => {
                    debug!("whats up?");
                    continue;
                }
            };
            let entity_event: EntityEvent = match EntityEvent::decode(msg.payload()) {
                Ok(e) => e,
                Err(err) => {
                    debug!(?err, "deserialzing error - bad message");
                    continue;
                }
            };
            let mut self_ref: AgentFinalizer = self.clone();
            tokio::spawn(async move { self_ref.dispatch(entity_event).await });
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AgentServer {
    mqtt: MqttAsyncClientInternal,
    integration_id: String,
    integration_service_id: String,
    name: String,
}

impl AgentServer {
    pub fn new(
        name: impl Into<String>,
        integration_id: impl Into<String>,
        integration_service_id: impl Into<String>,
    ) -> AgentServer {
        let client_id = format!("agent_server:{}", Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        AgentServer {
            name: name.into(),
            mqtt: MqttAsyncClientInternal { mqtt: cli },
            integration_id: integration_id.into(),
            integration_service_id: integration_service_id.into(),
        }
    }

    fn subscribe_topics(&self) -> (Vec<String>, Vec<i32>) {
        let inbound_channel = format!(
            "+/+/+/{}/{}/+/+/+/+",
            self.integration_id, self.integration_service_id,
        );
        (vec![inbound_channel], vec![2])
    }

    pub async fn dispatch(&mut self, entity_event: EntityEvent) -> Result<()> {
        async {
            let action_name = entity_event.action_name.clone();
            warn!(?entity_event, "please show me the money");
            let entity_event_locked = Arc::new(Mutex::new(entity_event));
            let result = match action_name.as_ref() {
                // This is a very dirty thing, and it should be removed once we
                // understand what we really want from the shape of these servers.
                "create" => match self.name.as_ref() {
                    "global" => self.create_global(entity_event_locked.clone()).await,
                    "aws" => self.create_aws(entity_event_locked.clone()).await,
                    _ => Ok(()),
                },
                "sync" => match self.name.as_ref() {
                    "global" => self.sync_global(entity_event_locked.clone()).await,
                    "aws" => self.sync_aws(entity_event_locked.clone()).await,
                    _ => Ok(()),
                },
                _ => Err(SshKeyError::InvalidEntityEventInvalidActionName),
            };
            warn!(?result, "dispatch result");
            if let Err(e) = result {
                {
                    let mut entity_event = entity_event_locked.lock().await;
                    entity_event.fail(e);
                }
                if let Err(send_err) = self.send(entity_event_locked).await {
                    error!(?send_err, "failed_to_send_message");
                }
            };
            Ok(())
        }
        .instrument(debug_span!("server_dispatch"))
        .await
    }

    async fn generate_result_topic(&self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> String {
        let entity_event = entity_event_locked.lock().await;
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/result",
            entity_event.billing_account_id,
            entity_event.organization_id,
            entity_event.workspace_id,
            entity_event.integration_id,
            entity_event.integration_service_id,
            entity_event.entity_id,
            "action",
            entity_event.action_name,
            entity_event.id,
        );
        topic
    }

    async fn generate_finalized_topic(
        &self,
        entity_event_locked: Arc<Mutex<EntityEvent>>,
    ) -> String {
        let entity_event = entity_event_locked.lock().await;
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}/finalized",
            entity_event.billing_account_id,
            entity_event.organization_id,
            entity_event.workspace_id,
            entity_event.integration_id,
            entity_event.integration_service_id,
            entity_event.entity_id,
            "action",
            entity_event.action_name,
            entity_event.id,
        );
        topic
    }

    pub async fn send(&self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> Result<()> {
        async {
            let mut payload = Vec::new();
            debug!("making payload");
            let finalized = {
                debug!("blocked myself");
                let entity_event = entity_event_locked.lock().await;
                debug!("locked myself");
                entity_event.encode(&mut payload)?;
                debug!("encoded myself");
                entity_event.finalized
            };
            debug!(?finalized, "finalized myself");
            if finalized {
                debug!("is finalized");
                debug!("sending result topic");
                let msg = mqtt::Message::new(
                    self.generate_result_topic(entity_event_locked.clone())
                        .await,
                    payload.clone(),
                    0,
                );
                self.mqtt.publish(msg).compat().await?;
                debug!("sending finalized topic");
                let msg = mqtt::Message::new(
                    self.generate_finalized_topic(entity_event_locked.clone())
                        .await,
                    payload,
                    2,
                );
                self.mqtt.publish(msg).compat().await?;
            } else {
                debug!("not finalized");
                debug!("sending result topic");
                let msg = mqtt::Message::new(
                    self.generate_result_topic(entity_event_locked.clone())
                        .await,
                    payload,
                    0,
                );
                self.mqtt.publish(msg).compat().await?;
            }
            Ok(())
        }
        .instrument(debug_span!("agent_server_send"))
        .await
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
    async fn spawn_command(
        &self,
        mut cmd: Command,
        entity_event_locked: Arc<Mutex<EntityEvent>>,
        capture_output: CaptureOutput,
    ) -> Result<CommandResult> {
        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.log(format!("---- Running Command ----"));
            entity_event.log(format!("{:?}", cmd));
            entity_event.log(format!("---- Output ----"));
            entity_event.error_log(format!("---- Running Command ----"));
            entity_event.error_log(format!("{:?}", cmd));
            entity_event.error_log(format!("---- Error Output ----"));
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!(?cmd, "running");

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or(SshKeyError::NoIoPipe)?;
        drop(stdin);

        let (stdout_tx, stdout_rx) = if capture_output.stdout() {
            let (stdout_tx, stdout_rx) = oneshot::channel::<String>();
            (Some(stdout_tx), Some(stdout_rx))
        } else {
            (None, None)
        };

        let stdout = BufReader::new(child.stdout.take().ok_or(SshKeyError::NoIoPipe)?);
        let std_entity_event = entity_event_locked.clone();
        let std_self = self.clone();
        let stdout_handle = tokio::spawn(async move {
            let mut stdout_capture: Option<String> = if stdout_tx.is_some() {
                Some(String::new())
            } else {
                None
            };
            let mut lines = stdout.lines();
            while let Some(result_line) = lines.next().await {
                let line = match result_line {
                    Ok(line) => line,
                    Err(err) => {
                        error!(?err, "line_read_error");
                        continue;
                    }
                };
                if stdout_tx.is_some() {
                    // Safe because stdout_capture is none if stdout_tx is none
                    stdout_capture
                        .as_mut()
                        .unwrap()
                        .push_str(&format!("{}\n", &line));
                }
                debug!(?line);
                {
                    {
                        let mut entity_event = std_entity_event.lock().await;
                        entity_event.log(line);
                    }
                    match std_self.send(std_entity_event.clone()).await {
                        Ok(_) => (),
                        Err(err) => {
                            error!(?err, "cannot_send_line_to_mqtt");
                            continue;
                        }
                    }
                }
            }
            if stdout_tx.is_some() {
                match stdout_tx.unwrap().send(stdout_capture.unwrap()) {
                    Ok(()) => debug!("sent_stdout_capture_tx"),
                    Err(_) => warn!("failed_to_send_capture_tx"),
                }
            }
        });

        let (stderr_tx, stderr_rx) = if capture_output.stderr() {
            let (stderr_tx, stderr_rx) = oneshot::channel::<String>();
            (Some(stderr_tx), Some(stderr_rx))
        } else {
            (None, None)
        };

        let stderr = BufReader::new(child.stderr.take().ok_or(SshKeyError::NoIoPipe)?);
        let std_err_entity_event = entity_event_locked.clone();
        let std_err_self = self.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut stderr_capture: Option<String> = if stderr_tx.is_some() {
                Some(String::new())
            } else {
                None
            };

            let mut lines = stderr.lines();
            while let Some(result_line) = lines.next().await {
                let line = match result_line {
                    Ok(line) => line,
                    Err(err) => {
                        error!(?err, "line_read_error");
                        continue;
                    }
                };
                if stderr_tx.is_some() {
                    // Safe because stdout_capture is none if stdout_tx is none
                    stderr_capture
                        .as_mut()
                        .unwrap()
                        .push_str(&format!("{}\n", &line));
                }
                debug!(?line);
                {
                    {
                        let mut entity_event = std_err_entity_event.lock().await;
                        entity_event.error_log(line);
                    }
                    match std_err_self.send(std_err_entity_event.clone()).await {
                        Ok(_) => (),
                        Err(err) => {
                            error!(?err, "cannot_send_line_to_mqtt");
                            continue;
                        }
                    }
                }
            }
            if stderr_tx.is_some() {
                match stderr_tx.unwrap().send(stderr_capture.unwrap()) {
                    Ok(()) => debug!("sent_stderr_capture_tx"),
                    Err(_) => warn!("failed_to_send_stderr_capture_tx"),
                }
            }
        });

        let (_stdout_result, _stderr_result, child_result) =
            tokio::join!(stdout_handle, stderr_handle, child);

        let stdout_string_option = if stdout_rx.is_some() {
            let mut rx = stdout_rx.unwrap();
            match rx.try_recv() {
                Ok(s) => Some(s),
                Err(e) => return Err(SshKeyError::from(e)),
            }
        } else {
            None
        };

        let stderr_string_option = if stderr_rx.is_some() {
            let mut rx = stderr_rx.unwrap();
            match rx.try_recv() {
                Ok(s) => Some(s),
                Err(e) => return Err(SshKeyError::from(e)),
            }
        } else {
            None
        };

        let child_status = child_result.map_err(SshKeyError::IoError)?;

        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.log(format!("---- Finished Command ----"));
            entity_event.error_log(format!("---- Finished Command ----"));
        }

        Ok(CommandResult::new(
            child_status,
            stdout_string_option,
            stderr_string_option,
        ))
    }

    #[tracing::instrument]
    pub async fn create_aws(&mut self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> Result<()> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut ec2 = ec2::Ec2Client::connect("http://localhost:4001").await?;
        let result = {
            let mut entity_event = entity_event_locked.lock().await;

            let key_name = if entity_event.input_entity.is_some() {
                entity_event.input_entity.as_ref().unwrap().id.to_string()
            } else {
                return Err(SshKeyError::MissingInputEntity);
            };
            entity_event.log("Creating Key Pair in EC2");
            let result = ec2
                .create_key_pair(ec2::CreateKeyPairRequest {
                    context: Some(ec2::Context {
                        billing_account_id: entity_event.billing_account_id.to_string(),
                        organization_id: entity_event.organization_id.to_string(),
                        workspace_id: entity_event.workspace_id.to_string(),
                        ..Default::default()
                    }),
                    key_name,
                    dry_run: false,
                })
                .await?
                .into_inner();

            if result.error.is_some() {
                let e = result.error.as_ref().unwrap();
                entity_event.error_log("Request failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                return Err(SshKeyError::ExternalRequest);
            }
            entity_event.log("Creation successful!\n");
            entity_event.log(format!("Request ID: {}\n", result.request_id));
            entity_event.log(format!("Key Fingerprint: {}\n", result.key_fingerprint));
            entity_event.log("Key Material:\n");
            entity_event.log(format!("{}\n", result.key_material));
            entity_event.log(format!("Key Name: {}\n", result.key_name));
            entity_event.log(format!("Key Pair ID: {}\n", result.key_pair_id));
            result
        };

        let tempdir = TempDir::new()?;
        let filename = tempdir.path().join("newkey");
        tokio::fs::write(&filename, &result.key_material).await?;
        let file_metadata = tokio::fs::metadata(&filename).await?;
        let mut file_perms = file_metadata.permissions();
        file_perms.set_mode(0o600);
        tokio::fs::set_permissions(&filename, file_perms).await?;

        debug!(?tempdir, ?filename, "tempfile_created");

        let key_format = {
            let entity_event = entity_event_locked.lock().await;
            match KeyFormat::from_i32(entity_event.input_entity.as_ref().unwrap().key_format) {
                Some(key_format) => key_format,
                // TODO: Fix this error
                None => return Err(SshKeyError::MissingInputEntity),
            }
        };

        let mut ssh_public_key_cmd = Command::new("ssh-keygen");
        ssh_public_key_cmd.current_dir(tempdir.path());
        ssh_public_key_cmd.arg("-e");
        ssh_public_key_cmd.arg("-f");
        ssh_public_key_cmd.arg(filename.to_string_lossy().as_ref());
        ssh_public_key_cmd.arg("-m");
        ssh_public_key_cmd.arg(format!("{}", key_format));
        let mut ssh_public_key_out = self
            .spawn_command(
                ssh_public_key_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        let filename_pub = tempdir.path().join("newkey.pub");
        tokio::fs::write(
            &filename_pub,
            &ssh_public_key_out.stdout().unwrap_or(&"".to_string()),
        )
        .await?;
        let pub_file_metadata = tokio::fs::metadata(&filename_pub).await?;
        let mut pub_file_perms = pub_file_metadata.permissions();
        pub_file_perms.set_mode(0o644);
        tokio::fs::set_permissions(&filename_pub, pub_file_perms).await?;

        let mut ssh_fingerprint_cmd = Command::new("ssh-keygen");
        ssh_fingerprint_cmd.current_dir(tempdir.path());
        ssh_fingerprint_cmd.arg("-l");
        ssh_fingerprint_cmd.arg("-f");
        ssh_fingerprint_cmd.arg(filename.to_string_lossy().as_ref());
        let mut ssh_fingerprint_out = self
            .spawn_command(
                ssh_fingerprint_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        let mut ssh_babble_cmd = Command::new("ssh-keygen");
        ssh_babble_cmd.current_dir(tempdir.path());
        ssh_babble_cmd.arg("-B");
        ssh_babble_cmd.arg("-f");
        ssh_babble_cmd.arg(filename.to_string_lossy().as_ref());
        let mut ssh_babble_out = self
            .spawn_command(
                ssh_babble_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.output_entity = entity_event.input_entity.clone();
            let output_entity = entity_event
                .output_entity
                .as_mut()
                .ok_or(SshKeyError::MissingOutputEntity)?;
            output_entity.private_key = result.key_material.to_string();
            output_entity.public_key = ssh_public_key_out.try_stdout()?;
            output_entity.fingerprint = ssh_fingerprint_out.try_stdout()?;
            output_entity.bubble_babble = ssh_babble_out.try_stdout()?;
            entity_event.success();
        }

        debug!("sending success");
        self.send(entity_event_locked.clone()).await?;
        debug!("sent success");

        Ok(())
    }

    pub async fn sync_global(
        &mut self,
        entity_event_locked: Arc<Mutex<EntityEvent>>,
    ) -> Result<()> {
        async {
            {
                let mut entity_event = entity_event_locked.lock().await;
                entity_event.output_entity = entity_event.input_entity.clone();
                entity_event.log("Synchronized State");
                entity_event.success();
            }
            debug!("sending success");
            self.send(entity_event_locked.clone()).await?;
            debug!("sent success");
            Ok(())
        }
        .instrument(debug_span!(
            "agent_server_sync_global",
            ?entity_event_locked
        ))
        .await
    }

    pub async fn sync_aws(&mut self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> Result<()> {
        async {
            let mut ec2 = ec2::Ec2Client::connect("http://localhost:4001").await?;
            {
                let mut entity_event = entity_event_locked.lock().await;

                entity_event.log("Synchronizing Key Pair in EC2");
                let result = ec2
                    .describe_key_pairs(ec2::DescribeKeyPairsRequest {
                        context: Some(ec2::Context {
                            billing_account_id: entity_event.billing_account_id.to_string(),
                            organization_id: entity_event.organization_id.to_string(),
                            workspace_id: entity_event.workspace_id.to_string(),
                            ..Default::default()
                        }),
                        key_names: vec![entity_event.entity_id.clone()],
                    })
                    .await?
                    .into_inner();
                if result.error.is_some() {
                    let e = result.error.as_ref().unwrap();
                    entity_event.error_log("Request failed\n");
                    entity_event.error_log(format!("Code: {}\n", e.code));
                    entity_event.error_log(format!("Message: {}\n", e.message));
                    entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                    return Err(SshKeyError::ExternalRequest);
                }
                if result.key_pairs.len() > 0 {
                    entity_event.log("Sync successful!\n");
                    entity_event.log(format!(
                        "Fingerprint: {}\n",
                        result.key_pairs[0].key_fingerprint
                    ));
                    entity_event.output_entity = entity_event.input_entity.clone();
                    entity_event.log("Synchronized State");
                    entity_event.success();
                }
            }
            debug!("sending success");
            self.send(entity_event_locked.clone()).await?;
            debug!("sent success");
            Ok(())
        }
        .instrument(debug_span!("agent_server_sync_aws", ?entity_event_locked))
        .await
    }

    #[tracing::instrument]
    pub async fn create_global(
        &mut self,
        entity_event_locked: Arc<Mutex<EntityEvent>>,
    ) -> Result<()> {
        let tempdir = TempDir::new()?;
        let filename = tempdir.path().join("newkey");
        debug!(?tempdir, ?filename, "tempfile_created");

        // Before this gets used in anger, the user needs to give us the passphrase
        let mut ssh_keygen_cmd = Command::new("ssh-keygen");
        ssh_keygen_cmd.current_dir(tempdir.path());
        {
            let entity_event = entity_event_locked.lock().await;
            let input_entity = entity_event
                .input_entity
                .as_ref()
                .ok_or(SshKeyError::MissingInputEntity)?;
            let key_type =
                KeyType::from_i32(input_entity.key_type).ok_or(SshKeyError::KeyTypeInvalid)?;
            let key_format = KeyFormat::from_i32(input_entity.key_format)
                .ok_or(SshKeyError::KeyFormatInvalid)?;

            ssh_keygen_cmd.arg("-t");
            ssh_keygen_cmd.arg(format!("{}", key_type));
            ssh_keygen_cmd.arg("-m");
            ssh_keygen_cmd.arg(format!("{}", key_format));
            ssh_keygen_cmd.arg("-b");
            ssh_keygen_cmd.arg(format!("{}", input_entity.bits));
            ssh_keygen_cmd.arg("-C");
            ssh_keygen_cmd.arg(&input_entity.name[..]);
            ssh_keygen_cmd.arg("-f");
            ssh_keygen_cmd.arg(filename.to_string_lossy().as_ref());
            ssh_keygen_cmd.arg("-N");
            ssh_keygen_cmd.arg("");
        }

        self.spawn_command(
            ssh_keygen_cmd,
            entity_event_locked.clone(),
            CaptureOutput::None,
        )
        .await?
        .success()?;

        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.output_entity = entity_event.input_entity.clone();
            let output_entity = entity_event
                .output_entity
                .as_mut()
                .ok_or(SshKeyError::MissingInputEntity)?;

            let private_key = fs::read(&filename).await?;
            output_entity.private_key = String::from_utf8(private_key)?;

            let public_key_name = format!("{}.pub", filename.display());
            let public_key = fs::read(&public_key_name).await?;
            output_entity.public_key = String::from_utf8(public_key)?;
        }

        let mut ssh_fingerprint_cmd = Command::new("ssh-keygen");
        ssh_fingerprint_cmd.current_dir(tempdir.path());
        ssh_fingerprint_cmd.arg("-l");
        ssh_fingerprint_cmd.arg("-f");
        ssh_fingerprint_cmd.arg(filename.to_string_lossy().as_ref());
        let mut ssh_fingerprint_out = self
            .spawn_command(
                ssh_fingerprint_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        let mut ssh_babble_cmd = Command::new("ssh-keygen");
        ssh_babble_cmd.current_dir(tempdir.path());
        ssh_babble_cmd.arg("-B");
        ssh_babble_cmd.arg("-f");
        ssh_babble_cmd.arg(filename.to_string_lossy().as_ref());
        let mut ssh_babble_out = self
            .spawn_command(
                ssh_babble_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        let mut ssh_random_cmd = Command::new("ssh-keygen");
        ssh_random_cmd.current_dir(tempdir.path());
        ssh_random_cmd.arg("-l");
        ssh_random_cmd.arg("-v");
        ssh_random_cmd.arg("-f");
        ssh_random_cmd.arg(filename.to_string_lossy().as_ref());
        let mut ssh_random_out = self
            .spawn_command(
                ssh_random_cmd,
                entity_event_locked.clone(),
                CaptureOutput::Stdout,
            )
            .await?
            .success()?;

        {
            let mut entity_event = entity_event_locked.lock().await;
            let output_entity = entity_event
                .output_entity
                .as_mut()
                .ok_or(SshKeyError::MissingOutputEntity)?;
            output_entity.fingerprint = ssh_fingerprint_out.try_stdout()?;
            output_entity.bubble_babble = ssh_babble_out.try_stdout()?;
            output_entity.random_art = ssh_random_out.try_stdout()?;
            entity_event.success();
        }

        self.send(entity_event_locked.clone()).await?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Whats the right value? Who knows? God only knows. Ask the Beach Boys.
        let mut rx = self.mqtt.get_stream(1000).compat();
        println!("Connecting to the MQTT server...");
        let (server_uri, ver, session_present) = self
            .mqtt
            .connect(mqtt::ConnectOptions::new())
            .compat()
            .await?;
        // Make the connection to the broker
        println!("Connected to: '{}' with MQTT version {}", server_uri, ver);
        if !session_present {
            let (subscribe_channels, subscribe_qos) = self.subscribe_topics();
            // Subscribe to multiple topics
            self.mqtt
                .subscribe(&subscribe_channels[0], subscribe_qos[0])
                //.subscribe_many(&subscribe_channels, &subscribe_qos)
                .compat()
                .await?;
        }

        // Just wait for incoming messages by running the receiver stream
        // in this thread.
        println!("Waiting for messages...");
        while let Some(stream_msg) = rx.next().await {
            let msg = match stream_msg {
                Ok(maybe_msg) => match maybe_msg {
                    Some(msg) => msg,
                    None => {
                        debug!("you don't have a message, eh?");
                        continue;
                    }
                },
                Err(_) => {
                    debug!("whats up?");
                    continue;
                }
            };
            let entity_event: EntityEvent = match EntityEvent::decode(msg.payload()) {
                Ok(e) => e,
                Err(err) => {
                    debug!(?err, "deserialzing error - bad message");
                    continue;
                }
            };
            if entity_event.input_entity.is_none() {
                warn!(?entity_event, "Missing input entity on event");
                continue;
            }
            let mut self_ref = self.clone();
            tokio::spawn(async move { self_ref.dispatch(entity_event).await });
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AgentClient {
    mqtt: MqttAsyncClientInternal,
}

#[derive(Clone)]
struct MqttAsyncClientInternal {
    mqtt: mqtt::AsyncClient,
}

impl std::fmt::Debug for MqttAsyncClientInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MQTT Async Client")
    }
}

impl Deref for MqttAsyncClientInternal {
    type Target = mqtt::AsyncClient;

    fn deref(&self) -> &Self::Target {
        &self.mqtt
    }
}

impl DerefMut for MqttAsyncClientInternal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mqtt
    }
}

impl AgentClient {
    pub async fn new() -> Result<AgentClient> {
        // Create a client & define connect options
        let client_id = format!("agent_client:{}", Uuid::new_v4());

        let cli = mqtt::AsyncClientBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id(client_id.as_ref())
            .persistence(false)
            .finalize();

        cli.connect(mqtt::ConnectOptions::new()).compat().await?;

        Ok(AgentClient {
            mqtt: MqttAsyncClientInternal { mqtt: cli },
        })
    }

    pub async fn dispatch(&self, entity_event: &EntityEvent) -> Result<()> {
        if entity_event.input_entity.is_none() {
            return Err(SshKeyError::MissingInputEntity);
        }
        match &entity_event.action_name[..] {
            "create" => self.send(entity_event).await,
            "sync" => self.send(entity_event).await,
            _ => Err(SshKeyError::InvalidEntityEventInvalidActionName),
        }
    }

    // Eventually, we need to be able to dispatch to an individual agent id, so
    // that people can run specific agents for their billing account. We can
    // do that by just putting it in the EntityEvent stuct, and if it is
    // filled in, we use it.
    fn generate_topic(&self, entity_event: &EntityEvent) -> String {
        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            entity_event.billing_account_id,
            entity_event.organization_id,
            entity_event.workspace_id,
            entity_event.integration_id,
            entity_event.integration_service_id,
            entity_event.entity_id,
            "action",
            entity_event.action_name,
            entity_event.id,
        );
        topic
    }

    pub async fn send(&self, entity_event: &EntityEvent) -> Result<()> {
        let mut payload = Vec::new();
        entity_event.encode(&mut payload)?;
        // We are very close to the broker - so no need to pretend that we are at
        // risk of not receiving our messages. Right?
        let msg = mqtt::Message::new(self.generate_topic(entity_event), payload, 0);
        self.mqtt.publish(msg).compat().await?;
        Ok(())
    }
}
