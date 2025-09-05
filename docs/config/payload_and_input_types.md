---
title: Payload types and publish input types
---

Payload types
=============

Each payload type has its own section below.

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

Each publish input type has its own section below.

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
- What it does: Inline hex or file path to hex.
- Fields: content and/or path.

json
----
- What it does: Inline JSON or file path.
- Fields: content and/or path.

yaml
----
- What it does: Inline YAML or file path.
- Fields: content and/or path.

base64
------
- What it does: Inline base64 or file path.
- Fields: content and/or path.

null
----
- What it does: No content is provided.

Validation
----------
- For content/path variants, at least one of content or path must be provided.

QoS in YAML
-----------
- QoS for subscription outputs and publish can be specified as 0, 1, 2.
