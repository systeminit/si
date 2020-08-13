use crate::agent_execute_sender::{AgentExecuteSender, AgentExecuteSenderError};
use crate::error::{AccountError, Result};
pub use crate::protobuf::{
    ChangeSet, ChangeSetExecuteReply, ChangeSetExecuteRequest, ChangeSetStatus, Item,
};
use futures::stream::StreamExt;
use si_data::{DataError, DataStorableChangeSetEventType, Db};
use tracing::{debug, info_span};
use tracing_futures::Instrument as _;

// TODO: We should switch from doing associations through the item lookup, and instead
// we should have a function that returns them as the result of a method here. So
// `changeSet` gets an `Entries` method, which will return all the entries in order.
//
// Then the execute method can use that to retrieve the list, and work it.
//
//CREATE INDEX adv_siStorable_changeSetId ON `si`(`siStorable`.`changeSetId`)
impl ChangeSet {
    pub async fn entries(&self, db: &Db) -> Result<Vec<String>> {
        let span = info_span!(
            "si.account.change_set.entries",
            change_set.id = tracing::field::Empty,
            change_set.name =
                tracing::field::display(self.name.as_ref().unwrap_or(&"".to_string())),
            change_set.entry_count = tracing::field::debug(&self.entry_count),
            change_set.count = tracing::field::Empty,
            db.cb.query = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();
            let change_set_id = self
                .id
                .as_ref()
                .ok_or(AccountError::MissingField("id".into()))?;
            span.record("change_set.id", &tracing::field::display(&change_set_id));

            let entries_query = format!(
                "SELECT `{bucket_name}`.* 
                           FROM `{bucket_name}`
                           WHERE siStorable.changeSetId = \"{change_set_id}\"
                           ORDER BY siStorable.changeSetEntryCount",
                bucket_name = db.bucket_name,
                change_set_id = change_set_id,
            );
            span.record("db.cb.query", &tracing::field::display(&entries_query));

            let mut result = db.cluster.query(entries_query, None).await?;

            let mut entries = vec![];
            let mut result_stream = result.rows_as::<serde_json::Value>()?;
            while let Some(row) = result_stream.next().await {
                match row {
                    Ok(row_json) => {
                        if row_json["id"].is_string() {
                            // as_str() is guaranteed to be present, per the docs
                            entries.push(row_json["id"].as_str().unwrap().into());
                        }
                    }
                    Err(e) => return Err(AccountError::CouchbaseError(e)),
                }
            }

            span.record("change_set.count", &tracing::field::display(entries.len()));

            Ok(entries)
        }
        .instrument(span)
        .await
    }

