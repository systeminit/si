pub mod data;
pub mod filters;
pub mod handlers;
pub mod models;

use nats::asynk::Connection;
use crate::data::Db;
use si_settings::Settings;
use warp::Filter;

pub async fn start(db: Db, nats: Connection, settings: Settings) {
    let api = filters::api(&db, &nats, &settings.jwt_encrypt.key);
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

    let routes = api
        .with(warp::trace::request())
        .recover(handlers::handle_rejection)
        .with(cors);
    println!(
        "*** Listening on http://0.0.0.0:{} ***",
        settings.service.port
    );
    warp::serve(routes)
        .run(([0, 0, 0, 0], settings.service.port))
        .await;
}
