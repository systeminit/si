use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

use si_data::{DataQuery, Db};

use crate::handlers;
use crate::models;

// Backend API
//
// Create - POST /entity/
// Update - PUT /entity/{id}
// Delete - DELETE /entity/{id}
// Get    - GET /entity/{id}
// List   - GET /entity?{query}
//

// This is where you inject the context!
pub fn entities(
    db: &Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    entities_list(db.clone())
        .or(entities_get(db.clone()))
        .or(entities_update_field(db.clone()))
}

pub fn entities_list(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entities")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::query::<models::ListQuery>())
        .and_then(handlers::list_entities)
}

pub fn entities_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entities" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and_then(handlers::get_entity)
}

pub fn entities_update_field(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entities" / String)
        .and(warp::patch())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::query::<models::SetField>())
        .and_then(handlers::update_entity_field)
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
