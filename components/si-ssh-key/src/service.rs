use si_data::{
    query_expression_option, Db, ListResult, Query, QueryBooleanLogic, QueryComparison,
    QueryExpression, QueryExpressionOption, QueryFieldType,
};
use tonic::{Request, Response};
use tracing::{debug, debug_span};
use tracing_futures::Instrument;

use si_account::{authorize::authorize, BillingAccount, Workspace};

use crate::agent::AgentClient;
use crate::error::{SshKeyError, TonicResult};
use crate::model::component::{Component, KeyFormat, KeyType};
use crate::model::entity::{Entity, EntityEvent};
use crate::protobuf::{self, ssh_key_server, ImplicitConstraint};

#[derive(Debug)]
pub struct Service {
    db: Db,
    agent: AgentClient,
}

impl Service {
    pub fn new(db: Db, agent: AgentClient) -> Service {
        Service { db, agent }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn agent(&self) -> &AgentClient {
        &self.agent
    }
}

async fn pick_component(
    db: &Db,
    request: Request<protobuf::PickComponentRequest>,
) -> TonicResult<protobuf::PickComponentReply> {
    let req = request.get_ref();
    if req.name != "" {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "name".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: req.name.to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        debug!(?query, "checking by name");
        let mut name_check_result: ListResult<Component> = db
            .list(&Some(query), 1, "", 0, "global", "")
            .await
            .map_err(SshKeyError::ListComponentsError)?;
        debug!(?name_check_result, "checking by name result");
        if name_check_result.len() == 1 {
            debug!("chosen by name");
            return Ok(Response::new(protobuf::PickComponentReply {
                // Safe because we checked the length above
                component: name_check_result.items.pop(),
                ..Default::default()
            }));
        } else {
            debug!("name does not match exactly");
            return Err(tonic::Status::from(SshKeyError::PickComponent(
                "name must match exactly, and was not found".to_string(),
            )));
        }
    }
    if req.display_name != "" {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "displayName".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: req.display_name.to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let mut name_check_result: ListResult<Component> = db
            .list(&Some(query), 1, "", 0, "global", "")
            .await
            .map_err(SshKeyError::ListComponentsError)?;
        if name_check_result.len() == 1 {
            return Ok(Response::new(protobuf::PickComponentReply {
                // Safe because we checked the length above
                component: name_check_result.items.pop(),
                ..Default::default()
            }));
        } else {
            return Err(tonic::Status::from(SshKeyError::PickComponent(
                "displayName must match exactly, and was not found".to_string(),
            )));
        }
    }

    debug!("solving like a motherfucker");

    // DEFAULT VALUES
    let key_type: KeyType; // = KeyType::Rsa;
    let key_format: KeyFormat; // = KeyFormat::Rfc4716;
    let bits: u32; // = 2048;

    // Someday, this will matter - but for now, there is only one integration.
    //
    // Here, future self, I gift to you these variables. Do with them what you will.
    //
    // let integration = "Global";
    // let integration_service = "SSH Key";

    let mut implicit_constraints: Vec<ImplicitConstraint> = Vec::new();

    // KEY TYPE GOES FIRST...

    // Means you have some kind of a type provided as a constraint
    //if req.key_type != 0 {
    if req.key_type == 0 {
        key_type = KeyType::Rsa;
        implicit_constraints.push(ImplicitConstraint {
            field: "keyType".to_string(),
            value: key_type.to_string(),
        });
    } else {
        key_type = match req.key_type {
            0 => unreachable!("You cannot get here"),
            1 => KeyType::Rsa,
            2 => KeyType::Dsa,
            3 => KeyType::Ecdsa,
            4 => KeyType::Ed25519,
            _ => return Err(tonic::Status::from(SshKeyError::KeyTypeInvalid)),
        };
    }

    // THEN SOLVE FOR BITS

    // If you didn't supply bits, we pick the right number of bits
    // on your behalf
    if req.bits == 0 {
        bits = match key_type {
            KeyType::Rsa => 2048,
            KeyType::Dsa => 1024,
            // No idea if this is right, but lets go for bigger. Because,
            // you know... better.
            KeyType::Ecdsa => 521,
            KeyType::Ed25519 => 256,
        };
        implicit_constraints.push(ImplicitConstraint {
            field: "bits".to_string(),
            value: bits.to_string(),
        });
    } else {
        // You provided me bits, and I need to check that the bits are valid
        // for your key_type.
        bits = match key_type {
            KeyType::Rsa => match req.bits {
                1024 | 2048 | 3072 | 4096 => req.bits,
                value => {
                    return Err(tonic::Status::from(SshKeyError::BitsInvalid(
                        key_type.to_string(),
                        value,
                    )))
                }
            },
            KeyType::Dsa => match req.bits {
                1024 => req.bits,
                value => {
                    return Err(tonic::Status::from(SshKeyError::BitsInvalid(
                        key_type.to_string(),
                        value,
                    )))
                }
            },
            KeyType::Ecdsa => match req.bits {
                256 | 384 | 521 => req.bits,
                value => {
                    return Err(tonic::Status::from(SshKeyError::BitsInvalid(
                        key_type.to_string(),
                        value,
                    )))
                }
            },
            KeyType::Ed25519 => match req.bits {
                256 => req.bits,
                value => {
                    return Err(tonic::Status::from(SshKeyError::BitsInvalid(
                        key_type.to_string(),
                        value,
                    )))
                }
            },
        };
    }

    // SOLVE FOR FORMAT
    if req.key_format == 0 {
        key_format = KeyFormat::Rfc4716;
        implicit_constraints.push(ImplicitConstraint {
            field: "keyFormat".to_string(),
            value: key_format.to_string(),
        });
    } else {
        key_format = match req.key_format {
            0 => unreachable!("You cannot get here"),
            1 => KeyFormat::Rfc4716,
            2 => KeyFormat::Pkcs8,
            3 => KeyFormat::Pem,
            _ => return Err(tonic::Status::from(SshKeyError::KeyFormatInvalid)),
        };
    }

    let query = Query {
        items: vec![
            QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "keyType".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: key_type.to_string(),
                    ..Default::default()
                })),
            },
            QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "keyFormat".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: key_format.to_string(),
                    ..Default::default()
                })),
            },
            QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "bits".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    field_type: QueryFieldType::Int as i32,
                    value: bits.to_string(),
                    ..Default::default()
                })),
            },
        ],
        boolean_term: QueryBooleanLogic::And as i32,
        ..Default::default()
    };
    let mut name_check_result: ListResult<Component> = db
        .list(&Some(query), 1, "", 0, "global", "")
        .await
        .map_err(SshKeyError::ListComponentsError)?;
    if name_check_result.len() == 1 {
        return Ok(Response::new(protobuf::PickComponentReply {
            // Safe because we checked the length above
            component: name_check_result.items.pop(),
            implicit_constraints,
            ..Default::default()
        }));
    } else {
        return Err(tonic::Status::from(SshKeyError::PickComponent(
            "our algo is very, very wrong. woopsie poopsie".to_string(),
        )));
    }
}

