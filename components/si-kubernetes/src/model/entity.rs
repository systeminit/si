use crate::protobuf::deployment::{
    self, Constraints, CreateEntityRequest, EditPropObjectRequest, EditPropObjectYamlRequest,
    ListEntitiesReply, ListEntitiesRequest, ListEntityEventsReply, ListEntityEventsRequest,
    PickComponentReply, PickComponentRequest,
};
use si_account::Workspace;
use si_cea::entity::prelude::*;
use std::convert::{TryFrom, TryInto};

pub use crate::protobuf::deployment::entity::{Deployment, State};
pub use crate::protobuf::deployment::{Entity, EntityEvent};

gen_entity!(
    type_name: "kubernetes_deployment_entity",
    order_by_fields: [
        "kubernetesVersion",
        "state"
    ],
    validate_fn: |self| {
        if self.display_name == "" {
            return Err(CeaError::ValidationError("missing display name".to_string()));
        }
        if self.name == "" {
            return Err(CeaError::ValidationError("missing name".to_string()));
        }
        Ok(())
    }
);

impl Entity {
    pub async fn edit_prop_object(&mut self, req: &EditPropObjectRequest) -> CeaResult<()> {
        let prop = req.prop.as_ref().ok_or(CeaError::RequestMissingProp)?;
        self.object = Some(prop.try_into()?);
        self.generate_yaml_from_object()?;
        Ok(())
    }

    pub async fn edit_prop_object_yaml(
        &mut self,
        req: &EditPropObjectYamlRequest,
    ) -> CeaResult<()> {
        let prop = req.prop.as_ref().ok_or(CeaError::RequestMissingProp)?;
        self.object_yaml = Some(prop.into());
        self.generate_object_from_yaml()?;
        Ok(())
    }

    pub async fn from_request_and_component(
        db: &si_data::Db,
        req: &CreateEntityRequest,
        pick_component: PickComponentReply,
        workspace: Workspace,
    ) -> CeaResult<Self> {
        // Safe, because we didn't error way earlier.
        let component = pick_component.component.unwrap();
        let implicit_constraints = pick_component.implicit_constraints;

        let constraints: Option<Constraints> = match &req.constraints {
            Some(c) => Some(c.into()),
            None => None,
        };

        // Don't do it! Not forever. So wasteful. But... so much less
        // shit!
        let k8s_entity_data = match req.props {
            Some(ref props) => match props.object {
                Some(ref object_spec) => {
                    let req_json = serde_json::to_string(object_spec)?;
                    let k8s_entity_data: Deployment = serde_json::from_str(&req_json)?;
                    Some(k8s_entity_data)
                }
                None => None,
            },
            None => None,
        };

        let mut e = Entity {
            tenant_ids: vec![
                workspace.billing_account_id.clone(),
                workspace.organization_id.clone(),
                workspace.id.clone(),
            ],
            name: req.name.clone(),
            display_name: req.display_name.clone(),
            description: req.description.clone(),
            component_id: component.id,
            integration_id: component.integration_id,
            integration_service_id: component.integration_service_id,
            workspace_id: workspace.id,
            organization_id: workspace.organization_id,
            billing_account_id: workspace.billing_account_id,
            constraints,
            implicit_constraints,
            kubernetes_version: component.kubernetes_version,
            object: k8s_entity_data,
            ..Default::default()
        };
        e.generate_yaml_from_object()?;

        db.validate_and_insert_as_new(&mut e).await?;

        Ok(e)
    }

    fn generate_yaml_from_object(&mut self) -> CeaResult<()> {
        if let Some(ref object) = self.object {
            let yaml_string = serde_yaml::to_string(object)?;
            self.object_yaml = Some(yaml_string);
        }

        Ok(())
    }

    fn generate_object_from_yaml(&mut self) -> CeaResult<()> {
        if let Some(ref object_yaml) = self.object_yaml {
            let object: Deployment = serde_yaml::from_str(object_yaml)?;
            self.object = Some(object);
        }

        Ok(())
    }
}

impl From<&PickComponentRequest> for Constraints {
    fn from(pcr: &PickComponentRequest) -> Self {
        Constraints {
            name: pcr.name.clone(),
            display_name: pcr.display_name.clone(),
            integration_id: pcr.integration_id.clone(),
            integration_service_id: pcr.integration_service_id.clone(),
            kubernetes_version: pcr.kubernetes_version.clone(),
        }
    }
}

impl TryFrom<&deployment::edit_prop_object_request::DeploymentRequest> for Deployment {
    type Error = CeaError;

    fn try_from(
        input: &deployment::edit_prop_object_request::DeploymentRequest,
    ) -> std::result::Result<Self, Self::Error> {
        // NOTE(fnichol): Here is our hack to convert one protobuf type into an identically
        // structured but technically different type. Basically, serialize our input type into a
        // JSON string, then deserialize that string into the output type. Clever no? Can anything
        // go wrong? I suspect: pretty much yea, at some point. But it's pretty cool for now; we
        // can maaagic!
        let json_string = serde_json::to_string(input)?;
        serde_json::from_str(&json_string).map_err(CeaError::JsonString)
    }
}

gen_entity_event!(
    type_name: "kubernetes_deployment_entity_event",
    action_names: [
        "create",
        "sync",
        "apply",
        "edit"
    ]
);
