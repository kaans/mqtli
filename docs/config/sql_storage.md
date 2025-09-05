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
