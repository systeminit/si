# Providers

This document is an extension of the [Input and Output Sockets](input-output-sockets.md) document.
Please refer to that document prior to reading this one.

## Description

Essentially, `InputSockets` have been renamed to `InternalProviders` and `OutputSockets` have been renamed to
`ExternalProviders`.
While the initial document did not explicitly dictate what domain objects _had_ to be named, this document exists to
clear the air for future readers.

The primary differentiator between each socket was where they could provide data to.
The new "provider" naming scheme highlights this key differentiator while retaining the secondary differentiator: where each socket
consumed data from.
Thus, the following ruleset was derived from the original socket document as well as from early experiences implementing sockets:

- `InternalProviders`
  - can only provide data within its `SchemaVariant`
  - if marked as "implicit" or an "internal consumer" --> can only consume data within its `SchemaVariant`
  - if _not_ marked as "implicit" or an "internal consumer" --> can only consume data from _other_ `SchemaVariants` than its own
- `ExternalProviders`
  - can only provide data to other `SchemaVariants` than its own
  - can only consume data within its `SchemaVariant`

Please note: the exact data shape and ideas may drift from this document, but this is purely meant to give a high level
design overview for historical purposes.