#[tonic::async_trait]
impl ssh_key_server::SshKey for Service {
    async fn sync_entity(
        &self,
        request: Request<protobuf::SyncEntityRequest>,
    ) -> TonicResult<protobuf::SyncEntityReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "sync_entity",
                &billing_account,
            )
            .await?;

            let entity = self
                .db
                .get(&req.entity_id)
                .await
                .map_err(|_| SshKeyError::EntityMissing)?;

            let mut entity_event = EntityEvent::new(user_id, "sync", &entity);
            self.db
                .validate_and_insert_as_new(&mut entity_event)
                .await
                .map_err(SshKeyError::CreateEntityEvent)?;

            self.agent.dispatch(&entity_event).await?;

            Ok(Response::new(protobuf::SyncEntityReply {
                event: Some(entity_event),
                ..Default::default()
            }))
        }
        .instrument(debug_span!("sync_entity", ?request))
        .await
    }

    async fn list_entity_events(
        &self,
        request: Request<protobuf::ListEntityEventsRequest>,
    ) -> TonicResult<protobuf::ListEntityEventsReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_entity_events",
                &billing_account,
            )
            .await?;

            let scope = if req.scope_by_tenant_id == "" {
                billing_account_id
            } else {
                req.scope_by_tenant_id.as_ref()
            };

            let list_result: ListResult<EntityEvent> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(SshKeyError::ListEntityEventsError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        scope,
                        "",
                    )
                    .await
                    .map_err(SshKeyError::ListEntityEventsError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListEntityEventsReply::default()));
            }

            Ok(Response::new(protobuf::ListEntityEventsReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_entity_events", ?request))
        .await
    }

    async fn create_entity(
        &self,
        request: Request<protobuf::CreateEntityRequest>,
    ) -> TonicResult<protobuf::CreateEntityReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            // NOTE: Someday, make this work on the workspace. Going to need to actually
            // think about it, instead of just blatting out what works. ;)
            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "create_entity",
                &billing_account,
            )
            .await?;

            let workspace: Workspace = self
                .db
                .get(&req.workspace_id)
                .await
                .map_err(|_| SshKeyError::WorkspaceMissing)?;

            let constraints = match &req.constraints {
                Some(constraint) => constraint.clone(),
                None => protobuf::PickComponentRequest::default(),
            };

            let selected_component = pick_component(&self.db, tonic::Request::new(constraints))
                .await?
                .into_inner();

            let mut entity = Entity::from_request_and_component(req, selected_component, workspace);
            self.db
                .validate_and_insert_as_new(&mut entity)
                .await
                .map_err(SshKeyError::CreateEntity)?;

            let mut entity_event = EntityEvent::new(user_id, "create", &entity);
            self.db
                .validate_and_insert_as_new(&mut entity_event)
                .await
                .map_err(SshKeyError::CreateEntityEvent)?;

            self.agent.dispatch(&entity_event).await?;

            Ok(Response::new(protobuf::CreateEntityReply {
                entity: Some(entity),
                event: Some(entity_event),
            }))
        }
        .instrument(debug_span!("create_entity", ?request))
        .await
    }

    async fn pick_component(
        &self,
        request: Request<protobuf::PickComponentRequest>,
    ) -> TonicResult<protobuf::PickComponentReply> {
        let metadata = request.metadata();
        let user_id = metadata
            .get("userId")
            .ok_or(SshKeyError::InvalidAuthentication)?
            .to_str()
            .map_err(SshKeyError::GrpcHeaderToString)?;
        let billing_account_id = metadata
            .get("billingAccountId")
            .ok_or(SshKeyError::InvalidAuthentication)?
            .to_str()
            .map_err(SshKeyError::GrpcHeaderToString)?;

        let billing_account: BillingAccount = self
            .db
            .get(billing_account_id)
            .await
            .map_err(|_| SshKeyError::BillingAccountMissing)?;

        authorize(
            &self.db,
            user_id,
            billing_account_id,
            "pick_component",
            &billing_account,
        )
        .await?;

        return pick_component(&self.db, request).await;
    }

    async fn list_components(
        &self,
        request: Request<protobuf::ListComponentsRequest>,
    ) -> TonicResult<protobuf::ListComponentsReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_components",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<Component> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(SshKeyError::ListComponentsError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        "global",
                        "",
                    )
                    .await
                    .map_err(SshKeyError::ListComponentsError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListComponentsReply::default()));
            }

            Ok(Response::new(protobuf::ListComponentsReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_components", ?request))
        .await
    }

    async fn get_component(
        &self,
        request: Request<protobuf::GetComponentRequest>,
    ) -> TonicResult<protobuf::GetComponentReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            let component: Component =
                self.db
                    .get(req.component_id.to_string())
                    .await
                    .map_err(|e| {
                        debug!(?e, "component_get_failed");
                        SshKeyError::ComponentMissing
                    })?;
            debug!(?component, "found");

            // NOTE: Once we actually can authorize on something other than your
            // billing account, this should get fixed.
            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "read_global_component",
                &billing_account,
            )
            .await?;

            Ok(Response::new(protobuf::GetComponentReply {
                component: Some(component),
            }))
        }
        .instrument(debug_span!("get_component", ?request))
        .await
    }

    async fn list_entities(
        &self,
        request: Request<protobuf::ListEntitiesRequest>,
    ) -> TonicResult<protobuf::ListEntitiesReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| SshKeyError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_entities",
                &billing_account,
            )
            .await?;

            let scope = if req.scope_by_tenant_id == "" {
                billing_account_id
            } else {
                req.scope_by_tenant_id.as_ref()
            };

            let list_result: ListResult<Entity> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(SshKeyError::ListEntitiesError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        scope,
                        "",
                    )
                    .await
                    .map_err(SshKeyError::ListEntitiesError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListEntitiesReply::default()));
            }

            Ok(Response::new(protobuf::ListEntitiesReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_entities", ?request))
        .await
    }

    async fn get_entity(
        &self,
        request: Request<protobuf::GetEntityRequest>,
    ) -> TonicResult<protobuf::GetEntityReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(SshKeyError::InvalidAuthentication)?
                .to_str()
                .map_err(SshKeyError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let entity: Entity = self.db.get(req.entity_id.to_string()).await.map_err(|e| {
                debug!(?e, "entity_get_failed");
                SshKeyError::EntityMissing
            })?;
            debug!(?entity, "found");

            authorize(&self.db, user_id, billing_account_id, "get_entity", &entity).await?;

            Ok(Response::new(protobuf::GetEntityReply {
                entity: Some(entity),
            }))
        }
        .instrument(debug_span!("get_entity", ?request))
        .await
    }
}
