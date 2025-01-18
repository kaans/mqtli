use mqtlib::config::subscription::Subscription;
use mqtlib::mqtt::{MqttReceiveEvent, MqttService};
use rumqttc::v5::Incoming;
use rumqttc::Incoming as IncomingV311;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tracing::{error, info};

pub fn start_subscription_task(
    mqtt_service: Arc<Mutex<dyn MqttService>>,
    sender: Sender<MqttReceiveEvent>,
    topics: Vec<(Subscription, String)>,
) {
    let mut receiver_connect = sender.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = receiver_connect.recv().await {
            match event {
                MqttReceiveEvent::V5(rumqttc::v5::Event::Incoming(Incoming::ConnAck(_)))
                | MqttReceiveEvent::V311(rumqttc::Event::Incoming(IncomingV311::ConnAck(_))) => {
                    for (subscription, topic) in topics.iter() {
                        info!(
                            "Subscribing to topic {} with QoS {:?}",
                            topic,
                            subscription.qos()
                        );
                        if let Err(e) = mqtt_service
                            .lock()
                            .await
                            .subscribe(topic.clone(), *subscription.qos())
                            .await
                        {
                            error!("Could not subscribe to topic {}: {}", topic, e);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}
