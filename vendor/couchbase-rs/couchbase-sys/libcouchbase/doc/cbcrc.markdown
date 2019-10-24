# cbcrc(4) - Configuration file for Couchbase command line tools

## SYNOPSIS

`~/.cbcrc`

## DESCRIPTION

cbcrc is an optional configuration file used to provide default values for the
cbc(1) and cbc-pillowfight(1) utilities. It should be placed in

Each entry in the cbcrc file is a line with a key-value pair in the following
form:

    # optional comments
    key=value

The keys may be specified in random order, and if you specify the same
key multiple times the last value "wins". The following keys exists:

* `connstr`:
  This is the URI-like string used to connect to the cluster. Its format
  consists of a _scheme_ followed by a list of hosts, an optional
  bucket for the path and an optional `?` followed by key-value options.

  Using custom REST API ports

    http://localhost:9000,localhost:9001,localhost:9002

  Using custom memcached ports:

    couchbase://localhost:9100,localhost:9200,localhost:9300

* `user`:
    This is the user name used during authentication to your bucket

* `password`:
    This is the password used during authentication to your bucket

* `timeout`:
    The timeout value to use for the operations.

## NOTES

* You can generate such a file from the cbc(1) itself using the `write-config`
  subcommand

* Most other options can be specified within the connection string

## SEE ALSO

cbc(1), cbc-pillowfight(1)
