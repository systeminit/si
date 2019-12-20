# Changelog

## 1.0.0-alpha.3

### Enhancements

 - Added support for automatic and custom Query `client_context_id` option.
 - Updated libcouchbase to `3.0.0-alpha.4`

## 1.0.0-alpha.2

### Fixes

 - Fixed a double import bug that slipped into 1.0.0-alpha.1
 
### Enhancements

 - Added SharedCluster and SharedBucket so it can be used in a multithreaded environment.
 - Added support for N1QL positional and named arguments.
 - Added support for Analytics positional and named arguments.

## 1.0.0-alpha.1

This is the first pre-release of the Couchbase Rust SDK 1.0.0, rendering the previous 0.x releases obsolete.

The API has been completely reworked and it is based on `libcouchbase` 3.0.0-alpha.3. Subsequent releases
will contain proper release notes over the changes.