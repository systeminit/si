use super::{FuncAssociations, FuncError, FuncResult, PropAssociation};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    AttributePrototype, DalContext, Func, FuncBackendKind, FuncId,
    Prop, QualificationPrototype, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub is_builtin: bool,
    pub is_revertable: bool,
    pub associations: Option<FuncAssociations>,
}

async fn attribute_proto_prop_association(
    ctx: &DalContext,
    proto: &AttributePrototype,
) -> FuncResult<Option<PropAssociation>> {
    if proto.context.is_least_specific_field_kind_prop()? {
        let prop = Prop::get_by_id(ctx, &proto.context.prop_id())
            .await?
            .ok_or(FuncError::PropNotFound)?;

        Ok(Some(PropAssociation {
            prop_id: *prop.id(),
            name: prop.name().to_string(),
            schema_variant_id: proto.context.schema_variant_id(),
            component_id: if proto.context.component_id().is_some() {
                Some(proto.context.component_id())
            } else {
                None
            },
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let associations = match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            let protos = QualificationPrototype::find_for_func(&ctx, func.id()).await?;

            let mut schema_variant_ids = vec![];
            let mut component_ids = vec![];

            for proto in protos {
                if proto.context().schema_variant_id().is_some() {
                    schema_variant_ids.push(proto.context().schema_variant_id());
                } else if proto.context().component_id().is_some() {
                    component_ids.push(proto.context().component_id());
                }
            }

            Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            })
        }
        FuncBackendKind::JsAttribute => {
            let protos = AttributePrototype::find_for_func(&ctx, func.id()).await?;

            let mut props = vec![];
            for proto in protos {
                let prop = attribute_proto_prop_association(&ctx, &proto).await?;
                if let Some(prop_assoc) = prop {
                    props.push(prop_assoc);
                }
            }

            Some(FuncAssociations::Attribute { props })
        }
        _ => None,
    };

    let is_revertable = super::is_func_revertable(&ctx, &func).await?;

    Ok(Json(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.is_builtin(),
        is_revertable,
        associations,
    }))
}
