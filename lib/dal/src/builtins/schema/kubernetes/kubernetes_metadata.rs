use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

use crate::builtins::schema::kubernetes::doc_url;

pub async fn create_metadata_prop(
    ctx: &DalContext<'_, '_, '_>,
    is_name_required: bool,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut metadata_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "metadata",
        PropKind::Object,
        Some(parent_prop_id),
        None,
    )
    .await?;
    metadata_prop
        .set_doc_link(
            ctx,
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

        let mut name_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "name",
            PropKind::String,
            Some(*metadata_prop.id()),
            None,
        )
        .await?;
        name_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
                )),
            )
            .await?;
    }

    {
        let mut generate_name_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "generateName",
            PropKind::String,
            Some(*metadata_prop.id()),
            None,
        )
        .await?;
        generate_name_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "reference/kubernetes-api/common-definitions/object-meta/#ObjectMeta",
                )),
            )
            .await?;
    }

    {
        // Note: should this come from a k8s namespace component configuring us?
        let mut namespace_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "namespace",
            PropKind::String,
            Some(*metadata_prop.id()),
            None,
        )
        .await?;
        namespace_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "concepts/overview/working-with-objects/namespaces/",
                )),
            )
            .await?;
    }

    {
        let mut labels_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labels",
            PropKind::Map,
            Some(*metadata_prop.id()),
            None,
        )
        .await?;
        labels_prop
            .set_doc_link(
                ctx,
                Some(doc_url("concepts/overview/working-with-objects/labels/")),
            )
            .await?;
        let mut labels_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "labelValue",
            PropKind::String,
            Some(*labels_prop.id()),
            None,
        )
        .await?;
        labels_value_prop
            .set_doc_link(
                ctx,
                Some(doc_url("concepts/overview/working-with-objects/labels/")),
            )
            .await?;
    }

    {
        let mut annotations_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "annotations",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
            None,
        )
        .await?;
        annotations_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "concepts/overview/working-with-objects/annotations/",
                )),
            )
            .await?;
        let mut annotations_value_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "annotationValue",
            PropKind::String,
            Some(*annotations_prop.id()),
            None,
        )
        .await?;
        annotations_value_prop
            .set_doc_link(
                ctx,
                Some(doc_url(
                    "concepts/overview/working-with-objects/annotations/",
                )),
            )
            .await?;
    }

    Ok(metadata_prop)
}
