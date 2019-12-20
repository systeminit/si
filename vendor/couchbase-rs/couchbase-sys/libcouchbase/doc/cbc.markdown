# cbc(1) - Couchbase Client Commandline Utility

## SYNOPSIS

`cbc` _COMMAND_ [_OPTIONS_]<br>
`cbc help`<br>
`cbc version`<br>
`cbc cat` _KEYS_ ... [_OPTIONS_]<br>
`cbc create` _KEY_ _-V VALUE_ [_OPTIONS_]<br>
`cbc create` _KEY_ [_OPTIONS_]<br>
`cbc cp` _FILES_ ... [_OPTIONS_]<br>
`cbc incr` _KEY_ [_OPTIONS_]<br>
`cbc decr` _KEY_ [_OPTIONS_]<br>
`cbc touch` _KEY_ [_OPTIONS_]<br>
`cbc rm` _KEY_ [_OPTIONS_]<br>
`cbc hash` _KEY_ [_OPTIONS_]<br>
`cbc stats` _KEYS_ ... [_OPTIONS_]<br>
`cbc observe` _KEYS_ ... [_OPTIONS_]<br>
`cbc view` _VIEWPATH_ [_OPTIONS_]<br>
`cbc lock` _KEY_ [_OPTIONS_]<br>
`cbc unlock` _KEY_ _CAS_ [_OPTIONS_]<br>
`cbc admin` _-P PASSWORD_ _RESTAPI_ [_OPTIONS_]<br>
`cbc bucket-create` _-P PASSWORD_ _NAME_ [_OPTIONS_]<br>
`cbc bucket-delete` _-P PASSWORD_ _NAME_ [_OPTIONS_]<br>
`cbc bucket-flush` _NAME_ [_OPTIONS_]<br>
`cbc role-list` [_OPTIONS_]<br>
`cbc user-list` [_OPTIONS_]<br>
`cbc user-upsert` _NAME_ [_OPTIONS_]<br>
`cbc user-delete` _NAME_ [_OPTIONS_]<br>
`cbc connstr` _SPEC_<br>
`cbc query` _QUERY_ ... [_OPTIONS_]<br>
`cbc write-config` [_OPTIONS_ ...]<br>
`cbc strerror` _HEX-OR-DECIMAL-CODE_<br>
`cbc ping` [_OPTIONS_ ...]<br>
`cbc watch` [_KEYS_ ...] [_OPTIONS_ ...]<br>
`cbc keygen` [_KEYS_ ...] [_OPTIONS_ ...]<br>


## DESCRIPTION

`cbc` is a utility for communicating with a Couchbase cluster.

`cbc` should be invoked with the command name first and then a series of command
options appropriate for the specific command. `cbc help` will always show the full
list of available commands.

<a name="OPTIONS"></a>
## OPTIONS

Options may be read either from the command line, or from a configuration file
(see cbcrc(4)):

The following common options may be applied to most of the commands

@@common-options.markdown@@

<a name="additional-options"></a>
## ADDITIONAL OPTIONS

The following options may be included in the connection string (via the `-U`
option) as URI-style query params (e.g.
`couchbase://host/bucket?option1=value1&option2=value2`) or as individual
key=value pairs passed to the `-D` switch (e.g. `-Doption1=value1
-Doption2=value`). The `-D` will internally build the connection string,
and is provided as a convenience for options to be easily passed on the
command-line

@@common-additional-options.markdown@@

## COMMANDS

The following commands are supported by `cbc`. Unless otherwise specified, each
command supports all of the options above.

### cat

Write the value of keys to standard output.

This command requires that at least one key may be passed to it, but may accept
multiple keys. The keys should be specified as positional arguments after the
command.

