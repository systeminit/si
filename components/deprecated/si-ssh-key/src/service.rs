use names;
use tonic::{Request, Response};
use tracing::{event, span, Level};
use tracing_futures::Instrument;

use crate::agent::Agent;
use crate::data::{Db, ListResult};
use crate::error::{Error, Result, TonicResult};
use crate::ssh_key::{
    query_expression_option::Qe, server::SshKey, Component, CreateEntityReply, CreateEntityRequest,
    Entity, GetComponentReply, GetComponentRequest, GetEntityReply, GetEntityRequest, KeyFormat,
    KeyType, ListComponentsReply, ListComponentsRequest, ListEntitiesReply, ListEntitiesRequest,
    OrderByDirection, PageToken, PickComponentReply, PickComponentRequest, Query,
    QueryBooleanLogic, QueryComparison, QueryExpression, QueryExpressionOption, QueryFieldType,
};

#[derive(Debug)]
pub struct Service {
    db: Db,
    component_order_by: Vec<&'static str>,
    entity_order_by: Vec<&'static str>,
}

impl Service {
    pub fn new(db: Db) -> Result<Service> {
        Ok(Service {
            db: db,
            component_order_by: vec![
                "naturalKey",
                "bits",
                "displayName",
                "integration",
                "keyType",
                "keyFormat",
            ],
            entity_order_by: vec![
                "naturalKey",
                "bits",
                "displayName",
                "integration",
                "keyType",
                "keyFormat",
                "comment",
            ],
        })
    }

    pub fn is_component_order_by_valid<S: AsRef<str>>(&self, order_by: S) -> Result<()> {
        let test = order_by.as_ref();

        match self.component_order_by.iter().find(|o| *o == &test) {
            Some(_) => Ok(()),
            None => Err(Error::OrderBy),
        }
    }

    pub fn is_entity_order_by_valid<S: AsRef<str>>(&self, order_by: S) -> Result<()> {
        let test = order_by.as_ref();

        match self.entity_order_by.iter().find(|o| *o == &test) {
            Some(_) => Ok(()),
            None => Err(Error::OrderBy),
        }
    }
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &KeyType::Rsa => "RSA".to_string(),
            &KeyType::Dsa => "DSA".to_string(),
            &KeyType::Ecdsa => "ECDSA".to_string(),
            &KeyType::Ed25519 => "ED25519".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl std::fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &KeyFormat::Rfc4716 => "RFC4716".to_string(),
            &KeyFormat::Pkcs8 => "PKCS8".to_string(),
            &KeyFormat::Pem => "PEM".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl std::fmt::Display for OrderByDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            &OrderByDirection::Asc => "ASC".to_string(),
            &OrderByDirection::Desc => "DESC".to_string(),
        };
        write!(f, "{}", msg)
    }
}

#[tonic::async_trait]
impl SshKey for Service {
    async fn get_component(
        &self,
        request: Request<GetComponentRequest>,
    ) -> TonicResult<Response<GetComponentReply>> {
        async {
            event!(Level::INFO, ?request);
            let req = request.into_inner();
            let component = self.db.get(req.component_id).await?;

            let reply = GetComponentReply {
                component: Some(component),
            };
            let response = Response::new(reply);
            event!(Level::INFO, ?response);
            Ok(response)
        }
        .instrument(span!(Level::INFO, "get_component"))
        .await
    }

