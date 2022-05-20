use crate::{
    builtins::{schema::create_prop, BuiltinsResult},
    DalContext, Prop, PropId, PropKind, StandardModel,
};

use super::kubernetes::doc_url;

pub async fn create_selector_prop(
    ctx: &DalContext<'_, '_>,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut selector_prop =
        create_prop(ctx, "selector", PropKind::Object, Some(parent_prop_id)).await?;
    selector_prop
        .set_doc_link(
            ctx,
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
            )),
        )
        .await?;

    {
        let mut match_labels_prop = create_prop(
            ctx,
            "matchLabels",
            PropKind::Map, // How to specify it as an array of strings?
            Some(*selector_prop.id()),
        )
        .await?;
        match_labels_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
                )),
            )
            .await?;
        let mut match_labels_value_prop = create_prop(
            ctx,
            "labelValue",
            PropKind::String,
            Some(*match_labels_prop.id()),
        )
        .await?;
        match_labels_value_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
                )),
            )
            .await?;
    }

    Ok(selector_prop)
}
