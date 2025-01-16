use mqtlib::mqtt::{MessageEvent, MqttService};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;

pub fn start_publish_task(
    mut receiver_publish: Receiver<MessageEvent>,
    mqtt_service_publish: Arc<Mutex<dyn MqttService>>,
) {
    tokio::spawn(async move {
        while let Ok(MessageEvent::Publish(event)) = receiver_publish.recv().await {
            mqtt_service_publish.lock().await.publish(event).await;
        }
    });
}
