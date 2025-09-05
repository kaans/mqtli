---
title: Topics
---

Topics
======

This page lists each topic option as its own section.

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
