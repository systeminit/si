use crate::builtins::schema::kubernetes::doc_url;
use crate::builtins::schema::MigrationDriver;
use crate::{BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

impl MigrationDriver {
    pub async fn create_kubernetes_deployment_spec_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
    ) -> BuiltinsResult<Prop> {
        let spec_prop = self
            .create_prop(
                ctx,
                "spec",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/#DeploymentSpec",
                )),
            )
            .await?;

        let _replicas_prop = self
            .create_prop(
                ctx,
                "replicas",
                PropKind::Integer,
                None,
                Some(*spec_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/deployment-v1/#DeploymentSpec",
                )),
            )
            .await?;

        let _selector_prop = self
            .create_kubernetes_selector_prop(ctx, *spec_prop.id())
            .await?;
        let _template_prop = self
            .create_kubernetes_pod_template_spec_prop(ctx, *spec_prop.id())
            .await?;

        Ok(spec_prop)
    }

    async fn create_kubernetes_pod_template_spec_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
    ) -> BuiltinsResult<Prop> {
        let template_prop = self
            .create_prop(
                ctx,
                "template",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-template-v1/#PodTemplateSpec",
                )),
            )
            .await?;

        let _metadata_prop = self
            .create_kubernetes_metadata_prop(
                ctx,
                true, // is name required, note: bool is not ideal here tho
                *template_prop.id(),
            )
            .await?;

        let _spec_prop = self
            .create_kubernetes_pod_spec_prop(ctx, *template_prop.id())
            .await?;

        Ok(template_prop)
    }

    async fn create_kubernetes_pod_spec_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
    ) -> BuiltinsResult<Prop> {
        let spec_prop = self
            .create_prop(
                ctx,
                "spec",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#PodSpec",
                )),
            )
            .await?;

        let containers_prop = self
            .create_prop(
                ctx,
                "containers",
                PropKind::Array,
                None,
                Some(*spec_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#containers",
                )),
            )
            .await?;
        let _containers_element_prop = self
            .create_kubernetes_container_prop(ctx, *containers_prop.id())
            .await?;

        Ok(spec_prop)
    }

    async fn create_kubernetes_container_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
    ) -> BuiltinsResult<Prop> {
        let container_prop = self
            .create_prop(
                ctx,
                "container",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#Container",
                )),
            )
            .await?;

        let _name_prop = self
            .create_prop(
                ctx,
                "name",
                PropKind::String,
                None,
                Some(*container_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#Container",
                )),
            )
            .await?;

        let _image_prop = self
            .create_prop(
                ctx,
                "image",
                PropKind::String,
                None,
                Some(*container_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#image",
                )),
            )
            .await?;

        let ports_prop = self
            .create_prop(
                ctx,
                "ports",
                PropKind::Array,
                None,
                Some(*container_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#ports",
                )),
            )
            .await?;
        let _ports_element_prop = self
            .create_kubernetes_container_port_prop(ctx, *ports_prop.id())
            .await?;

        Ok(container_prop)
    }

    async fn create_kubernetes_container_port_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: PropId,
    ) -> BuiltinsResult<Prop> {
        let port_prop = self
            .create_prop(
                ctx,
                "port",
                PropKind::Object,
                None,
                Some(parent_prop_id),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#ports",
                )),
            )
            .await?;

        let container_port_prop = self
            .create_prop(
                ctx,
                "containerPort",
                PropKind::Integer,
                None,
                Some(*port_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#ports",
                )),
            )
            .await?;

        let _protocol_prop = self
            .create_prop(
                ctx,
                "protocol",
                PropKind::String,
                None,
                Some(*port_prop.id()),
                Some(doc_url(
                    "reference/kubernetes-api/workload-resources/pod-v1/#ports",
                )),
            )
            .await?;

        Ok(container_port_prop)
    }
}
