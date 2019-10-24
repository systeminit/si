# cbc-n1qlback(1) - Stress Test for Couchbase Query (N1QL)

## SYNOPSIS

`cbc-n1qlback` -f QUERYFILE [_OPTIONS_]

## DESCRIPTION

`cbc-n1qlback` creates a specified number of threads each executing a set
of user defined queries.

`cbc-n1qlback` requires that it be passed the path to a file containing
the queries to execute; one per line. The query should be in the format of
the actual HTTP POST body (in JSON format) to be sent to the server.
For simple queries, only the `statement` field needs to be set:

    {"statement":"SELECT country FROM `travel-sample`"}
    {"statement":"SELECT country, COUNT(country) FROM `travel-sample` GROUP BY country"}


For more complex queries (for example, placeholders, custom options), you may
refer to the N1QL REST API reference.

`n1qlback` requires that any resources (data items, indexes) are already
defined.

## OPTIONS

The following options control workload generation:

* `-f` `--queryfile`=_PATH_:
  Path to a file containing the query bodies to execute in JSON format, one
  query per line. See above for the format.

* `-t`, `--num-threads`=_NTHREADS_:
  Set the number of threads (and thus the number of client instances) to run
  concurrently. Each thread is assigned its own client object.


The following options control how `cbc-n1qlback` connects to the cluster

@@common-options.markdown@@

* `-e`, `--error-log`=_PATH_:
  Path to a file, where the command will write failed queries along with error details.
  Use this option to figure out why `ERRORS` metric is not zero.


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

## EXAMPLES

The following will create a file with 3 queries and 5 threads alternating
between them. It also creates indexes on the `travel-sample` bucket

    cbc n1ql -U couchbase://192.168.72.101/a_bucket 'CREATE INDEX ix_name ON `travel-sample`(name)'
    cbc n1ql -U couchbase://192.168.72.101/a_bucket 'CREATE INDEX ix_country ON `travel-sample`(country)'

    cat queries.txt <<EOF
    {"statement":"SELECT country FROM `travel-sample` WHERE `travel-sample`.country = \"United States\""}
    {"statement":"SELECT name FROM `travel-sample` LIMIT 10"}
    {"statement":"SELECT country, COUNT(country) FROM `travel-sample` GROUP BY country"}
    EOF

    cbc-n1qlback -U couchbase://192.168.72.101/a_bucket -t 5 -f queries.txt

## BUGS

This command's options are subject to change.

## SEE ALSO

cbc(1), cbc-pillowfight(1), cbcrc(4)

## HISTORY

The `cbc-n1qlback` tool was first introduced in libcouchbase 2.4.10
