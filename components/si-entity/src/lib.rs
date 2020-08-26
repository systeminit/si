mod filters;
mod handlers;
mod models;

use si_data::Db;
use si_settings::Settings;
use warp::Filter;

pub async fn start(db: Db, settings: Settings) {
    let api = filters::entities(&db);
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
