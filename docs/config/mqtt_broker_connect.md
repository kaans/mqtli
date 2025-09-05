---
title: Broker connection
---

Broker connection
=================

Use these options to tell MQTli how to connect to your MQTT broker, which protocol/version to use, whether to use TLS, and (optionally) a last‑will message.

Broker host
-----------
Specify the hostname or IP address of the MQTT broker you want to connect to.
- Values: string.
- Default: localhost.
- How to set:
  - CLI: --host
  - ENV: BROKER_HOST
  - YAML: broker.host

Broker port
-----------
Choose the network port on which your MQTT broker is listening.
- Values: integer.
- Default: 1883.
- How to set: --port | BROKER_PORT | broker.port

Protocol
--------
Select the transport to connect to the broker, either a raw TCP socket or a WebSocket connection.
- Values: tcp | websocket.
- Default: tcp.
- How to set: --protocol | BROKER_PROTOCOL | broker.protocol

Client ID
---------
Set a unique identifier for this client instance on the broker.
- Values: string.
- Default: mqtli.
- How to set: --client-id | BROKER_CLIENT_ID | broker.client_id

MQTT version
------------
Choose the MQTT protocol version for the connection.
- Values: v311 | v5.
- Default: v5.
- How to set: --mqtt-version | BROKER_MQTT_VERSION | broker.mqtt_version

Keep alive
----------
Set how often the client sends keep‑alive pings to the broker (in seconds).
- Values: integer seconds, must be >= 5.
- Default: 5.
- How to set: --keep-alive | BROKER_KEEP_ALIVE | broker.keep_alive

Username
--------
Provide a username for authenticating to the broker (optional).
- Values: string.
- Default: empty (unset).
- How to set: --username | BROKER_USERNAME | broker.username

Password
--------
Provide the password that pairs with the username for authentication (optional).
- Values: string.
- Default: empty (unset).
- How to set: --password | BROKER_PASSWORD | broker.password
- Note: Username and password must be provided together.

Use TLS
-------
Enable TLS encryption for the connection to secure traffic between client and broker.
- Values: true | false.
- Default: false.
- How to set: --use-tls | BROKER_USE_TLS | broker.use_tls

TLS CA file
-----------
Provide the path to a PEM‑encoded CA certificate used to verify the broker’s certificate.
- Values: file path (string).
- Default: empty (unset).
- How to set: --ca-file | BROKER_TLS_CA_FILE | broker.tls_ca_file

TLS client certificate
----------------------
Specify the PEM‑encoded client certificate file for mutual TLS authentication (use with a matching key).
- Values: file path (string).
- Default: empty (unset).
- How to set: --client-cert | BROKER_TLS_CLIENT_CERTIFICATE_FILE | broker.tls_client_certificate
- Note: Must be provided together with TLS client key.

TLS client key
--------------
Specify the unencrypted PKCS#8 client private key file for mutual TLS (pairs with the client certificate).
- Values: file path (string).
- Default: empty (unset).
- How to set: --client-key | BROKER_TLS_CLIENT_KEY_FILE | broker.tls_client_key
- Note: Must be provided together with TLS client certificate.

TLS version
-----------
Limit which TLS protocol versions are allowed during the handshake.
- Values: all | v12 | v13.
- Default: all.
- How to set: --tls-version | BROKER_TLS_VERSION | broker.tls_version

Last will — topic
-----------------
Set the topic where the broker will publish your last‑will message if the client disconnects unexpectedly.
- Values: string.
- Default: empty (unset; last will disabled if topic missing).
- How to set: --last-will-topic | BROKER_LAST_WILL_TOPIC | broker.last_will.topic

Last will — payload
-------------------
Provide the UTF‑8 string payload that will be sent in the last‑will message.
- Values: string.
- Default: empty.
- How to set: --last-will-payload | BROKER_LAST_WILL_PAYLOAD | broker.last_will.payload

Last will — QoS
---------------
Choose the Quality of Service level used when the last‑will message is published.
- Values: 0 | 1 | 2.
- Default: 0.
- How to set: --last-will-qos | BROKER_LAST_WILL_QOS | broker.last_will.qos

Last will — retain
------------------
Decide whether the last‑will message should be stored by the broker as a retained message.
- Values: true | false.
- Default: false.
- How to set: --last-will-retain | BROKER_LAST_WILL_RETAIN | broker.last_will.retain

YAML example
```yaml
broker:
  host: localhost
  port: 1883
  protocol: tcp
  client_id: mqtli
  mqtt_version: v5
  keep_alive: 5
  use_tls: false
  # username: ""
  # password: ""
  # tls_ca_file: "ca.pem"
  # tls_client_certificate: "client.crt"
  # tls_client_key: "client.key"
  # tls_version: all  # all|v12|v13
  # last_will:
  #   topic: lwt
  #   payload: "Good bye"
  #   qos: 0
  #   retain: false
```

Notes
- keep_alive must be at least 5 seconds.
- If username is set, password must also be set (and vice versa).
- TLS client certificate and key must be provided together.


Examples
--------
Example A — Plain TCP, no TLS
```yaml
broker:
  host: localhost
  port: 1883
  protocol: tcp
  mqtt_version: v5
  keep_alive: 10
  use_tls: false
```

Example B — TLS with CA validation
```yaml
broker:
  host: broker.example.com
  port: 8883
  use_tls: true
  tls_ca_file: "ca.pem"
  mqtt_version: v311
  keep_alive: 20
```

Example C — Mutual TLS (client certificate + key)
```yaml
broker:
  host: secure.example.net
  port: 8883
  use_tls: true
  tls_ca_file: "ca.pem"
  tls_client_certificate: "client.crt"
  tls_client_key: "client.key"
  tls_version: v13
```

Example D — WebSocket connection
```yaml
broker:
  host: ws-broker.example.com
  port: 9001
  protocol: websocket
  use_tls: false
```

Example E — Last‑Will configured
```yaml
broker:
  host: localhost
  port: 1883
  last_will:
    topic: lwt/mqtli
    payload: "bye"
    qos: 1
    retain: true
```
