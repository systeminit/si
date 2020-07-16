use crate::{
    error::{Error, Result},
    message::{content_type, WireMessage},
    metadata::Topic,
    qos::QoS,
};
use futures::{Stream, StreamExt};
use paho_mqtt::{
    AsyncClient, ConnectOptions, CreateOptionsBuilder, Message as MqttMessage, MessageBuilder,
    PersistenceType, Properties, Property, PropertyCode,
};
use std::{convert::TryInto, fmt, marker::Unpin, time::Duration};
use tokio::sync::mpsc::{self, Sender};
use tracing::{info, warn};

#[derive(Clone)]
pub struct Transport {
    mqtt: AsyncClient,
}

impl Transport {
    pub async fn create(
        server_uri: impl Into<String>,
        client_name: impl AsRef<str>,
    ) -> Result<Self> {
        let mqtt = CreateOptionsBuilder::new()
            .server_uri(server_uri)
            .client_id(format!(
                "{}:{}",
                client_name.as_ref(),
                crate::uuid::uuid_string()
            ))
            .persistence(PersistenceType::None)
            .mqtt_version(paho_mqtt::MQTT_VERSION_5)
            .max_buffered_messages(100)
            .create_client()?;

        mqtt.connect(ConnectOptions::new()).await?;

        Ok(Self { mqtt })
    }

    pub async fn send<T>(&self, message: T) -> Result<()>
    where
        T: TryInto<WireMessage>,
        T::Error: Into<Error>,
    {
        let message = message.try_into().map_err(Into::into)?;

        let mut props = Properties::new();
        props.push(Property::new(PropertyCode::PayloadFormatIndicator, 1u8)?)?;
        props.push(Property::new(
            PropertyCode::ContentType,
            content_type(message.payload_type),
        )?)?;
        let mqtt_msg = MessageBuilder::new()
            .topic(message.topic)
            .qos(message.qos.into())
            .payload(message.payload)
            .properties(props)
            .finalize();

        self.mqtt.publish(mqtt_msg).await?;

        Ok(())
    }

    pub async fn subscribe_to(
        &mut self,
        subscriptions: Vec<(Topic, QoS)>,
    ) -> Result<SubscribedTransport<impl Stream<Item = Option<MqttMessage>>>> {
        let rx = self.mqtt.get_stream(1000);

        let (topics, qoses): (Vec<_>, Vec<_>) = subscriptions
            .into_iter()
            .map(|(topic, qos)| (String::from(topic), i32::from(qos)))
            .unzip();

        let mqtt = self.mqtt.clone();
        mqtt.subscribe_many(&topics, &qoses).await?;

        Ok(SubscribedTransport { mqtt, rx })
    }
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transport")
            .field("mqtt", &"mqtt::AsyncClient")
            .finish()
    }
}

pub struct SubscribedTransport<R: Stream<Item = Option<MqttMessage>>> {
    mqtt: AsyncClient,
    rx: R,
}

impl<R: Stream<Item = Option<MqttMessage>> + Send + Unpin + 'static> SubscribedTransport<R> {
    pub fn messages(self) -> impl Stream<Item = WireMessage> {
        let (tx, rx) = mpsc::channel(1000);
        tokio::spawn(stream_with_reconnect(self.mqtt, self.rx, tx));

        rx
    }
}

impl<R: Stream<Item = Option<MqttMessage>> + Send + Unpin + 'static + fmt::Debug> fmt::Debug
    for SubscribedTransport<R>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubscribedTransport")
            .field("mqtt", &"mqtt::AsyncClient")
            .field("rx", &self.rx)
            .finish()
    }
}

async fn stream_with_reconnect<R: Stream<Item = Option<MqttMessage>> + Send + Unpin + 'static>(
    mqtt: AsyncClient,
    mut rx: R,
    mut tx: Sender<WireMessage>,
) -> Result<()> {
    while let Some(maybe_msg) = rx.next().await {
        match maybe_msg {
            Some(msg) => {
                let wire_msg = match msg.try_into() {
                    Ok(m) => m,
                    Err(err) => {
                        warn!(?err, "failed to convert mqtt message to wire message");
                        continue;
                    }
                };
                tx.send(wire_msg).await.map_err(|_| Error::ChannelClosed)?
            }
            None => {
                info!("lost connection to mqtt, attempting to reconnect");
                while let Err(err) = mqtt.reconnect().await {
                    warn!(?err, "error reconnecting to MQTT");
                    tokio::time::delay_for(Duration::from_millis(1000)).await;
                }
                continue;
            }
        }
    }
    Ok(())
}
