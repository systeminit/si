use crate::builtins::schema::MigrationDriver;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

impl MigrationDriver {
    #[allow(dead_code)]
    pub async fn create_kubernetes_template_prop(
        &self,
        ctx: &DalContext,
        parent_prop_id: Option<PropId>,
    ) -> BuiltinsResult<Prop> {
        let template_prop = self
            .create_prop(
                ctx,
                "template",
                PropKind::Object,
                None,
                parent_prop_id,
                None,
            )
            .await?;

        {
            let _optional_metadata_prop = self
                .create_kubernetes_metadata_prop(ctx, false, *template_prop.id())
                .await?;
        }

        {
            let _spec_prop = self
                .create_kubernetes_spec_prop(ctx, *template_prop.id())
                .await?;
        }
        Ok(template_prop)
    }
}
