use crate::{
    builtins::{schema::create_prop, BuiltinsResult},
    DalContext, Prop, PropId, PropKind, StandardModel,
};

use super::kubernetes::doc_url;

pub async fn create_metadata_prop(
    ctx: &DalContext<'_, '_>,
    is_name_required: bool,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let mut metadata_prop =
        create_prop(ctx, "metadata", PropKind::Object, Some(parent_prop_id)).await?;
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

        let mut name_prop =
            create_prop(ctx, "name", PropKind::String, Some(*metadata_prop.id())).await?;
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
        let mut generate_name_prop = create_prop(
            ctx,
            "generateName",
            PropKind::String,
            Some(*metadata_prop.id()),
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
        // NOTE(nick): since k8s namespace uses metadata, should we drop this field? My gut says
        // no, but maybe we can hide this in "web".
        let mut namespace_prop = create_prop(
            ctx,
            "namespace",
            PropKind::String,
            Some(*metadata_prop.id()),
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
        let mut labels_prop =
            create_prop(ctx, "labels", PropKind::Map, Some(*metadata_prop.id())).await?;
        labels_prop
            .set_doc_link(
                ctx,
                Some(doc_url("concepts/overview/working-with-objects/labels/")),
            )
            .await?;
        let mut labels_value_prop =
            create_prop(ctx, "labelValue", PropKind::String, Some(*labels_prop.id())).await?;
        labels_value_prop
            .set_doc_link(
                ctx,
                Some(doc_url("concepts/overview/working-with-objects/labels/")),
            )
            .await?;
    }

    {
        let mut annotations_prop = create_prop(
            ctx,
            "annotations",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
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
        let mut annotations_value_prop = create_prop(
            ctx,
            "annotationValue",
            PropKind::String,
            Some(*annotations_prop.id()),
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
