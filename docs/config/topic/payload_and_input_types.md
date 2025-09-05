---
title: Payload types and publish input types
---

Payload types
=============

This page explains the formats you can work with and how MQTli treats them when converting and rendering messages—for example, human‑readable text/JSON/YAML, binary encodings like hex/base64/raw, and structured formats such as Protobuf and Sparkplug. Choose a payload type to match your topic’s data so conversions and outputs behave as expected.

Text
----
UTF‑8 text payloads.
- Typical use: human‑readable strings.
- Notes: Can convert to most other formats; invalid UTF‑8 in conversions will be preserved with replacement when displayed.

JSON
----
JSON documents.
- Typical use: structured data.
- Notes: If converted from binary, the decoded data must be valid UTF‑8 JSON.

YAML
----
YAML documents.
- Notes: If converted from binary, decoded data must be valid UTF‑8 YAML.

Hex
---
Hex‑encoded bytes (lower/upper accepted when read; shown lower‑case).
- Typical use: inline binary representation in YAML.

Base64
------
Base64‑encoded bytes (with padding).
- Typical use: inline binary representation in YAML.

Raw
---
Uninterpreted bytes.
- Notes: Everything can convert to raw.

Protobuf
--------
Protobuf‑encoded bytes.
- Attributes (when used as payload):
  - definition: path to .proto
  - message: fully qualified message name
- Notes: Text cannot convert directly into protobuf.

Sparkplug
---------
Eclipse Sparkplug payloads (protobuf‑based).

Sparkplug JSON
--------------
JSON representation compatible with Sparkplug payloads.

Conversions
-----------
- See README “Supported Payload formats and conversion” for the conversion table. Many conversions are supported; text lacks structure and cannot be converted into protobuf directly.

Publish input types
===================

This section describes how to provide data for publishing: inline content or file‑based sources in various formats. Pick the input type that matches how you want to supply the message, and MQTli will convert it to the topic’s payload format when possible.

text
----
Inline text.
- Fields: content and/or path.

raw
---
Read raw bytes from a file.
- Fields: path (required).

hex
---
Inline hex or file path to hex.
- Fields: content and/or path.

json
----
Inline JSON or file path.
- Fields: content and/or path.

yaml
----
Inline YAML or file path.
- Fields: content and/or path.

base64
------
Inline base64 or file path.
- Fields: content and/or path.

null
----
No content is provided.

Validation
----------
- For content/path variants, at least one of content or path must be provided.

QoS in YAML
-----------
- QoS for subscription outputs and publish can be specified as 0, 1, 2.


Examples
--------
Example 1 — Hex input published to a protobuf topic, YAML output on subscribe
```yaml
topics:
  - topic: telemetry/node1
    payload:
      type: protobuf
      definition: messages.proto
      message: Proto.Message
    publish:
      enabled: true
      input:
        type: hex
        content: "AB12CD34"
      trigger:
        - type: periodic
          interval: 1000
    subscription:
      enabled: true
      outputs:
        - format: { type: yaml }
```

Example 2 — Base64 input to JSON payload with extract filter
```yaml
topics:
  - topic: app/data
    payload: { type: json }
    publish:
      enabled: true
      input:
        type: base64
        content: "eyJtZXNzYWdlIjogImhlbGxvIn0="  # {"message":"hello"}
      filters:
        - type: to_json
        - type: extract_json
          jsonpath: $.message
      trigger:
        - type: periodic
          interval: 1500
```
