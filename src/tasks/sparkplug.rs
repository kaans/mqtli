use mqtlib::mqtt::MessageEvent;
use mqtlib::payload::PayloadFormat;
use mqtlib::sparkplug::SparkplugNetwork;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;

pub fn start_sparkplug_monitor(
    sparkplug_network: Arc<Mutex<SparkplugNetwork>>,
    mut receiver: Receiver<MessageEvent>,
) {
    tracing::debug!("Starting sparkplug network monitor");

    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(MessageEvent::ReceivedUnfiltered(message)) => {
                    if let PayloadFormat::Sparkplug(payload) = message.payload {
                        tracing::debug!("Received sparkplug message on topic {}", message.topic);
                        tracing::trace!("{}", payload);
                        if let Err(e) = sparkplug_network
                            .lock()
                            .await
                            .try_parse_message(message.topic, payload)
                        {
                            tracing::error!("Error while parsing sparkplug message: {e:?}");
                        }
                    }
                }
                Err(RecvError::Lagged(skipped_messages)) => {
                    tracing::warn!("Receiver skipped {skipped_messages} messages");
                }
                Err(RecvError::Closed) => break,
                _ => {}
            }
        }

        tracing::debug!("Sparkplug network monitor exited");
    });
}
