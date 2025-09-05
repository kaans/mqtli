---
title: Quickstart
---

Quickstart
==========

This guide helps you get MQTli running quickly, either by downloading a prebuilt binary or building from source. It also shows how to connect to a local MQTT broker on localhost:1883 without TLS, optionally using username/password.

Option 1 — Download a prebuilt binary
-------------------------------------
- Visit the Releases page and download the binary for your platform:
  - https://github.com/kaans/mqtli/releases
- Extract the archive if needed.
- Run the executable to see available options:
  - Windows (PowerShell): `./mqtli.exe --help`
  - Linux/macOS: `./mqtli --help`

Option 2 — Build from source with Cargo
---------------------------------------
Prerequisites
- Rust toolchain installed: https://rustup.rs
- (Optional) Docker if you want to run a local Mosquitto broker via docker-compose.

Build
- From the repository root, build a release binary:
  - `cargo build --release`
- The binary will be located at:
  - `target/release/mqtli` (Linux/macOS)
  - `target/release/mqtli.exe` (Windows)

Run
- Show help:
  - `target/release/mqtli[.exe] --help`

Running a local MQTT broker (localhost:1883)
-------------------------------------------
You can spin up a local Eclipse Mosquitto broker using the provided docker-compose.yaml in the repository root.

Steps (with Docker and Docker Compose installed):
1) From the repository root, start Mosquitto:
   - Windows PowerShell: `docker compose up -d mosquitto`
2) This maps broker port 1883 on your machine. You should now be able to connect to `localhost:1883` without TLS.
3) To stop it later: `docker compose down`

Minimal configuration to connect to localhost:1883 (no TLS)
-----------------------------------------------------------
You can run MQTli with CLI flags only, or via a YAML config file. Below is a minimal config file example for a non‑TLS broker on localhost:1883.

Create a file `config.yaml` next to your executable with the following content:

```yaml
# These are the default values. Change them if required.

broker:
#host: localhost
#port: 1883
#protocol: Tcp      # or websocket if your broker supports WS on a port
#mqtt_version: v5   # or v311
#keep_alive: 5
#use_tls: false
# Optional auth (see next section for username/password)

# At least one topic section is needed for useful work.
# Example: subscribe to all sensors and print output to console.
topics:
  - topic: sensor/one
    subscription:
      enabled: true
      outputs:
        - format:
            type: text # default output is console
    payload:
      type: text
    publish:
      enabled: true
      input:
        type: text
        content: "Sensor is alive"
      trigger:
        - type: periodic
          interval: 2000

  - topic: sensor/two
    subscription:
      enabled: true
      outputs:
        - format:
            type: text
      filters:
        - type: extract_json
          jsonpath: $.temperature
        - type: prepend
          content: "Temperature: "
        - type: append
          content: " °C"
    payload:
      type: json

  - topic: sensor/two
    payload:
      type: json
    publish:
      enabled: true
      input:
        type: json
        content: "{\"measurements\": {\"temperature\": 12.34} }"
      filters:
        - type: extract_json
          jsonpath: $.measurements
      trigger:
        - type: periodic # default trigger: periodic with no count (indefinitely) and interval 1 second
          interval: 2000
```

Optional username/password authentication
----------------------------------------
If your broker requires basic auth, add the following to the `broker` section in your `config.yaml`:

```yaml
broker:
  host: localhost
  port: 1883
  protocol: tcp
  mqtt_version: v5
  keep_alive: 5
  use_tls: false
  username: your_user
  password: your_password
```

Alternatively, you can pass these via CLI or environment variables:
- CLI: `mqtli --host localhost --port 1883 --username your_user --password your_password`
- ENV: `BROKER_HOST=localhost BROKER_PORT=1883 BROKER_USERNAME=your_user BROKER_PASSWORD=your_password mqtli`

Quick try: subscribe and publish
--------------------------------
- Subscribe to a topic and print messages as JSON:
  - `mqtli --host localhost --port 1883 --subscribe 'test/topic' --output-format json`
- Publish a JSON message to the same topic:
  - `mqtli --host localhost --port 1883 --publish 'test/topic' --input-format json --message '{"hello":"world"}'`

Notes
-----
- For the full list of CLI arguments and environment variables, run `mqtli --help` or see the main README.
- More configuration samples: `config.default.yaml` and `config.example.yaml` in the repository root.
- Prebuilt binaries are available at: https://github.com/kaans/mqtli/releases
