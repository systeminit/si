use crate::builtins::schema::create_prop;
use crate::builtins::BuiltinsResult;
use crate::{AttributeReadContext, DalContext};
use crate::{Prop, PropId, PropKind, StandardModel};

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub async fn create_metadata_prop(
    ctx: &DalContext<'_, '_>,
    is_name_required: bool,
    parent_prop_id: Option<PropId>,
    base_attribute_read_context: AttributeReadContext,
) -> BuiltinsResult<Prop> {
    let metadata_prop = create_prop(
        ctx,
        "metadata",
        PropKind::Object,
        parent_prop_id,
        base_attribute_read_context,
    )
    .await?;

    {
        // TODO: add validation
        //validation: [
        //  {
        //    kind: ValidatorKind.Regex,
        //    regex: "^[A-Za-z0-9](?:[A-Za-z0-9-]{0,251}[A-Za-z0-9])?$",
        //    message: "Kubernetes names must be valid DNS subdomains",
        //    link:
        //      "https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-subdomain-names",
        //  },
        //],
        if is_name_required {
            // TODO: add a required field validation here
        }

        let _name_prop = create_prop(
            ctx,
            "name",
            PropKind::String,
            Some(*metadata_prop.id()),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        let _generate_name_prop = create_prop(
            ctx,
            "generateName",
            PropKind::String,
            Some(*metadata_prop.id()),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        // Note: should this come from a k8s namespace component configuring us?
        let _namespace_prop = create_prop(
            ctx,
            "namespace",
            PropKind::String,
            Some(*metadata_prop.id()),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        let labels_prop = create_prop(
            ctx,
            "labels",
            PropKind::Map,
            Some(*metadata_prop.id()),
            base_attribute_read_context,
        )
        .await?;
        let _string_for_labels_prop = create_prop(
            ctx,
            "label",
            PropKind::String,
            Some(*labels_prop.id()),
            base_attribute_read_context,
        )
        .await?;
    }

    {
        let annotations_prop = create_prop(
            ctx,
            "annotations",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
            base_attribute_read_context,
        )
        .await?;
        let _string_for_annotations_prop = create_prop(
            ctx,
            "annotation",
            PropKind::String,
            Some(*annotations_prop.id()),
            base_attribute_read_context,
        )
        .await?;
    }

    Ok(metadata_prop)
}
