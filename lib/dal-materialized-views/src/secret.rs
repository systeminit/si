use dal::{
    DalContext,
    Prop,
    SchemaVariant,
    Secret,
    SecretId,
    prop::PropPath,
};
use si_frontend_mv_types::secret::Secret as SecretMv;
use telemetry::prelude::*;

use crate::Error;

pub mod secret_definition;
pub mod secret_definition_list;
pub mod secret_list;

#[instrument(name = "dal_materialized_views.secret", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext, secret_id: SecretId) -> super::Result<SecretMv> {
    let ctx = &ctx;
    let secret_definition_path = PropPath::new(["root", "secret_definition"]);

    let secret = Secret::get_by_id(ctx, secret_id).await?;
    let mut maybe_secret_prop = None;

    // find which prop this is for
    let all_secret_svs = SchemaVariant::list_default_secret_defining_ids(ctx).await?;
    for schema_variant_id in all_secret_svs {
        let maybe_secret_definition_prop_id =
            Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &secret_definition_path).await?;

        // We have found a schema variant with a secret definition!
        if maybe_secret_definition_prop_id.is_some() {
            // Get the secret output socket corresponding to the definition. There should only be one
            // output socket as secret defining schema variants are required to have one and only one.
            let secret_output_socket =
                SchemaVariant::find_output_socket_for_secret_defining_id(ctx, schema_variant_id)
                    .await?;
            if secret_output_socket.name() == secret.definition() {
                maybe_secret_prop = maybe_secret_definition_prop_id;
            }
        }
    }
    let definition_id = match maybe_secret_prop {
        Some(prop) => prop,
        None => return Err(Error::Secret(dal::SecretError::SecretNotFound(secret_id))),
    };
    Ok(SecretMv {
        id: secret_id,
        name: secret.name().to_owned(),
        label: secret.definition().to_owned(),
        description: secret.description().to_owned(),
        is_usable: secret.can_be_decrypted(ctx).await?,
        definition_id: definition_id.into(),
    })
}
