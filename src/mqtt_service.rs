use std::sync::Arc;

use log::{debug, error, info};
use rumqttc::v5::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::v5::ConnectReturnCode;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::mqtl_config::{MqttBrokerConnectArgs, Topic};

#[derive(Error, Debug)]
pub enum MqttServiceError {
    #[error("Not authorized, provide valid credentials")]
    NotAuthorized(#[source] ConnectionError)
}

type OnConnectCallback = fn();

pub struct MqttService<'a> {
    client: Option<AsyncClient>,
    config: &'a MqttBrokerConnectArgs,

    subscribe_topics: Arc<Mutex<Vec<Topic>>>,

    on_connect_callback: Option<OnConnectCallback>,
    task_handle: Option<JoinHandle<()>>,
}

impl MqttService<'_> {
    pub fn new(mqtt_connect_args: &MqttBrokerConnectArgs) -> MqttService {
        MqttService {
            client: None,
            config: mqtt_connect_args,

            subscribe_topics: Arc::new(Mutex::new(vec![])),
            on_connect_callback: None,
            task_handle: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), MqttServiceError> {
        debug!("Connection to {}:{} with client id {}", self.config.host(),
            self.config.port(), self.config.client_id());
        let mut options = MqttOptions::new(self.config.client_id(),
                                           self.config.host(),
                                           *self.config.port());

        debug!("Setting keep alive to {} seconds", self.config.keep_alive().as_secs());
        options.set_keep_alive(*self.config.keep_alive());

        if self.config.username().is_some() && self.config.password().is_some() {
            info!("Using username/password for authentication {} {}",
                self.config.username().clone().unwrap(),
                self.config.password().clone().unwrap());
            options.set_credentials(self.config.username().clone().unwrap(),
                                    self.config.password().clone().unwrap());
        }

        let (client, mut event_loop) = AsyncClient::new(options, 10);
        self.client = Option::from(client.clone());

        let subscribe_topics = self.subscribe_topics.clone();
        let on_connect_callback = self.on_connect_callback.clone();

        let task_handle: JoinHandle<()> = MqttService::start_connection_task(event_loop,
                                                                             client,
                                                                             subscribe_topics,
                                                                             on_connect_callback)
            .await;

        self.task_handle = Some(task_handle);

        Ok(())
    }

    async fn start_connection_task(mut event_loop: EventLoop,
                                   client: AsyncClient,
                                   subscribe_topics: Arc<Mutex<Vec<Topic>>>,
                                   on_connect_callback: Option<OnConnectCallback>) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        info!("Received {:?}", event);

                        match event {
                            Event::Incoming(event) => {
                                match event {
                                    Incoming::ConnAck(_) => {
                                        info!("Connected to broker");

                                        if on_connect_callback.is_some() { on_connect_callback.unwrap()() }

                                        for topic in subscribe_topics.lock().await.iter() {
                                            client.subscribe(topic.topic(), *topic.qos()).await.expect("could not subscribe");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Event::Outgoing(_event) => {}
                        }
                    }
                    Err(e) => {
                        match e {
                            ConnectionError::ConnectionRefused(ConnectReturnCode::NotAuthorized) => {
                                error!("Not authorized, check if the credentials are valid");
                                return;
                            }
                            _ => {
                                error!("Error while processing mqtt loop: {:?}", e);
                            }
                        }
                    }
                }
            }
        })
    }

    pub async fn subscribe(&mut self, topic: Topic) {
        info!("Subscribing to topic {} with QoS {:?}", topic.topic(), topic.qos());

        self.subscribe_topics.lock().await.push(topic);
    }

    pub fn set_on_connect_callback(&mut self, fun: Option<OnConnectCallback>) {
        self.on_connect_callback = fun;
    }

    pub async fn await_task(self) {
        if self.task_handle.is_some() {
            self.task_handle.unwrap().await.expect("Could not join thread");
        }
    }
}