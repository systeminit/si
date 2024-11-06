use axum::Router;

use axum_util::AppState;

const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/admin", sdf_server_v2_admin::v2_routes(state.clone()))
        .nest(
            &format!("{PREFIX}/audit-logs"),
            sdf_server_v2_audit_log::v2_routes(),
        )
        .nest(PREFIX, sdf_server_v2_change_set::v2_routes(state.clone()))
        .nest(&format!("{PREFIX}/funcs"), sdf_server_v2_func::v2_routes())
        .nest(
            &format!("{PREFIX}/modules"),
            sdf_server_v2_module::v2_routes(),
        )
        .nest(
            &format!("{PREFIX}/schema-variants"),
            sdf_server_v2_variant::v2_routes(),
        )
        .nest(
            &format!("{PREFIX}/management"),
            sdf_server_v2_management::v2_routes(),
        )
        .nest(&format!("{PREFIX}/views"), sdf_server_v2_view::v2_routes())
}
