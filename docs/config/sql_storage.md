---
title: SQL storage
---

SQL storage
===========

Configure a database connection that outputs of type sql can use to persist incoming messages. This is useful for logging, analytics, and archiving; you provide a connection string, then reference it from subscription outputs that execute your INSERT statements.

Connection string
-----------------
Database connection string used by SQL outputs and optional storage features.
- Values: URL‑like string. Supported schemes: sqlite, mariadb, mysql, postgresql.
- Default: unset.
- How to set in YAML: sql_storage.connection_string
- Examples accepted:
  - sqlite::memory:
  - sqlite://        (temporary file)
  - sqlite:data.db   (no authority)
  - sqlite://data.db (with authority)


Placeholders for SQL statements
-------------------------------
When you configure a SQL output insert_statement, you can embed placeholders in double braces like {{name}}. At runtime, mqtli replaces these with values from the MQTT message, the current time, or decoded Sparkplug payload/topic fields. Some placeholders expand to literal values; others become a database bind/parameter (so the binary payload can be sent safely). Below is the complete list supported by the current implementation.

### Basic message placeholders

- {{topic}}

  The full MQTT topic of the received message.
  - Definition: Replaced with the literal topic string.
  - Example value: sensor/siteA/temperature

- {{retain}}

  Whether the message had the MQTT retain flag set.
  - Definition: Replaced with 1 if retained, otherwise 0.
  - Example value: 0

- {{qos}}

  The MQTT Quality of Service level for the message.
  - Definition: Replaced with the numeric QoS (0, 1, or 2).
  - Example value: 1

- {{created_at}}

  Time when the SQL statement is generated, in Unix epoch seconds.
  - Definition: Replaced with seconds since 1970‑01‑01 UTC.
  - Example value: 1736149123

- {{created_at_millis}}

  Time when the SQL statement is generated, in Unix epoch milliseconds.
  - Definition: Replaced with milliseconds since 1970‑01‑01 UTC.
  - Example value: 1736149123567

- {{created_at_iso}}

  Human‑readable UTC timestamp when the SQL is generated.
  - Definition: Replaced with formatted timestamp using pattern %Y-%m-%d %H:%M:%S%.3f (UTC).
  - Example value: 2025-09-05 13:27:45.123

- {{payload}}

  The raw message payload as bytes, bound as a parameter.
  - Definition: Replaced with a database placeholder token appropriate for the configured driver (e.g., ? for SQLite/MySQL, $1 for Postgres). The actual bytes are sent via a bind parameter.
  - Example value in SQL: INSERT ... VALUES($1)  (Postgres) or INSERT ... VALUES(?)  (SQLite/MySQL)

### Sparkplug placeholders

Applicable when the topic conforms to Sparkplug and the payload is Sparkplug (binary) or Sparkplug JSON as noted. If a placeholder is used outside the matching context, it will be replaced with an empty string or left null as described.

- {{sp_version}}

  Sparkplug protocol version parsed from the topic.
  - Definition: Replaced with the version token from the topic, such as spBv1.0.
  - Example value: spBv1.0

- {{sp_message_type}}

  Sparkplug message type parsed from the topic (e.g., NBIRTH, NDATA, DBIRTH, DDATA, STATE).
  - Definition: Replaced with the textual message type from the topic.
  - Example value: NDATA

- {{sp_group_id}}

  Sparkplug group identifier parsed from the topic (Edge Node topics).
  - Definition: Replaced with the group ID string.
  - Example value: FactoryA

- {{sp_edge_node_id}}

  Sparkplug Edge Node ID parsed from the topic (Edge Node topics).
  - Definition: Replaced with the edge node identifier string.
  - Example value: Edge01

- {{sp_device_id}}

  Sparkplug Device ID parsed from the topic when present (DDATA/DBIRTH). Empty if not present.
  - Definition: Replaced with the device ID string or an empty string when absent.
  - Example value: Pump_3

- {{sp_metric_level}}

  Additional metric level segments in the topic after the known Sparkplug tokens.
  - Definition: If present, replaced with a single‑quoted joined path of the extra segments (joined by /). If not present, replaced with the literal null (without quotes) so it can be inserted into SQL as NULL.
  - Example value: 'floor1/line2' or null

