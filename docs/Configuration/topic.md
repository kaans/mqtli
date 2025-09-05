---
title: Topics
---

Topics
======

Use this section to define the topics you care about: specify the MQTT topic or pattern, declare how payloads on that topic should be interpreted, choose how incoming messages are displayed or forwarded, and optionally automate publishing. Wildcards (+, #) and automatic format conversion let you build flexible flows for both subscribe and publish.

Topic
-----
Provide the MQTT topic or pattern this entry applies to.
- Values: string. Supports + and # wildcards for matching incoming topics.
- Default: none (required).
- How to set: YAML: topics[].topic

Payload
-------
Declare the expected payload format used by messages on this topic.
- Values: json | yaml | protobuf | sparkplug | sparkplug_json | hex | base64 | text | raw (plus attributes for protobuf/sparkplug).
- Default: text in some contexts; recommended to set explicitly.
- How to set: YAML: topics[].payload.{type,...}
- See also: Payload types page for attributes like definition/message for protobuf.

Subscription
------------
Configure how received messages should be output (format, targets, and optional filters).
- Values: object; see the Subscription page.
- Default: unset.
- How to set: YAML: topics[].subscription

Publish
-------
Define how to automatically publish messages on this topic, including input, triggers, filters, and QoS/retain.
- Values: object; see the Publish page.
- Default: unset.
- How to set: YAML: topics[].publish

YAML example
------------
```yaml
topics:
  - topic: mqtli/topic
    subscription:
      enabled: true
      qos: 0
      outputs:
        - format: { type: yaml }
          target: { type: console }
    payload:
      type: protobuf
      # definition: messages.proto
      # message: Proto.Message
    publish:
      enabled: false
      input:
        type: text
        content: "hello"
      trigger:
        - type: periodic
          interval: 1000
```

Notes
- You can configure the same MQTT topic multiple times for different flows.
- Payload conversion is automatic between payload.type, publish.input.type, and subscription.output.format.type when possible.


More examples
-------------
Example 1 — Wildcard subscription and two outputs
```yaml
topics:
  - topic: sensors/+/data
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target: { type: console }
        - format: { type: base64 }
          target:
            type: file
            path: sensors.b64
            overwrite: false
            append: "\n"
```

Example 2 — Same topic with separate publish flow
```yaml
topics:
  - topic: devices/1/cmd
    payload: { type: json }
    publish:
      enabled: true
      input:
        type: json
        content: '{"cmd":"reset"}'
      trigger:
        - type: periodic
          interval: 5000

  - topic: devices/1/cmd
    subscription:
      enabled: true
      outputs:
        - format: { type: text }
          target: { type: console }
```

Example 3 — Protobuf payload with YAML output and hex input for publish
```yaml
topics:
  - topic: telemetry/node1
    payload:
      type: protobuf
      definition: messages.proto
      message: Proto.Message
    subscription:
      enabled: true
      outputs:
        - format: { type: yaml }
          target: { type: console }
    publish:
      enabled: true
      qos: 1
      input:
        type: hex
        content: "AB12CD34"
      trigger:
        - type: periodic
          interval: 1000
```
