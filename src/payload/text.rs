use std::str::from_utf8;

use log::error;
use rumqttc::v5::mqttbytes::v5::Publish;

pub struct PayloadTextHandler {}

impl PayloadTextHandler {
    pub fn handle_publish(value: &Publish) {
        match from_utf8(value.payload.as_ref()) {
            Ok(content) => {
                println!("{}", content);
            }
            Err(e) => {
                error!("Could not convert payload to UTF 8 string: {e:?}");
            }
        }
    }
}