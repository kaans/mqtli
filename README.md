# MQTli

MQTli is a multi-topic payload-converting MQTT cli client written in Rust.

It can be configured to automatically convert between different payload formats
when reading input data for publish and outputting data for subscribe.
The supported data formats and the conversion rules are listed under [supported payload formats](#supported-formats)

## How to use

1. Download the latest release for your platform [from the releases](https://github.com/kaans/mqtli/releases/latest).
2. Extract the executable to a folder on your local hard drive
3. Optional: Add the path to the executable to your environments PATH variable, so that you can execute the
   program from any folder
4. Copy the [config.default.yaml](https://github.com/kaans/mqtli/blob/main/config.default.yaml) to
   your local directory, rename it to `config.yaml` and adjust it with your required settings
5. Execute the command `mqtli` (Optionally specify the name of the config
   file `mqtli --config-file config.default.yaml`)


## Supported features:

* Configure multiple topics with the following settings for each:
    * Input: periodically send messages (publish)
    * Payload: define the payload format of the topic
    * Output: print incoming payload to console or file (subscribe)
* Automatically convert the message payload between input, payload, and output, if they differ
* Configuration via cli arguments and config file (yaml)
* MQTT v5 and v3.1.1
* TLS support (v1.2 and v1.3)
* Websocket support (unencrypted and TLS)
* Client authentication via username/password
* Client authentication via TLS certificates
* Last will
* QoS 0, 1, 2


## Configuration

The configuration can be passed to the tool via the following ways in descending priority:

1. command line arguments
2. environment variables
3. config file

If a config entry was not explicitly specified, the default values applies.

All but the topic configuration can be given as a command line argument or environment variable. The topic configuration
can only be specified in the config file because it would be too complex to specify in another way.

> NOTE: The program does not do anything really useful if no topics are specified or all specified topics are disabled.
> It is fine to just use the command line arguments or environment variables to connect to the broker. 
> If you want to see subscribe to topics or publish messages, you need to configure them in the config file.

### CLI arguments and environment variables

The following lists all possible command line arguments and environment variables (also available via `mqtli --help`):

```
Usage: mqtli.exe [OPTIONS]

Options:
      --help                       Print help
      --version                    Print version
  -c, --config-file <CONFIG_FILE>  Path to the config file (default: config.yaml) [env: CONFIG_FILE_PATH=]

Broker:
  -h, --host <HOST>                  The ip address or hostname of the broker (default: localhost) [env: BROKER_HOST=]
  -p, --port <PORT>                  The port the broker is listening on (default: 1883) [env: BROKER_PORT=]
      --protocol <PROTOCOL>          The protocol to use to communicate with the broker (default: tcp) [env: BROKER_PROTOCOL=] [possible values: tcp, websocket]
  -i, --client-id <CLIENT_ID>        The client id for this mqtli instance (default: mqtli) [env: BROKER_CLIENT_ID=]
  -v, --mqtt-version <MQTT_VERSION>  The MQTT version to use (default: v5) [env: BROKER_MQTT_VERSION=] [possible values: v311, v5]
      --keep-alive <KEEP_ALIVE>      Keep alive time in seconds (default: 5 seconds) [env: BROKER_KEEP_ALIVE=]
  -u, --username <USERNAME>          (optional) Username used to authenticate against the broker; if used then username must be given too (default: empty) [env: BROKER_USERNAME=]
  -w, --password <PASSWORD>          (optional) Password used to authenticate against the broker; if used then password must be given too (default: empty) [env: BROKER_PASSWORD=]

TLS:
      --use-tls <USE_TLS>
          If specified, TLS is used to communicate with the broker (default: false) [env: BROKER_USE_TLS=] [possible values: true, false]
      --ca-file <TLS_CA_FILE>
          Path to a PEM encoded ca certificate to verify the broker's certificate (default: empty) [env: BROKER_TLS_CA_FILE=]
      --client-cert <TLS_CLIENT_CERTIFICATE>
          (optional) Path to a PEM encoded client certificate for authenticating against the broker; must be specified with client-key (default: empty) [env: BROKER_TLS_CLIENT_CERTIFICATE_FILE=]
      --client-key <TLS_CLIENT_KEY>
          (optional) Path to a PKCS#8 encoded, unencrypted client private key for authenticating against the broker; must be specified with client-cert (default: empty) [env: BROKER_TLS_CLIENT_KEY_FILE=]
      --tls-version <TLS_VERSION>
          TLS version to used (default: all) [env: BROKER_TLS_VERSION=] [possible values: all, v12, v13]

Last will:
      --last-will-payload <PAYLOAD>  The UTF-8 encoded payload of the will message (default: empty) [env: BROKER_LAST_WILL_PAYLOAD=]
      --last-will-topic <TOPIC>      The topic where the last will message will be published (default: empty) [env: BROKER_LAST_WILL_TOPIC=]
      --last-will-qos <QOS>          Quality of Service (default: 0) (possible values: 0 = at most once; 1 = at least once; 2 = exactly once) [env: BROKER_LAST_WILL_QOS=]
      --last-will-retain <RETAIN>    If true, last will message will be retained, else not (default: false) [env: BROKER_LAST_WILL_RETAIN=] [possible values: true, false]

Logging:
  -l, --log-level <LOG_LEVEL>  Log level (default: info) (possible values: trace, debug, info, warn, error, off) [env: LOG_LEVEL=]
```

### Config file

In addition to all configuration values from command line arguments, the topics can be configured via the config file.

See [config.default.yaml](https://github.com/kaans/mqtli/blob/main/config.default.yaml) for all possible configuration
values including their defaults.

#### Topics and automatic conversion between payload formats

The general idea behind the topics configuration is that each topic on the mqtt broker is used for transporting messages
of the same type and data format, but possibly different content. Even though the MQTT specification does not at all
apply
any restrictions how topics may be used, it is common practise to only use the same data formats for the payload of a
specific topic. In case the structure or data format of the payload of a topic differs between two messages, it
is recommended to use different topics for these messages.

For each topic, the following three main aspects can be configured:

1. **The format of the payload of the messages on the topic**

    The format is defined once for all message on the topic, assuming that the format of the payload does not change between
    messages.
    Depending on the format, several options may be passed, see [supported payload formats](#supported-formats).
    
    For example, all messages on the topic may be formatted as `hex` string or `JSON` value.

2. **The display of received messages on subscribed topics**

    If enabled, a subscription for the topic is registered on connect. Each subscription may have several independent
    outputs.
    Each output has a format type and a target.
    
    * *Format type* (default: Text): This may be one of the types defined
      in [supported payload formats](#supported-formats).
      It defines which format the received message will be displayed in. If the format type
      of the topic is different, an automatic conversion is attempted. If it fails, an error is displayed. See the
      referenced chapter to see which conversions are currently possible.
      * *Target* (default: Console): The target defines where the message is being printed out. Currently, the following
        targets
        are supported:
          * *Console*: Prints the message to the stdin console.
          * *File*: Prints the message to a file. Apart from the path to the output file, string for prepending or appending
            or the
            behavior for overwriting can be specified.

3. **The format of messages published on the topics**

    When messages are published to a topic, for example via a periodic trigger, the message may be specified
    in another format than the payload of the topic. If the payload format of the published message is not the same format
    as the payload format of the topic, the payload will automatically be converted to the payload format of the topic. If a conversion is not possible, it will fail and an error will be printed. See [supported payload formats](#supported-formats) for possible conversions.
    
    For example, it might be easier to specify a binary payload as hex or base64 encoded string than as raw
    bytes. This way, the payload could be written directly into the `config.yaml` file instead of an external
    file (YAML files only accept UTF-8 content; a binary payload may contain invalid bytes).

> One of the most important advantages of this separate definition of format types is that it is then possible to
> automatically convert
> between formats. For example:
> * The payload format of the topic is protobuf
> * The published messages are written as hex string for storing it directly in the config.yaml
> * The received messages on subscribed topics are displayed as json and written to a file as raw (bytes)
>
> Even though protobuf is not human-readable by itself (as it is encoded using bytes), this setup allows to read
> messages
> on the topic as human-readable json while storing received messages as original bytes in a file (for later use or
> whatsoever).
> The message to publish does not need to be stored as bytes but can be encoded to a hex string which will automatically
> be decoded to protobuf before being published.

#### Example config: Protobuf as topic format, no TLS

```yaml
broker:
  client_id: "my_client_id"
  username: "yourusernamehere"
  password: "yourpasswordhere"

  use_tls: false

  last_will:
    topic: "mqtli/lwt"
    payload: "Good bye"

topics:
  - topic: mqtli/test
    subscription:
      enabled: true
      outputs:
        - format: yaml # target is console; protobuf message will be shown as yaml
        - format: base64
          target:
            type: file
            path: "log.txt"
            overwrite: false
            prepend: "MESSAGE: " # prepends the string "MESSAGE: " to the beginning of the base64 encoded message
            append: "\n" # appends a new line to the end of the message
    payload:
      type: protobuf
      definition: "messages.proto" # path to file containing message definition
      message: "Proto.Message" # package_name.message_name
    publish:
      enabled: true
      input:
        type: hex
        content: AB23F6E983 # this must be a valid protobuf message according to the payload format (encoded as hex) 
      trigger:
        - type: periodic # default trigger: periodic with no count (indefinitely) and interval 1 second

```


## <a name="supported-formats"></a>Supported Payload formats and conversion

### Raw (binary)

#### Options

#### Conversions

| Convert to      | Description                                                                                                                | Example                              | Possible failures |
|-----------------|----------------------------------------------------------------------------------------------------------------------------|--------------------------------------|-------------------|
| Text<br>(UTF-8) | Depending on the config ("raw_as") it is encoded as hex, base64 or UTF-8                                                   | INPUT => 494e505554                  |                   |
| Raw             | No conversion needed                                                                                                       | INPUT => INPUT                       |                   |
| Hex             | Converts input to hex                                                                                                      | INPUT => 494e505554                  |                   |
| Base64          | Converts input to base64                                                                                                   | INPUT => SU5QVVQ=                    |                   |
| JSON            | Puts input text into field content of new JSON object<br>Depending on the config ("raw_as") it is encoded as hex or base64 | INPUT => { "content": "494e505554" } |                   |
| YAML            | Puts input text into field content of new YAML object<br>Depending on the config ("raw_as") it is encoded as hex or base64 | INPUT => content: 494e505554         |                   |
| Protobuf        | Raw bytes must represent a valid protobuf message                                                                          | <raw_bytes> => <protobuf_message>    |                   |

### Text (UTF-8)

#### Options

| Name   | Description                                                                                                         | Default | Possible values   |
|--------|---------------------------------------------------------------------------------------------------------------------|---------|-------------------|
| raw_as | Type as which raw values will be rendered; if utf8, invalid characters will be displayed with replacement character | hex     | hex, base64, utf8 |

#### Conversions

| Convert to      | Description                                           | Example                         | Possible failures |
|-----------------|-------------------------------------------------------|---------------------------------|-------------------|
| Text<br>(UTF-8) | No conversion needed                                  | INPUT => INPUT                  |                   |
| Raw             | No conversion needed                                  | INPUT => INPUT                  |                   |
| Hex             | Converts input to hex                                 | INPUT => 494e505554             |                   |
| Base64          | Converts input to base64                              | INPUT => SU5QVVQ=               |                   |
| JSON            | Puts input text into field content of new JSON object | INPUT => { "content": "INPUT" } |                   |
| YAML            | Puts input text into field content of new YAML object | INPUT => content: INPUT         |                   |
| Protobuf        | Not supported                                         |                                 |                   |

### Hex

#### Options

#### Conversions

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
    <th class="tg-c8dp">Possible failures</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Decodes hex String and tries to convert bytes to hex, base64 or UTF-8 (depending on the "raw_as" setting)</td>
    <td class="tg-0pky">494e505554 =&gt; INPUT</td>
    <td class="tg-0lax">- length of hex string is not even</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">Decodes hex to raw bytes and stores them</td>
    <td class="tg-0pky">494e505554 =&gt; INPUT</td>
    <td class="tg-0lax">None</td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">494e505554 =&gt; 494e505554</td>
    <td class="tg-0lax">None</td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">Decodes hex String and converts bytes to Base64</td>
    <td class="tg-0pky">494e505554 =&gt; SU5QVVQ=</td>
    <td class="tg-0lax">- length of hex string is not even</td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">Puts hex string into field content of new JSON object</td>
    <td class="tg-0pky">INPUT =&gt; { "content": "494e505554" }</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">Puts hex string into field content of new YAML object</td>
    <td class="tg-0pky">INPUT =&gt; content: 494e505554</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">Decoded bytes must represent a valid protobuf message</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### Base64

#### Options

#### Conversions

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
    <th class="tg-c8dp">Possible failures</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Decodes base64 String and tries to convert bytes to hex, base64 or UTF-8 (depending on the "raw_as" setting)</td>
    <td class="tg-0pky">SU5QVVQ= =&gt; INPUT</td>
    <td class="tg-0lax">- wrong length, padding, or bytes in base64 string</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">Decodes base64 to raw bytes and stores them</td>
    <td class="tg-0pky">SU5QVVQ= =&gt; INPUT</td>
    <td class="tg-0lax">None</td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">Decodes base64 String and converts bytes to Hex</td>
    <td class="tg-0pky">SU5QVVQ= =&gt; 494e505554</td>
    <td class="tg-0lax">- wrong length, padding, or bytes in base64 string</td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">SU5QVVQ= =&gt; SU5QVVQ=</td>
    <td class="tg-0lax">None</td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">Puts base64 string into field content of new JSON object</td>
    <td class="tg-0pky">INPUT =&gt; { "content": "SU5QVVQ=" }</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">Puts base64 string into field content of new YAML object</td>
    <td class="tg-0pky">INPUT =&gt; content: SU5QVVQ=</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">Decoded bytes must represent a valid protobuf message</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### JSON

#### Options

| Name   | Description                                                                                                         | Default | Possible values   |
|--------|---------------------------------------------------------------------------------------------------------------------|---------|-------------------|
| raw_as | Type as which raw values will be rendered; if utf8, invalid characters will be displayed with replacement character | hex     | hex, base64, utf8 |

#### Conversions

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
    <th class="tg-c8dp">Possible failures</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Read the string from the field "content" and store it</td>
    <td class="tg-0pky">{ "content": "INPUT" } =&gt; INPUT</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a string</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">Read the string from the field "content" and store it as raw bytes</td>
    <td class="tg-0pky">{ "content": "INPUT" } =&gt; INPUT</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a string</td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">Read the string from the field "content" validate that it is a valid hex string and store the string as is</td>
    <td class="tg-0pky">{ "content": "494e505554" } =&gt; 494e505554</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid hex string</td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">Read the string from the field "content" validate that it is a valid base64 string and store the string as is</td>
    <td class="tg-0pky">{ "content": "SU5QVVQ=" } =&gt; SU5QVVQ=</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid base64 string</td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">{ "content": "INPUT" } =&gt; { "content": "INPUT" }</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">Convert the JSON structure to YAML structure, leaving the content unaltered</td>
    <td class="tg-0pky">{ "content": "INPUT" } =&gt; content: INPUT</td>
    <td class="tg-0lax">- Invalid content like too large numbers or missing fields</td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">JSON structure must be equal to the protobuf message</td>
    <td class="tg-0pky">{ "distance": 32, "name": "Path" } => [1] distance: 32\n[2] name: Path</td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### YAML

#### Options

| Name   | Description                                                                                                         | Default | Possible values   |
|--------|---------------------------------------------------------------------------------------------------------------------|---------|-------------------|
| raw_as | Type as which raw values will be rendered; if utf8, invalid characters will be displayed with replacement character | hex     | hex, base64, utf8 |

#### Conversions

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
    <th class="tg-c8dp">Possible failures</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Read the string from the field "content" and store it</td>
    <td class="tg-0pky">content: INPUT =&gt; INPUT</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a string</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">Read the string from the field "content" and store it as raw bytes</td>
    <td class="tg-0pky">content: INPUT =&gt; INPUT</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a string</td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">Read the string from the field "content" validate that it is a valid hex string and store the string as is</td>
    <td class="tg-0pky">content: 494e505554 =&gt; 494e505554</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid hex string</td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">Read the string from the field "content" validate that it is a valid base64 string and store the string as is</td>
    <td class="tg-0pky">content: SU5QVVQ= =&gt; SU5QVVQ=</td>
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid base64 string</td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">Convert the YAML structure to JSON structure, leaving the content unaltered</td>
    <td class="tg-0pky">content: INPUT =&gt; { "content": "INPUT" }</td>
    <td class="tg-0lax">- Invalid content like too large numbers or missing fields</td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">content: INPUT =&gt; content: INPUT</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">YAML structure must be equal to the protobuf message</td>
    <td class="tg-0pky">distance: 32\nname: Path => [1] distance: 32\n[2] name: Path</td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### Protobuf

#### Options

| Name       | Description                                                                  | Default | Possible values        |
|------------|------------------------------------------------------------------------------|---------|------------------------|
| definition | Path to the file containing the definition of the protobuf message           | none    | a valid path to a file |
| message    | Name of the message from the definition file to use for decoding the payload | none    | a valid message name   |

#### Conversions

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
    <th class="tg-c8dp">Possible failures</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Converts the protobuf message to a human-readable text format:<br/>
    [field number] field name = value (field type)</td>
    <td class="tg-0pky">Proto.Response (Message)<br/>
  [1] distance = 32 (Int32)<br/>
  [2] Proto.Inner (Message)<br/>
  &nbsp;&nbsp;[1] kind = kindof (String)<br/>
  [3] position = "POSITION_INSIDE" (Enum Proto.Position)<br/>
  [4] raw = "wyg=" (Bytes)</td>
    <td class="tg-0lax">- field not found in message specification</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">Stores the encoded protobuf message as bytes</td>
    <td class="tg-0pky">msg_bytes =&gt; msg_bytes</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">Stores the encoded protobuf message as hex string</td>
    <td class="tg-0pky">INPUT =&gt; 494e505554</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">Stores the encoded protobuf message as base64 strings</td>
    <td class="tg-0pky">INPUT =&gt; SU5QVVQ=</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">Convert the protobuf structure to JSON structure, leaving the content unaltered</td>
    <td class="tg-0pky">[1] content=INPUT =&gt; { "content": "INPUT" }</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">Convert the protobuf structure to YAML structure, leaving the content unaltered</td>
    <td class="tg-0pky">[1] content=INPUT =&gt; content: INPUT</td>
    <td class="tg-0lax"></td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">[1] distance: 32\n[2] name: Path => [1] distance: 32\n[2] name: Path</td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

## Future plans

* Single-topic clients for each subscribe and publish
    * publish one message (or the same message repeatedly) to a single topic
    * subscribe for one topic
    * this mode is only configurable via cli args
* Support MQTT5 attributes
  * user properties
  * content-type (to automatically detect the format of a topic)
  * other attributes
