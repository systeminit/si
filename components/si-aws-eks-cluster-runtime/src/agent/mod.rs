use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use paho_mqtt as mqtt;
use prost::Message;
use tokio;
use tokio::sync::Mutex;

use tracing::{debug, error, warn};
use uuid::Uuid;

use si_data::Db;
use si_external_api_gateway::aws::eks;

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::process::ExitStatus;
use std::sync::Arc;

use crate::error::{AwsEksClusterRuntimeError, Result};
use crate::model::entity::EntityEvent;
use crate::AwsStatus;

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
        let finalized_topic = "+/+/+/+/+/+/action/+/+/finalized".to_string();
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
            "sync" => match self.name.as_ref() {
                "aws" => self.sync_aws(entity_event_locked.clone()).await,
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

    #[tracing::instrument]
    pub async fn sync_aws(&mut self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> Result<()> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        //
        //let mut eks = eks::EksClient::connect("http://localhost:4001").await?;
        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.log("Sync successful!\n");
            entity_event.output_entity = entity_event.input_entity.clone();
            //let output_entity = entity_event
            //    .output_entity
            //    .as_mut()
            //    .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)?;
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

    #[tracing::instrument]
    pub async fn create_aws(&mut self, entity_event_locked: Arc<Mutex<EntityEvent>>) -> Result<()> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut eks = eks::EksClient::connect("http://localhost:4001").await?;

        let result = {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.log("Creating EKS Cluster\n");

            let result = eks
                .create_cluster(build_create_cluster_request(&mut entity_event)?)
                .await?
                .into_inner();

            match result.error {
                Some(e) => {
                    entity_event.error_log("Request failed\n");
                    entity_event.error_log(format!("Code: {}\n", e.code));
                    entity_event.error_log(format!("Message: {}\n", e.message));
                    entity_event.error_log(format!("Request ID: {}\n", e.request_id));
                    return Err(AwsEksClusterRuntimeError::ExternalRequest);
                }
                None => {
                    entity_event.log("Creation successful!\n");
                    entity_event.log(format!("{:?}\n", result));
                    debug!("YOU DID IT!");
                }
            };

            result
        };

        {
            let mut entity_event = entity_event_locked.lock().await;
            entity_event.output_entity = entity_event.input_entity.clone();
            let mut output = entity_event
                .output_entity
                .as_mut()
                .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)?;
            let cluster = result
                .cluster
                .ok_or(AwsEksClusterRuntimeError::CreateClusterReplyMissingCluster)?;
            output.aws_status = match cluster.status.as_ref() {
                "CREATING" => AwsStatus::Creating.into(),
                "ACTIVE" => AwsStatus::Active.into(),
                "DELETING" => AwsStatus::Deleting.into(),
                "FAILED" => AwsStatus::Failed.into(),
                "UPDATING" => AwsStatus::Updating.into(),
                invalid => {
                    return Err(AwsEksClusterRuntimeError::InvalidCreateClusterStatus(
                        invalid.to_string(),
                    ))
                }
            };
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

fn build_create_cluster_request(
    entity_event: &mut EntityEvent,
) -> Result<eks::CreateClusterRequest> {
    let input_entity = if entity_event.input_entity.is_some() {
        entity_event.input_entity.as_ref().unwrap()
    } else {
        return Err(AwsEksClusterRuntimeError::MissingInputEntity);
    };

    enum WhoDat {
        Adam,
        Fletcher,
    }
    const WHODAT: WhoDat = WhoDat::Fletcher;

    let security_group_ids = match WHODAT {
        WhoDat::Adam => vec!["sg-070f1067".to_string()],
        WhoDat::Fletcher => vec!["sg-bc6539db".to_string()],
    };
    let subnet_ids = match WHODAT {
        WhoDat::Adam => vec![
            "subnet-0c064a76".to_string(),
            "subnet-08a11544".to_string(),
            "subnet-ae9a8dc6".to_string(),
        ],
        WhoDat::Fletcher => vec![
            "subnet-223e1c58".to_string(),
            "subnet-5a867231".to_string(),
            "subnet-a26ae0ee".to_string(),
        ],
    };
    let role_arn = match WHODAT {
        WhoDat::Adam => "arn:aws:iam::835304779882:role/eksServiceRole".to_string(),
        WhoDat::Fletcher => "arn:aws:iam::167069368189:role/eksServiceRole".to_string(),
    };

    let logging = if input_entity.cloudwatch_logs {
        Some(
            eks::Logging::builder()
                .cluster_logging(vec![eks::logging::LogSetup::builder()
                    .enabled(true)
                    // TODO fn: Chances are we want to ask for which logging types are
                    // enabled, but for now, you get them all.
                    .types(vec![
                        "api".to_string(),
                        "audit".to_string(),
                        "authenticator".to_string(),
                        "controllerManager".to_string(),
                        "scheduler".to_string(),
                    ])
                    .build()])
                .build(),
        )
    } else {
        None
    };

    let resources_vpc_config = Some(
        eks::create_cluster_request::VpcConfigRequest::builder()
            .endpoint_private_access(false)
            .endpoint_public_access(true)
            .public_access_cidrs(Default::default())
            .security_group_ids(security_group_ids)
            .subnet_ids(subnet_ids)
            .build(),
    );
    let mut tags = vec![
        eks::Tag::builder()
            .key("si:id".to_string())
            .value(input_entity.id.to_string())
            .build(),
        eks::Tag {
            key: "si:id".to_string(),
            value: input_entity.id.to_string(),
        },
        eks::Tag {
            key: "si:name".to_string(),
            value: input_entity.name.to_string(),
        },
        eks::Tag {
            key: "si:displayName".to_string(),
            value: input_entity.display_name.to_string(),
        },
    ];
    for tag in input_entity.tags.iter() {
        tags.push(
            eks::Tag::builder()
                .key(tag.key.clone())
                .value(tag.value.clone())
                .build(),
        );
    }

    let request = eks::CreateClusterRequest::builder()
        .context(Some(eks::Context {
            billing_account_id: entity_event.billing_account_id.to_string(),
            organization_id: entity_event.organization_id.to_string(),
            workspace_id: entity_event.workspace_id.to_string(),
            ..Default::default()
        }))
        .version(input_entity.kubernetes_version.to_string())
        .role_arn(role_arn)
        .name(input_entity.id.replace(':', "-"))
        .logging(logging)
        .client_request_token(entity_event.id.to_string())
        .resources_vpc_config(resources_vpc_config)
        .tags(tags)
        .build();

    Ok(request)
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