In addition to the options in the [OPTIONS](#OPTIONS) section, the following options are supported:

* `r`, `--replica`=_all|INDEX_:
  Read the value from a replica server. The value for this option can either be
  the string `all` which will cause the client to request the value from each
  replica, or `INDEX` where `INDEX` is a 0-based replica index.

* `e`, `--expiry`=_EXPIRATION_:
  Specify that this operation should be a _get-and-touch_ operation in which the
  key's expiry time is updated along with retrieving the item.


### create

### cp

Create a new item in the cluster, or update the value of an existing item.
By default this command will read the value from standard input unless the
`--value` option is specified.

The `cp` command functions the same, except it operates on a list of files. Each file is
stored in the cluster under the name specified on the command line.

In addition to the options in the [OPTIONS](#OPTIONS) section, the following options are supported:

* `-V`, `--value`=_VALUE_:
  The value to store in the cluster. If omitted, the value is read from standard input. This
  option is valid only for the `create` command.

* `f`, `--flags`=_ITEMFLAGS_:
  A 32 bit unsigned integer to be stored alongside the value. This number is returned
  when the item is retrieved again. Other clients commonly use this value to determine
  the type of item being stored.

* `e`, `--expiry`=_EXPIRATION_:
  The number of time in seconds from now at which the item should expire.

* `M`, `--mode`=_upsert|insert|replace_:
  Specify the storage mode. Mode can be one of `insert` (store item if it does
  not yet exist), `replace` (only store item if key already exists), or
  `upsert` (unconditionally store item)

* `d`, `--durability`=_LEVEL_:
  Specify durability level for mutation operations. Known values are: "none",
  "majority", "majority\_and\_persist\_on\_master", "persist\_to\_majority".

* `p`, `--persist-to`=_NUMNODES_:
  Wait until the item has been persisted to at least `NUMNODES` nodes' disk. If
  `NUMNODES` is 1 then wait until only the master node has persisted the item for
  this key. You may not specify a number greater than the number of nodes actually
  in the cluster.

* `r` `--replicate-to`=_NREPLICAS_:
  Wait until the item has been replicated to at least `NREPLICAS` replica nodes.
  The bucket must be configured with at least one replica, and at least `NREPLICAS`
  replica nodes must be online.


### observe

Retrieve persistence and replication information for items.

This command will print the status of each key to standard error.

See the [OPTIONS](#OPTIONS) for accepted options

### incr

### decr

These commands increment or decrement a _counter_ item in the cluster. A _counter_
is a value stored as an ASCII string which is readable as a number, thus for example
`42`.

These commands will by default refuse to operate on an item which does not exist in
the cluster.

The `incr` and `decr` command differ with how they treat the `--delta` argument. The
`incr` command will treat the value as a _positive_ offset and increment the current
value by the amount specified, whereas the `decr` command will treat the value as a
_negative_ offset and decrement the value by the amount specified.

In addition to [OPTIONS](#OPTIONS), the following options are supported:

* `--initial=_DEFAULT_`:
  Set the initial value for the item if it does not exist in the cluster. The value
  should be an unsigned 64 bit integer. If this option is not specified and the item
  does not exist, the operation will fail. If the item _does_ exist, this option is
  ignored.

* `--delta`=_DELTA_:
  Set the absolute delta by which the value should change. If the command is `incr`
  then the value will be _incremented_ by this amount. If the command is `decr` then
  the value will be _decremented_ by this amount. The default value for this option is
  `1`.

* `-e`, `--expiry`=_EXPIRATION_:
  Set the expiration time for the key, in terms of seconds from now.


### hash

Display mapping information for a key.

This command diplays mapping information about a key. The mapping information
indicates which _vBucket_ the key is mapped to, and which server is currently the
master node for the given _vBucket_.

See the [OPTIONS](#OPTIONS) for accepted options

<a name="lock"></a>
### lock

Lock an item in the cluster.

This will retrieve and lock an item in the cluster, making it inaccessible for
modification until it is unlocked (see [unlock](#unlock)).

In addition to the common options ([OPTIONS](#OPTIONS)), this command accepts the following
options:

* `e`, `--expiry`=_LOCKTIME_:
  Specify the amount of time the lock should be held for. If not specified, it will
  default to the server side maximum of 15 seconds.

<a name="unlock"></a>
### unlock

Unlock a previously locked item.

This command accepts two mandatory positional arguments which are the key and _CAS_ value.
The _CAS_ value should be specified as printed from the [lock][] command (i.e. with the
leading `0x` hexadecimal prefix).

See the [OPTIONS](#OPTIONS) for accepted options


### rm

Remove an item from the cluster.

This command will remove an item from the cluster. If the item does not exist, the
operation will fail.


See the [OPTIONS](#OPTIONS) for accepted options


### stats

Retrieve a list of cluster statistics. If positional arguments are passed to this
command, only the statistics classified under those keys will be retrieved. See the
server documentation for a full list of possible statistics categories.

This command will contact each server in the cluster and retrieve that node's own set
of statistics.

The statistics are printed to standard output in the form of `SERVER STATISTIC VALUE`
where _SERVER_ is the _host:port_ representation of the node from which has provided this
statistic, _STATISTIC_ is the name of the current statistical key, and _VALUE_ is the
value for this statistic.


See the [OPTIONS](#OPTIONS) for accepted options

### watch

Retrieve a list of cluster statistics, select specified sub-keys and aggregate values
across the cluster. Then continuously poll the stats and display the difference with
the previous values. If the list of stat sub-keys not specified, the command will use
`cmd_total_ops`, `cmd_total_gets`, `cmd_total_sets`.

In addition to the options in the [OPTIONS](#OPTIONS) section, the following options are supported:
* `-n`, `--interval`=_VALUE_:
  Update interval in seconds (default `1` second).

### keygen

Output list of keys that equally distribute amongst every vbucket.

In addition to the options in the [OPTIONS](#OPTIONS) section, the following options are supported:
* `--keys-per-vbucket`=_VALUE_:
  Number of keys to generate per vBucket (default `1`).

### write-config

Write the configuration file based on arguments passed.

### strerror

Decode library error code

### version

Display information about the underlying version of _libcouchbase_ to which the
`cbc` binary is linked.

### verbosity

Set the memcached logging versbosity on the cluster. This affects how the memcached
processes write their logs. This command accepts a single positional argument which
is a string describing the verbosity level to be set. The options are `detail`, `debug`
`info`, and `warning`.

### ping

Sends NOOP-like request to every service on each cluster node, and report time it took to response.

* `--details`:
  Provide more details about status of the service.

### view

Execute an HTTP request against the server's view (CAPI) interface.

The request may be one to create a design document, view a design document, or query a
view.

To create a design document, the definition of the document (in JSON) should be piped
to the command on standard input.

This command accepts one positional argument which is the _path_ (relative to the
bucket) to execute. Thus to query the `brewery_beers` view in the `beer` design
document within the `beer-sample` bucket one would do:
    cbc view -U couchbase://localhost/beer-sample _design/beer/_view/brewery_beers

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `-X`, `--method`=_GET|PUT|POST|DELETE_:
  Specify the HTTP method to use for the specific request. The default method is `GET`
  to query a view. To delete an existing design document, specify `DELETE`, and to
  create a new design document, specify `PUT`.

### query

Execute a N1QL Query. The cluster must have at least one query node enabled.

The query itself is passed as a positional argument on the commandline. The
query may contain named placeholders (in the format of `$param`), whose values
may be supplied later on using the `--qarg='$param=value'` syntax.

It is recommended to place the statement in single quotes to avoid shell
expansion.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options
are recognized:

* `-Q`, `--qopt`=_SETTING=VALUE_:
  Specify additional options controlling the execution of the query. This can
  be used for example, to set the `scan_consistency` of the query.

* `-A`, `--qarg`=_PLACEHOLDER=VALUE_:
  Supply values for placeholders found in the query string. The placeholders
  must evaluate to valid JSON values.

* `--prepare`:
  Prepare query before issuing. Default is FALSE.

* `--analytics`:
  Perform query to analytics service. Default is FALSE.

### admin

Execute an administrative request against the management REST API.
Note that in order to perform an administrative API you will need to provide
_administrative_ credentials to `cbc admin`. This means the username and password
used to log into the administration console.

This command accepts a single positional argument which is the REST API endpoint
(i.e. HTTP path) to execute.

If the request requires a _body_, it should be supplied via standard input

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `-X`, `--method`=_GET|PUT|POST|DELETE_:
  Specify the HTTP method to use for the specific request. The default method is
  `GET`.

### bucket-create

Create a bucket in the cluster.

This command will create a bucket with the name specified as the lone positional
argument on the command line.

As this is an administrative command, the `--username` and `--password` options should
be supplied administrative credentials.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `--bucket-type`=_couchbase|memcached_:
  Specify the type of bucket to create. A _couchbase_ bucket has persistence to disk and
  replication. A _memached_ bucket is in-memory only and does not replicate.

* `--ram-quota`=_QUOTA_:
  Specify the maximum amount of memory the bucket should occupy (per node) in megabytes.
  If not specified, the default is _512_.

* `--bucket-password`=_PASSWORD_:
  Specify the password to secure this bucket. If passed, this password will be required
  by all clients attempting to connect to the bucket. If ommitted, this bucket may be
  accessible to everyone for both read and write access.

* `--num-replicas`=_REPLICAS_:
  Specify the amount of replicas the bucket should have. This will set the number of nodes
  each item will be replicated to. If not specified the default is _1_.


### bucket-flush

This command will flush the bucket with the name specified as the lone positional
argument on the command line.

This command does not require administrative level credentials, however it does
require that _flush_ be enabled for the bucket.

See the [OPTIONS](#OPTIONS) for accepted options

### role-list

List accessible RBAC user roles in the cluster.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `-r`, `--raw`:
  Print unformatted server response in JSON form.

### user-list

List users in the cluster.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `-r`, `--raw`:
  Print unformatted server response in JSON form.

### user-upsert

Create or update a user in the cluster. Takes user ID as an argument.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `--domain`=_local|remote_:
  The domain, where user account defined. If not specified, the default is _local_.

* `--full-name`=_FULL_NAME_:
  The user's fullname. If not specified, the default is empty string.

* `--role`=_ROLE_:
  The role associated with user (can be specified multiple times if needed).

* `--user-password`=_PASSWORD_:
  The password for the user.

### user-delete

Delete a user in the cluster. Takes user ID as an argument.

In addition to the [OPTIONS](#OPTIONS) specified above, the following options are recognized:

* `--domain`=_local|remote_:
  The domain, where user account defined. If not specified, the default is _local_.


### connstr

This command will parse a connection string into its constituent parts and
display them on the screen. The command takes a single positional argument
which is the string to parse.

## EXAMPLES


### CONNECTION EXAMPLES

The following shows how to connect to various types of buckets. These examples
all show how to retrieve the key `key`. See
[OPERATION EXAMPLES](#OPERATION EXAMPLES) for more information on specific
sub-commands.

Connect to a bucket (`a_bucket`) on a cluster on a remote host (for servers version 5.x+).
It uses account 'myname' and asks password interactively:

    cbc cat key -U couchbase://192.168.33.101/a_bucket -u myname -P-

Run against a password-less bucket (`a_bucket`) on a cluster on a remote host (for servers older than 5.x):

    cbc cat key -U couchbase://192.168.33.101/a_bucket

Connect to an SSL cluster at `secure.net`. The certificate for the cluster is
stored locally at `/home/couchbase/couchbase_cert.pem`:

    cbc cat key -U couchbases://secure.net/topsecret_bucket?certpath=/home/couchbase/couchbase_cert.pem

Connect to an SSL cluster at `secure.net`, ignoring certificate verification.
This is insecure but handy for testing:

    cbc cat key -U couchbases://secure.net/topsecret_bucket?ssl=no_verify

Connect to a password protected bucket (`protected`) on a remote host (for servers older than 5.x):

    cbc cat key -U couchbase://remote.host.net/protected -P-
    Bucket password:

Connect to a password protected bucket (for servers older than 5.x), specifying the password on the
command line (INSECURE, but useful for testing dummy environments)

    cbc cat key -U couchbase://remote.host.net/protected -P t0ps3cr3t

Connect to a bucket running on a cluster with a custom REST API port

    cbc cat key -U http://localhost:9000/default

Connec to bucket running on a cluster with a custom memcached port

    cbc cat key -U couchbase://localhost:12000/default

Connect to a *memcached* (http://memcached.org)
cluster using the binary protocol. A vanilla memcached cluster is not the same
as a memcached bucket residing within a couchbase cluster (use the normal
`couchbase://` scheme for that):

    cbc cat key -U memcached://host1,host2,host3,host4


Connect to a cluster using the HTTP protocol for bootstrap, and set the
operation timeout to 5 seconds

    cbc cat key -U couchbase://host/bucket -Dbootstrap_on=http -Doperation_timeout=5


### OPERATION EXAMPLES

Store a file to the cluster:

    $ cbc cp mystuff.txt
    mystuff.txt         Stored. CAS=0xe15dbe22efc1e00


Retrieve persistence/replication information about an item (note that _Status_
is a set of bits):

    $ cbc observe mystuff.txt
    mystuff              [Master] Status=0x80, CAS=0x0


Display mapping information about keys:

    $cbc hash foo bar baz
    foo: [vBucket=115, Index=3] Server: cbnode3:11210, CouchAPI: http://cbnode3:8092/default
    bar: [vBucket=767, Index=0] Server: cbnode1:11210, CouchAPI: http://cbnode1:8092/default
    baz: [vBucket=36, Index=2] Server: cbnode2:11210, CouchAPI: http://cbnode2:8092/default

Create a bucket:

    $ cbc bucket-create --bucket-type=memcached --ram-quota=100 --password=letmein -u Administrator -P 123456 mybucket
    Requesting /pools/default/buckets
    202
      Cache-Control: no-cache
      Content-Length: 0
      Date: Sun, 22 Jun 2014 22:43:56 GMT
      Location: /pools/default/buckets/mybucket
      Pragma: no-cache
      Server: Couchbase Server

Flush a bucket:

    $ cbc bucket-flush default
    Requesting /pools/default/buckets/default/controller/doFlush


    200
      Cache-Control: no-cache
      Content-Length: 0
      Date: Sun, 22 Jun 2014 22:53:44 GMT
      Pragma: no-cache
      Server: Couchbase Server

Delete a bucket:

    $ cbc bucket-delete mybucket -P123456
    Requesting /pools/default/buckets/mybucket
    200
      Cache-Control: no-cache
      Content-Length: 0
      Date: Sun, 22 Jun 2014 22:55:58 GMT
      Pragma: no-cache
      Server: Couchbase Server

Use `cbc stats` to determine the minimum and maximum timeouts for a lock operation:

    $ cbc stats | grep ep_getl
    localhost:11210 ep_getl_default_timeout 15
    localhost:11210 ep_getl_max_timeout 30


Create a design document:

    $ echo '{"views":{"all":{"map":"function(doc,meta){emit(meta.id,null)}"}}}' | cbc view -X PUT _design/blog
    201
      Cache-Control: must-revalidate
      Content-Length: 32
      Content-Type: application/json
      Date: Sun, 22 Jun 2014 23:03:40 GMT
      Location: http://localhost:8092/default/_design/blog
      Server: MochiWeb/1.0 (Any of you quaids got a smint?)
    {"ok":true,"id":"_design/blog"}


Query a view:

    $ cbc view _design/blog/_view/all?limit=5
    200
      Cache-Control: must-revalidate
      Content-Type: application/json
      Date: Sun, 22 Jun 2014 23:06:09 GMT
      Server: MochiWeb/1.0 (Any of you quaids got a smint?)
      Transfer-Encoding: chunked
    {"total_rows":20,"rows":[
    {"id":"bin","key":"bin","value":null},
    {"id":"check-all-libev-unit-tests.log","key":"check-all-libev-unit-tests.log","value":null},
    {"id":"check-all-libevent-unit-tests.log","key":"check-all-libevent-unit-tests.log","value":null},
    {"id":"check-all-select-unit-tests.log","key":"check-all-select-unit-tests.log","value":null},
    {"id":"cmake_install.cmake","key":"cmake_install.cmake","value":null}
    ]
    }


Issue a N1QL query:

    $ cbc query 'SELECT * FROM `travel-sample` WHERE type="airport" AND city=$city' -Qscan_consistency=request_plus -A'$city=\"Reno\"'


Ping cluster services:

    $ cbc ping --details  -Ucouchbase://192.168.1.101
    {
       "version" : 1,
       "config_rev" : 54,
       "id" : "0x1d67af0",
       "sdk" : "libcouchbase/2.8.4",
       "services" : {
          "fts" : [
             {
                "id" : "0x1d75e90",
                "latency_us" : 1500,
                "local" : "192.168.1.12:35232",
                "remote" : "192.168.1.101:8094",
                "status" : "ok"
             },
             {
                "id" : "0x1da6800",
                "latency_us" : 2301,
                "local" : "192.168.1.12:40344",
                "remote" : "192.168.1.103:8094",
                "status" : "ok"
             },
             {
                "id" : "0x1da3270",
                "latency_us" : 2820,
                "local" : "192.168.1.12:42730",
                "remote" : "192.168.1.102:8094",
                "status" : "ok"
             },
             {
                "details" : "LCB_ENETUNREACH (0x31): The remote host was unreachable - is your network OK?",
                "latency_us" : 3071733,
                "remote" : "192.168.1.104:8094",
                "status" : "error"
             }
          ],
          "kv" : [
             {
                "id" : "0x1d6bde0",
                "latency_us" : 3700,
                "local" : "192.168.1.12:42006",
                "remote" : "192.168.1.101:11210",
                "scope" : "default",
                "status" : "ok"
             },
             {
                "id" : "0x1dadcf0",
                "latency_us" : 5509,
                "local" : "192.168.1.12:39936",
                "remote" : "192.168.1.103:11210",
                "scope" : "default",
                "status" : "ok"
             },
             {
                "id" : "0x1dac500",
                "latency_us" : 5594,
                "local" : "192.168.1.12:33868",
                "remote" : "192.168.1.102:11210",
                "scope" : "default",
                "status" : "ok"
             },
             {
                "latency_us" : 2501688,
                "remote" : "192.168.1.104:11210",
                "scope" : "default",
                "status" : "timeout"
             }
          ],
          "n1ql" : [
             {
                "id" : "0x1d7f280",
                "latency_us" : 3235,
                "local" : "192.168.1.12:54210",
                "remote" : "192.168.1.101:8093",
                "status" : "ok"
             },
             {
                "id" : "0x1d76f20",
                "latency_us" : 4625,
                "local" : "192.168.1.12:58454",
                "remote" : "192.168.1.102:8093",
                "status" : "ok"
             },
             {
                "id" : "0x1da44b0",
                "latency_us" : 4477,
                "local" : "192.168.1.12:36678",
                "remote" : "192.168.1.103:8093",
                "status" : "ok"
             },
             {
                "details" : "LCB_ENETUNREACH (0x31): The remote host was unreachable - is your network OK?",
                "latency_us" : 3071843,
                "remote" : "192.168.1.104:8093",
                "status" : "error"
             }
          ],
          "views" : [
             {
                "id" : "0x1da55c0",
                "latency_us" : 1762,
                "local" : "192.168.1.12:52166",
                "remote" : "192.168.1.103:8092",
                "status" : "ok"
             },
             {
                "id" : "0x1da20d0",
                "latency_us" : 2016,
                "local" : "192.168.1.12:59420",
                "remote" : "192.168.1.102:8092",
                "status" : "ok"
             },
             {
                "id" : "0x1d6a740",
                "latency_us" : 2567,
                "local" : "192.168.1.12:38614",
                "remote" : "192.168.1.101:8092",
                "status" : "ok"
             },
             {
                "details" : "LCB_ENETUNREACH (0x31): The remote host was unreachable - is your network OK?",
                "latency_us" : 3071798,
                "remote" : "192.168.1.104:8092",
                "status" : "error"
             }
          ]
       }
    }


## FILES

cbc(1) and cbc-pillowfight(1) may also read options from cbcrc(4). The default
path for `cbcrc` is `$HOME/.cbcrc`, but may be overridden by setting the
`CBC_CONFIG` evironment variable to an alternate path.

## BUGS

The options in this utility and their behavior are subject to change. This script
should be used for experiemntation only and not inside production scripts.


## SEE ALSO

cbc-pillowfight(1), cbcrc(4)


## History

The cbc command first appeared in version 0.3.0 of the library. It was significantly
rewritten in version 2.4.0
