# cbc-subdoc(1) - Interactively Inspect Document Using Subdocument API

## SYNOPSIS

`cbc-subdoc` [_OPTIONS_]

## DESCRIPTION

`cbc-subdoc` runs an interactive shell with commands from subdocument API.

<a name="options"></a>
## OPTIONS

Options may be read either from the command line, or from a configuration file
(see cbcrc(4)):

The following options control workload generation:

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

<a name="commands"></a>
## COMMANDS

### help

Show list of accessible commands with short help.

### LOOKUP COMMANDS

The following options are supported for lookup commands:

* `-?`, `--help`:
   Display built-in help

* `-p`, `--path` _PATH_:
   JSON path in the document. Read more about paths in the [documentation][n1ql-paths].

* `-x`, `--xattr` _PATH_:
   JSON path in the extended attributes.

* `-d`, `--deleted`
   Access XATTR attributes of deleted documents.

### get

`get` [OPTIONS...] KEY...

Retrieve path from the item on the server.

This command requires that at least one key passed to it. If no paths are specified,
it will fetch full document.

### exists

`exists` [OPTIONS...] KEY...

Check if path exists in the item on the server.

This command requires that at least one key and one path are passed to it. Command has
alias `exist`.

### size

`size` [OPTIONS...] KEY...

Count the number of elements in an array or dictionary. The command has alias `get-count`.

This command requires that at least one key and one path passed to it.

### MUTATION COMMANDS

The mutation commands below support the following options:

* `-x`, `--xattr` _PATH=VALUE_:
   Store XATTR path (exentnded attributes).

* `-p`, `--path` _PATH=VALUE_:
   JSON path in the document. Read more about paths in the [documentation][n1ql-paths].

* `-e`, `--expiry` _TIME_:
   Expiration time in seconds. Relative (up to 30 days) or absolute (as Unix timestamp).

* `-i`, `--intermediates`:
   Create intermediate paths [Default=FALSE].

* `-u`, `--upsert`:
   Create document if it does not exist [Default=FALSE].

### replace

`replace` [OPTIONS...] KEY...

Replace the value at the specified path.

### dict-add

`dict-add` [OPTIONS...] KEY...

Add the value at the given path, if the given path does not exist.

### dict-upsert

`dict-upsert` [OPTIONS...] KEY...

Unconditionally set the value at the path.

### array-add-first

`array-add-first` [OPTIONS...] KEY...

Prepend the value(s) to the array. All array operations may accept multiple objects.
See examples below.

### array-add-last

`array-add-last` [OPTIONS...] KEY...

Append the value(s) to the array.

### array-add-unique

`array-add-unique` [OPTIONS...] KEY...

Add the value to the array indicated by the path, if the value is not already in the array.

### array-insert

`array-insert` [OPTIONS...] KEY...

Add the value at the given array index. Path must include index, e.g. `my.list[4]`

### counter

Increment or decrement an existing numeric path. The value must be 64-bit integer.

### set

`set` [OPTIONS...] KEY VALUE

Store document on the server.

This command requires exactly two argument, key and value. Command has alias `upsert`.
If no XATTR specified, the command will add its version to the path `_cbc.version`.

* `-x`, `--xattr` _PATH=VALUE_:
   Store XATTR path (exentnded attributes)

* `-e`, `--expiry` _TIME_:
   Expiration time in seconds. Relative (up to 30 days) or absolute (as Unix timestamp)

### remove

`remove` [OPTIONS...] KEY...

Remove path in the item on the server.

This command requires at least one key. If no paths specified, it will remove whole document.

* `-p`, `--path` _PATH_:
   JSON path in the document. Read more about paths in the [documentation][n1ql-paths].

* `-x`, `--xattr` _PATH_:
   JSON path in the extended attributes.

<a name="examples"></a>
## EXAMPLES

Connect to the server and wait for commands:

    cbc subdoc -u Administrator -P password -U couchbase://192.168.33.101/a_bucket
    subdoc>

Create new document `foo` with empty JSON document:

    subdoc> upsert foo {}
    foo                  CAS=0x14d766f19a720000

