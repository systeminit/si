use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::DalContext;
use crate::{Prop, PropId, PropKind, StandardModel};

#[allow(clippy::too_many_arguments)]
pub async fn create_selector_prop(
    ctx: &DalContext<'_, '_>,
    parent_prop_id: Option<PropId>,
) -> SchemaResult<Prop> {
    let selector_prop = create_prop(ctx, "selector", PropKind::Object, parent_prop_id).await?;

    {
        let match_expressions_prop = create_prop(
            ctx,
            "matchExpressions",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*selector_prop.id()),
        )
        .await?;

        {
            let _key_prop = create_prop(
                ctx,
                "key",
                PropKind::String,
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }

        {
            // TODO: validate to ensure it's either "In", "NotInt", "Exists", "DoesNotExist"
            // Is there a selector widget? If so how to enable it
            let _operator_prop = create_prop(
                ctx,
                "operator",
                PropKind::String,
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }

        {
            let _values_prop = create_prop(
                ctx,
                "values",
                PropKind::Array, // How to specify it as an array of strings?
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }
    }

    {
        let _match_labels_prop = create_prop(
            ctx,
            "matchLabels",
            PropKind::Array, // How to specify it as an array of strings?
            Some(*selector_prop.id()),
        )
        .await?;
    }
    Ok(selector_prop)
}
