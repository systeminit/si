use axum::{
    Json,
    response::IntoResponse,
};
use dal::{
    ChangeSet,
    Component,
    DalContext,
    Prop,
    Schema,
    SchemaVariant,
    Secret,
    SecretAlgorithm,
    SecretVersion,
    diagram::view::View,
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
};
use sdf_extract::{
    PosthogEventTracker,
    workspace::WorkspaceDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_id::KeyPairPk;

use super::Result;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Provider {
    #[serde(rename_all = "camelCase")]
    Aws {
        region: String,
    },
    #[serde(rename_all = "camelCase")]
    Azure {
        location: String,
        subscription_id: String,
    },
    Hetzner {},      // No other data for Hetzner besides the credential!
    Digitalocean {}, // No other data for DigitalOcean besides the credential!
    Gcp {},          // No other data for Google Cloud Platform besides the credential!
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialData {
    pub name: String,
    pub description: Option<String>,
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub credential: CredentialData,
    pub provider: Provider,
}

const INITIALIZATION_CHANGE_SET_NAME: &str = "Initialization";

pub async fn create_initialize_apply(
    WorkspaceDalContext(ref mut ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse> {
    let mut initialization_change_set =
        ChangeSet::fork_head(ctx, INITIALIZATION_CHANGE_SET_NAME).await?;
    ctx.write_audit_log(
        AuditLogKind::CreateChangeSet,
        INITIALIZATION_CHANGE_SET_NAME.to_string(),
    )
    .await?;

    let mut initialization_changeset_ctx =
        ctx.clone_with_new_visibility(initialization_change_set.id.into());
    let initialization_result =
        initialize_and_apply(&mut initialization_changeset_ctx, request).await;

    // Ensure that if something went wrong on initialization, we abandon the change set
    if initialization_result.is_err() {
        let old_status = initialization_change_set.status;
        initialization_change_set.abandon(ctx).await?;
        ctx.write_audit_log(
            AuditLogKind::AbandonChangeSet {
                from_status: old_status.into(),
            },
            initialization_change_set.name,
        )
        .await?;

        return initialization_result;
    }

    // TODO set fields on this new event
    tracker.track(
        ctx,
        "onboarded",
        serde_json::json!({
            "change_set_name": INITIALIZATION_CHANGE_SET_NAME,
        }),
    );

    Ok(())
}

async fn initialize_and_apply(ctx: &mut DalContext, request: Request) -> Result<()> {
    // TODO: there's still some audit logs missing on this function
    let view_id = View::get_id_for_default(ctx).await?;
    let credential_name = request.credential.name;

    let (
        credential_schema_name,
        credential_prop_path,
        region_or_location_schema,
        region_or_location,
        region_prop_path,
        region_credential_path,
        subscription_id,
    ) = match &request.provider {
        Provider::Aws { region } => (
            "AWS Credential",
            "/secrets/AWS Credential",
            Some("Region"),
            region.clone(),
            "/domain/region",
            "/secrets/credential",
            None,
        ),
        Provider::Azure {
            location,
            subscription_id,
        } => (
            "Microsoft Credential",
            "/secrets/Microsoft Credential",
            Some("Microsoft.Resources/locations"),
            location.clone(),
            "/domain/name",
            "/secrets/Microsoft Credential",
            Some(subscription_id.clone()),
        ),
        Provider::Hetzner {} => (
            "Hetzner::Credential::ApiToken",
            "/secrets/Hetzner::Credential::ApiToken",
            None,
            "".to_string(),
            "",
            "",
            None,
        ),
        Provider::Digitalocean {} => (
            "DigitalOcean Credential",
            "/secrets/DigitalOcean Credential",
            None,
            "".to_string(),
            "",
            "",
            None,
        ),
        Provider::Gcp {} => (
            "Google Cloud Credential",
            "/secrets/Google Cloud Credential",
            None,
            "".to_string(),
            "",
            "",
            None,
        ),
    };

    // Create the credential component
    let credential_component = {
        let schema = Schema::get_or_install_by_name(ctx, credential_schema_name).await?;
        let sv_id = SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;
        let component = Component::new(ctx, &credential_name, sv_id, view_id).await?;

        // Create the secret and set it to the credential
        let secret = Secret::new(
            ctx,
            &credential_name,
            credential_schema_name.to_owned(),
            request.credential.description,
            &request.credential.crypted,
            request.credential.key_pair_pk,
            request.credential.version,
            request.credential.algorithm,
        )
        .await?;

        ctx.write_audit_log(
            AuditLogKind::CreateSecret {
                name: secret.name().to_string(),
                secret_id: secret.id(),
            },
            secret.name().to_string(),
        )
        .await?;

        // Find the credential prop and attach the secret
        {
            let credential_prop = Prop::find_prop_by_path(
                ctx,
                sv_id,
                &PropPath::new(["root", "secrets", credential_schema_name]),
            )
            .await?;

            let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;
            let credential_attribute_value_id =
                property_values.find_by_prop_id_or_err(credential_prop.id())?;

            Secret::attach_for_attribute_value(
                ctx,
                credential_attribute_value_id,
                Some(secret.id()),
            )
            .await?;
        }

        let variant = SchemaVariant::get_by_id(ctx, sv_id).await?;

        ctx.write_audit_log(
            AuditLogKind::CreateComponent {
                name: credential_name.clone(),
                component_id: component.id(),
                schema_variant_id: sv_id,
                schema_variant_name: variant.display_name().to_owned(),
            },
            credential_name.clone(),
        )
        .await?;

        component
    };

    // creating a region or schema, skip this section for GCP, Hetzner, and DigitalOcean
    if let Some(schema_to_create) = region_or_location_schema {
        if let Some(sub_id) = subscription_id {
            let comp_name = "SubscriptionIdComponent".to_string();
            let schema_name = "Microsoft.Resources/subscription";
            let schema = Schema::get_or_install_by_name(ctx, schema_name).await?;
            let sv_id = SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;
            let component = Component::new(ctx, comp_name.clone(), sv_id, view_id).await?;

            let variant = SchemaVariant::get_by_id(ctx, sv_id).await?;

            ctx.write_audit_log(
                AuditLogKind::CreateComponent {
                    name: comp_name.clone(),
                    component_id: component.id(),
                    schema_variant_id: sv_id,
                    schema_variant_name: variant.display_name().to_owned(),
                },
                comp_name.clone(),
            )
            .await?;

            {
                let sources = serde_json::from_value(serde_json::json!({
                    "/domain/subscriptionId" : sub_id,
                }))?;

                dal::update_attributes(ctx, component.id(), sources).await?;
            }
        };

        // Create the region/location component
        let region_component = {
            let schema = Schema::get_or_install_by_name(ctx, schema_to_create).await?;
            let sv_id = SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;
            let component = Component::new(ctx, &region_or_location, sv_id, view_id).await?;

            let variant = SchemaVariant::get_by_id(ctx, sv_id).await?;

            ctx.write_audit_log(
                AuditLogKind::CreateComponent {
                    name: region_or_location.clone(),
                    component_id: component.id(),
                    schema_variant_id: sv_id,
                    schema_variant_name: variant.display_name().to_owned(),
                },
                region_or_location.clone(),
            )
            .await?;

            component
        };

        // Set the region/location value and subscribe the region's credential to the cred component
        {
            let sources = serde_json::from_value(serde_json::json!({
                region_prop_path : region_or_location,
                region_credential_path: {
                "$source": { "component": credential_component.id().to_string(), "path": credential_prop_path }
                },
            }))?;

            dal::update_attributes(ctx, region_component.id(), sources).await?;
        }
    }

    // Commit so the components are created and the DVU is triggered
    ctx.commit().await?;

    // Await the DVU before applying the changeset
    ChangeSet::wait_for_dvu(ctx, true).await?;

    let change_set = ctx.change_set()?.clone();
    let old_status = change_set.status;

    ChangeSet::prepare_for_force_apply(ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        change_set.name.clone(),
    )
    .await?;

    ChangeSet::apply_to_base_change_set(ctx).await?;

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name.clone())
        .await?;

    Ok(())
}
