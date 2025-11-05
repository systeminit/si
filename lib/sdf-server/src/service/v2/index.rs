use axum::{
    Router,
    routing::{
        get,
        post,
    },
};
use sdf_core::index::IndexResult;

use super::AccessBuilder;
use crate::AppState;

mod get_change_set_index;
mod get_front_end_object;
mod rebuild_change_set_index;

pub fn v2_change_set_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_change_set_index::get_change_set_index))
        .route("/mjolnir", get(get_front_end_object::get_front_end_object))
        .route(
            "/multi_mjolnir",
            post(get_front_end_object::get_multiple_front_end_objects),
        )
        .route(
            "/rebuild",
            post(rebuild_change_set_index::rebuild_change_set_index),
        )
}
