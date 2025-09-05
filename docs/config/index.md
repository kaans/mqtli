---
title: Configuration Reference
---

Configuration sources and precedence
====================================

You can configure MQTli via three sources. If a setting is provided in multiple places, the following precedence applies (highest first):

1) Command line arguments
2) Environment variables
3) YAML configuration file (config.yaml by default)

If a value is not supplied, built‑in defaults apply. Complex topic configuration is only supported via the YAML file.

Quick reference of sources
- CLI: Run mqtli --help for the full list of flags.
- ENV: See the mapping in each section below (e.g., BROKER_HOST, BROKER_PORT, ...).
- YAML: See examples in config.default.yaml and the examples in this documentation.

Sections in this reference
- Top‑level settings: docs/config/mqtli_config.md
- Broker connection settings: docs/config/mqtt_broker_connect.md
- Topic configuration: docs/config/topic.md
- Subscription and outputs: docs/config/subscription.md
- Publish and triggers: docs/config/publish.md
- Filters: docs/config/filter.md
- Payload and input types: docs/config/payload_and_input_types.md
- SQL storage: docs/config/sql_storage.md

Notes on overrides and merging
- CLI/ENV options only cover the broker/logging and a few top‑level aspects. Topic definitions cannot be supplied via CLI/ENV; they must be in YAML.
- When combining sources, scalar values are overridden by higher‑precedence sources. Collections (like topics list) are not merged from CLI/ENV; they come from YAML.
- For booleans provided via CLI, both --flag true and dedicated presence/absence styles may appear; see --help output for exact forms.
