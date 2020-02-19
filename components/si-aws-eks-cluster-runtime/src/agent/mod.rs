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

use tracing::{debug, error, warn};
use uuid::Uuid;

use si_data::Db;
use si_external_api_gateway::aws::eks;

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::os::unix::fs::PermissionsExt;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;

use crate::error::{AwsEksClusterRuntimeError, Result};
use crate::model::entity::EntityEvent;

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
            Err(AwsEksClusterRuntimeError::CommandFailed(self))
        }
    }

    pub fn try_stdout(&mut self) -> Result<String> {
        self.stdout
            .take()
            .ok_or(AwsEksClusterRuntimeError::CommandExpectedOutput)
    }

    pub fn stdout(&self) -> Option<&String> {
        self.stdout.as_ref()
    }

    pub fn try_stderr(&mut self) -> Result<String> {
        self.stderr
            .take()
            .ok_or(AwsEksClusterRuntimeError::CommandExpectedOutput)
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
            .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)?;
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

    #[tracing::instrument]
    pub async fn dispatch(&mut self, entity_event: EntityEvent) -> Result<()> {
        let action_name = entity_event.action_name.clone();
        warn!(?entity_event, "please show me the money");
        let entity_event_locked = Arc::new(Mutex::new(entity_event));
        let result = match action_name.as_ref() {
            // This is a very dirty thing, and it should be removed once we
            // understand what we really want from the shape of these servers.
            "create" => match self.name.as_ref() {
                "aws" => self.create_aws(entity_event_locked.clone()).await,
                _ => Ok(()),
            },
            _ => Err(AwsEksClusterRuntimeError::InvalidEntityEventInvalidActionName),
        };
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
        let mut payload = Vec::new();
        let finalized = {
            let entity_event = entity_event_locked.lock().await;
            entity_event.encode(&mut payload)?;
            entity_event.finalized
        };
        if finalized {
            let msg = mqtt::Message::new(
                self.generate_result_topic(entity_event_locked.clone())
                    .await,
                payload.clone(),
                0,
            );
            self.mqtt.publish(msg).compat().await?;
            let msg = mqtt::Message::new(
                self.generate_finalized_topic(entity_event_locked.clone())
                    .await,
                payload,
                2,
            );
            self.mqtt.publish(msg).compat().await?;
        } else {
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
        let stdin = child
            .stdin
            .take()
            .ok_or(AwsEksClusterRuntimeError::NoIoPipe)?;
        drop(stdin);

        let (stdout_tx, stdout_rx) = if capture_output.stdout() {
            let (stdout_tx, stdout_rx) = oneshot::channel::<String>();
            (Some(stdout_tx), Some(stdout_rx))
        } else {
            (None, None)
        };

        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or(AwsEksClusterRuntimeError::NoIoPipe)?,
        );
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

        let stderr = BufReader::new(
            child
                .stderr
                .take()
                .ok_or(AwsEksClusterRuntimeError::NoIoPipe)?,
        );
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
                Err(e) => return Err(AwsEksClusterRuntimeError::from(e)),
            }
        } else {
            None
        };

        let stderr_string_option = if stderr_rx.is_some() {
            let mut rx = stderr_rx.unwrap();
            match rx.try_recv() {
                Ok(s) => Some(s),
                Err(e) => return Err(AwsEksClusterRuntimeError::from(e)),
            }
        } else {
            None
        };

        let child_status = child_result.map_err(AwsEksClusterRuntimeError::IoError)?;

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
        let mut eks = eks::EksClient::connect("http://localhost:4001").await?;
        let result = {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.log("Creating EKS Cluster");

            let input_entity = if entity_event.input_entity.is_some() {
                entity_event.input_entity.as_ref().unwrap()
            } else {
                return Err(AwsEksClusterRuntimeError::MissingInputEntity);
            };

            let logging = if input_entity.cloudwatch_logs {
                Some(eks::create_cluster_request::ClusterLogging {
                    types: vec![
                        "api".to_string(),
                        "audit".to_string(),
                        "authenticator".to_string(),
                        "controllerManager".to_string(),
                        "scheduler".to_string(),
                    ],
                    enabled: true,
                })
            } else {
                None
            };

            let tags = vec![
                eks::create_cluster_request::TagRequest {
                    key: "si:id".to_string(),
                    value: input_entity.id.to_string(),
                },
                eks::create_cluster_request::TagRequest {
                    key: "si:name".to_string(),
                    value: input_entity.name.to_string(),
                },
                eks::create_cluster_request::TagRequest {
                    key: "si:displayName".to_string(),
                    value: input_entity.display_name.to_string(),
                },
            ];

            let result = eks
                .create_cluster(eks::CreateClusterRequest {
                    context: Some(eks::Context {
                        billing_account_id: entity_event.billing_account_id.to_string(),
                        organization_id: entity_event.organization_id.to_string(),
                        workspace_id: entity_event.workspace_id.to_string(),
                        ..Default::default()
                    }),
                    version: input_entity.kubernetes_version.to_string(),
                    // So, this works if you are Adam! Ha!. Cheers!
                    role_arn: "arn:aws:iam::835304779882:role/eksServiceRole".to_string(),
                    //role_arn: "arn:aws:iam::167069368189:role/eksServiceRole".to_string(),
                    name: input_entity.id.replace(':', "-").to_string(),
                    logging,
                    client_request_token: entity_event.id.to_string(),
                    resources_vpc_config: Some(eks::create_cluster_request::VpcConfigRequest {
                        // Adam Subnets! ;P
                        subnet_ids: vec![
                            "subnet-0c064a76".to_string(),
                            "subnet-08a11544".to_string(),
                            "subnet-ae9a8dc6".to_string(),
                        ],
                        security_group_ids: vec!["sg-070f1067".to_string()],
                        // Fletcher subnets! :)
                        //subnet_ids: vec![
                        //    "subnet-223e1c58".to_string(),
                        //    "subnet-5a867231".to_string(),
                        //    "subnet-a26ae0ee".to_string(),
                        //],
                        //security_group_ids: vec!["sg-bc6539db".to_string()],
                        endpoint_public_access: true,
                        endpoint_private_access: false,
                    }),
                    tags,
                })
                .await?
                .into_inner();

            if result.error.is_some() {
                let e = result.error.as_ref().unwrap();
                entity_event.error_log("Request failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                return Err(AwsEksClusterRuntimeError::ExternalRequest);
            }
            entity_event.log("Creation successful!\n");
            entity_event.log(format!("{:?}", result));
            debug!("YOU DID IT!");
            result
        };

        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.output_entity = entity_event.input_entity.clone();
            let output_entity = entity_event
                .output_entity
                .as_mut()
                .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)?;
            //output_entity.private_key = result.key_material.to_string();
            //output_entity.public_key = ssh_public_key_out.try_stdout()?;
            //output_entity.fingerprint = ssh_fingerprint_out.try_stdout()?;
            //output_entity.bubble_babble = ssh_babble_out.try_stdout()?;
            entity_event.success();
        }

        debug!("sending success");
        self.send(entity_event_locked.clone()).await?;
        debug!("sent success");

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
            return Err(AwsEksClusterRuntimeError::MissingInputEntity);
        }
        match &entity_event.action_name[..] {
            "create" => self.send(entity_event).await,
            _ => Err(AwsEksClusterRuntimeError::InvalidEntityEventInvalidActionName),
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
