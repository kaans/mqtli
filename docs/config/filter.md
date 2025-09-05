---
title: Filters
---

Filters
=======

This page lists each filter option as its own section.

Where filters apply
-------------------
- On subscription: transform received messages before output.
- Before publish: transform the input message before it’s sent.

Automatic conversion
--------------------
- Filters will try to convert input to a required intermediate type as needed (e.g., to JSON or Text). If conversion is impossible or fails, processing stops with an error.

Filter: extract_json
--------------------
Extract values from JSON via JSONPath.
- Input: JSON
- Output: JSON
- Attributes:
  - jsonpath: string (e.g., $.data.temp)

Filter: to_upper
----------------
Convert ASCII letters to upper case.
- Input: Text
- Output: Text

Filter: to_lower
----------------
Convert ASCII letters to lower case.
- Input: Text
- Output: Text

Filter: prepend
---------------
Prepend text to a Text message.
- Input: Text
- Output: Text
- Attributes:
  - content: string

Filter: append
--------------
Append text to a Text message.
- Input: Text
- Output: Text
- Attributes:
  - content: string

Filter: to_text
---------------
Convert any payload to Text.
- Input: Any
- Output: Text

Filter: to_json
---------------
Convert any payload to JSON (when possible).
- Input: Any
- Output: JSON

YAML example
------------
```yaml
topics:
  - topic: mqtli/json
    subscription:
      enabled: true
      outputs:
        - format: { type: json }
      filters:
        - type: extract_json
          jsonpath: $.name
    payload:
      type: json
```
