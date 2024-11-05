use axum::Router;

use crate::AppState;

pub mod admin;
pub mod audit_log;
pub mod change_set;
pub mod func;
pub mod management;
pub mod module;
pub mod variant;
pub mod view;

const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";

// use this if you don't need to pass in the change_set id
const CHANGE_SET_PREFIX: &str = "/workspaces/:workspace_id/change-sets";

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/admin", admin::v2_routes(state.clone()))
        .nest(&format!("{PREFIX}/audit-logs"), audit_log::v2_routes())
        .nest(CHANGE_SET_PREFIX, change_set::v2_routes(state.clone()))
        .nest(&format!("{PREFIX}/funcs"), func::v2_routes())
        .nest(&format!("{PREFIX}/modules"), module::v2_routes())
        .nest(&format!("{PREFIX}/schema-variants"), variant::v2_routes())
        .nest(&format!("{PREFIX}/management"), management::v2_routes())
        .nest(&format!("{PREFIX}/views"), view::v2_routes())
}
