* `-U`, `--spec`=_SPEC_:
  A string describing the cluster to connect to. The string is in a URI-like syntax,
  and may also contain other options. See the [EXAMPLES](#examples) section for information.
  Typically such a URI will look like `couchbase://host1,host2,host3/bucket`.

  The default for this option is `couchbase://localhost/default`

* `-u`, `--username`=_USERNAME_:
  Specify the _username_ for the bucket. Since Couchbase 5.x this is mandatory
  switch, and it must specify the name of the user exisiting on cluster (read
  more at "Security/Authorization" section of the server manual). For older servers
  this field should be either left empty or set to the name of the bucket itself.

* `-P`, `--password`=_PASSWORD_:
* `-P -`, `--password=-`:
  Specify the password for the bucket. As for servers before 5.x this was only
  needed if the bucket is protected with a password. For cluster version after 5.x,
  the password is mandatory, and should match the selected account (read more at
  "Security/Authorization" section of the server manual).

  Specifying the `-` as the password indicates that the program should prompt for the
  password. You may also specify the password on the commandline, directly,
  but is insecure as command line arguments are visible via commands such as `ps`.

* `-T`, `--timings`:
  Dump command timings at the end of execution. This will display a histogram
  showing the latencies for the commands executed.

* `-v`, `--verbose`:
  Specify more information to standard error about what the client is doing. You may
  specify this option multiple times for increased output detail.

* `-D`, `--cparam`=OPTION=VALUE:
  Provide additional client options. Acceptable options can also be placed
  in the connection string, however this option is provided as a convenience.
  This option may be specified multiple times, each time specifying a key=value
  pair (for example, `-Doperation_timeout=10 -Dconfig_cache=/foo/bar/baz`).
  See [ADDITIONAL OPTIONS](#additional-options) for more information

* `-y`, `--compress`:
  Enable compressing of documents. When the library is compiled with compression
  support, this option will enable Snappy compression for outgoing data.
  Incoming compressed data handled automatically regardless of this option.
  Note, that because the compression support have to be negotiated with the
  server, first packets might be sent uncompressed even when this switch
  was specified. This is because the library might queue data commands before
  socket connection has been established, and the library will negotiate
  compression feature. If it is known that all server support compression
  repeating the switch (like `-yy`) will force compression for all outgoing
  mutations, even scheduled before establishing connection.

* `--truststorepath`=_PATH_:
  The path to the server's SSL certificate. This is typically required for SSL
  connectivity unless the certificate has already been added to the OpenSSL
  installation on the system (only applicable with `couchbases://` scheme)

* `--certpath`=_PATH_:
  The path to the server's SSL certificate. This is typically required for SSL
  connectivity unless the certificate has already been added to the OpenSSL
  installation on the system (only applicable with `couchbases://` scheme).
  This also should contain client certificate when certificate authentication
  used, and in this case other public certificates could be extracted into
  `truststorepath` chain.

* `--keypath`=_PATH_:
  The path to the client SSL private key. This is typically required for SSL
  client certificate authentication. The certificate itself have to go first
  in chain specified by `certpath` (only applicable with `couchbases://` scheme)

* `--dump`:
  Dump verbose internal state after operations are done.
