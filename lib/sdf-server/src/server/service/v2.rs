use axum::Router;

use crate::server::state::AppState;

pub mod func;
pub mod module;
pub mod variant;

pub fn routes() -> Router<AppState> {
    const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";
    Router::new()
        .nest(
            &format!("{PREFIX}/schema-variants"),
            crate::server::service::v2::variant::v2_routes(),
        )
        .nest(
            &format!("{PREFIX}/funcs"),
            crate::server::service::v2::func::v2_routes(),
        )
        .nest(
            &format!("{PREFIX}/modules"),
            crate::server::service::v2::module::v2_routes(),
        )
}