    async fn list_components(
        &self,
        request: Request<ListComponentsRequest>,
    ) -> TonicResult<Response<ListComponentsReply>> {
        async {
            event!(Level::INFO, ?request);
            let req = request.into_inner();

            // "" is the default value for a protocol buffer string
            let mut order_by = if req.order_by == "" {
                "naturalKey".to_string()
            } else {
                // Make this safe
                req.order_by
            };

            let order_by_direction = OrderByDirection::from_i32(req.order_by_direction)
                .ok_or(Error::InvalidOrderByDirection)?;

            // 0 is the default value for a protocol buffer int32
            let mut page_size = if req.page_size == 0 {
                10
            } else {
                req.page_size
            };

            let mut query = req.query;

            let mut item_id = String::new();

            if req.page_token != "" {
                let page_token = PageToken::unseal(&req.page_token, &self.db.page_secret_key)?;
                query = page_token.query;
                order_by = page_token.order_by;
                page_size = page_token.page_size;
                item_id = page_token.item_id;
            }

            self.is_component_order_by_valid(&order_by)?;

            let list_result: ListResult<Component> = self
                .db
                .list(
                    "component:ssh_key",
                    &query,
                    page_size,
                    &order_by,
                    &order_by_direction.to_string(),
                    &item_id,
                )
                .await?;
            event!(Level::DEBUG, ?list_result);

            // Skip the rest of the logic - the answer is there is nothing
            if list_result.items.len() == 0 {
                return Ok(Response::new(ListComponentsReply::default()));
            }

            let mut reply = ListComponentsReply::default();
            reply.total_count = list_result.total_count;
            reply.component = list_result.items;

            // If we have another page, seal up the info in a page token
            if list_result.next_item_id != "" {
                let mut next_page_token = PageToken::default();
                next_page_token.query = query;
                next_page_token.page_size = page_size;
                next_page_token.order_by = order_by;
                next_page_token.item_id = list_result.next_item_id;
                event!(Level::DEBUG, ?next_page_token);
                reply.next_page_token = next_page_token.seal(&self.db.page_secret_key)?;
            }

            let response = Response::new(reply);
            event!(Level::INFO, ?response);

            Ok(response)
        }
        .instrument(span!(Level::INFO, "list_components"))
        .await
    }

    //async fn pick_component(
    //    &self,
    //    request: Request<PickComponentRequest>,
    //) -> TonicResult<Response<PickComponentReply>> {
    //    async {
    //        event!(Level::INFO, ?request);
    //        let req = request.into_inner();
    //        let component = self.db.get(req.component_id).await?;

    //        let reply = PickComponentReply {
    //            component: Some(component),
    //        };
    //        let response = Response::new(reply);
    //        event!(Level::INFO, ?response);
    //        Ok(response)
    //    }
    //        .instrument(span!(Level::INFO, "pick_component"))
    //        .await
    //}

