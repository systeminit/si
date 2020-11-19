use nats::asynk::Connection;
use sodiumoxide::crypto::secretbox;
use warp::{filters::BoxedFilter, Filter};

use crate::data::Db;

use crate::handlers;
use crate::models;

#[cfg(debug_assertions)]
pub fn api(
    db: &Db,
    nats: &Connection,
    secret_key: &secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    billing_accounts(db, nats)
        .or(organizations(db, nats))
        .or(nodes(db, nats))
        .or(change_sets(db, nats))
        .or(users(db, nats, secret_key))
        .or(workspaces(db, nats))
        .or(updates(db, nats))
        .or(entities(db))
        .or(systems(db))
        .or(edges(db, nats))
        .or(change_set_participants(db, nats))
        .or(secrets(db, nats))
        .boxed()
}

#[cfg(not(debug_assertions))]
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
        .or(entities(db))
        .or(systems(db))
}

// The Web Socket Update API
pub fn updates(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("updates")
        .and(warp::ws())
        .and(with_db(db.clone()))
        .and(with_nats(nats.clone()))
        .and(warp::query::<models::update::WebsocketToken>())
        .and_then(handlers::updates::update)
        .boxed()
}

// Workspaces API
//   workspaces: GET
pub fn workspaces(db: &Db, _nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    workspaces_list(db.clone())
        .or(workspaces_get(db.clone()))
        .boxed()
}

pub fn workspaces_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("workspaces" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("workspace".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn workspaces_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("workspaces")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("workspace".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

// Edges API
//   edges: GET - list
//     edges/{id}: GET - get
//     edges/{id}: DELETE - delete
//     edges/allPredecessorEdges?object_id|node_id: GET - get_all_predecessor_edges
pub fn edges(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    edges_list(db.clone())
        .or(edges_all_predecessors(db.clone()))
        .or(edges_all_successors(db.clone()))
        .or(edges_get(db.clone()))
        .or(edges_delete(db.clone(), nats.clone()))
        .boxed()
}

pub fn edges_all_predecessors(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / "allPredecessors")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::edge::AllPredecessorsRequest>())
        .and_then(handlers::edges::all_predecessors)
        .boxed()
}

pub fn edges_all_successors(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / "allSuccessors")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::edge::AllSuccessorsRequest>())
        .and_then(handlers::edges::all_successors)
        .boxed()
}

pub fn edges_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("edge".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn edges_delete(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / String)
        .and(warp::delete())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::edges::delete)
        .boxed()
}

pub fn edges_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("edge".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

// User API
//
// users/login: POST
pub fn users(
    db: &Db,
    _nats: &Connection,
    secret_key: &secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    users_login(db.clone(), secret_key.clone()).boxed()
}

pub fn users_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("users" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("user".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn users_login(db: Db, secret_key: secretbox::Key) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("users" / "login")
        .and(warp::post())
        .and(with_db(db))
        .and(with_secret_key(secret_key))
        .and(warp::body::json::<models::user::LoginRequest>())
        .and_then(handlers::users::login)
        .boxed()
}

// Billing Account API
//
// billingAccounts
//   billing_account_create: POST
//pub fn billing_accounts(
//    db: &Db,
//    nats: &Connection,
//) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {

pub fn billing_accounts(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    //impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    billing_accounts_create(db.clone(), nats.clone())
        .or(billing_accounts_get(db.clone()))
        .or(billing_accounts_get_public_key(db.clone()))
        .boxed()
}

pub fn billing_accounts_create(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    //impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("billingAccounts")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::body::json::<models::billing_account::CreateRequest>())
        .and_then(handlers::billing_accounts::create)
        .boxed()
}

pub fn billing_accounts_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("billingAccounts" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("billingAccount".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn billing_accounts_get_public_key(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("billingAccounts" / String / "publicKey")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("billingAccount".into()))
        .and_then(handlers::billing_accounts::get_public_key)
        .boxed()
}

// Organization API
// organizations
//   organization_get: GET
pub fn organizations(db: &Db, _nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    organizations_list(db.clone())
        .or(organizations_get(db.clone()))
        .boxed()
}

pub fn organizations_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("organizations" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("organization".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn organizations_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("organizations")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("organization".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

// Nodes API
//
// nodes
//   nodes_create: POST
pub fn nodes(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    nodes_create(db.clone(), nats.clone())
        .or(nodes_get(db.clone()))
        .or(nodes_patch(db.clone(), nats.clone()))
        .or(nodes_object_get(db.clone()))
        .or(nodes_object_patch(db.clone(), nats.clone()))
        .boxed()
}

pub fn nodes_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("node".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn nodes_patch(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String)
        .and(warp::patch())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::PatchRequest>())
        .and_then(handlers::nodes::patch)
        .boxed()
}

pub fn nodes_create(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::CreateRequest>())
        .and_then(handlers::nodes::create)
        .boxed()
}

pub fn nodes_object_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String / "object")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::nodes::get_object)
        .boxed()
}

pub fn nodes_object_patch(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String / "object")
        .and(warp::patch())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::ObjectPatchRequest>())
        .and_then(handlers::nodes::object_patch)
        .boxed()
}

