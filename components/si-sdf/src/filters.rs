use sodiumoxide::crypto::secretbox;
use warp::{filters::BoxedFilter, Filter};

use crate::data::{EventLogFS, NatsConn, PgPool};
use crate::veritech::Veritech;

use crate::handlers;
use crate::models::{self, OutputLineStream};

pub fn api(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
    event_log_fs: &EventLogFS,
    secret_key: &secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    billing_accounts(pg, nats_conn, veritech)
        .or(signup_dal(pg, nats_conn, veritech))
        .or(session_dal(pg, secret_key))
        .or(users(pg, secret_key))
        .or(organizations(pg))
        .or(nodes(pg, nats_conn, veritech))
        .or(change_sets(pg, nats_conn, veritech))
        .or(workspaces(pg))
        .or(updates(pg, nats_conn))
        .or(entities(pg))
        .or(events(pg))
        .or(event_logs(pg, event_log_fs))
        .or(systems(pg))
        .or(edges(pg, nats_conn))
        .or(change_set_participants(pg))
        .or(secrets(pg, nats_conn))
        .or(api_clients(pg, nats_conn, secret_key))
        .or(cli(pg, nats_conn, veritech))
        .recover(handlers::handle_rejection)
        .boxed()
}

// The Web Socket CLI API
pub fn cli(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("cli")
        .and(warp::ws())
        .and(with_pg(pg.clone()))
        .and(with_nats_conn(nats_conn.clone()))
        .and(with_veritech(veritech.clone()))
        .and(warp::query::<models::update::WebsocketToken>())
        .and_then(handlers::cli::cli)
        .boxed()
}

// The Web Socket Update API
pub fn updates(pg: &PgPool, nats_conn: &NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("updates")
        .and(warp::ws())
        .and(with_pg(pg.clone()))
        .and(with_nats_conn(nats_conn.clone()))
        .and(warp::query::<models::update::WebsocketToken>())
        .and_then(handlers::updates::update)
        .boxed()
}

pub fn events(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    events_list(pg.clone()).or(events_get(pg.clone())).boxed()
}

pub fn events_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("events" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::events::get)
        .boxed()
}

pub fn events_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("events")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::events::list)
        .boxed()
}

pub fn event_logs(pg: &PgPool, event_log_fs: &EventLogFS) -> BoxedFilter<(impl warp::Reply,)> {
    event_logs_get(pg.clone())
        .or(event_logs_list(pg.clone()))
        .or(event_logs_get_output(pg.clone(), event_log_fs.clone()))
        .boxed()
}

pub fn event_logs_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("eventLogs" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::event_logs::get)
        .boxed()
}

pub fn event_logs_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("eventLogs")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::event_logs::list)
        .boxed()
}

pub fn event_logs_get_output(
    pg: PgPool,
    event_log_fs: EventLogFS,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("eventLogs" / String / "output" / OutputLineStream)
        .and(warp::get())
        .and(with_pg(pg))
        .and(with_event_log_fs(event_log_fs))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::event_logs::get_output)
        .boxed()
}

// Workspaces API
pub fn workspaces(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    workspaces_list(pg.clone())
        .or(workspaces_get(pg.clone()))
        .boxed()
}

pub fn workspaces_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("workspaces" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::workspaces::get)
        .boxed()
}

pub fn workspaces_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("workspaces")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::workspaces::list)
        .boxed()
}

// Edges API
pub fn edges(pg: &PgPool, nats_conn: &NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    edges_list(pg.clone())
        .or(edges_all_predecessors(pg.clone()))
        .or(edges_all_successors(pg.clone()))
        .or(edges_get(pg.clone()))
        .or(edges_delete(pg.clone(), nats_conn.clone()))
        .boxed()
}

pub fn edges_all_predecessors(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / "allPredecessors")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::edge::AllPredecessorsRequest>())
        .and_then(handlers::edges::all_predecessors)
        .boxed()
}

pub fn edges_all_successors(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / "allSuccessors")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::edge::AllSuccessorsRequest>())
        .and_then(handlers::edges::all_successors)
        .boxed()
}

pub fn edges_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::edges::get)
        .boxed()
}

