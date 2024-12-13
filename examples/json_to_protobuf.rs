extern crate mqtlib;

use base64::Engine;
use mqtlib::payload::json::PayloadFormatJson;
use mqtlib::payload::protobuf::PayloadFormatProtobuf;
use std::fs::read_to_string;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    const JSON_INPUT_FILE: &str = "test/data/message.json";
    const PROTO_DEFINITION_FILE: &str = "test/data/message.proto";
    const PROTO_MESSAGE_NAME: &str = "Response";

    let json_string: String = read_to_string(JSON_INPUT_FILE)?;
    let json: PayloadFormatJson = PayloadFormatJson::try_from(json_string)?;

    //println!("JSON:\n{}", json);

    let protobuf = PayloadFormatProtobuf::convert_from(
        json.into(),
        &PathBuf::from(PROTO_DEFINITION_FILE),
        PROTO_MESSAGE_NAME,
    )?;

    //println!("PROTOBUF:\n{:#?}", protobuf);

    let result: Vec<u8> = protobuf.try_into()?;

    println!(
        "BASE64 encoded protobuf result: {}",
        base64::engine::general_purpose::STANDARD.encode(result)
    );

    Ok(())
}
