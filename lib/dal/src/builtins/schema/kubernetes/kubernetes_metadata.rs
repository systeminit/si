use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

use crate::builtins::schema::kubernetes::doc_url;

pub async fn create_metadata_prop(
    ctx: &DalContext,
    is_name_required: bool,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let metadata_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "metadata",
        PropKind::Object,
        None,
        Some(parent_prop_id),
        Some(doc_url(
            "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
        )),
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

        let _name_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "name",
            PropKind::String,
            None,
            Some(*metadata_prop.id()),
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
            )),
        )
        .await?;
    }

    {
        let _generate_name_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "generateName",
            PropKind::String,
            None,
            Some(*metadata_prop.id()),
            Some(doc_url(
                "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
            )),
        )
        .await?;
    }

    {
        // Note: should this come from a k8s namespace component configuring us?
        let _namespace_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "namespace",
            PropKind::String,
            None,
            Some(*metadata_prop.id()),
            Some(doc_url(
                "concepts/overview/working-with-objects/namespaces/",
            )),
        )
        .await?;
    }

    {
        let labels_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labels",
            PropKind::Map,
            None,
            Some(*metadata_prop.id()),
            Some(doc_url("concepts/overview/working-with-objects/labels/")),
        )
        .await?;
        let _labels_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labelValue",
            PropKind::String,
            None,
            Some(*labels_prop.id()),
            Some(doc_url("concepts/overview/working-with-objects/labels/")),
        )
        .await?;
    }

    {
        let annotations_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "annotations",
            PropKind::Map,
            None, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
            Some(doc_url(
                "concepts/overview/working-with-objects/annotations/",
            )),
        )
        .await?;
        let _annotations_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "annotationValue",
            PropKind::String,
            None,
            Some(*annotations_prop.id()),
            Some(doc_url(
                "concepts/overview/working-with-objects/annotations/",
            )),
        )
        .await?;
    }

    Ok(metadata_prop)
}
