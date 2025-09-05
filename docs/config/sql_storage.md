---
title: SQL storage
---

SQL storage
===========

This page lists each SQL storage option as its own section.

Connection string
-----------------
Database connection string used by SQL outputs and optional storage features.
- Values: URL‑like string. Supported schemes: sqlite, mariadb, mysql, postgresql.
- Default: unset.
- How to set: YAML: sql_storage.connection_string
- Examples accepted:
  - sqlite::memory:
  - sqlite://        (temporary file)
  - sqlite:data.db   (no authority)
  - sqlite://data.db (with authority)


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
              INSERT INTO logs(ts, payload_json)
              VALUES (CURRENT_TIMESTAMP, ?);
```

Example 2 — SQLite file and mixed outputs
```yaml
sql_storage:
  connection_string: "sqlite://data.db"

topics:
  - topic: sensor/one
    payload: { type: json }
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
          target: { type: console }
        - format: { type: base64 }
          target:
            type: sql
            insert_statement: |
              INSERT INTO samples(ts, payload_b64)
              VALUES (CURRENT_TIMESTAMP, ?);
```
