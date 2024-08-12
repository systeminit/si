use axum::Router;

use crate::server::state::AppState;

pub mod admin;
pub mod func;
pub mod module;
pub mod variant;

const WORKSPACE_ONLY_PREFIX: &str = "/workspaces/:workspace_id";
const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest(
            &format!("{WORKSPACE_ONLY_PREFIX}/admin"),
            admin::v2_routes(),
        )
        .nest(&format!("{PREFIX}/schema-variants"), variant::v2_routes())
        .nest(&format!("{PREFIX}/funcs"), func::v2_routes())
        .nest(&format!("{PREFIX}/modules"), module::v2_routes())
}