Fetch document with virtual XATTR, containing its metadata:

    subdoc> get -x $document foo
    foo                  CAS=0x14d766f19a720000
    0. Size=194, RC=0x00 Success (Not an error)
    {"CAS":"0x14d766f19a720000","vbucket_uuid":"0x0000ef56295d9206",
    "seqno":"0x0000000000000021","exptime":0,"value_bytes":2,
    "datatype":["json","xattr"],"deleted":false,"last_modified":"1501782188"}
    1. Size=2, RC=0x00 Success (Not an error)
    {}

Increment counter with path `site.hits` twice and set document expiration to 5 seconds.
Note that it sends `-i` option to create `site` JSON object automatically:

    subdoc> counter -e 5 -i -p site.hits=1 foo
    foo                  CAS=0x14d76764f3b60000
    0. Size=1, RC=0x00 Success (Not an error)
    1
    subdoc> counter -e 5 -p site.hits=1 foo
    foo                  CAS=0x14d76765ea2b0000
    0. Size=1, RC=0x00 Success (Not an error)
    2
    subdoc> get foo
    foo                  CAS=0x14d76765ea2b0000
    0. Size=19, RC=0x00 Success (Not an error)
    {"site":{"hits":2}}

    ... wait for 5 seconds ...

    subdoc> get foo
    foo                  The key does not exist on the server (0xd)

Add into array at path `ratings` value `5`. Note, that switch `-u` will ask server
to create the document if it does not exist:

    subdoc> array-add-first -u -p ratings=5 foo
    foo                  CAS=0x14d76814fbb00000
    0. Size=0, RC=0x00 Success (Not an error)
    subdoc> get foo
    foo                  CAS=0x14d76814fbb00000
    0. Size=15, RC=0x00 Success (Not an error)
    {"ratings":[5]}

Add several objects at once into `ratings` array:

    subdoc> array-add-last -p ratings=4,6,7 foo
    foo                  CAS=0x14d7687097c50000
    0. Size=0, RC=0x00 Success (Not an error)
    subdoc> get foo
    foo                  CAS=0x14d7687097c50000
    0. Size=21, RC=0x00 Success (Not an error)
    {"ratings":[5,4,6,7]}

Remove rating with index 2 in array (third number):

    subdoc> remove -p ratings[2] foo
    foo                  CAS=0x14d76885efd90000
    0. Size=0, RC=0x00 Success (Not an error)
    subdoc> get foo
    foo                  CAS=0x14d76885efd90000
    0. Size=19, RC=0x00 Success (Not an error)
    {"ratings":[5,4,7]}

Insert new rating instead of removed one:

    subdoc> array-insert -p ratings[2]=10 foo
    foo                  CAS=0x14d768a6daee0000
    0. Size=0, RC=0x00 Success (Not an error)
    subdoc> get foo
    foo                  CAS=0x14d768a6daee0000
    0. Size=22, RC=0x00 Success (Not an error)
    {"ratings":[5,4,10,7]}

Fetch number of the items in the `ratings` array:

    subdoc> size -p ratings foo
    foo                  CAS=0x14d768a6daee0000
    0. Size=1, RC=0x00 Success (Not an error)
    4

Create document with spaces (surround the value with single quotes,
and escape inner single quotes with backslash `\`):

    subdoc> upsert bar '{"text": "hello world"}'
    bar                  CAS=0x14d768bc25270000
    subdoc> get bar
    bar                  CAS=0x14d768bc25270000
    0. Size=23, RC=0x00 Success (Not an error)
    {"text": "hello world"}

## TODO

Port tool to Windows platform. Currently linenoise only supports UNIX-like
systems, but there are unofficial patches for Windows.

## INTERFACE STABILITY

This command's options should be considered uncommitted and are subject to change.

## SEE ALSO

cbc(1), cbcrc(4),
https://developer.couchbase.com/documentation/server/current/developer-guide/sub-doc-api.html

## HISTORY

The `cbc-subdoc` tool was first introduced in libcouchbase 2.7.7.

[n1ql-paths]: https://developer.couchbase.com/documentation/server/current/n1ql/n1ql-intro/queriesandresults.html#story-h2-2
