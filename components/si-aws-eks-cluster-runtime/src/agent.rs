use crate::model::entity::EntityEvent;
use si_cea::agent::dispatch::Dispatch;
use si_cea::{
    gen_dispatch, gen_dispatch_keys, gen_dispatch_setup, gen_dispatcher, CeaResult,
    MqttAsyncClientInternal,
};
use si_data::Db;

gen_dispatcher!(self_ident: self);

#[async_trait::async_trait]
impl Dispatch<EntityEvent> for Dispatcher {
    gen_dispatch_keys!(self);

    async fn setup(&mut self, db: &Db) -> CeaResult<()> {
        gen_dispatch_setup!(self, db, {
            integration_name: "aws",
            integration_service_name: "eks",
            dispatch[
                ("create", aws::create),
                ("sync", aws::sync),
                ("add_nodegroup", aws::add_nodegroup)
            ]
        });
        Ok(())
    }

    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
        integration_service_id: String,
        action_name: String,
    ) -> CeaResult<()> {
        gen_dispatch!(
            self,
            mqtt_client,
            entity_event,
            integration_service_id,
            action_name,
            dispatch[
                aws::create,
                aws::sync,
                aws::add_nodegroup
            ]
        );
        Ok(())
    }
}

mod aws {
    use crate::model::entity::{Bool, ClusterStatus, Entity, EntityEvent, NodegroupStatus, Tag};
    use si_cea::{CeaError, CeaResult, EntityEvent as _, MqttAsyncClientInternal};
    use si_external_api_gateway::aws::eks;
    use std::convert::{TryFrom, TryInto};
    use std::str::FromStr;
    use tracing::debug_span;
    use tracing_futures::Instrument as _;

    // NOTE(fnichol): We know this is temporary, right?
    enum WhoDat {
        Adam,
        Fletcher,
        Production,
    }
    const WHODAT: WhoDat = WhoDat::Production;

    fn hardcoded_security_group_ids() -> Vec<String> {
        match WHODAT {
            WhoDat::Adam => vec!["sg-070f1067".to_string()],
            WhoDat::Fletcher => vec!["sg-bc6539db".to_string()],
            WhoDat::Production => vec!["sg-070f1067".to_string()],
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
            WhoDat::Production => vec![
                "subnet-0c064a76".to_string(),
                "subnet-08a11544".to_string(),
                "subnet-ae9a8dc6".to_string(),
            ],
        }
    }

    fn hardcoded_role_arn() -> String {
        match WHODAT {
            WhoDat::Adam => "arn:aws:iam::835304779882:role/eksServiceRole".to_string(),
            WhoDat::Fletcher => "arn:aws:iam::167069368189:role/eksServiceRole".to_string(),
            WhoDat::Production => "arn:aws:iam::835304779882:role/eksServiceRole".to_string(),
        }
    }

    fn hardcoded_node_iam_role_name() -> String {
        match WHODAT {
            WhoDat::Adam => "arn:aws:iam::835304779882:role/nodeInstanceRole".to_string(),
            WhoDat::Production => "arn:aws:iam::835304779882:role/nodeInstanceRole".to_string(),
            WhoDat::Fletcher => "arn:aws:iam::167069368189:role/NodeInstanceRole".to_string(),
        }
    }

    fn hardcoded_ec2_ssh_key() -> String {
        match WHODAT {
            WhoDat::Adam => "si_key".to_string(),
            WhoDat::Production => "si_key".to_string(),
            WhoDat::Fletcher => "fnichol".to_string(),
        }
    }

