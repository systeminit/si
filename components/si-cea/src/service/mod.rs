use crate::agent::client::AgentClient;
use si_data::Db;
use std::convert::TryFrom;

pub mod auth;

pub mod prelude {
    pub use super::{authnz, into_response};
    pub use crate::{
        component_get, component_list, component_pick, edit, entity_create, entity_event_list,
        entity_get, entity_list, sync, CeaError, Component as _, Entity as _, EntityEvent as _,
        ListRequest as _, Service as CeaService, TonicResult,
    };
    pub use std::convert::TryFrom as _;
    pub use tonic::Request as TonicRequest;
    pub use tracing::{debug, debug_span};
    pub use tracing_futures::Instrument as _;
}

#[derive(Debug)]
pub struct Service {
    pub db: Db,
    pub agent: AgentClient,
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

pub async fn authnz<T>(
    db: &Db,
    request: &tonic::Request<T>,
    endpoint: impl AsRef<str>,
) -> crate::CeaResult<crate::Authentication>
where
    T: std::fmt::Debug,
{
    tracing::debug!(?request);
    let auth = crate::Authentication::try_from(request)?;
    auth.authorize_on_billing_account(db, endpoint.as_ref())
        .await?;

    Ok(auth)
}

pub fn into_response<O, T>(ouput: O) -> crate::TonicResult<T>
where
    T: From<O>,
{
    Ok(tonic::Response::new(ouput.into()))
}

#[macro_export]
macro_rules! component_get {
    ($component:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyComponent = $component;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            authnz(db, &request, ENDPOINT).await?;

            let component_id = request
                .into_inner()
                .component_id
                .ok_or(CeaError::InvalidComponentGetRequestMissingId)?;

            into_response(MyComponent::get(db, &component_id).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! component_list {
    ($component:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyComponent = $component;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            into_response(MyComponent::list(db, &list_request).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! component_pick {
    ($component:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyComponent = $component;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            authnz(db, &request, ENDPOINT).await?;

            let constraints = request
                .into_inner()
                .constraints
                .ok_or(CeaError::InvalidComponentPickRequestMissingConstraints)?;

            into_response(MyComponent::pick(db, &constraints).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! entity_create {
    ($component:ty, $entity:ty, $entity_event:ty, $endpoint:expr, $request:expr, $db:expr, $agent:expr $(,)?) => {{
        type MyComponent = $component;
        type MyEntity = $entity;
        type MyEntityEvent = $entity_event;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;
        let agent = $agent;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let inner = request.into_inner();
            let name = inner
                .name
                .ok_or(CeaError::InvalidEntityCreateRequestMissingField("name"))?;
            let display_name =
                inner
                    .display_name
                    .ok_or(CeaError::InvalidEntityCreateRequestMissingField(
                        "display_name",
                    ))?;
            let description = inner.description;
            let properties = inner.properties.unwrap_or_default();
            let constraints = inner.constraints.unwrap_or_default();
            let workspace_id =
                inner
                    .workspace_id
                    .ok_or(CeaError::InvalidEntityCreateRequestMissingField(
                        "workspace_id",
                    ))?;

            let workspace = db.get(&workspace_id).await?;
            let (implicit_constraints, component) = MyComponent::pick(db, &constraints).await?;

            let entity = MyEntity::from_request_and_component(
                db,
                name,
                display_name,
                description,
                properties,
                constraints,
                component,
                implicit_constraints,
                workspace,
            )
            .await?;
            let entity_event = MyEntityEvent::create(db, auth.user_id(), "create", &entity).await?;
            agent.dispatch(&entity_event).await?;

            into_response((entity, entity_event))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! entity_list {
    ($entity:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyEntity = $entity;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            into_response(MyEntity::list(db, &list_request).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! entity_get {
    ($entity:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyEntity = $entity;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            authnz(db, &request, ENDPOINT).await?;

            let entity_id = request
                .into_inner()
                .entity_id
                .ok_or(CeaError::InvalidEntityGetRequestMissingId)?;

            into_response(MyEntity::get(db, &entity_id).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! sync {
    ($entity:ty, $entity_event:ty, $endpoint:expr, $request:expr, $db:expr, $agent:expr $(,)?) => {{
        type MyEntity = $entity;
        type MyEntityEvent = $entity_event;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;
        let agent = $agent;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let entity_id = request
                .into_inner()
                .entity_id
                .ok_or(CeaError::InvalidEntityGetRequestMissingId)?;

            let entity = MyEntity::get(db, &entity_id).await?;
            let entity_event = MyEntityEvent::create(db, auth.user_id(), ENDPOINT, &entity).await?;
            agent.dispatch(&entity_event).await?;

            into_response(entity_event)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! edit {
    ($entity:ty, $entity_event:ty, $endpoint:expr, $request:expr, $db:expr, $agent:expr, $inner_property:expr, $edit_property:expr $(,)?) => {{
        type MyEntity = $entity;
        type MyEntityEvent = $entity_event;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;
        let agent = $agent;
        let inner_property = $inner_property;
        let edit_property = $edit_property;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut inner = request.into_inner();
            let entity_id = inner
                .entity_id
                .take()
                .ok_or(CeaError::InvalidEntityEditRequestMissingId)?;
            let property =
                inner_property(inner).ok_or(CeaError::InvalidEntityEditRequestMissingProperty)?;

            let mut entity = MyEntity::get(db, &entity_id).await?;
            let previous_entity = entity.clone();
            entity.set_state_transition();
            edit_property(&mut entity, property)?;
            entity.save(db).await?;

            let entity_event = MyEntityEvent::create_with_previous_entity(
                db,
                auth.user_id(),
                ENDPOINT,
                &entity,
                previous_entity,
            )
            .await?;
            agent.dispatch(&entity_event).await?;

            into_response(entity_event)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}

#[macro_export]
macro_rules! entity_event_list {
    ($entity_event:ty, $endpoint:expr, $request:expr, $db:expr $(,)?) => {{
        type MyEntityEvent = $entity_event;
        const ENDPOINT: &str = $endpoint;
        let request = $request;
        let db = $db;

        async {
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            into_response(MyEntityEvent::list(db, &list_request).await?)
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }};
}