// Entity API
//
// entities
//   entities_list: GET
pub fn entities(db: &Db) -> BoxedFilter<(impl warp::Reply,)> {
    entities_list(db.clone())
        .or(entities_get(db.clone()))
        .boxed()
}

pub fn entities_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("entities" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::entities::get)
        .boxed()
}

pub fn entities_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("entities")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("entity".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

// Systems API
//
// systems
//   systems_list: GET
pub fn systems(db: &Db) -> BoxedFilter<(impl warp::Reply,)> {
    systems_list(db.clone()).or(systems_get(db.clone())).boxed()
}

pub fn systems_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("systems" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("system".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn systems_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("systems")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("system".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
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
pub fn change_sets(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    change_set_create(db.clone(), nats.clone())
        .or(change_set_patch(db.clone(), nats.clone()))
        .or(edit_session_create(db.clone(), nats.clone()))
        .or(edit_session_patch(db.clone(), nats.clone()))
        .boxed()
}

pub fn change_set_create(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::CreateRequest>())
        .and_then(handlers::change_sets::create)
        .boxed()
}

pub fn change_set_patch(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String)
        .and(warp::patch())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::PatchRequest>())
        .and_then(handlers::change_sets::patch)
        .boxed()
}

pub fn edit_session_create(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String / "editSessions")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::edit_session::CreateRequest>())
        .and_then(handlers::edit_sessions::create)
        .boxed()
}

pub fn edit_session_patch(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String / "editSessions" / String)
        .and(warp::patch())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::edit_session::PatchRequest>())
        .and_then(handlers::edit_sessions::patch)
        .boxed()
}

// changeSetParticipants
//   list
pub fn change_set_participants(db: &Db, _nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    change_set_participants_list(db.clone()).boxed()
}

pub fn change_set_participants_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSetParticipants")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("changeSetParticipant".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

// Secrets API
pub fn secrets(db: &Db, nats: &Connection) -> BoxedFilter<(impl warp::Reply,)> {
    secrets_list(db.clone())
        .or(secrets_get(db.clone()))
        .or(secrets_create(db.clone(), nats.clone()))
        .boxed()
}

pub fn secrets_list(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("secret".into()))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::list_models)
        .boxed()
}

pub fn secrets_get(db: Db) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::header::<String>("authorization"))
        .and(with_string("secret".into()))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::get_model_change_set)
        .boxed()
}

pub fn secrets_create(db: Db, nats: Connection) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets")
        .and(warp::post())
        .and(with_db(db))
        .and(with_nats(nats))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::secret::CreateRequest>())
        .and_then(handlers::secrets::create)
        .boxed()
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
