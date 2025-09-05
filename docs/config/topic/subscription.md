---
title: Subscription and outputs
---

Subscription and outputs
========================

Use these settings to control how incoming messages are handled on a subscribed topic: enable/disable the subscription, choose the QoS, decide how the payload is rendered, and send it to one or more targets such as console, files, another topic, or an SQL database. You can also chain filters to transform messages before output.

Enabled
-------
Turn the subscription on or off for this topic.
- Values: true | false.
- Default: true.
- How to set in YAML: subscription.enabled

QoS
---
Set the Quality of Service level to use when receiving messages.
- Values: 0 | 1 | 2.
- Default: 0.
- How to set in YAML: subscription.qos

Outputs
-------
Declare one or more outputs for received messages, each with its own format and target.
- Values: list of output objects.
- Default: empty list.
- How to set in YAML: subscription.outputs

Output — format.type
--------------------
Choose how the message is rendered for this output.
- Values: see Payload types page (e.g., json, yaml, text, hex, base64, raw, protobuf, sparkplug).
- Default: text (if omitted for some targets) — specify explicitly for clarity.
- How to set in YAML: subscription.outputs[].format.type

Output — target (console)
-------------------------
Print messages to the console.
- Values: type: console.
- Default: console is assumed if target omitted.
- How to set in YAML: subscription.outputs[].target.type: console

Output — target (file)
----------------------
Write messages to a file on disk.
- Values:
  - path: file path (string) — required
  - overwrite: bool (default false)
  - prepend: string (optional)
  - append: string (default "\n")
- How to set in YAML: subscription.outputs[].target.{path,overwrite,prepend,append}

Output — target (topic)
-----------------------
Forward the received payload to another MQTT topic.
- Values:
  - topic: string
  - qos: 0|1|2 (default 0)
  - retain: true|false (default false)
- How to set in YAML: subscription.outputs[].target.{topic,qos,retain}

Output — target (sql)
---------------------
Insert each received payload into a database using a custom SQL statement.
- Values:
  - insert_statement: string
- How to set in YAML: subscription.outputs[].target.insert_statement (plus top‑level sql_storage configured)

Filters
-------
Optionally transform received messages before output using a chain of filters.
- Values: list of filters; see [Filters page](filter.md)
- Default: empty list.
- How to set in YAML: subscription.filters

YAML example
------------
```yaml
subscription:
  enabled: true
  qos: 0
  outputs:
    - format: { type: json }
      target: { type: console }
    - format: { type: base64 }
      target:
        type: file
        path: log.txt
        overwrite: false
        prepend: "MESSAGE: "
        append: "\n"
    - format: { type: text }
      target:
        type: topic
        topic: other/topic
        qos: 0
        retain: false
  filters:
    - type: extract_json
      jsonpath: $.name
```


More examples
-------------
Example 1 — Console + file outputs
```yaml
subscription:
  enabled: true
  qos: 1
  outputs:
    - format: { type: yaml }
      target: { type: console }
    - format: { type: text }
      target:
        type: file
        path: received.txt
        overwrite: false
        append: "\n"
```

Example 2 — Forward to another topic with QoS/retain
```yaml
subscription:
  enabled: true
  qos: 0
  outputs:
    - format: { type: raw }
      target:
        type: topic
        topic: archive/raw
        qos: 1
        retain: true
```

Example 3 — Insert into SQL and extract a field
```yaml
subscription:
  enabled: true
  outputs:
    - format: { type: json }
      target:
        type: sql
        insert_statement: |
          INSERT INTO messages(ts, payload_json)
          VALUES (CURRENT_TIMESTAMP, ?);
  filters:
    - type: extract_json
      jsonpath: $.data
```
