use crate::{entity_event::EntityEvent, error::CeaResult};
use si_transport::{Header, Message, QoS, Transport};
use tracing::{debug, warn};

/// Quality of service level for the last status/stream message.
///
/// Currently, lines of output (that is, a line of standard out or standard error) are aggregated
/// in an `EntityEvent` object, the output for all prior messages are contained in all subsequent
/// messages. This means that if the delivery of one message cannot be guaranteed this isn't the
/// end of the world and at most some output might be missing on the consumer side. In this way,
/// the last message is no more special than any other message that preceded it.
///
/// Therefore, all last messages are sent with `QoS::AtMostOnce` which doesn't enforce delivery
/// guarantees.
const SEND_LAST_QOS: QoS = QoS::AtMostOnce;

/// Quality of service level for finalize messages.
///
/// A finalize message is assumed to be either [idempotent] or at least eventually consistent. That
/// is, in the event that there are multiple finalize messages for an object, a finalizing
/// operation should succeed when called more than once.
///
/// Therefore all finalize message are sent with `QoS::AtLeastOnce` to guarantee delivery of at
/// least *one* message.
///
/// [idempotent]: https://en.wikipedia.org/wiki/Idempotence
const SEND_FINALIZED_QOS: QoS = QoS::AtLeastOnce;

pub struct EntityEventDispatch;

impl EntityEventDispatch {
    pub fn prepare_entity_event(entity_event: &mut impl EntityEvent) -> CeaResult<()> {
        if let Err(err) = entity_event.input_entity() {
            warn!(?err, "missing input entity on event");
            return Err(err.into());
        }
        if let Err(err) = entity_event.init_output_entity() {
            warn!(?err, "cannot initialize output entity on event");
            return Err(err.into());
        }

        Ok(())
    }

    pub async fn finish_entity_event(
        result: CeaResult<()>,
        transport: &Transport,
        entity_event: &mut impl EntityEvent,
        stream_header: Header,
        finalized_header: Header,
    ) -> CeaResult<()> {
        match result {
            Ok(()) => {
                debug!(?entity_event, "event success");
                if let Err(err) = entity_event.succeeded() {
                    warn!(?err, "error setting event to succeeded");
                    return Err(err.into());
                }
            }
            Err(err) => {
                debug!(?entity_event, "event failed");
                warn!(?err, "error when dispatching event");
                if let Err(err) = entity_event.failed(err) {
                    warn!(?err, "error setting event to failed");
                    return Err(err.into());
                }
            }
        };

        if let Err(err) = transport
            .send(Message::new(
                stream_header,
                SEND_LAST_QOS,
                None::<Header>,
                &entity_event,
            ))
            .await
        {
            warn!(?err, "error sending last event stream message");
            return Err(err.into());
        };

        if entity_event.finalized().unwrap_or(false) {
            if let Err(err) = transport
                .send(Message::new(
                    finalized_header,
                    SEND_FINALIZED_QOS,
                    None::<Header>,
                    &entity_event,
                ))
                .await
            {
                warn!(?err, "error sending finalized message");
                return Err(err.into());
            };
        }

        Ok(())
    }
}