    async fn create_entity(
        &self,
        request: Request<CreateEntityRequest>,
    ) -> TonicResult<Response<CreateEntityReply>> {
        async {
            event!(Level::INFO, ?request);
            let req = request.into_inner();

            let tenant_id = if req.tenant_id == "" {
                // When this is fleshed out, this should be a legit validation, where we check that the
                // tenant id exists, and that the user making the request has the capabilities required
                return Err(tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "All entities must have a tenant_id",
                ));
            } else {
                req.tenant_id
            };

            let name = if req.name == "" {
                let mut generator = names::Generator::with_naming(names::Name::Numbered);
                generator.next().unwrap() // There is always a name, so this is safe.
            } else {
                req.name
            };

            let description = if req.description == "" {
                name.clone()
            } else {
                req.description
            };

            let query = if req.constraint.is_none() {
                Some(Query {
                    items: vec![QueryExpressionOption {
                        qe: Some(Qe::Expression(QueryExpression {
                            field: "name".to_string(),
                            comparison: QueryComparison::Equals as i32,
                            value: "RSA 3072 RFC4716".to_string(),
                            ..Default::default()
                        })),
                    }],
                    ..Default::default()
                })
            } else {
                req.constraint
            };

            let constraint_result: ListResult<Component> = self
                .db
                .list("component:ssh_key", &query, 1, "bits", "DESC", "")
                .await?;
            event!(Level::DEBUG, ?constraint_result);

            // If we are bigger than 1, we start adding default constraints until
            // we wind up with the default again.
            //
            // This is.. not a sexy pattern. but it's going to work, I think.
            //
            // It super isn't going to work. You have to be much smarter about the
            // internals, and you have to know whats being queried for in the first
            // place.
            let maybe_prototype_component = if constraint_result.total_count == 1 {
                constraint_result.items.into_iter().next() // Safe because we checked above
            } else {
                let extra_search_constraints = vec![
                    QueryExpressionOption {
                        qe: Some(Qe::Expression(QueryExpression {
                            field: "keyFormat".to_string(),
                            comparison: QueryComparison::Equals as i32,
                            value: "RFC4716".to_string(),
                            ..Default::default()
                        })),
                    },
                    QueryExpressionOption {
                        qe: Some(Qe::Expression(QueryExpression {
                            field: "keyType".to_string(),
                            comparison: QueryComparison::Equals as i32,
                            value: "RSA".to_string(),
                            ..Default::default()
                        })),
                    },
                    QueryExpressionOption {
                        qe: Some(Qe::Expression(QueryExpression {
                            field: "bits".to_string(),
                            comparison: QueryComparison::Equals as i32,
                            field_type: QueryFieldType::Int as i32,
                            value: "3072".to_string(),
                        })),
                    },
                ];
                let mut c_query = Query {
                    items: vec![QueryExpressionOption {
                        qe: Some(Qe::Query(query.unwrap())),
                        ..Default::default()
                    }],
                    boolean_term: QueryBooleanLogic::And as i32,
                    ..Default::default()
                };
                let mut final_item = None;
                for cqe in extra_search_constraints.into_iter() {
                    c_query.items.push(cqe);
                    let new_constraint_result: ListResult<Component> = self
                        .db
                        .list(
                            "component:ssh_key",
                            &Some(c_query.clone()),
                            1,
                            "bits",
                            "DESC",
                            "",
                        )
                        .await?;
                    if new_constraint_result.total_count == 1 {
                        final_item = new_constraint_result.items.into_iter().next();
                        break;
                    }
                }
                final_item
            };

            let prototype_component = match maybe_prototype_component {
                Some(c) => c,
                None => return Err(tonic::Status::from(Error::ComponentNotFound)),
            };

            event!(Level::INFO, ?prototype_component);

            let mut agent = Agent::new(
                prototype_component,
                Entity {
                    name: name,
                    description: description,
                    tenant_id: tenant_id,
                    ..Default::default()
                },
            );

            agent.create().await?;

            self.db
                .insert(agent.entity().id.clone(), agent.entity())
                .await?;

            let response = Response::new(CreateEntityReply {
                entity: Some(agent.into_entity()),
            });
            Ok(response)
        }
        .instrument(span!(Level::INFO, "create_entity"))
        .await
    }

    async fn get_entity(
        &self,
        request: Request<GetEntityRequest>,
    ) -> TonicResult<Response<GetEntityReply>> {
        async {
            event!(Level::INFO, ?request);
            let req = request.into_inner();
            let entity: Entity = self.db.get(req.entity_id).await?;

            if entity.tenant_id != req.tenant_id {
                return Err(tonic::Status::from(Error::InvalidTenant));
            }

            let reply = GetEntityReply {
                entity: Some(entity),
            };
            let response = Response::new(reply);
            event!(Level::INFO, ?response);

            Ok(response)
        }
        .instrument(span!(Level::INFO, "get_entity"))
        .await
    }

    async fn list_entities(
        &self,
        request: Request<ListEntitiesRequest>,
    ) -> TonicResult<Response<ListEntitiesReply>> {
        async {
            event!(Level::INFO, ?request);
            let req = request.into_inner();

            // "" is the default value for a protocol buffer string
            let mut order_by = if req.order_by == "" {
                "naturalKey".to_string()
            } else {
                // Make this safe
                req.order_by
            };

            let order_by_direction = OrderByDirection::from_i32(req.order_by_direction)
                .ok_or(Error::InvalidOrderByDirection)?;

            // 0 is the default value for a protocol buffer int32
            let mut page_size = if req.page_size == 0 {
                10
            } else {
                req.page_size
            };

            let mut query = req.query;

            let mut item_id = String::new();

            if req.page_token != "" {
                let page_token = PageToken::unseal(&req.page_token, &self.db.page_secret_key)?;
                query = page_token.query;
                order_by = page_token.order_by;
                page_size = page_token.page_size;
                item_id = page_token.item_id;
            }

            self.is_entity_order_by_valid(&order_by)?;

            let list_result: ListResult<Entity> = self
                .db
                .list(
                    "entity:ssh_key",
                    &query,
                    page_size,
                    &order_by,
                    &order_by_direction.to_string(),
                    &item_id,
                )
                .await?;
            event!(Level::DEBUG, ?list_result);

            // Skip the rest of the logic - the answer is there is nothing
            if list_result.items.len() == 0 {
                return Ok(Response::new(ListEntitiesReply::default()));
            }

            let mut reply = ListEntitiesReply::default();
            reply.total_count = list_result.total_count;
            reply.entity = list_result.items;

            // If we have another page, seal up the info in a page token
            if list_result.next_item_id != "" {
                let mut next_page_token = PageToken::default();
                next_page_token.query = query;
                next_page_token.page_size = page_size;
                next_page_token.order_by = order_by;
                next_page_token.item_id = list_result.next_item_id;
                event!(Level::DEBUG, ?next_page_token);
                reply.next_page_token = next_page_token.seal(&self.db.page_secret_key)?;
            }

            let response = Response::new(reply);
            event!(Level::INFO, ?response);

            Ok(response)
        }
        .instrument(span!(Level::INFO, "list_entities"))
        .await
    }
}
