use std::io::ErrorKind;
use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info};
use rumqttc::{AsyncClient, ConnectionError, EventLoop, MqttOptions, StateError};
use rumqttc::{ConnectReturnCode, LastWill};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

use crate::config::mqtli_config::MqttBrokerConnect;
use crate::mqtt::{
    get_transport_parameters, MessagePublishData, MqttReceiveEvent, MqttService, MqttServiceError,
    QoS,
};

pub struct MqttServiceV311 {
    client: Option<AsyncClient>,
    config: Arc<MqttBrokerConnect>,
}

impl MqttServiceV311 {
    pub fn new(config: Arc<MqttBrokerConnect>) -> MqttServiceV311 {
        MqttServiceV311 {
            client: None,
            config,
        }
    }

    async fn start_connection_task(
        mut event_loop: EventLoop,
        client: AsyncClient,
        channel: broadcast::Sender<MqttReceiveEvent>,
        mut receiver_exit: Receiver<()>,
    ) -> JoinHandle<()> {
        let client_exit = client.clone();

        tokio::task::spawn(async move {
            loop {
                if receiver_exit.recv().await.is_ok() {
                    if let Err(e) = client_exit.disconnect().await {
                        error!("Error while disconnecting client on exit signal: {e:?}");
                    }
                    return;
                }
            }
        });

        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        debug!("Received {:?}", &event);
                        let _ = channel.send(MqttReceiveEvent::V311(event));
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
        channel: broadcast::Sender<MqttReceiveEvent>,
        receiver_exit: Receiver<()>,
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

        let task_handle: JoinHandle<()> =
            Self::start_connection_task(event_loop, client.clone(), channel, receiver_exit).await;

        self.client = Option::from(client);

        Ok(task_handle)
    }

    async fn disconnect(&self) -> Result<(), MqttServiceError> {
        if let Some(client) = self.client.as_ref() {
            return Ok(client.disconnect().await?);
        }

        Ok(())
    }

    async fn publish(&self, payload: MessagePublishData) {
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

    async fn subscribe(&mut self, topic: String, qos: QoS) -> Result<(), MqttServiceError> {
        if let Some(client) = &self.client {
            return client
                .subscribe(topic.clone(), qos.into())
                .await
                .map_err(MqttServiceError::from);
        }

        Err(MqttServiceError::NotConnected)
    }
}
