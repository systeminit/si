use crate::error::{AwsEksClusterRuntimeError, Result};
use crate::model::entity::{Entity, EntityEvent};
use crate::{ClusterStatus, NodegroupStatus};
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::StreamExt;
use paho_mqtt as mqtt;
use prost::Message;
use si_data::Db;
use si_external_api_gateway::aws::eks;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use tokio;
use tracing::{debug, error, warn};
use uuid::Uuid;

// NOTE(fnichol): We know this is temporary, right?
enum WhoDat {
    Adam,
    Fletcher,
}
const WHODAT: WhoDat = WhoDat::Fletcher;

fn hardcoded_security_group_ids() -> Vec<String> {
    match WHODAT {
        WhoDat::Adam => vec!["sg-070f1067".to_string()],
        WhoDat::Fletcher => vec!["sg-bc6539db".to_string()],
    }
}

fn hardcoded_subnet_ids() -> Vec<String> {
    match WHODAT {
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
    }
}

fn hardcoded_role_arn() -> String {
    match WHODAT {
        WhoDat::Adam => "arn:aws:iam::835304779882:role/eksServiceRole".to_string(),
        WhoDat::Fletcher => "arn:aws:iam::167069368189:role/eksServiceRole".to_string(),
    }
}

fn hardcoded_node_iam_role_name() -> String {
    match WHODAT {
        WhoDat::Adam => "i:dont:have:one:yet".to_string(),
        WhoDat::Fletcher => "arn:aws:iam::167069368189:role/NodeInstanceRole".to_string(),
    }
}

