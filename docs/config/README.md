---
title: Configuration
---

Configuration
=======================

Use these settings to shape how MQTli runs: connect to your broker (host/port, protocol, TLS, last‑will), control logging verbosity, define topics to subscribe/publish with automatic format conversion, choose an overall mode, and optionally enable a database connection for SQL outputs.

The precedence of sources is as follows: `CLI > ENV > YAML`

This means that CLI parameters overwrite environment variables and YAML configurations. Environment variables are overwrite YAML configurations.

Broker
------
Configure how MQTli connects to your MQTT broker, including host/port, protocol, TLS, and optional last‑will.
- Default: If omitted, sensible defaults are used (host localhost; port 1883; protocol tcp; mqtt_version v5; client_id mqtli; keep_alive 5s; TLS off).
- How to set:
  - CLI/ENV: See the Broker connection page for the full list (e.g., --host, --port, --protocol, --use-tls, …)
  - YAML: broker: {...}
- See also: Broker connection page for detailed per‑option sections.

Log level
---------
Control how verbose the application logs are during execution.
- Values: trace | debug | info | warn | error | off.
- Default: info.
- How to set: --log-level | LOG_LEVEL | log_level

Topics
------
Define one or more topics, specifying payload format, how to output received messages, and how to publish automatically.
- Values: list of topic entries.
- Default: none (empty list). Without topics, the client won’t subscribe/publish anything automatically.
- How to set: YAML only: topics: [ ... ]
- See also: Topics page for full schema and examples.

Mode
----
Select the overall operating mode for the application.
- Values: multi_topic | publish | subscribe | sparkplug.
- Default: multi_topic.
- How to set: YAML: mode

SQL storage
-----------
Configure an optional database connection used by SQL outputs and storage features.
- Values: object with connection_string.
- Default: unset.
- How to set: YAML: sql_storage.connection_string
- See also: [SQL storage page](sql_storage.md)

YAML example (top level)
```yaml
broker:
  host: localhost
  port: 1883
  protocol: tcp
  client_id: mqtli
  mqtt_version: v5
  keep_alive: 5
  use_tls: false

log_level: info

# topics:
#   - ...

# sql_storage:
#   connection_string: "sqlite::memory:"
```

Notes
- Keep‑alive must be at least 5 seconds (see Broker > Keep alive).
- Username and password must be provided together when used.
- TLS client certificate and key must be provided together.


Examples
--------
Example 1 — Minimal localhost (no TLS), one console subscription
```yaml
broker:
  host: localhost
  port: 1883
  protocol: tcp
  mqtt_version: v5
  keep_alive: 5
  use_tls: false

log_level: info

topics:
  - topic: sensors/+/data
    subscription:
      enabled: true
      qos: 0
      outputs:
        - format: { type: json }
```

Example 2 — TLS with CA, forward to another topic, and periodic publisher
```yaml
broker:
  host: broker.example.com
  port: 8883
  use_tls: true
  tls_ca_file: "ca.pem"

log_level: warn

topics:
  - topic: devices/1/cmd
    payload: { type: json }
    publish:
      enabled: true
      qos: 1
      retain: false
      input:
        type: json
        content: '{"cmd":"ping"}'
      trigger:
        - type: periodic
          interval: 2000
          initial_delay: 0

  - topic: devices/1/resp
    subscription:
      enabled: true
      outputs:
        - format: { type: text }
          target:
            type: topic
            topic: devices/1/archive
            qos: 0
            retain: false
```

Example 3 — SQL storage with file logging and JSON filter
```yaml
sql_storage:
  connection_string: "sqlite::memory:"

log_level: debug

topics:
  - topic: app/logs
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target: { type: console }
        - format: { type: text }
          target:
            type: file
            path: app.log
            overwrite: false
            append: "\n"
        - format: { type: json }
          target:
            type: sql
            insert_statement: |
              INSERT INTO logs(ts, payload_json)
              VALUES (CURRENT_TIMESTAMP, ?);
    filters:
      - type: extract_json
        jsonpath: $.message
```

Example 4 — Last‑will and Sparkplug payloads
```yaml
broker:
  host: iot.example.net
  port: 1883
  last_will:
    topic: lwt/clients/mqtli
    payload: "offline"
    qos: 0
    retain: true

topics:
  - topic: spBv1.0/+/NDATA/plant1
    payload:
      type: sparkplug
    subscription:
      enabled: true
      outputs:
        - format: { type: sparkplug_json }
          target: { type: console }
```
