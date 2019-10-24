* `operation_timeout=SECONDS`:
  Specify the operation timeout in seconds. This is the time the client will
  wait for an operation to complete before timing it out. The default is `2.5`
* `config_cache=PATH`:
  Enables the client to make use of a file based configuration cache rather
  than connecting for the bootstrap operation. If the file does not exist, the
  client will first connect to the cluster and then cache the bootstrap information
  in the file.
* `truststorepath=PATH`:
  The path to the server's SSL certificate. This is typically required for SSL
  connectivity unless the certificate has already been added to the OpenSSL
  installation on the system (only applicable with `couchbases://` scheme)
* `certpath=PATH`:
  The path to the server's SSL certificate. This is typically required for SSL
  connectivity unless the certificate has already been added to the OpenSSL
  installation on the system (only applicable with `couchbases://` scheme).
  This also should contain client certificate when certificate authentication
  used, and in this case other public certificates could be extracted into
  `truststorepath` chain.
* `keypath=PATH`:
  The path to the client SSL private key. This is typically required for SSL
  client certificate authentication. The certificate itself have to go first
  in chain specified by `certpath` (only applicable with `couchbases://` scheme)
* `ipv6=allow`:
  Enable IPv6.
* `ssl=no_verify`:
  Temporarily disable certificate verification for SSL (only applicable with
  `couchbases://` scheme). This should only be used for quickly debugging SSL
  functionality.
* `sasl_mech_force=MECHANISM`:
  Force a specific _SASL_ mechanism to be used when performing the initial
  connection. This should only need to be modified for debugging purposes.
  The currently supported mechanisms are `PLAIN` and `CRAM-MD5`
* `bootstrap_on=<both,http,cccp>`:
  Specify the bootstrap protocol the client should use when attempting to connect
  to the cluster. Options are: `cccp`: Bootstrap using the Memcached protocol
  (supported on clusters 2.5 and greater); `http`: Bootstrap using the HTTP REST
  protocol (supported on any cluster version); and `both`: First attempt bootstrap
  over the Memcached protocol, and use the HTTP protocol if Memcached bootstrap fails.
  The default is `both`

* `enable_tracing=true/false`: Activate/deactivate end-to-end tracing.

* `tracing_orphaned_queue_flush_interval=SECONDS`: Flush interval for orphaned
  spans queue in default tracer. This is the time the tracer will wait between
  repeated attempts to flush most recent orphaned spans.
  Default value is 10 seconds.

* `tracing_orphaned_queue_size=NUMBER`: Size of orphaned spans queue in default
  tracer. Queues in default tracer has fixed size, and it will remove
  information about older spans, when the limit will be reached before flushing
  time.
  Default value is 128.

* `tracing_threshold_queue_flush_interval=SECONDS`: Flush interval for spans
  with total time over threshold in default tracer. This is the time the tracer
  will wait between repeated attempts to flush threshold queue.
  Default value is 10 seconds.

* `tracing_threshold_queue_size=NUMBER`: Size of threshold queue in default
  tracer. Queues in default tracer has fixed size, and it will remove
  information about older spans, when the limit will be reached before flushing
  time.
  Default value is 128.

* `tracing_threshold_kv=SECONDS`: Minimum time for the tracing span of KV
  service to be considered by threshold tracer.
  Default value is 0.5 seconds.

* `tracing_threshold_n1ql=SECONDS`: Minimum time for the tracing span of N1QL
  service to be considered by threshold tracer.
  Default value is 1 second.

* `tracing_threshold_view=SECONDS`: Minimum time for the tracing span of VIEW
  service to be considered by threshold tracer.
  Default value is 1 second.

* `tracing_threshold_fts=SECONDS`: Minimum time for the tracing span of FTS
  service to be considered by threshold tracer.
  Default value is 1 second.

* `tracing_threshold_analytics=SECONDS`: Minimum time for the tracing span of
  ANALYTICS service to be considered by threshold tracer.
  Default value is 1 second.
