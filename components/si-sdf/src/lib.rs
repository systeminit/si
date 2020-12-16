use crate::data::{EventLogFS, NatsConn, PgPool};
use crate::veritech::Veritech;
use si_settings::Settings;
use warp::Filter;

pub mod cli;
pub mod data;
pub mod filters;
pub mod handlers;
pub mod models;
pub mod veritech;

pub static mut PAGE_SECRET_KEY: Option<sodiumoxide::crypto::secretbox::Key> = None;

pub fn page_secret_key() -> &'static sodiumoxide::crypto::secretbox::Key {
    unsafe {
        PAGE_SECRET_KEY
            .as_ref()
            .expect("cannot unwrap page secret key - it should be set before you call this!")
    }
}

pub async fn start(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    event_log_fs: EventLogFS,
    settings: Settings,
) {
    // This is safe because we only ever reference this key *after* this function is
    // called.
    unsafe {
        PAGE_SECRET_KEY = Some(settings.paging.key.clone());
    }
    println!("*** Initializing the Update Clock Service ***");
    crate::models::update_clock::init_update_clock_service(&settings);
    let api = filters::api(
        &pg,
        &nats_conn,
        &veritech,
        &event_log_fs,
        &settings.jwt_encrypt.key,
    );
    let cors = warp::cors::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Access-Control-Allow-Origin",
            "Authorization",
            "Content-Type",
        ])
        .allow_methods(vec!["HEAD", "GET", "PUT", "POST", "DELETE", "PATCH"]);

    let routes = api.with(warp::trace::request()).with(cors);
    println!(
        "*** Listening on http://0.0.0.0:{} ***",
        settings.service.port
    );
    warp::serve(routes)
        .run(([0, 0, 0, 0], settings.service.port))
        .await;
}
