use nats::asynk::Connection;
use sodiumoxide::crypto::secretbox;
use warp::Filter;

use si_data::Db;

use crate::handlers;
use crate::models;

// The full API for this service
pub fn api(
    db: &Db,
    nats: &Connection,
    secret_key: &secretbox::Key,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    billing_accounts(db, nats)
        .or(organizations(db, nats))
        .or(nodes(db, nats))
        .or(change_sets(db, nats))
        .or(users(db, nats, secret_key))
        .or(workspaces(db, nats))
        .or(updates(db, nats))
}

// The Web Socket Update API
pub fn updates(
    db: &Db,
    nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("updates")
        .and(warp::ws())
        .and(with_db(db.clone()))
        .and(with_nats(nats.clone()))
        .and(warp::query::<models::update::WebsocketToken>())
        .and_then(handlers::updates::update)
}

// Workspaces API
//   workspaces: GET
pub fn workspaces(
    db: &Db,
    _nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    workspaces_list(db.clone()).or(workspaces_get(db.clone()))
}

pub fn workspaces_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("workspaces" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("workspace".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
}

pub fn workspaces_list(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("workspaces")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("workspace".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
}

// User API
//
// users/login: POST
pub fn users(
    db: &Db,
    _nats: &Connection,
    secret_key: &secretbox::Key,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    users_login(db.clone(), secret_key.clone())
}

pub fn users_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("user".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
}

pub fn users_login(
    db: Db,
    secret_key: secretbox::Key,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / "login")
        .and(warp::post())
        .and(with_db(db))
        .and(with_secret_key(secret_key))
        .and(warp::body::json::<models::user::LoginRequest>())
        .and_then(handlers::users::login)
}

// Billing Account API
//
// billingAccounts
//   billing_account_create: POST
pub fn billing_accounts(
    db: &Db,
    nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    billing_accounts_create(db.clone(), nats.clone())
}

pub fn billing_accounts_create(
    db: Db,
    nats: Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("billingAccounts")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::body::json::<models::billing_account::CreateRequest>())
        .and_then(handlers::billing_accounts::create)
}

// Organization API
// organizations
//   organization_get: GET
pub fn organizations(
    db: &Db,
    _nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    organizations_list(db.clone()).or(organizations_get(db.clone()))
}

pub fn organizations_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("organizations" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("organization".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
}

pub fn organizations_list(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("organizations")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("organization".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
}

// Nodes API
//
// nodes
//   nodes_create: POST
pub fn nodes(
    db: &Db,
    _nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    nodes_create(db.clone())
        .or(nodes_get(db.clone()))
        .or(nodes_object_get(db.clone()))
        .or(nodes_object_patch(db.clone()))
}

pub fn nodes_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("node".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
}

pub fn nodes_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::CreateRequest>())
        .and_then(handlers::nodes::create)
}

pub fn nodes_object_get(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes" / String / "object")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("entity".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
}

pub fn nodes_object_patch(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nodes" / String / "object")
        .and(warp::patch())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::PatchRequest>())
        .and_then(handlers::nodes::patch)
}

// Change Sets API
//
// changeSets
//   change_set_create: POST
//   {change_set_id}
//      PATCH
//          - { execute?hypothetical }
//      editSessions
//          edit_session_create: POST
//
pub fn change_sets(
    db: &Db,
    _nats: &Connection,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    change_set_create(db.clone())
        .or(change_set_patch(db.clone()))
        .or(edit_session_create(db.clone()))
}

pub fn change_set_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("changeSets")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::CreateRequest>())
        .and_then(handlers::change_sets::create)
}

pub fn change_set_patch(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("changeSets" / String)
        .and(warp::patch())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::PatchRequest>())
        .and_then(handlers::change_sets::patch)
}

pub fn edit_session_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("changeSets" / String / "editSessions")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::edit_session::CreateRequest>())
        .and_then(handlers::edit_sessions::create)
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_nats(
    nats: Connection,
) -> impl Filter<Extract = (Connection,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || nats.clone())
}

fn with_secret_key(
    secret_key: secretbox::Key,
) -> impl Filter<Extract = (secretbox::Key,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || secret_key.clone())
}

fn with_string(
    thingy: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || thingy.clone())
}
