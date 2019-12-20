# Internal source code structure

This directory contains the source code for libcouchbase. Here is a brief
listing of the various subcomponents and what they do:

* `internal.h` contains the top level internal declarations.

* `config_static.h` contains statically inferred (via macros) platform information.

* `instance.c` contains functions related to creating and destroying the actual
  `lcb_t` handle. It also contains some convenience functions.

* `bootstrap.{c,h}` contains the top-level logic for _retrieving_ the cluster
  configuration

* `newconfig.c` contains the logic for _applying_ a new configuration

* `bucketconfig/*` is a directory which contains the low level transport logic used
  to retrieve a new configuration

* `callbacks.c` contains the functions used to implement the library's `set_callback`
  accessor functions.

* `operations/*` is a directory which contains the top level entry points for memcached
  requests

* `handler.c` contains the response handlers for memcached.

* `getconfig.c` contains the implementation for requesting a config from an existing
  server.

* `connspec.{c,h}` contains the connection string parsing logic

* `cntl.c` contains the handlers for the `lcb_cntl()` family of functions

* `dump.c` contains the handlers for the `lcb_dump()` function

* `gethrtime.c` contains platform-dependent implementations of a nanosecond timer

* `hashset.{c,h}` contains the implementation for a set (unique collection of pointers)

* `list.{c,h}` contains the implementation for a double-linked list

* `sllist.h, sllist-inl.h` contain the implementation for a single-linked list

* `logging.{c,h}` contains the implementation for the library's logging mechanism

* `hostlist.{c,h}` defines a list of hosts, with features for de-duping and converting
  to other structures

* `nodeinfo.c` contains the implementation for the `lcb_get_node()` function.

* `packetutils.{c,h}` contains utilities and macros for handling partial memcached
  response packets

* `wait.c` contains the implementation of `lcb_wait()`

* `timings.c` contains the implementation of `lcb_get_timings()`

* `trace.h` contains macros for DTrace functionality

* `utilities.c` contains cross-platform utilities (such as temporary directory,
  getting environment variables)

* `iofactory.c` contains the plugin intialization/loading functionality for I/O
  plugins

* `retrychk.c` contains logic which determines if a rety is necessary under certain
  conditions

* `retryq.{c,h}` contains an internal retried operations, and are placed there if they
  are eligible for retries.

* `aspend.h` contains definitions for pending operations which are meant to block
  calls to `lcb_wait()` (implementation in instance.c)

* `lcbio/*` is a directory which contains the cross platform/cross model IO
  implementation. Most I/O is done in this subdirectory

* `http/*` contains the API implementation for user-level HTTP requests

* `lcbht/*` contains an HTTP response parsing API/implementation

* `mc/*` contains the memcached/Couchbase structure and packet/buffer allocation
  and scheduling logic.

* `mcserver/*` contains the operation/failure/IO logic for memcached connections

* `vbucket/*` contains the raw vBucket config parsing and hashing/mapping implementation
  (formerly known as "libvbucket")

* `rdb/*` contains an extensible pooled read buffer implementation

* `netbuf/*` contains an extensible high performance output buffer implementation

* `rigbuffer.{c,h}` contains a circular buffer implementation.

* `ssl/*` contains the OpenSSL interfacing routines

* `strcodecs/*` contains utility functions to encode/decode strings to/from
  various formats.
