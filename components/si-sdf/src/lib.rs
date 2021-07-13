use si_data::{EventLogFS, NatsConn, PgPool};
use si_model::Veritech;
use si_settings::Settings;
use warp::{trace::Info, Filter};

pub mod cli;
pub mod filters;
pub mod handlers;
pub mod resource_scheduler;
pub mod telemetry;
pub mod update;

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
    let tracing = warp::trace(|info: Info| {
        use tracing::field::{display, Empty};

        let span = tracing::info_span!(
            "request",
            http.method = %info.method(),
            // http.url, ex: https://www.foo.bar/search?q=Yep#SemConv
            http.target = %info.path(),
            http.host = Empty,
            // http.scheme, ex: https
            // http.status_code: ex: 200
            http.flavor = ?info.version(),
            http.user_agent = Empty,
            net.transport = "ip_tcp",
            net.peer.ip = Empty,
            net.peer.port = Empty,
        );

        if let Some(host) = info.host() {
            span.record("http.host", &display(host));
        }
        if let Some(user_agent) = info.user_agent() {
            span.record("http.user_agent", &display(user_agent));
        }
        if let Some(remote_addr) = info.remote_addr() {
            span.record("net.peer.ip", &display(remote_addr.ip()));
            span.record("net.peer.port", &display(remote_addr.port()));
        }

        span
    });
    let routes = api.with(cors).with(tracing);

    println!(
        "*** Listening on http://0.0.0.0:{} ***",
        settings.service.port
    );
    warp::serve(routes)
        .run(([0, 0, 0, 0], settings.service.port))
        .await;
}
