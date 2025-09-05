---
title: Publish and triggers
---

Publish and triggers
====================

Use these settings to automate sending messages on a topic: choose the data source (inline text/JSON/YAML, hex/base64, or file), schedule when to send with triggers, set QoS/retain flags, and optionally transform the payload with filters. Conversions are applied automatically when the input format differs from the topic’s payload format.

Enabled
-------
Turns publishing on or off for the topic entry.
- Values: true | false.
- Default: true.
- How to set in YAML: publish.enabled

QoS
---
Quality of Service level for sending messages.
- Values: 0 | 1 | 2.
- Default: 0.
- How to set in YAML: publish.qos

Retain
------
Whether published messages are retained by the broker.
- Values: true | false.
- Default: false.
- How to set in YAML: publish.retain

Input — type
------------
Select how the message data is provided.
- Values: text | raw | hex | json | yaml | base64 | null.
- Default: text (empty content/path).
- How to set in YAML: publish.input.type

Input — content
---------------
Inline content for the message (for text/json/yaml/hex/base64).
- Values: string.
- Default: empty (unset).
- How to set in YAML: publish.input.content

Input — path
------------
File path from which to read the message.
- Values: string (path). For raw, path is required; for other types, content and/or path may be used.
- Default: empty (unset).
- How to set in YAML: publish.input.path

Trigger — type
--------------
Select a trigger mechanism. Currently periodic is supported.
- Values: periodic.
- Default: periodic with 1s interval if not specified but triggers present.
- How to set in YAML: publish.trigger[].type

Trigger — interval
------------------
Period between publishes in milliseconds.
- Values: integer milliseconds.
- Default: 1000.
- How to set in YAML: publish.trigger[].interval

Trigger — count
---------------
Number of messages to publish; omit for infinite.
- Values: integer (u32), optional.
- Default: unset (infinite).
- How to set in YAML: publish.trigger[].count

Trigger — initial_delay
-----------------------
Initial delay before the first publish, in milliseconds.
- Values: integer milliseconds.
- Default: 1000.
- How to set in YAML: publish.trigger[].initial_delay

Filters
-------
Optional chain to transform the message before sending.
- Values: list of filters; see [Filters page](filter.md)
- Default: empty list.
- How to set in YAML: publish.filters

Notes
- If input type and the topic payload.type differ, automatic conversion is attempted.
- For binary data, consider hex or base64 inline strings or use a file path.

YAML example
------------
```yaml
publish:
  enabled: true
  qos: 0
  retain: false
  input:
    type: text
    content: "hello"
  trigger:
    - type: periodic
      interval: 1000
      # count: 10
      initial_delay: 0
  filters:
    - type: to_upper
```


More examples
-------------
Example 1 — Periodic with count and initial delay
```yaml
publish:
  enabled: true
  qos: 1
  retain: false
  input:
    type: json
    content: '{"ping":true}'
  trigger:
    - type: periodic
      interval: 500
      count: 5
      initial_delay: 100
```

Example 2 — Read raw bytes from file
```yaml
publish:
  enabled: true
  qos: 0
  input:
    type: raw
    path: payload.bin
  trigger:
    - type: periodic
      interval: 2000
```

Example 3 — Hex inline with filter chain to upper text
```yaml
publish:
  enabled: true
  input:
    type: hex
    content: "48656c6c6f"  # "Hello"
  filters:
    - type: to_text
    - type: to_upper
  trigger:
    - type: periodic
      interval: 1000
```
