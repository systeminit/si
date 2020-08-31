use si_data::{DataError, Db};

pub use crate::protobuf::{
    ChangeSet, EditSession, EditSessionRevertReply, EditSessionRevertRequest,
    EditSessionUnrevertReply, EditSessionUnrevertRequest,
};

use futures::stream::StreamExt;
use tracing::info_span;
use tracing_futures::Instrument as _;

impl EditSession {
    pub async fn revert(
        db: &Db,
        request: EditSessionRevertRequest,
    ) -> Result<EditSessionRevertReply, DataError> {
        let span = info_span!(
            "si.account.edit_session.revert",
            si.account.edit_session.id = tracing::field::Empty,
            db.cb.query = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();

            let edit_session_id = request
                .id
                .ok_or_else(|| DataError::RequiredField("id".into()))?;
            span.record(
                "edit_session.id",
                &tracing::field::display(&edit_session_id),
            );

            let mut edit_session = EditSession::get(db, &edit_session_id).await?;
            edit_session.reverted = Some(true);
            let response_session = edit_session.clone();
            edit_session.save(db).await?;

            let change_set_id = edit_session
                .si_properties
                .and_then(|si_properties| si_properties.change_set_id)
                .ok_or_else(|| DataError::RequiredField("siProperties.changeSetId".into()))?;

            let update_query = format!(
                "UPDATE `{bucket_name}`
                       SET siStorable.reverted = true
                               WHERE siStorable.changeSetId = \"{change_set_id}\"
                                 AND siStorable.editSessionId = \"{edit_session_id}\"",
                bucket_name = db.bucket_name,
                change_set_id = change_set_id,
                edit_session_id = edit_session_id,
            );
            span.record("db.cb.query", &tracing::field::display(&update_query));

            tracing::warn!(?update_query, "wtf");
            db.cluster.query(update_query, None).await?;
            tracing::warn!("made it");

            Ok(EditSessionRevertReply {
                item: Some(response_session),
            })
        }
        .instrument(span)
        .await
    }

    pub async fn unrevert(
        db: &Db,
        request: EditSessionUnrevertRequest,
    ) -> Result<EditSessionUnrevertReply, DataError> {
        let span = info_span!(
            "si.account.edit_session.revert",
            edit_session.id = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();

            let edit_session_id = request
                .id
                .ok_or_else(|| DataError::RequiredField("id".into()))?;
            span.record(
                "edit_session.id",
                &tracing::field::display(&edit_session_id),
            );

            let mut edit_session = EditSession::get(db, &edit_session_id).await?;
            edit_session.reverted = Some(false);
            edit_session.save(db).await?;

            Ok(EditSessionUnrevertReply {
                item: Some(edit_session),
            })
        }
        .instrument(span)
        .await
    }
}