### Sparkplug Host Application (STATE) placeholders

These placeholders apply when the topic is a Sparkplug HostApplication topic and the payload is Sparkplug JSON for STATE messages.

- {{sp_host_id}}

  Sparkplug host identifier from the topic.
  - Definition: Replaced with the host ID string.
  - Example value: HostAlpha

- {{sp_host_online}}

  Online/offline state from the STATE payload.
  - Definition: Replaced with the string value of the online field from the JSON (e.g., "true" or "false").
  - Example value: true

- {{sp_host_timestamp}}

  Timestamp string from the STATE payload.
  - Definition: Replaced with the string value of the timestamp field from the JSON.
  - Example value: 1693922198123

### Sparkplug metric placeholders

When the topic is a Sparkplug EdgeNode topic and the payload is binary Sparkplug, the engine iterates over every metric in the payload and produces one SQL statement per metric. The following placeholders describe each metric.

- {{sp_metric_name}}

  Metric name if provided in the payload.
  - Definition: Replaced with the metric name string, or an empty string if absent.
  - Example value: temperature

- {{sp_metric_value}}

  The raw metric value encoded as bytes, bound as a parameter.
  - Definition: Replaced with a database placeholder token (e.g., $1 for Postgres, ? for SQLite/MySQL) and the corresponding value is bound based on the underlying Sparkplug type (int/float/bool/string/bytes/dataset/template/extension). If the metric value is missing, an empty byte array is bound.
  - Example usage in SQL: INSERT ... VALUES($1)

Notes
-----
- For Postgres, placeholders are numbered like $1, $2, ...; for SQLite and MySQL they are positional ? markers. The expansion for {{payload}} and {{sp_metric_value}} uses whatever the active driver requires.
- Time placeholders are generated on ingestion, not copied from MQTT message timestamps.
- If you use Sparkplug placeholders on a topic/payload combination that does not match the expected Sparkplug shape, they will resolve to empty strings (or null for {{sp_metric_level}}) and a warning may be logged.

Examples
--------
Example 1 — Use in‑memory SQLite and write subscription output to SQL
```yaml
sql_storage:
  connection_string: "sqlite::memory:"

topics:
  - topic: app/logs
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target:
            type: sql
            insert_statement: |
              INSERT INTO logs(ts, topic, qos, retained, payload)
              VALUES ("{{created_at_iso}}", "{{topic}}", {{qos}}, {{retain}}, {{payload}});
```

Example 2 — Sparkplug metric fan‑out to rows
```yaml
topics:
  - topic: spBv1.0/GroupA/NDATA/Edge01
    payload: { type: sparkplug }
    subscription:
      enabled: true
      outputs:
        - format: { type: sparkplug }
          target:
            type: sql
            insert_statement: |
              INSERT INTO sp_metrics(
                  ts_unix_ms, group_id, edge_node_id, device_id, metric, value
              ) VALUES (
                  {{created_at_millis}}, "{{sp_group_id}}", "{{sp_edge_node_id}}", "{{sp_device_id}}", "{{sp_metric_name}}", {{sp_metric_value}}
              );
```

Example 3 — MySQL (or MariaDB) with JSON logs table
```yaml
sql_storage:
  connection_string: "mysql://user:password@localhost:3306/mydb"

topics:
  - topic: app/logs
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target:
            type: sql
            insert_statement: |
              INSERT INTO logs(ts_unix_ms, topic, qos, retained, payload)
              VALUES ({{created_at_millis}}, "{{topic}}", {{qos}}, {{retain}}, {{payload}});
```

Example 4 — PostgreSQL with JSON logs table
```yaml
sql_storage:
  connection_string: "postgresql://user:password@localhost:5432/mydb"

topics:
  - topic: app/logs
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target:
            type: sql
            insert_statement: |
              INSERT INTO logs(ts_unix_ms, topic, qos, retained, payload)
              VALUES ({{created_at_millis}}, "{{topic}}", {{qos}}, {{retain}}, {{payload}});
```