pub fn edges_delete(pg: PgPool, nats_conn: NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges" / String)
        .and(warp::delete())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::edges::delete)
        .boxed()
}

pub fn edges_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("edges")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::edges::list)
        .boxed()
}

// User API
pub fn users(pg: &PgPool, secret_key: &secretbox::Key) -> BoxedFilter<(impl warp::Reply,)> {
    users_login(pg.clone(), secret_key.clone()).boxed()
}

pub fn users_login(pg: PgPool, secret_key: secretbox::Key) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("users" / "login")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_secret_key(secret_key))
        .and(warp::body::json::<models::user::LoginRequest>())
        .and_then(handlers::users::login)
        .boxed()
}

// API Clients API
pub fn api_clients(
    pg: &PgPool,
    nats_conn: &NatsConn,
    secret_key: &secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    api_client_create(pg.clone(), nats_conn.clone(), secret_key.clone())
        .or(api_client_get(pg.clone()))
        .or(api_client_list(pg.clone()))
        .boxed()
}

pub fn api_client_create(
    pg: PgPool,
    nats_conn: NatsConn,
    secret_key: secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("apiClients")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_secret_key(secret_key))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::api_client::CreateRequest>())
        .and_then(handlers::api_clients::create)
        .boxed()
}

pub fn api_client_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("apiClients" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::api_clients::get)
        .boxed()
}

pub fn api_client_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("apiClients")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::api_clients::list)
        .boxed()
}

// Session DAL
pub fn session_dal(pg: &PgPool, secret_key: &secretbox::Key) -> BoxedFilter<(impl warp::Reply,)> {
    session_dal_login(pg.clone(), secret_key.clone())
        .or(session_dal_restore_authentication(pg.clone()))
        .boxed()
}

pub fn session_dal_login(
    pg: PgPool,
    secret_key: secretbox::Key,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessionDal" / "login")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_secret_key(secret_key))
        .and(warp::body::json::<handlers::session_dal::LoginRequest>())
        .and_then(handlers::session_dal::login)
        .boxed()
}

pub fn session_dal_restore_authentication(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("sessionDal" / "restoreAuthentication")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::session_dal::restore_authentication)
        .boxed()
}

// Signup DAL
pub fn signup_dal(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    signup_dal_create_billing_account(pg.clone(), nats_conn.clone(), veritech.clone()).boxed()
}

pub fn signup_dal_create_billing_account(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("signupDal" / "createBillingAccount")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::body::json::<handlers::signup_dal::CreateRequest>())
        .and_then(handlers::signup_dal::create_billing_account)
        .boxed()
}

// Billing Account API
pub fn billing_accounts(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    billing_accounts_create(pg.clone(), nats_conn.clone(), veritech.clone())
        .or(billing_accounts_get(pg.clone()))
        .or(billing_accounts_get_public_key(pg.clone()))
        .boxed()
}

pub fn billing_accounts_create(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("billingAccounts")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::body::json::<models::billing_account::CreateRequest>())
        .and_then(handlers::billing_accounts::create)
        .boxed()
}

pub fn billing_accounts_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("billingAccounts" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::billing_accounts::get)
        .boxed()
}

pub fn billing_accounts_get_public_key(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("billingAccounts" / String / "publicKey")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(with_string("billingAccount".into()))
        .and_then(handlers::billing_accounts::get_public_key)
        .boxed()
}

// Organization
pub fn organizations(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    organizations_list(pg.clone())
        .or(organizations_get(pg.clone()))
        .boxed()
}

pub fn organizations_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("organizations" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::organizations::get)
        .boxed()
}

pub fn organizations_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("organizations")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::organizations::list)
        .boxed()
}

// Nodes API
pub fn nodes(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    nodes_create(pg.clone(), nats_conn.clone(), veritech.clone())
        .or(nodes_get(pg.clone()))
        .or(nodes_patch(pg.clone(), nats_conn.clone(), veritech.clone()))
        .or(nodes_object_get(pg.clone()))
        .or(nodes_object_patch(
            pg.clone(),
            nats_conn.clone(),
            veritech.clone(),
        ))
        .boxed()
}

