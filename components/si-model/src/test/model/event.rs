use names::{Generator, Name};

use crate::test::model::billing_account::NewBillingAccount;

use si_data::{NatsConn, PgPool};
use crate::{Event, EventKind};

pub async fn create_event(pg: &PgPool, nats_conn: &NatsConn, nba: &NewBillingAccount) -> Event {
    let event = Event::new(
        &pg,
        &nats_conn,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");
    event
}
