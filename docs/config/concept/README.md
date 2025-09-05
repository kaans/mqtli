---
title: Conceptual Overview
---

Conceptual overview of MQTli
============================

What is MQTli?
--------------
MQTli is a command‑line tool that connects to an MQTT broker, subscribes to and/or publishes on topics, and abstracts away the pain of inconsistent payload formats across topics. It can automatically convert between different input and output formats and apply filter chains to transform the data on the fly. Results can be sent to the console, written to files, inserted into a SQL database, or published back to another topic. This enables powerful ETL‑style workflows: transform data from one topic and route it to another topic or persist selected fields into a database table.

High‑level architecture
-----------------------
At a high level, MQTli is configured with:
- Broker connection: How to connect/authenticate to the MQTT broker (TCP/WebSocket, TLS, credentials, client ID, keep‑alive, last‑will, etc.).
- Topics: A list of topic entries. Each entry may define:
  - Subscription: Whether to listen on the topic, optionally with QoS. Each subscription can have multiple outputs (console/file/sql/topic) and an optional filter chain that transforms the payload before output.
  - Publish: Whether to publish to the topic, where the input content comes from (inline text/JSON/file/stdin), optional filters to transform the input prior to publishing, and triggers (e.g., periodic) that define when to publish.
  - Payload: The expected input format of the topic (e.g., text, json, bytes) to assist with parsing/conversion.

Data flow (simplified)
----------------------
1) Connect to broker using the broker section of the configuration.
2) For each configured topic:
   - If subscription is enabled:
     a) Receive messages from the broker.
     b) Normalize/parse payload according to the topic’s payload type.
     c) Apply the configured filter chain (zero or more filters). Filters can convert between formats and/or mutate content.
     d) Send the transformed result to each configured output (console, file, SQL, or another topic).
   - If publish is enabled:
     a) Build the input payload (text/JSON/file/stdin).
     b) Apply the configured filter chain.
     c) Convert to the required wire format and publish to the broker.

Why payload abstraction matters
-------------------------------
Different topics often carry payloads using inconsistent styles (plain text, JSON objects, CSV‑like lines, arbitrary bytes). MQTli’s payload handling and filters allow you to:
- Parse and validate inbound payloads.
- Extract, reshape, or enrich fields.
- Convert between formats (e.g., JSON → text, text → JSON) automatically as needed by the chosen outputs.
- Keep your downstream consumers stable even if upstream payloads vary by topic.

Core components (high‑level)
----------------------------
- Broker connector
  Establishes and maintains the MQTT connection. Manages protocol selection (v3.1.1/v5), TCP vs WebSocket, optional TLS, authentication, keep‑alive, and last‑will configuration.

- Topic registry
  A list of topic configurations. Each topic acts as a logical processing unit that can independently subscribe, publish, transform, and route messages.

- Payload handler
  Interprets the raw bytes for a topic according to the declared payload type (e.g., text, json), enabling reliable conversion and filter operations.

- Filter engine
  Executes a sequence of filters to transform content. Filters can extract JSON paths, append/prepend text, map fields, coerce types, or perform other format conversions. The engine ensures the output of one filter becomes the input to the next, forming a pipeline.

- Output plugins
  Pluggable sinks for transformed data. Out of the box:
  - Console: Print as text or JSON for quick inspection.
  - File: Append or write to files for logging or archival.
  - SQL: Map fields into columns and insert rows into a relational database.
  - Topic: Publish the result to another MQTT topic (enabling routing and fan‑out patterns).

- Publisher with triggers
  Publishes content to topics based on configured triggers (for example, periodic timers). Content can be inline, read from files/stdin, or generated via filters, then published in the desired format.

Filters and transformations
---------------------------
Filters are applied in order and can both convert formats and mutate the data. Typical examples include:
- extract_json with a JSONPath (e.g., $.measurements.temperature) to focus on a specific field or sub‑object.
- prepend/append text around a value (e.g. add a unit to a number and print it on the console or to a file)
- casting or formatting steps (e.g., numeric → string).
- restructuring objects (e.g., remapping keys), when supported by the available filter set.

Because filters run for both subscriptions and publications, you can normalize incoming data and also synthesize outgoing messages.

Outputs in depth
----------------
- Console output
  - Quick way to observe transformed data. Useful during development or when running MQTli as a monitoring tool.

- File output
  - Write transformed payloads to files. Typical use is operational logging, audit trails, or creating CSV/JSONL feeds for later batch processing.

- SQL storage
  - Define how transformed fields map to table columns. MQTli will insert rows for each message after filters are applied. This is ideal for persisting metrics or events for reporting/analytics.

- Topic output
  - Publish the transformed result on another MQTT topic. This enables patterns like: listen on sensor/raw, transform, then publish to sensor/normalized.

ETL with MQTli
--------------
Thanks to the combination of subscriptions, filters, and outputs, MQTli performs streaming ETL:
- Extract: Subscribe to topics; parse payloads (e.g., JSON) using the payload handler.
- Transform: Apply one or more filters to clean, enrich, or reshape the data.
- Load: Send results to your destination: console, file, SQL database, or a different MQTT topic.

Common patterns
---------------
- Normalize JSON to text for dashboards

  Subscribe to a topic where a sensor value, like a temperature, is transmitted as JSON. Extract the temperature
  using JSONPath (e.g. `$.temperature`), prepend "Temperature: ", append " °C" and print to console.

- Topic‑to‑topic transformation (routing)

  Subscribe to a topic, extract sub‑fields, reshape the JSON, and publish the normalized data to another topic.

- Periodic publisher

  Configure publish with a periodic trigger that emits a heartbeat or a synthesized JSON blob every N seconds.

Where to find the details
-------------------------
- [Quickstart](../quickstart.md) — build/run and a minimal config to get going.
- [Broker connection](../config/mqtt_broker_connect.md) — all connection options.
- Topic configuration:
  - [Payload and input types](../config/topic/payload_and_input_types.md)
  - [Filters](../config/topic/filter.md)
  - [Subscription (and outputs)](../config/topic/subscription.md)
  - [Publish](../config/topic/publish.md)
- [SQL storage](../config/sql_storage.md) — mapping fields to a relational table.

Configuration sketch (YAML)
---------------------------
Below is a compact sketch showing how the pieces relate. See Quickstart and reference pages for the full syntax and options.

```yaml
broker:
  host: localhost
  port: 1883
  use_tls: false

topics:
  - topic: sensor/raw
    payload:
      type: json
    subscription:
      enabled: true
      outputs:
        - format: { type: text }    # console
        - file: { path: "out.log" } # file sink
        - sql:
            dsn: "postgres://user:pw@localhost/db"
            table: "measurements"
            mapping: { time: "$.ts", device_id: "$.id", temperature: "$.t" }
      filters:
        - type: extract_json
          jsonpath: "$.measurements"

  - topic: sensor/normalized
    publish:
      enabled: true
      input:
        type: json
        content: '{"heartbeat": true}'
      filters:
        - type: append
          content: "" # placeholder to show filters can run here too
      trigger:
        - type: periodic
          interval: 5000
```

Key takeaways
-------------
- MQTli connects to your MQTT broker and manages protocol/TLS/auth details for you.
- You define a list of topics with subscribe and/or publish behavior.
- Payload types and filters let you automatically convert between formats and transform data.
- Outputs route transformed data to console, files, SQL databases, or other topics.
- Combining these pieces enables powerful streaming ETL patterns without writing code.