fn hardcoded_ec2_ssh_key() -> String {
    match WHODAT {
        WhoDat::Adam => "adam".to_string(),
        WhoDat::Fletcher => "fnichol".to_string(),
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
    pub async fn dispatch(&mut self, mut entity_event: EntityEvent) -> Result<()> {
        warn!(?entity_event, "please show me the money");
        let result = match entity_event.action_name.as_ref() {
            // This is a very dirty thing, and it should be removed once we
            // understand what we really want from the shape of these servers.
            "create" => match self.name.as_ref() {
                "aws" => self.create_aws(&mut entity_event).await,
                _ => Ok(()),
            },
            "sync" => match self.name.as_ref() {
                "aws" => self.sync_aws(&mut entity_event).await,
                _ => Ok(()),
            },
            // NOTE(fnichol): A Nodegroup shouldn't be a part of the cluster component/entity, but
            // we're going to mash this together for the moment to see the entire flow and pretend
            // as though it's a different internal state for a cluster
            "add_nodegroup" => match self.name.as_ref() {
                "aws" => self.add_nodegroup_aws(&mut entity_event).await,
                _ => Ok(()),
            },
            _ => Err(AwsEksClusterRuntimeError::InvalidEntityEventInvalidActionName),
        };

        if let Err(e) = result {
            entity_event.fail(e);
            if let Err(send_err) = self.send(&mut entity_event).await {
                error!(?send_err, "failed_to_send_message");
            }
        };

        Ok(())
    }

    async fn generate_result_topic(&self, entity_event: &EntityEvent) -> String {
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

    async fn generate_finalized_topic(&self, entity_event: &EntityEvent) -> String {
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

    #[tracing::instrument]
    pub async fn send(&self, entity_event: &mut EntityEvent) -> Result<()> {
        let mut payload = Vec::new();
        let finalized = {
            entity_event.encode(&mut payload)?;
            entity_event.finalized
        };
        if finalized {
            let msg = mqtt::Message::new(
                self.generate_result_topic(entity_event).await,
                payload.clone(),
                0,
            );
            self.mqtt.publish(msg).compat().await?;
            let msg = mqtt::Message::new(
                self.generate_finalized_topic(entity_event).await,
                payload,
                2,
            );
            self.mqtt.publish(msg).compat().await?;
        } else {
            let msg =
                mqtt::Message::new(self.generate_result_topic(entity_event).await, payload, 0);
            self.mqtt.publish(msg).compat().await?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub async fn create_aws(&mut self, entity_event: &mut EntityEvent) -> Result<()> {
        entity_event.log("Creating EKS cluster in AWS\n");
        entity_event.set_output();

        let reply = self.eks_create_cluster(entity_event).await?;
        entity_event.output_as_mut()?.update_from_cluster(
            reply
                .cluster
                .ok_or(AwsEksClusterRuntimeError::ReplyMissingCluster)?,
        )?;
        entity_event.success();

        self.send(entity_event).await
    }

    #[tracing::instrument]
    pub async fn sync_aws(&mut self, entity_event: &mut EntityEvent) -> Result<()> {
        entity_event.log("Synchronizing EKS cluster in AWS\n");
        entity_event.set_output();

        let reply = self.eks_describe_cluster(entity_event).await?;
        entity_event.output_as_mut()?.update_from_cluster(
            reply
                .cluster
                .ok_or(AwsEksClusterRuntimeError::ReplyMissingCluster)?,
        )?;
        if !entity_event.output()?.nodegroup_name.is_empty() {
            let reply = self.eks_describe_nodegroup(entity_event).await?;
            entity_event.output_as_mut()?.update_from_nodegroup(
                reply
                    .nodegroup
                    .ok_or(AwsEksClusterRuntimeError::ReplyMissingNodegroup)?,
            )?;
        }
        entity_event.success();

        self.send(entity_event).await
    }

    #[tracing::instrument]
    pub async fn add_nodegroup_aws(&mut self, entity_event: &mut EntityEvent) -> Result<()> {
        entity_event.log("Adding a nodegroup to an EKS cluster in AWS\n");
        entity_event.set_output();

        let reply = self.eks_create_nodegroup(entity_event).await?;
        entity_event.output_as_mut()?.update_from_nodegroup(
            reply
                .nodegroup
                .ok_or(AwsEksClusterRuntimeError::ReplyMissingNodegroup)?,
        )?;
        entity_event.success();

        self.send(entity_event).await
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

    #[tracing::instrument]
    async fn eks_create_cluster(
        &self,
        entity_event: &mut EntityEvent,
    ) -> Result<eks::CreateClusterReply> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut eks = eks::EksClient::connect("http://localhost:4001").await?;

        entity_event.log("Performing CreateCluster\n");

        let input = entity_event
            .input_entity
            .as_ref()
            .ok_or(AwsEksClusterRuntimeError::MissingInputEntity)?;

        let reply = eks
            .create_cluster(eks::CreateClusterRequest::from(input))
            .await?
            .into_inner();

        match reply.error {
            Some(e) => {
                entity_event.error_log("CreateCluster failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));

                Err(AwsEksClusterRuntimeError::ExternalRequest)
            }
            None => {
                entity_event.log("CreateCluster successful\n");
                entity_event.log(format!("{:?}\n", reply));

                Ok(reply)
            }
        }
    }

    #[tracing::instrument]
    async fn eks_create_nodegroup(
        &self,
        entity_event: &mut EntityEvent,
    ) -> Result<eks::CreateNodegroupReply> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut eks = eks::EksClient::connect("http://localhost:4001").await?;

        entity_event.log("Performing CreateNodegroup\n");

        let input = entity_event
            .input_entity
            .as_ref()
            .ok_or(AwsEksClusterRuntimeError::MissingInputEntity)?;

        let reply = eks
            .create_nodegroup(eks::CreateNodegroupRequest::from(input))
            .await?
            .into_inner();

        match reply.error {
            Some(e) => {
                entity_event.error_log("CreateNodegroup failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));

                Err(AwsEksClusterRuntimeError::ExternalRequest)
            }
            None => {
                entity_event.log("CreateNodegroup successful\n");
                entity_event.log(format!("{:?}\n", reply));

                Ok(reply)
            }
        }
    }

    #[tracing::instrument]
    async fn eks_describe_cluster(
        &self,
        entity_event: &mut EntityEvent,
    ) -> Result<eks::DescribeClusterReply> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut client = eks::EksClient::connect("http://localhost:4001").await?;

        entity_event.log("Performing DescribeCluster\n");

        let input = entity_event
            .input_entity
            .as_ref()
            .ok_or(AwsEksClusterRuntimeError::MissingInputEntity)?;

        let reply = client
            .describe_cluster(eks::DescribeClusterRequest::from(input))
            .await?
            .into_inner();

        match reply.error {
            Some(e) => {
                entity_event.error_log("DescribeCluster failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));

                Err(AwsEksClusterRuntimeError::ExternalRequest)
            }
            None => {
                entity_event.log("DescribeCluster successful\n");
                entity_event.log(format!("{:?}\n", reply));

                Ok(reply)
            }
        }
    }

    #[tracing::instrument]
    async fn eks_describe_nodegroup(
        &self,
        entity_event: &mut EntityEvent,
    ) -> Result<eks::DescribeNodegroupReply> {
        // More evidence this should be refactored - why connect multiple times, rather than
        // multiplexing? Even if we need N connections, better to manage it higher up.
        let mut client = eks::EksClient::connect("http://localhost:4001").await?;

        entity_event.log("Performing DescribeNodegroup\n");

        let input = entity_event
            .input_entity
            .as_ref()
            .ok_or(AwsEksClusterRuntimeError::MissingInputEntity)?;

        let reply = client
            .describe_nodegroup(eks::DescribeNodegroupRequest::from(input))
            .await?
            .into_inner();

        match reply.error {
            Some(e) => {
                entity_event.error_log("DescribeNodegroup failed\n");
                entity_event.error_log(format!("Code: {}\n", e.code));
                entity_event.error_log(format!("Message: {}\n", e.message));
                entity_event.error_log(format!("Request ID: {}\n", e.request_id));

                Err(AwsEksClusterRuntimeError::ExternalRequest)
            }
            None => {
                entity_event.log("DescribeNodegroup successful\n");
                entity_event.log(format!("{:?}\n", reply));

                Ok(reply)
            }
        }
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
            "create" | "sync" | "add_nodegroup" => self.send(entity_event).await,
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

impl Entity {
    fn update_from_cluster(&mut self, cluster: eks::Cluster) -> Result<()> {
        self.cluster_name = cluster.name;
        self.cluster_status = ClusterStatus::from_str(&cluster.status)?.into();

        Ok(())
    }

    fn update_from_nodegroup(&mut self, nodegroup: eks::Nodegroup) -> Result<()> {
        self.nodegroup_name = nodegroup.nodegroup_name;
        self.nodegroup_status = NodegroupStatus::from_str(&nodegroup.status)?.into();
        // NOTE(fnichol): The SSH key info should come in populated via the Entity on creation, but
        // since that requires proper Entity to Entity linking and we're not quite there, I'm going
        // to pretend it was there and we're "round trip updating" this field--which may or may not
        // end up being what we want (i.e. is this field immutable in this context and updated
        // elsewhere in the system?)
        self.node_group_ssh_key_id = nodegroup
            .remote_access
            .ok_or(AwsEksClusterRuntimeError::ReplyMissingNodegroupRemoteAccess)?
            .ec2_ssh_key;

        Ok(())
    }
}

impl EntityEvent {
    fn set_output(&mut self) {
        self.output_entity = self.input_entity.clone();
    }

    fn output(&self) -> Result<&Entity> {
        self.output_entity
            .as_ref()
            .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)
    }

    fn output_as_mut(&mut self) -> Result<&mut Entity> {
        self.output_entity
            .as_mut()
            .ok_or(AwsEksClusterRuntimeError::MissingOutputEntity)
    }
}

impl From<&Entity> for Option<eks::Context> {
    fn from(input: &Entity) -> Self {
        Some(
            eks::Context::builder()
                .billing_account_id(input.billing_account_id.to_string())
                .organization_id(input.organization_id.to_string())
                .workspace_id(input.workspace_id.to_string())
                .build(),
        )
    }
}

impl From<&Entity> for eks::CreateClusterRequest {
    fn from(input: &Entity) -> Self {
        eks::CreateClusterRequest::builder()
            .context(input.into())
            .version(input.kubernetes_version.to_string())
            .role_arn(hardcoded_role_arn())
            .name(cluster_name_for(input))
            .logging(input.into())
            .client_request_token(input.id.to_string())
            .resources_vpc_config(input.into())
            .tags(cluster_tags(input))
            .build()
    }
}

impl From<&Entity> for eks::CreateNodegroupRequest {
    fn from(input: &Entity) -> Self {
        eks::CreateNodegroupRequest::builder()
            .context(input.into())
            .cluster_name(input.cluster_name.to_string())
            .nodegroup_name(nodegroup_name_for(input))
            .node_role(hardcoded_node_iam_role_name())
            .remote_access(input.into())
            .scaling_config(input.into())
            .subnets(hardcoded_subnet_ids())
            .tags(nodegroup_tags(input))
            .version(input.kubernetes_version.to_string())
            .build()
    }
}

impl From<&Entity> for eks::DescribeClusterRequest {
    fn from(input: &Entity) -> Self {
        eks::DescribeClusterRequest::builder()
            .context(input.into())
            .name(cluster_name_for(input))
            .build()
    }
}

impl From<&Entity> for eks::DescribeNodegroupRequest {
    fn from(input: &Entity) -> Self {
        eks::DescribeNodegroupRequest::builder()
            .cluster_name(input.cluster_name.to_string())
            .nodegroup_name(input.nodegroup_name.to_string())
            .build()
    }
}

impl From<&Entity> for Option<eks::Logging> {
    fn from(input: &Entity) -> Self {
        if input.cloudwatch_logs {
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
        }
    }
}

impl From<&Entity> for Option<eks::NodegroupScalingConfig> {
    fn from(input: &Entity) -> Self {
        Some(
            eks::NodegroupScalingConfig::builder()
                .desired_size(input.node_group_desired_size)
                .max_size(input.node_group_maximum_size)
                .min_size(input.node_group_minimum_size)
                .build(),
        )
    }
}

impl From<&Entity> for Option<eks::RemoteAccessConfig> {
    fn from(_input: &Entity) -> Self {
        Some(
            eks::RemoteAccessConfig::builder()
                .ec2_ssh_key(hardcoded_ec2_ssh_key())
                .build(),
        )
    }
}

impl From<&Entity> for Option<eks::VpcConfigRequest> {
    fn from(_input: &Entity) -> Self {
        Some(
            eks::VpcConfigRequest::builder()
                .endpoint_private_access(false)
                .endpoint_public_access(true)
                .security_group_ids(hardcoded_security_group_ids())
                .subnet_ids(hardcoded_subnet_ids())
                .build(),
        )
    }
}

impl FromStr for ClusterStatus {
    type Err = AwsEksClusterRuntimeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s {
            "CREATING" => Self::Creating,
            "ACTIVE" => Self::Active,
            "DELETING" => Self::Deleting,
            "FAILED" => Self::Failed,
            "UPDATING" => Self::Updating,
            invalid => {
                return Err(AwsEksClusterRuntimeError::InvalidClusterStatus(
                    invalid.to_string(),
                ))
            }
        };

        Ok(r)
    }
}

impl FromStr for NodegroupStatus {
    type Err = AwsEksClusterRuntimeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s {
            "CREATING" => Self::NodegroupCreating,
            "ACTIVE" => Self::NodegroupActive,
            "UPDATING" => Self::NodegroupUpdating,
            "DELETING" => Self::NodegroupDeleting,
            "CREATE_FAILED" => Self::NodegroupCreateFailed,
            "DELETE_FAILED" => Self::NodegroupDeleteFailed,
            "DEGRADED" => Self::NodegroupDegraded,
            invalid => {
                return Err(AwsEksClusterRuntimeError::InvalidNodegroupStatus(
                    invalid.to_string(),
                ))
            }
        };

        Ok(r)
    }
}

fn cluster_name_for(input: &Entity) -> String {
    input.id.replace(':', "-")
}

fn cluster_tags(input: &Entity) -> Vec<eks::Tag> {
    let mut tags = vec![
        eks::Tag::builder()
            .key("si:id".to_string())
            .value(input.id.to_string())
            .build(),
        eks::Tag::builder()
            .key("si:name".to_string())
            .value(input.name.to_string())
            .build(),
        eks::Tag::builder()
            .key("si:displayName".to_string())
            .value(input.display_name.to_string())
            .build(),
    ];
    for tag in input.tags.iter() {
        tags.push(
            eks::Tag::builder()
                .key(tag.key.clone())
                .value(tag.value.clone())
                .build(),
        );
    }

    tags
}

fn nodegroup_name_for(input: &Entity) -> String {
    // NOTE(fnichol): this naming is temporary and assumes a 1:1 relationship with a cluster
    // and a nodegroup. Also, there is a 63-character max length name, so there's that :/
    input
        .cluster_name
        .replace("aws_eks_cluster_runtime_entity", "aws_eks_nodegroup")
}

fn nodegroup_tags(input: &Entity) -> Vec<eks::Tag> {
    vec![
        eks::Tag::builder()
            .key("si:aws_eks_cluster_runtime:id".to_string())
            .value(input.id.to_string())
            .build(),
        eks::Tag::builder()
            .key("si:name".to_string())
            .value(nodegroup_name_for(input))
            .build(),
        // NOTE(fnichol): no `si:displayName` which should be added back
    ]
}