    pub async fn create(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Creating EKS cluster in AWS\n");
            entity_event.init_output();

            let reply = eks_create_cluster(entity_event).await?;
            entity_event.output_as_mut()?.update_from_cluster(
                reply
                    .cluster
                    .ok_or(CeaError::action_error("reply missing cluster info"))?,
            )?;

            Ok(())
        }
        .instrument(debug_span!("aws_eks_cluster_runtime_aws_create"))
        .await
    }

    pub async fn sync(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Synchronizing EKS cluster in AWS\n");
            entity_event.init_output();

            let reply = eks_describe_cluster(entity_event).await?;
            entity_event.output_as_mut()?.update_from_cluster(
                reply
                    .cluster
                    .ok_or(CeaError::action_error("reply missing cluster info"))?,
            )?;
            if !entity_event.output()?.nodegroup_name.is_empty() {
                let reply = eks_describe_nodegroup(entity_event).await?;
                entity_event.output_as_mut()?.update_from_nodegroup(
                    reply
                        .nodegroup
                        .ok_or(CeaError::action_error("reply missing nodegroup info"))?,
                )?;
            }

            Ok(())
        }
        .instrument(debug_span!("aws_eks_cluster_runtime_aws_sync"))
        .await
    }

    pub async fn add_nodegroup(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Adding a nodegroup to an EKS cluster in AWS\n");
            entity_event.init_output();

            let reply = eks_create_nodegroup(entity_event).await?;
            entity_event.output_as_mut()?.update_from_nodegroup(
                reply
                    .nodegroup
                    .ok_or(CeaError::action_error("reply missing nodegroup info"))?,
            )?;

            Ok(())
        }
        .instrument(debug_span!("aws_eks_cluster_runtime_aws_add_nodegroup"))
        .await
    }

    async fn eks_create_cluster(
        entity_event: &mut EntityEvent,
    ) -> CeaResult<eks::CreateClusterReply> {
        async {
            // More evidence this should be refactored - why connect multiple times, rather than
            // multiplexing? Even if we need N connections, better to manage it higher up.
            let mut eks = eks::EksClient::connect(si_external_api_gateway::gateway_url()).await?;

            entity_event.log("Performing CreateCluster\n");

            let input = entity_event
                .input_entity
                .as_ref()
                .ok_or(CeaError::MissingInputEntity)?;

            let reply = eks
                .create_cluster(eks::CreateClusterRequest::try_from(input)?)
                .await?
                .into_inner();

            match reply.error {
                Some(e) => {
                    entity_event.error_log("CreateCluster failed\n");
                    entity_event.error_log(format!("Code: {}\n", e.code));
                    entity_event.error_log(format!("Message: {}\n", e.message));
                    entity_event.error_log(format!("Request ID: {}\n", e.request_id));

                    Err(CeaError::ExternalRequest)
                }
                None => {
                    entity_event.log("CreateCluster successful\n");
                    entity_event.log(format!("{:?}\n", reply));

                    Ok(reply)
                }
            }
        }
        .instrument(debug_span!("eks_create_cluster"))
        .await
    }

    async fn eks_create_nodegroup(
        entity_event: &mut EntityEvent,
    ) -> CeaResult<eks::CreateNodegroupReply> {
        async {
            // More evidence this should be refactored - why connect multiple times, rather than
            // multiplexing? Even if we need N connections, better to manage it higher up.
            let mut eks = eks::EksClient::connect(si_external_api_gateway::gateway_url()).await?;

            entity_event.log("Performing CreateNodegroup\n");

            let input = entity_event
                .input_entity
                .as_ref()
                .ok_or(CeaError::MissingInputEntity)?;

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

                    Err(CeaError::ExternalRequest)
                }
                None => {
                    entity_event.log("CreateNodegroup successful\n");
                    entity_event.log(format!("{:?}\n", reply));

                    Ok(reply)
                }
            }
        }
        .instrument(debug_span!("eks_create_nodegroup"))
        .await
    }

    async fn eks_describe_cluster(
        entity_event: &mut EntityEvent,
    ) -> CeaResult<eks::DescribeClusterReply> {
        async {
            // More evidence this should be refactored - why connect multiple times, rather than
            // multiplexing? Even if we need N connections, better to manage it higher up.
            let mut client =
                eks::EksClient::connect(si_external_api_gateway::gateway_url()).await?;

            entity_event.log("Performing DescribeCluster\n");

            let input = entity_event
                .input_entity
                .as_ref()
                .ok_or(CeaError::MissingInputEntity)?;

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

                    Err(CeaError::ExternalRequest)
                }
                None => {
                    entity_event.log("DescribeCluster successful\n");
                    entity_event.log(format!("{:?}\n", reply));

                    Ok(reply)
                }
            }
        }
        .instrument(debug_span!("eks_describe_cluster"))
        .await
    }

    async fn eks_describe_nodegroup(
        entity_event: &mut EntityEvent,
    ) -> CeaResult<eks::DescribeNodegroupReply> {
        async {
            // More evidence this should be refactored - why connect multiple times, rather than
            // multiplexing? Even if we need N connections, better to manage it higher up.
            let mut client =
                eks::EksClient::connect(si_external_api_gateway::gateway_url()).await?;

            entity_event.log("Performing DescribeNodegroup\n");

            let input = entity_event
                .input_entity
                .as_ref()
                .ok_or(CeaError::MissingInputEntity)?;

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

                    Err(CeaError::ExternalRequest)
                }
                None => {
                    entity_event.log("DescribeNodegroup successful\n");
                    entity_event.log(format!("{:?}\n", reply));

                    Ok(reply)
                }
            }
        }
        .instrument(debug_span!("eks_describe_nodegroup"))
        .await
    }

    impl Entity {
        fn update_from_cluster(&mut self, cluster: eks::Cluster) -> CeaResult<()> {
            if !cluster.name.is_empty() {
                self.cluster_name = cluster.name;
            }
            if !cluster.status.is_empty() {
                self.cluster_status = ClusterStatus::from_str(&cluster.status)
                    .map_err(CeaError::conversion_error)?
                    .into();
            }
            if !cluster.endpoint.is_empty() {
                self.endpoint = cluster.endpoint;
            }
            if let Some(certificate_authority) = cluster.certificate_authority {
                if !certificate_authority.data.is_empty() {
                    self.certificate_authority = certificate_authority.data;
                }
            }
            if let Some(resouces_vpc_config) = cluster.resources_vpc_config {
                self.endpoint_private_access = Bool::from(
                    eks::Bool::try_from(resouces_vpc_config.endpoint_private_access)
                        .map_err(CeaError::conversion_error)?,
                )
                .into();
                self.endpoint_public_access = Bool::from(
                    eks::Bool::try_from(resouces_vpc_config.endpoint_public_access)
                        .map_err(CeaError::conversion_error)?,
                )
                .into();
            }
            if !cluster.tags.is_empty() {
                self.tags = cluster.tags.into_iter().map(Tag::from).collect();
            }
            if !cluster.version.is_empty() {
                self.kubernetes_version = cluster.version;
            }

            Ok(())
        }

        fn update_from_nodegroup(&mut self, mut nodegroup: eks::Nodegroup) -> CeaResult<()> {
            if !nodegroup.nodegroup_name.is_empty() {
                self.nodegroup_name = nodegroup.nodegroup_name;
            }
            if !nodegroup.status.is_empty() {
                self.nodegroup_status = NodegroupStatus::from_str(&nodegroup.status)
                    .map_err(CeaError::conversion_error)?
                    .into();
            }
            // NOTE(fnichol): The SSH key info should come in populated via the Entity on creation,
            // but since that requires proper Entity to Entity linking and we're not quite there,
            // I'm going to pretend it was there and we're "round trip updating" this field--which
            // may or may not end up being what we want (i.e. is this field immutable in this
            // context and updated elsewhere in the system?)
            self.nodegroup_ssh_key_id = nodegroup
                .remote_access
                .ok_or(CeaError::action_error(
                    "reply missing nodegroup remote access info",
                ))?
                .ec2_ssh_key;
            if !nodegroup.instance_types.is_empty() {
                self.nodegroup_instance_type = nodegroup.instance_types.remove(0);
            }
            if nodegroup.disk_size != 0 {
                self.nodegroup_disk_size = nodegroup.disk_size;
            }
            if let Some(scaling_config) = nodegroup.scaling_config {
                self.nodegroup_desired_size = scaling_config.desired_size;
                self.nodegroup_min_size = scaling_config.min_size;
                self.nodegroup_max_size = scaling_config.max_size;
            }

            Ok(())
        }
    }

    impl EntityEvent {
        fn init_output(&mut self) {
            self.output_entity = self.input_entity.clone();
        }

        fn output(&self) -> CeaResult<&Entity> {
            self.output_entity
                .as_ref()
                .ok_or(CeaError::MissingOutputEntity)
        }

        fn output_as_mut(&mut self) -> CeaResult<&mut Entity> {
            self.output_entity
                .as_mut()
                .ok_or(CeaError::MissingOutputEntity)
        }
    }

    impl From<eks::Tag> for Tag {
        fn from(input: eks::Tag) -> Self {
            Tag {
                key: input.key,
                value: input.value,
            }
        }
    }

    impl From<&Entity> for Option<eks::Context> {
        fn from(input: &Entity) -> Self {
            Some(eks::Context {
                billing_account_id: input.billing_account_id.to_string(),
                organization_id: input.organization_id.to_string(),
                workspace_id: input.workspace_id.to_string(),
                ..Default::default()
            })
        }
    }

    impl TryFrom<&Entity> for eks::CreateClusterRequest {
        type Error = CeaError;

        fn try_from(input: &Entity) -> std::result::Result<Self, Self::Error> {
            Ok(eks::CreateClusterRequest {
                context: input.into(),
                version: input.kubernetes_version.to_string(),
                role_arn: hardcoded_role_arn(),
                name: cluster_name_for(input),
                logging: input.into(),
                client_request_token: input.id.to_string(),
                resources_vpc_config: input.try_into()?,
                tags: cluster_tags(input),
            })
        }
    }

    impl From<&Entity> for eks::CreateNodegroupRequest {
        fn from(input: &Entity) -> Self {
            eks::CreateNodegroupRequest {
                context: input.into(),
                cluster_name: input.cluster_name.to_string(),
                disk_size: input.nodegroup_disk_size,
                instance_types: vec![input.nodegroup_instance_type.to_string()],
                nodegroup_name: nodegroup_name_for(input),
                node_role: hardcoded_node_iam_role_name(),
                remote_access: input.into(),
                scaling_config: input.into(),
                subnets: hardcoded_subnet_ids(),
                tags: nodegroup_tags(input),
                version: input.kubernetes_version.to_string(),
                ..Default::default()
            }
        }
    }

    impl From<&Entity> for eks::DescribeClusterRequest {
        fn from(input: &Entity) -> Self {
            eks::DescribeClusterRequest {
                context: input.into(),
                name: cluster_name_for(input),
            }
        }
    }

    impl From<&Entity> for eks::DescribeNodegroupRequest {
        fn from(input: &Entity) -> Self {
            eks::DescribeNodegroupRequest {
                context: input.into(),
                cluster_name: input.cluster_name.to_string(),
                nodegroup_name: input.nodegroup_name.to_string(),
            }
        }
    }

    impl From<&Entity> for Option<eks::Logging> {
        fn from(input: &Entity) -> Self {
            if input.cloudwatch_logs {
                Some(eks::Logging {
                    cluster_logging: vec![eks::logging::LogSetup {
                        enabled: true,
                        // TODO(fnichol): chances are we want to ask for which logging types are
                        // enabled, but for now, you get them all.
                        types: vec![
                            "api".to_string(),
                            "audit".to_string(),
                            "authenticator".to_string(),
                            "controllerManager".to_string(),
                            "scheduler".to_string(),
                        ],
                    }],
                })
            } else {
                None
            }
        }
    }

    impl From<&Entity> for Option<eks::NodegroupScalingConfig> {
        fn from(input: &Entity) -> Self {
            Some(eks::NodegroupScalingConfig {
                desired_size: input.nodegroup_desired_size,
                max_size: input.nodegroup_max_size,
                min_size: input.nodegroup_min_size,
            })
        }
    }

    impl From<&Entity> for Option<eks::RemoteAccessConfig> {
        fn from(_input: &Entity) -> Self {
            Some(eks::RemoteAccessConfig {
                ec2_ssh_key: hardcoded_ec2_ssh_key(),
                ..Default::default()
            })
        }
    }

    impl TryFrom<&Entity> for Option<eks::VpcConfigRequest> {
        type Error = CeaError;

        fn try_from(input: &Entity) -> std::result::Result<Self, Self::Error> {
            Ok(Some(eks::VpcConfigRequest {
                endpoint_private_access: Bool::try_from(input.endpoint_private_access)
                    .map_err(CeaError::conversion_error)?
                    .into(),
                endpoint_public_access: Bool::try_from(input.endpoint_public_access)
                    .map_err(CeaError::conversion_error)?
                    .into(),
                security_group_ids: hardcoded_security_group_ids(),
                subnet_ids: hardcoded_subnet_ids(),
                ..Default::default()
            }))
        }
    }

    impl From<Bool> for eks::Bool {
        fn from(input: Bool) -> Self {
            match input {
                Bool::Unknown => Self::Unknown,
                Bool::True => Self::True,
                Bool::False => Self::False,
            }
        }
    }

    impl From<eks::Bool> for Bool {
        fn from(input: eks::Bool) -> Self {
            match input {
                eks::Bool::Unknown => Self::Unknown,
                eks::Bool::True => Self::True,
                eks::Bool::False => Self::False,
            }
        }
    }

    fn cluster_name_for(input: &Entity) -> String {
        input.id.replace(':', "-")
    }

    fn cluster_tags(input: &Entity) -> Vec<eks::Tag> {
        let mut tags = vec![
            eks::Tag {
                key: "si:id".to_string(),
                value: input.id.to_string(),
            },
            eks::Tag {
                key: "si:name".to_string(),
                value: input.name.to_string(),
            },
            eks::Tag {
                key: "si:displayName".to_string(),
                value: input.display_name.to_string(),
            },
        ];
        for tag in input.tags.iter() {
            tags.push(eks::Tag {
                key: tag.key.clone(),
                value: tag.value.clone(),
            });
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
            eks::Tag {
                key: "si:aws_eks_cluster_runtime:id".to_string(),
                value: input.id.to_string(),
            },
            eks::Tag {
                key: "si:name".to_string(),
                value: nodegroup_name_for(input),
            },
            // NOTE(fnichol): no `si:displayName` which should be added back
        ]
    }
}
