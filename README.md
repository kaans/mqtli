# MQTli

MQTli is a multi-topic payload-converting MQTT cli client written in Rust.

It can be configured to automatically convert between different payload formats 
when reading input data for publish and outputting data for subscribe.
The supported data formats and the conversion rules are listed under [supported payload formats](#supported-formats)

## Supported features:

* Configure multiple topics with the following settings for each:
  * Input: periodically send messages (publish)
  * Payload: define the payload format of the topic
  * Output: print incoming payload to console or file (subscribe)
* Automatically convert the message payload between input, payload, and output
* Configuration via cli arguments and config file (yaml)
* MQTT v5 (only)
* TLS support (v1.2 and v1.3)
* Client authentication via username/password
* Client authentication via TLS certificates
* Last will
* QoS 0, 1, 2

## <a name="supported-formats"></a>Supported Payload formats and conversion

### Raw (binary)

<table class="tg">
<thead>
  <tr>
    <th class="tg-fr0y">Convert to</th>
    <th class="tg-fr0y">Description</th>
    <th class="tg-fr0y">Example</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-0cjc">Text<br>(UTF-8)</td>
    <td class="tg-0pky">Depending on the config ("raw_as") it is encoded as hex or base64</td>
    <td class="tg-0pky">INPUT =&gt; 494e505554</td>
  </tr>
  <tr>
    <td class="tg-0cjc">Raw</td>
    <td class="tg-0pky">No conversion needed</td>
    <td class="tg-0pky">INPUT =&gt; INPUT</td>
  </tr>
  <tr>
    <td class="tg-l23c">Hex</td>
    <td class="tg-0pky">Converts input to hex</td>
    <td class="tg-0pky">INPUT =&gt; 494e505554</td>
  </tr>
  <tr>
    <td class="tg-l23c">Base64</td>
    <td class="tg-0pky">Converts input to base64</td>
    <td class="tg-0pky">INPUT =&gt; SU5QVVQ=</td>
  </tr>
  <tr>
    <td class="tg-l23c">JSON</td>
    <td class="tg-0pky">Puts input text into field content of new JSON object<br/>Depending on the config ("raw_as") it is encoded as hex or base64</td>
    <td class="tg-0pky">INPUT =&gt; { "content": "494e505554" }</td>
  </tr>
  <tr>
    <td class="tg-l23c">YAML</td>
    <td class="tg-0pky">Puts input text into field content of new YAML object<br/>Depending on the config ("raw_as") it is encoded as hex or base64</td>
    <td class="tg-0pky">INPUT =&gt; content: 494e505554</td>
  </tr>
  <tr>
    <td class="tg-jilr">Protobuf</td>
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
  </tr>
</tbody>
</table>

### Text (UTF-8)

<table class="tg">
<thead>
  <tr>
    <th class="tg-c8dp">Convert to</th>
    <th class="tg-c8dp">Description</th>
    <th class="tg-c8dp">Example</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td class="tg-1o19">Text<br>(UTF-8)</td>
    <td class="tg-0lax">No conversion needed</td>
    <td class="tg-0lax">INPUT =&gt; INPUT</td>
  </tr>
  <tr>
    <td class="tg-1o19">Raw</td>
    <td class="tg-0lax">No conversion needed</td>
    <td class="tg-0lax">INPUT =&gt; INPUT</td>
  </tr>
  <tr>
    <td class="tg-a00j">Hex</td>
    <td class="tg-0lax">Converts input to hex</td>
    <td class="tg-0lax">INPUT =&gt; 494e505554</td>
  </tr>
  <tr>
    <td class="tg-a00j">Base64</td>
    <td class="tg-0lax">Converts input to base64</td>
    <td class="tg-0lax">INPUT =&gt; SU5QVVQ=</td>
  </tr>
  <tr>
    <td class="tg-a00j">JSON</td>
    <td class="tg-0lax">Puts input text into field content of new JSON object</td>
    <td class="tg-0lax">INPUT =&gt; { "content": "INPUT" }</td>
  </tr>
  <tr>
    <td class="tg-a00j">YAML</td>
    <td class="tg-0lax">Puts input text into field content of new YAML object</td>
    <td class="tg-0lax">INPUT =&gt; content: INPUT</td>
  </tr>
  <tr>
    <td class="tg-baly">Protobuf</td>
    <td class="tg-0lax">Not possible</td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### Hex

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
    <td class="tg-0pky">Decodes hex String and tries to convert bytes to UTF-8</td>
    <td class="tg-0pky">494e505554 =&gt; INPUT</td>
    <td class="tg-0lax">- length of hex string is not even<br>- decoded bytes contain non-UTF-8 characters</td>
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
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### Base64

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
    <td class="tg-0pky">Decodes base64 String and tries to convert bytes to UTF-8</td>
    <td class="tg-0pky">SU5QVVQ= =&gt; INPUT</td>
    <td class="tg-0lax">- wrong length, padding, or bytes in base64 string<br>- decoded bytes contain non-UTF-8 characters</td>
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
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### JSON

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
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid base64 string</td>
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
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### YAML

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
    <td class="tg-0lax">- field "content" does not exist or does not contain a valid base64 string</td>
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
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>

### Protobuf

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
    <td class="tg-0pky">Converts the protobuf message to a human readable text format:<br/>
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
    <td class="tg-0pky">Not possible</td>
    <td class="tg-0pky"></td>
    <td class="tg-0lax"></td>
  </tr>
</tbody>
</table>


## Future plans

* Support MQTT v3 and v3.1
* Support websockets
* Single-topic clients for each subscribe and publish
  * publish one message (or the same message repeatedly) to a single topic
  * subscribe for one topic
  * this mode is only configurable via cli args