pub fn nodes_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::nodes::get)
        .boxed()
}

pub fn nodes_patch(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String)
        .and(warp::patch())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::PatchRequest>())
        .and_then(handlers::nodes::patch)
        .boxed()
}

pub fn nodes_create(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::CreateRequest>())
        .and_then(handlers::nodes::create)
        .boxed()
}

pub fn nodes_object_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String / "object")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::nodes::get_object)
        .boxed()
}

pub fn nodes_object_patch(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("nodes" / String / "object")
        .and(warp::patch())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::node::ObjectPatchRequest>())
        .and_then(handlers::nodes::object_patch)
        .boxed()
}

// Entity API
pub fn entities(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    entities_list(pg.clone())
        .or(entities_get(pg.clone()))
        .boxed()
}

pub fn entities_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("entities" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::entities::get)
        .boxed()
}

pub fn entities_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("entities")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::entities::list)
        .boxed()
}

// Systems API
pub fn systems(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    systems_list(pg.clone()).or(systems_get(pg.clone())).boxed()
}

pub fn systems_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("systems" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::GetRequest>())
        .and_then(handlers::systems::get)
        .boxed()
}

pub fn systems_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("systems")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::systems::list)
        .boxed()
}

// Change Sets API
pub fn change_sets(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    change_set_create(pg.clone(), nats_conn.clone())
        .or(change_set_patch(
            pg.clone(),
            nats_conn.clone(),
            veritech.clone(),
        ))
        .or(edit_session_create(pg.clone(), nats_conn.clone()))
        .or(edit_session_patch(
            pg.clone(),
            nats_conn.clone(),
            veritech.clone(),
        ))
        .boxed()
}

pub fn change_set_create(pg: PgPool, nats_conn: NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::CreateRequest>())
        .and_then(handlers::change_sets::create)
        .boxed()
}

pub fn change_set_patch(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String)
        .and(warp::patch())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::change_set::PatchRequest>())
        .and_then(handlers::change_sets::patch)
        .boxed()
}

pub fn edit_session_create(pg: PgPool, nats_conn: NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String / "editSessions")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::edit_session::CreateRequest>())
        .and_then(handlers::edit_sessions::create)
        .boxed()
}

pub fn edit_session_patch(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSets" / String / "editSessions" / String)
        .and(warp::patch())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(with_veritech(veritech))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::edit_session::PatchRequest>())
        .and_then(handlers::edit_sessions::patch)
        .boxed()
}

// changeSetParticipants
pub fn change_set_participants(pg: &PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    change_set_participants_list(pg.clone()).boxed()
}

pub fn change_set_participants_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("changeSetParticipants")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::change_sets::list_participants)
        .boxed()
}

// Secrets API
pub fn secrets(pg: &PgPool, nats_conn: &NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    secrets_list(pg.clone())
        .or(secrets_get(pg.clone()))
        .or(secrets_create(pg.clone(), nats_conn.clone()))
        .boxed()
}

pub fn secrets_list(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets")
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<models::ListRequest>())
        .and_then(handlers::secrets::list)
        .boxed()
}

pub fn secrets_get(pg: PgPool) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets" / String)
        .and(warp::get())
        .and(with_pg(pg))
        .and(warp::header::<String>("authorization"))
        .and_then(handlers::secrets::get)
        .boxed()
}

pub fn secrets_create(pg: PgPool, nats_conn: NatsConn) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path!("secrets")
        .and(warp::post())
        .and(with_pg(pg))
        .and(with_nats_conn(nats_conn))
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json::<models::secret::CreateRequest>())
        .and_then(handlers::secrets::create)
        .boxed()
}

fn with_pg(
    pg: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pg.clone())
}

fn with_nats_conn(
    nats_conn: NatsConn,
) -> impl Filter<Extract = (NatsConn,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || nats_conn.clone())
}

fn with_event_log_fs(
    event_log_fs: EventLogFS,
) -> impl Filter<Extract = (EventLogFS,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || event_log_fs.clone())
}

fn with_veritech(
    veritech: Veritech,
) -> impl Filter<Extract = (Veritech,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || veritech.clone())
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
