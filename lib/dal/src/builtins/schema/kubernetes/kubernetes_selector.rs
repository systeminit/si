use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

use crate::builtins::schema::kubernetes::doc_url;

pub async fn create_selector_prop(
    ctx: &DalContext,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let selector_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "selector",
        PropKind::Object,
        None,
        Some(parent_prop_id),
        Some(doc_url(
            "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
        )),
    )
    .await?;

    {
        let match_labels_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "matchLabels",
            PropKind::Map,
            None,
            Some(*selector_prop.id()),
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
            )),
        )
        .await?;
        let _match_labels_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labelValue",
            PropKind::String,
            None,
            Some(*match_labels_prop.id()),
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
            )),
        )
        .await?;
    }

    Ok(selector_prop)
}