    pub async fn execute_task(db: Db, mut change_set: ChangeSet) -> Result<()> {
        let span = tracing::Span::current();
        let change_set_id = change_set
            .id
            .as_ref()
            .ok_or_else(|| DataError::RequiredField("id".into()))?;
        let user_id = change_set.created_by_user_id.clone().unwrap();

        let entries = change_set.entries(&db).await?;
        for entry_id in entries.into_iter() {
            let entry_span = info_span!(
                parent: &span,
                "si.account.change_set.execute.entry",
                change_set.id = &tracing::field::display(&change_set_id),
                change_set.name = &tracing::field::debug(&change_set.name),
                change_set.event_type = tracing::field::Empty,
                change_set.item_id = tracing::field::Empty,
            );

            let mut entry_json: serde_json::Value = db.get(entry_id).await?;
            // Safe because we always return 0, no matter what
            let event_type = DataStorableChangeSetEventType::from_i32(
                entry_json["siStorable"]["changeSetEventType"]
                    .as_i64()
                    .unwrap_or(0) as i32,
            )
            .unwrap();
            crate::EventLog::change_set_entry_execute_start(&db, &user_id, &entry_json)
                .await
                .expect("you die like a dead person");
            let mut real_json = entry_json.clone();
            match real_json.as_object_mut() {
                Some(entry) => {
                    let item_id = String::from(
                        entry["siStorable"]["itemId"]
                            .as_str()
                            .ok_or(AccountError::MissingField("siStorable.itemId".into()))?,
                    );
                    entry_span.record("change_set.item_id", &tracing::field::display(&item_id));
                    entry.insert("id".into(), serde_json::Value::String(item_id.clone()));
                    entry
                        .get_mut("siStorable")
                        .and_then(|si_storable| si_storable.as_object_mut())
                        .and_then(|si_storable| {
                            if si_storable["changeSetEventType"].is_string() {
                                // unwrap is safe, because we checked above.
                                entry_span.record(
                                    "change_set.event_type",
                                    &tracing::field::display(
                                        &si_storable["changeSetEventType"].as_str().unwrap(),
                                    ),
                                );
                            }
                            si_storable.remove("changeSetId");
                            si_storable.remove("itemId");
                            si_storable.remove("changeSetEntryCount");
                            si_storable
                                .insert("changeSetExecuted".into(), serde_json::Value::Bool(true))
                        });
                    db.upsert_raw(item_id, &entry).await?;
                }
                None => return Err(AccountError::InvalidJsonObject),
            }
            entry_json["siStorable"]
                .as_object_mut()
                .and_then(|si_storable| {
                    si_storable.insert(
                        "changeSetExecuted".into(),
                        serde_json::value::Value::Bool(true),
                    )
                });
            let change_set_object_id = entry_json["id"]
                .as_str()
                .ok_or(AccountError::MissingField("id".into()))?;
            db.upsert_raw(change_set_object_id, &entry_json).await?;
            if event_type == DataStorableChangeSetEventType::Action {
                // TODO: This should not happen here, but fuck it - it's a hack for now
                // anyway!
                let mut client =
                    AgentExecuteSender::create("change_set", "tcp://localhost:1883").await?;
                client.send(&entry_json).await?;

                let id = entry_json["id"]
                    .as_str()
                    .ok_or(AgentExecuteSenderError::MissingField("id".into()))?
                    .to_string();

                let to_check_count: isize = 18000;
                let mut check_count: isize = 0;
                loop {
                    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;

                    let raw_event: serde_json::Value = db.get(&id).await?;
                    let finalized = raw_event["finalized"].as_bool().unwrap_or(false);
                    if finalized {
                        break;
                    }
                    check_count = check_count + 1;
                    if check_count == to_check_count {
                        change_set.set_status(ChangeSetStatus::Failed);
                        change_set.save(&db).await?;
                        crate::EventLog::change_set_entry_execute_end(&db, &user_id, &entry_json)
                            .await?;
                        return Err(AccountError::ChangeSetEntityEventTimeout);
                    }
                }
                crate::EventLog::change_set_entry_execute_end(&db, &user_id, &entry_json).await?;
            } else {
                crate::EventLog::change_set_entry_execute_end(&db, &user_id, &entry_json).await?;
            }
        }

        change_set.set_status(ChangeSetStatus::Closed);
        change_set.save(&db).await?;
        let user_id = change_set
            .created_by_user_id
            .clone()
            .unwrap_or("bug".into());
        crate::EventLog::change_set_closed(&db, &user_id, &change_set).await?;
        tracing::error!("donezo chief");
        Ok(())
    }

    pub async fn execute(
        db: &Db,
        request: ChangeSetExecuteRequest,
    ) -> Result<ChangeSetExecuteReply> {
        let span = info_span!(
            "si.account.change_set.execute",
            change_set.id = tracing::field::Empty,
            change_set.name = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();

            let change_set_id = request
                .id
                .ok_or_else(|| DataError::RequiredField("id".into()))?;

            span.record("change_set.id", &tracing::field::display(&change_set_id));
            let mut change_set = ChangeSet::get(db, &change_set_id).await?;
            span.record("change_set.name", &tracing::field::debug(&change_set.name));

            change_set.set_status(ChangeSetStatus::Executing);
            change_set.save(db).await?;

            let user_id = change_set.created_by_user_id.clone().unwrap();
            crate::EventLog::change_set_execute(db, &user_id, &change_set).await?;

            let change_set_copy = change_set.clone();
            let new_db = db.clone();
            tokio::task::spawn(async {
                match ChangeSet::execute_task(new_db, change_set_copy).await {
                    Ok(()) => {}
                    Err(err) => debug!(?err, "Failed task"),
                }
            });

            Ok(ChangeSetExecuteReply {
                item: Some(change_set),
            })
        }
        .instrument(span)
        .await
    }
}
