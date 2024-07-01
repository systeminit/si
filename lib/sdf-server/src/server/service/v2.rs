use crate::server::state::AppState;
use axum::Router;

pub mod func;
pub mod variant;

pub fn routes() -> Router<AppState> {
    const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";
    let mut router: Router<AppState> = Router::new();
    router = router
        .nest(
            &format!("{PREFIX}/schema-variants"),
            crate::server::service::v2::variant::v2_routes(),
        )
        .nest(
            &format!("{PREFIX}/funcs"),
            crate::server::service::v2::func::v2_routes(),
        );
    router
}
