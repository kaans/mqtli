---
title: Top‑level configuration
---

Top‑level configuration
=======================

This page lists each top‑level setting with its own section. For precedence of sources see the Configuration Reference index (CLI > ENV > YAML).

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
- See also: SQL storage page.

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
