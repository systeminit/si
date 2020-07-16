use crate::{entity_event::EntityEvent, error::CeaResult};
use si_transport::{Header, Message, QoS, Transport};
use tracing::{debug, warn};

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
                QoS::AtMostOnce,
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
                    QoS::ExactlyOnce,
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
