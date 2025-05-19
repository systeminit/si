use dal::{
    DalContext,
    Secret,
};
use si_frontend_mv_types::secret::SecretList as SecretListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.secret_definition_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> crate::Result<SecretListMv> {
    let id = ctx.change_set_id();
    let ctx = &ctx;
    let mut secrets = Vec::new();
    for secret in Secret::list(ctx).await? {
        secrets.push(secret.id());
    }
    secrets.sort();

    Ok(SecretListMv {
        secrets: secrets.iter().map(|&secret_id| secret_id.into()).collect(),
        id,
    })
}
