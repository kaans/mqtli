use std::io::ErrorKind;
use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info};
use rumqttc::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions, StateError};
use rumqttc::{ConnectReturnCode, LastWill};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

use crate::config::mqtli_config::MqttBrokerConnect;
use crate::mqtt::{
    get_transport_parameters, MqttPublishEvent, MqttReceiveEvent, MqttService, MqttServiceError,
    QoS,
};

pub struct MqttServiceV311 {
    client: Option<AsyncClient>,
    config: Arc<MqttBrokerConnect>,
    topics: Arc<Mutex<Vec<(String, QoS)>>>,
}

impl MqttServiceV311 {
    pub fn new(config: Arc<MqttBrokerConnect>) -> MqttServiceV311 {
        MqttServiceV311 {
            client: None,
            config,
            topics: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn start_connection_task(
        mut event_loop: EventLoop,
        client: AsyncClient,
        topics: Arc<Mutex<Vec<(String, QoS)>>>,
        channel: Option<broadcast::Sender<MqttReceiveEvent>>,
    ) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        debug!("Received {:?}", &event);

                        match &event {
                            Event::Incoming(event) => {
                                if let Incoming::ConnAck(_) = event {
                                    info!("Connected to broker");

                                    for (topic, qos) in topics.lock().await.iter() {
                                        info!("Subscribing to topic {} with QoS {:?}", topic, qos);
                                        if let Err(e) = client.subscribe(topic, qos.into()).await {
                                            error!("Could not subscribe to topic {}: {}", topic, e);
                                        }
                                    }
                                }
                            }
                            Event::Outgoing(_event) => {}
                        }

                        if let Some(channel) = &channel {
                            let _ = channel.send(MqttReceiveEvent::V311(event));
                        }
                    }
                    Err(e) => match e {
                        ConnectionError::ConnectionRefused(ConnectReturnCode::NotAuthorized) => {
                            error!("Not authorized, check if the credentials are valid");
                            return;
                        }
                        ConnectionError::MqttState(StateError::Io(value)) => match value.kind() {
                            ErrorKind::ConnectionAborted => {
                                info!("Connection was terminated by the broker");
                                return;
                            }
                            e => {
                                error!("Connection error: {}", e);
                                return;
                            }
                        },
                        _ => {
                            error!("Error while processing mqtt loop: {}", e);
                            return;
                        }
                    },
                }
            }
        })
    }
}

#[async_trait]
impl MqttService for MqttServiceV311 {
    async fn connect(
        &mut self,
        channel: Option<broadcast::Sender<MqttReceiveEvent>>,
    ) -> Result<JoinHandle<()>, MqttServiceError> {
        let (transport, hostname) = get_transport_parameters(self.config.clone())?;

        info!(
            "Connecting to {} on port {} with client id {}",
            hostname,
            self.config.port(),
            self.config.client_id()
        );
        let mut options = MqttOptions::new(self.config.client_id(), hostname, *self.config.port());

        options.set_transport(transport);

        debug!(
            "Setting keep alive to {} seconds",
            self.config.keep_alive().as_secs()
        );
        options.set_keep_alive(*self.config.keep_alive());

        if self.config.username().is_some() && self.config.password().is_some() {
            info!("Using username/password for authentication");
            options.set_credentials(
                self.config.username().clone().unwrap(),
                self.config.password().clone().unwrap(),
            );
        } else {
            info!("Using anonymous access");
        }

        if let Some(last_will) = self.config.last_will() {
            info!(
                "Setting last will for topic {} [Payload length: {}, QoS {:?}; retain: {}]",
                last_will.topic(),
                last_will.payload().len(),
                last_will.qos(),
                last_will.retain(),
            );
            let last_will = LastWill::new(
                last_will.topic(),
                last_will.payload().clone(),
                last_will.qos().into(),
                *last_will.retain(),
            );
            options.set_last_will(last_will);
        }

        let (client, event_loop) = AsyncClient::new(options, 10);

        let topics = self.topics.clone();

        let task_handle: JoinHandle<()> =
            Self::start_connection_task(event_loop, client.clone(), topics, channel).await;

        self.client = Option::from(client);

        Ok(task_handle)
    }

    async fn disconnect(&self) -> Result<(), MqttServiceError> {
        if let Some(client) = self.client.as_ref() {
            return Ok(client.disconnect().await?);
        }

        Ok(())
    }

    async fn publish(&self, payload: MqttPublishEvent) {
        if let Some(client) = self.client.as_ref() {
            if let Err(e) = client
                .publish(
                    &payload.topic,
                    payload.qos.into(),
                    payload.retain,
                    payload.payload,
                )
                .await
            {
                error!("Error during publish: {}", e);
            } else {
                info!("Message published on topic {}", payload.topic);
            }
        }
    }

    async fn subscribe(&mut self, topic: String, qos: QoS) {
        self.topics.lock().await.push((topic, qos));
    }
}
