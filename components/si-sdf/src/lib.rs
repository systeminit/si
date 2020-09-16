pub mod data;
pub mod filters;
pub mod handlers;
pub mod models;

use si_data::Db;
use si_settings::Settings;
use warp::Filter;

pub async fn start(db: Db, settings: Settings) {
    let api = filters::api(&db);
    let routes = api
        .with(warp::trace::request())
        .recover(handlers::handle_rejection);
    println!(
        "*** Listening on http://0.0.0.0:{} ***",
        settings.service.port
    );
    warp::serve(routes)
        .run(([0, 0, 0, 0], settings.service.port))
        .await;
}
