---
title: Operating Modes
---

Operating Modes
================

MQTli can run in one of four mutually exclusive modes. You select a mode via a CLI argument. If no mode is specified, the default is multi topic mode.

## Selecting a mode

You select the operating mode with the --mode CLI argument (or the MODE environment variable). If you omit the flag entirely, MQTli starts in the default multi_topic mode. For example, running mqtli without arguments launches multi_topic; specifying mqtli --mode subscribe activates the subscribe-only behavior; mqtli --mode publish starts a session intended for sending data; and mqtli --mode sparkplug enables the Sparkplug-specific monitoring features. Only one mode can be active at a time in a single run.

### Multi topic (default)

In multi_topic mode, MQTli operates entirely from a configuration file and can simultaneously subscribe to many topics and publish to others. This is the most capable mode: it allows you to define subscriptions with multiple outputs (console, files, SQL, or forwarding to another topic), publishers driven by triggers (periodic timers or reactions to incoming messages), and chains of filters that transform payloads. The configuration file is required in this mode because it is the source of truth for topics and their behavior. Without a topics list in the file, no automatic subscribe or publish actions will occur. Broker connection settings may be provided on the CLI, via environment variables, or in the configuration file; whichever source you prefer will work.

To select multi topic mode, nothing has to be specified as it is the default.

### Subscribe only

Subscribe mode focuses on receiving messages and printing or otherwise handling them based on CLI/ENV settings. It is intended for single-topic use in a given invocation: you typically point MQTli at one topic or pattern to monitor, in contrast to the default multi topic mode which is designed to orchestrate multiple subscriptions and publishers at once via a configuration file. You do not need a configuration file for subscribe mode. If you provide one anyway, MQTli will read only the broker and other top‑level settings from it and will intentionally ignore any topics defined there. The topics list from YAML is not consulted in this mode. You can still control the broker connection parameters entirely from the CLI and environment variables if you prefer.

To select subscribe only mode, use: `mqtli subscribe`

### Publish only

Publish mode is intended for sending messages, and it targets single-topic publishing in a given run. You typically push data to one MQTT topic from the command line, unlike the multi topic mode which coordinates multiple publishers and subscriptions defined in a configuration file. This mode is driven by CLI/ENV options rather than a YAML topics list. A configuration file is not required. If a file is present, only the broker and other top‑level settings are used; any topics entries in the file are ignored while this mode is active. As with subscribe mode, you can provide all connection details on the command line or through environment variables.

To select publish only mode, use: `mqtli publish`

### Sparkplug mode

Sparkplug mode is designed to monitor a network of Sparkplug devices. When you enable this mode, MQTli subscribes to the predefined Sparkplug topics and decodes payloads accordingly. A configuration file is optional. If you supply one, its broker and top‑level settings are honored. Topic entries in the file are optional and, by default, are ignored in Sparkplug mode; if you want to include them in addition to the Sparkplug subscriptions, pass the --include-topics-from-file flag. You can further tailor Sparkplug subscriptions by selecting a default QoS with --qos (or SPARKPLUG_QOS) and by restricting the monitored groups using --include-group (or its short form --ig) with a comma‑separated list. If you do not set a QoS, QoS 0 is used.

To select sparkplug mode, use: `mqtli sp` or `mqtli sparkplug`

## See also

- [Top‑level settings](config)
- [Topic configuration](config/topic)
- [Sparkplug payloads](config/topic/payload_and_input_types.md)
