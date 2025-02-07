use mqtlib::config::subscription::{Output, OutputTarget};
use mqtlib::config::topic::TopicStorage;
use mqtlib::config::PayloadType;
use mqtlib::mqtt::{MessageEvent, MessagePublishData, MessageReceivedData};
use mqtlib::output::console::ConsoleOutput;
use mqtlib::output::file::FileOutput;
use mqtlib::output::OutputError;
use mqtlib::payload::PayloadFormat;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{debug, error};
use mqtlib::storage::SqlStorageImpl;

pub fn start_output_task(
    mut receiver: Receiver<MessageEvent>,
    topic_storage: Arc<TopicStorage>,
    sender_message: Sender<MessageEvent>,
    exclude_types: Vec<PayloadType>,
    db: Arc<Option<Box<dyn SqlStorageImpl>>>,
) {
    tokio::spawn(async move {

        loop {
            if let Ok(MessageEvent::ReceivedFiltered(message)) = receiver.recv().await {
                if !exclude_types.contains(&message.payload.clone().to_owned().into()) {
                    let outputs = topic_storage.get_outputs_for_topic(&message.topic);
                    for output in outputs {
                        if let Err(e) = write_to_output(sender_message.clone(), &message, output, db.clone()).await {
                            error!("Error while writing to output {}: {e:?}", output.target);
                        }
                    }
                }
            }
        }
    });
}

async fn write_to_output(
    sender_message: Sender<MessageEvent>,
    message: &MessageReceivedData,
    output: &Output,
    db: Arc<Option<Box<dyn SqlStorageImpl>>>,
) -> Result<(), OutputError> {
    let conv = PayloadFormat::try_from((message.payload.clone(), output.format()))?;
    match output.target() {
        OutputTarget::Console(_options) => ConsoleOutput::output_topic(
            &message.topic,
            conv.clone().try_into()?,
            conv,
            message.qos,
            message.retain,
        ),
        OutputTarget::File(file) => FileOutput::output(conv.try_into()?, file),
        OutputTarget::Topic(options) => {
            sender_message
                .send(MessageEvent::Publish(MessagePublishData::new(
                    options.topic().clone(),
                    *options.qos(),
                    *options.retain(),
                    conv.try_into()?,
                )))
                .map_err(OutputError::SendError)?;
            Ok(())
        }
        OutputTarget::Sql(sql) => {
            if let Some(db) = db.as_ref() {
                debug!("Writing to SQL storage");

                db.insert(sql.insert_statement.as_str(),
                          &message.topic,
                          message.qos,
                          message.retain,
                          &message.payload.clone()).await
                    .map(|_| ())
                    .map_err(OutputError::from)
            } else {
                Err(OutputError::SqlDatabaseNotInitialized)
            }
        },
    }
}
