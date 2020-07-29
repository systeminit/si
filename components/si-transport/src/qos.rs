use crate::error::Error;
use std::convert::TryFrom;

/// Quality of Service.
///
/// The Quality of Service (QoS) level is an agreement between the sender of a message and the
/// receiver of a message that defines the guarantee of delivery for a specific message. There are
/// 3 QoS levels in MQTT:
///
/// - At most once (0)
/// - At least once (1)
/// - Exactly once (2)
///
/// When you talk about QoS in MQTT, you need to consider the two sides of message delivery:
///
/// 1. Message delivery form the publishing client to the broker.
/// 2. Message delivery from the broker to the subscribing client.
///
/// Source: [Quality of Service 0,1 & 2 - MQTT Essentials: Part
/// 6](https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/)
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QoS {
    /// QoS 0 - at most once.
    ///
    /// The minimal QoS level is zero. This service level guarantees a best-effort delivery. There
    /// is no guarantee of delivery. The recipient does not acknowledge receipt of the message and
    /// the message is not stored and re-transmitted by the sender. QoS level 0 is often called
    /// "fire and forget" and provides the same guarantee as the underlying TCP protocol.
    ///
    /// Source: [Quality of Service 0,1 & 2 - MQTT Essentials: Part
    /// 6](https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/)
    AtMostOnce,
    /// QoS 1 - at least once.
    ///
    /// QoS level 1 guarantees that a message is delivered at least one time to the receiver. The
    /// sender stores the message until it gets a  PUBACK packet from the receiver that
    /// acknowledges receipt of the message. It is possible for a message to be sent or delivered
    /// multiple times.
    ///
    /// Source: [Quality of Service 0,1 & 2 - MQTT Essentials: Part
    /// 6](https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/)
    AtLeastOnce,
    /// QoS 2 - exactly once.
    ///
    /// QoS 2 is the highest level of service in MQTT. This level guarantees that each message is
    /// received only once by the intended recipients. QoS 2 is the safest and slowest quality of
    /// service level. The guarantee is provided by at least two request/response flows (a
    /// four-part handshake) between the sender and the receiver. The sender and receiver use the
    /// packet identifier of the original PUBLISH message to coordinate delivery of the message.
    ///
    /// Source: [Quality of Service 0,1 & 2 - MQTT Essentials: Part
    /// 6](https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/)
    ExactlyOnce,
}

impl Default for QoS {
    fn default() -> Self {
        Self::AtMostOnce
    }
}

impl From<QoS> for i32 {
    fn from(value: QoS) -> Self {
        match value {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2,
        }
    }
}

impl TryFrom<i32> for QoS {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            invalid => Err(Error::InvalidQoS(invalid)),
        }
    }
}
