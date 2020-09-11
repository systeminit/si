use warp::Filter;

use si_data::Db;

use crate::handlers;
use crate::models;

// The full API for this service
pub fn api(db: &Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    nodes(db).or(change_sets(db))
}

// Nodes API
//
// nodes
//   nodes_create: POST
pub fn nodes(db: &Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    nodes_create(db.clone()).or(nodes_get(db.clone()))
    //  .or(entities_get(db.clone()))
    //  .or(entities_update_field(db.clone()))
}

pub fn nodes_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::header::<String>("organizationId"))
        .and(warp::header::<String>("workspaceId"))
        .and(with_string("node".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model)
}

pub fn nodes_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::header::<String>("organizationId"))
        .and(warp::header::<String>("workspaceId"))
        .and(warp::header::<String>("changeSetId"))
        .and(warp::header::<String>("editSessionId"))
        .and(warp::body::json::<models::node::CreateRequest>())
        .and_then(handlers::nodes::create)
}

pub fn nodes_object_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes" / String / "object")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::header::<String>("organizationId"))
        .and(warp::header::<String>("workspaceId"))
        .and(with_string("entity".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model)
}

// Change Sets API
//
// changeSets
//   change_set_create: POST
//   {change_set_id}
//      editSessions
//          edit_session_create: POST
//
pub fn change_sets(
    db: &Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    change_set_create(db.clone()).or(edit_session_create(db.clone()))
    //  .or(entities_get(db.clone()))
    //  .or(entities_update_field(db.clone()))
}

pub fn change_set_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("changeSets")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::header::<String>("organizationId"))
        .and(warp::header::<String>("workspaceId"))
        .and(warp::query::<models::change_set::CreateRequest>())
        .and_then(handlers::change_sets::create)
}

pub fn edit_session_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("changeSets" / String / "editSessions")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("userId"))
        .and(warp::header::<String>("billingAccountId"))
        .and(warp::header::<String>("organizationId"))
        .and(warp::header::<String>("workspaceId"))
        .and(warp::query::<models::edit_session::CreateRequest>())
        .and_then(handlers::edit_sessions::create)
}

// This is where you inject the context!
//
//pub fn entities_list(
//    db: Db,
//) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//    warp::path!("entities")
//        .and(warp::get())
//        .and(with_db(db))
//        .and(warp::header::<String>("userId"))
//        .and(warp::header::<String>("billingAccountId"))
//        .and(warp::query::<models::ListQuery>())
//        .and_then(handlers::list_entities)
//}
//
//pub fn entities_get(
//    db: Db,
//) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//    warp::path!("entities" / String)
//        .and(warp::get())
//        .and(with_db(db))
//        .and(warp::header::<String>("userId"))
//        .and(warp::header::<String>("billingAccountId"))
//        .and_then(handlers::get_entity)
//}
//
//pub fn entities_update_field(
//    db: Db,
//) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//    warp::path!("entities" / String)
//        .and(warp::patch())
//        .and(with_db(db))
//        .and(warp::header::<String>("userId"))
//        .and(warp::header::<String>("billingAccountId"))
//        .and(warp::query::<models::SetField>())
//        .and_then(handlers::update_entity_field)
//}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_string(
    thingy: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || thingy.clone())
}
