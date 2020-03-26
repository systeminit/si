pub mod auth;

use crate::agent::client::AgentClient;
use si_data::Db;

pub mod prelude {

    pub use crate::{
        gen_service_action, gen_service_create_entity, gen_service_get, gen_service_list,
        gen_service_pick_component, Component as _, Entity as _, EntityEvent as _,
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

#[macro_export]
macro_rules! gen_service_action {
    ($self:ident, $request:ident, $endpoint:tt, $action:tt, $response:ident) => {
        async {
            let auth = si_cea::Authentication::try_from(&$request)?;
            auth.authorize_on_billing_account($self.db(), $endpoint)
                .await?;

            let req = $request.get_ref();

            let entity = Entity::get($self.db(), &req.entity_id).await?;

            let entity_event =
                EntityEvent::create($self.db(), auth.user_id(), $action, &entity).await?;

            $self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new($response {
                event: Some(entity_event),
                ..Default::default()
            }))
        }
        .instrument(debug_span!($endpoint, ?$request))
        .await
    };
}

#[macro_export]
macro_rules! gen_service_list {
    ($self:ident, $request:ident, $endpoint:tt, $list_type:ident) => {
        async {
            debug!(?$request);
            let auth = { si_cea::Authentication::try_from(&$request)? };
            auth.authorize_on_billing_account(&$self.db, $endpoint)
                .await?;
            {
                $request.get_mut().default_scope_by_tenant_id(&auth);
            }
            let req = $request.get_ref();

            let list_result = $list_type::list(&$self.db, req).await?;
            Ok(tonic::Response::new(list_result.into()))
        }
        .instrument(debug_span!($endpoint))
        .await
    };
}

#[macro_export]
macro_rules! gen_service_get {
    ($self:ident, $request:ident, $endpoint:tt, $get_type:ident, $get_field:ident, $response_ty:ident, $response_field:ident) => {
        async {
            let auth = si_cea::Authentication::try_from(&$request)?;

            auth.authorize_on_billing_account(&$self.db, $endpoint)
                .await?;

            let req = $request.get_ref();

            let item = $get_type::get(&$self.db, &req.$get_field).await?;

            Ok(tonic::Response::new($response_ty {
                $response_field: Some(item),
            }))
        }
        .instrument(debug_span!($endpoint, ?$request))
        .await
    };
}

#[macro_export]
macro_rules! gen_service_pick_component {
    ($self:ident, $request:ident, $endpoint:tt, $response_ty:ident) => {
        async {
            let auth = si_cea::Authentication::try_from(&$request)?;

            auth.authorize_on_billing_account(&$self.db, $endpoint)
                .await?;

            let req = $request.get_ref();

            let (implicit_constraints, chosen_component) = Component::pick(&$self.db, req).await?;
            let mut return_implicit: Vec<ImplicitConstraint> = implicit_constraints.into();

            Ok(tonic::Response::new($response_ty {
                component: Some(chosen_component),
                implicit_constraints: return_implicit,
                ..Default::default()
            }))
        }
        .instrument(debug_span!($endpoint, ?$request))
        .await
    };
}

#[macro_export]
macro_rules! gen_service_create_entity {
    ($self:ident, $request:ident, $endpoint:tt, $response_ty:ident) => {
        async {
            let auth = si_cea::Authentication::try_from(&$request)?;
            auth.authorize_on_billing_account(&$self.db, $endpoint)
                .await?;
            let req: &CreateEntityRequest = $request.get_ref();

            let workspace = $self.db.get(&req.workspace_id).await?;

            let constraints = match &req.constraints {
                Some(constraint) => constraint.clone(),
                None => PickComponentRequest::default(),
            };

            let (implicit_constraints, chosen_component) =
                Component::pick(&$self.db, &constraints).await?;
            let mut pick_component: PickComponentReply = PickComponentReply::default();
            pick_component.component = Some(chosen_component);
            pick_component.implicit_constraints = implicit_constraints.into();

            let entity =
                Entity::from_request_and_component(&$self.db, req, pick_component, workspace)
                    .await?;

            let entity_event =
                EntityEvent::create(&$self.db, auth.user_id(), "create", &entity).await?;

            $self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new($response_ty {
                entity: Some(entity),
                event: Some(entity_event),
            }))
        }
        .instrument(debug_span!($endpoint, ?$request))
        .await
    };
}
