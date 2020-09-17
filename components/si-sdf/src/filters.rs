use sodiumoxide::crypto::secretbox;
use warp::Filter;

use si_data::Db;

use crate::handlers;
use crate::models;

// The full API for this service
pub fn api(
    db: &Db,
    secret_key: &secretbox::Key,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    billing_accounts(db)
        .or(nodes(db))
        .or(change_sets(db))
        .or(users(db, secret_key))
}

// Authentication header check - rejects anything without

// User API
//
// users/login: POST
pub fn users(
    db: &Db,
    secret_key: &secretbox::Key,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    users_login(db.clone(), secret_key.clone())
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
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    billing_accounts_create(db.clone())
}

pub fn billing_accounts_create(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("billingAccounts")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::body::json::<models::billing_account::CreateRequest>())
        .and_then(handlers::billing_accounts::create)
}

// Nodes API
//
// nodes
//   nodes_create: POST
pub fn nodes(db: &Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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
