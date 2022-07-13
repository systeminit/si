use axum::Json;
use dal::{
    func::backend::js_qualification::FuncBackendJsQualificationArgs, generate_name,
    qualification_prototype::QualificationPrototypeContext, Component, ComponentId, Func,
    FuncBackendKind, FuncBackendResponseType, QualificationPrototype, QualificationPrototypeId,
    Schema, StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{QualificationError, QualificationResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateResponse {
    prototype_id: QualificationPrototypeId,
}

pub async fn create(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateRequest>,
) -> QualificationResult<Json<CreateResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let mut func = Func::new(
        &ctx,
        generate_name(None),
        FuncBackendKind::JsQualification,
        FuncBackendResponseType::Qualification,
    )
    .await?;

    let code = "/*
* Your qualification function
* The signature should never be changed
*
* The input type is `Component`
* The return type is `Qualification`
* 
* interface System {
*   name: string;
* }
*
* // The properties are derived from the fields in the Attributes panel
* interface Properties {
*   si: unknown;
*   domain: unknown
* }
*
* enum Kind {
*   Standard,
*   Credential
* }
*
* interface Data {
*   system: System | null;
*   kind: Kind;
*   properties: Properties;
* }
*
* interface Code {
*   format: string;
*   code: string | null;
* }
*
* interface Component {
*   data: Data;
*   parents: Component[]; // The parent's parents won't be available
*   codes: Code[];
* }
*
* interface Qualification {
*   qualified: boolean;
*   message: string;
* }
*/
async function qualification(component) {
  return {
    qualified: true,
    message: 'Component qualified'
  };
}";
    func.set_code_base64(&ctx, Some(base64::encode(code)))
        .await?;
    func.set_handler(&ctx, Some("qualification".to_owned()))
        .await?;

    let json = serde_json::to_value(&FuncBackendJsQualificationArgs::default())?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(QualificationError::ComponentNotFound(request.component_id))?;
    let variant = component
        .schema_variant(&ctx)
        .await?
        .ok_or(QualificationError::SchemaVariantNotFound)?;

    let mut prototype_context = QualificationPrototypeContext::new();
    prototype_context.set_schema_variant_id(*variant.id());

    let prototype = QualificationPrototype::new(
        &ctx,
        *func.id(),
        json,
        prototype_context,
        generate_name(None),
    )
    .await?;

    let mut components = Vec::new();
    if prototype.context().component_id().is_some() {
        let component_id = prototype.context().component_id();
        let component = Component::get_by_id(&ctx, &component_id)
            .await?
            .ok_or(QualificationError::ComponentNotFound(component_id))?;
        components.push(component);
    } else if prototype.context().schema_variant_id().is_some() {
        let variant_id = prototype.context().schema_variant_id();
        components.extend(Component::list_for_schema_variant(&ctx, variant_id).await?);
    } else if prototype.context().schema_id().is_some() {
        let schema_id = prototype.context().schema_id();
        let schema = Schema::get_by_id(&ctx, &schema_id)
            .await?
            .ok_or(QualificationError::SchemaNotFound(schema_id))?;
        for variant in schema.variants(&ctx).await? {
            components.extend(Component::list_for_schema_variant(&ctx, *variant.id()).await?);
        }
    }

    // TODO: actually use the system to filter qualifications
    let system_id = request.system_id.unwrap_or(SystemId::NONE);

    for component in components {
        ctx.enqueue_job(QualificationJob::new(*component.id(), *system_id, *prototype.id())).await;
    }

    txns.commit().await?;

    Ok(Json(CreateResponse {
        prototype_id: *prototype.id(),
    }))
}
