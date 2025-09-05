use mqtlib::mqtt::{MessageEvent, MqttService};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;

pub fn start_publish_task(
    mut receiver_publish: Receiver<MessageEvent>,
    mqtt_service_publish: Arc<Mutex<dyn MqttService>>,
) {
    tokio::spawn(async move {
        loop {
            match receiver_publish.recv().await {
                Ok(MessageEvent::Publish(event)) => {
                    mqtt_service_publish.lock().await.publish(event).await;
                }
                Ok(_) => {
                    // ignore other events
                }
                Err(_e) => {
                    break;
                }
            }
        }
    });
}
