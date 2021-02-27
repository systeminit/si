use crate::model::billing_account::NewBillingAccount;
use crate::model::event::create_event;

use si_data::{NatsConn, PgPool};
use si_model::{EventLog, EventLogLevel};

pub async fn create_event_log(
    pg: &PgPool,
    nats_conn: &NatsConn,
    nba: &NewBillingAccount,
) -> EventLog {
    let event = create_event(&pg, &nats_conn, &nba).await;

    let event_log = EventLog::new(
        &pg,
        &nats_conn,
        "logging your events",
        serde_json::json![{}],
        EventLogLevel::Info,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");
    event_log
}
