# Couchbase C Client

This is the C client library for [Couchbase](http://www.couchbase.com)
It communicates with the cluster and speaks the relevant protocols
necessary to connect to the cluster and execute data operations.

## Features

* Can function as either a synchronous or asynchronous library
* Callback Oriented
* Can integrate with most other asynchronous environments. You can write your
  code to integrate it into your environment. Currently support exists for
    * [libuv](http://github.com/joyent/libuv) (Windows and POSIX)
    * [libev](http://software.schmorp.de/pkg/libev.html) (POSIX)
    * [libevent](http://libevent.org/) (POSIX)
    * `select` (Windows and POSIX)
    * IOCP (Windows Only)
* Support for operation batching
* Cross Platform - Tested on Linux, OS X, and Windows.

## Building

Before you build from this repository, please check the
[installation page](https://developer.couchbase.com/server/other-products/release-notes-archives/c-sdk)
to see if there is a binary or release tarball available for your needs. Since the code here is
not part of an official release it has therefore not gone through our
release testing process.

### Dependencies

By default the library depends on:

* _libevent_ (or _libev_) for the primary I/O backend.
* _openssl_ for SSL transport.
* _CMake_ version 2.8.9 or greater (for building)

On Unix-like systems these dependencies are checked for by default
while on Windows they are not checked by default.

On Unix, the build system will expect to have _libevent_ or _libev_ installed,
unless building plugins is explicitly disabled (see further).

### Building on Unix-like systems

Provided is a convenience script called `cmake/configure`. It is a Perl
script and functions like a normal `autotools` script.

```shell
$ git clone git://github.com/couchbase/libcouchbase.git
$ cd libcouchbase && mkdir build && cd build
$ ../cmake/configure
$ make
$ ctest
```

### Building on Windows

Assuming `git` and Visual Studio 2010 are installed, from a `CMD` shell, do:

```
C:\> git clone git://github.com/couchbase/libcouchbase.git
C:\> mkdir lcb-build
C:\> cd lcb-build
C:\> cmake -G "Visual Studio 10" ..\libcouchbase
C:\> cmake --build .
```

This will generate and build a Visual Studio `.sln` file.

Windows builds are known to work on Visual Studio versions 2008, 2010 and
2012.

If you wish to link against OpenSSL, you should set the value of
`OPENSSL_ROOT_DIR` to the location of the installation path, as described
[here](https://github.com/Kitware/CMake/blob/master/Modules/FindOpenSSL.cmake)

## Running tests

To run tests, you can use either ctest directly or generated build targets.
For Unix-like:

```shell
make test
```

For windows:

```batchfile
cmake --build . --target alltests
ctest -C debug
```

By default tests will use [CouchbaseMock](https://github.com/couchbase/CouchbaseMock) project to simulate the Couchbase
Cluster. It allows to cover more different failure scenarios, although does not implement all kinds of APIs provided
by real server.

If you need to test against real server, you have to provide comma-separated configuration in `LCB_TEST_CLUSTER_CONF`
environment variable. For example, the following command will run tests against local cluster and bucket `default` using
administrator credentials:

```shell
export LCB_TEST_CLUSTER_CONF=couchbase://localhost,default,Administrator,password
make test
```
Note that specifying username will automatically switch to RBAC mode, which supported by Couchbase Server 5.0+. For old
servers the spec will look like `couchbase://localhost,default` or `couchbase://localhost,protected,,secret`.

Also tests expecting `beer-sample` bucket loaded. It comes with the server. Look at "Sample buckets" section of Admin
Console.

## Bugs, Support, Issues
You may report issues in the library in our issue tracked at
<https://issues.couchbase.com>. Sign up for an account and file an issue
against the _Couchbase C Client Library_ project.

The developers of the library hang out in IRC on `#libcouchbase` on
irc.freenode.net.


## Examples

* The `examples` directory
* Official client libraries using libcouchbase
    * [node.js](http://github.com/couchbase/couchnode)
    * [Python](http://github.com/couchbase/couchbase-python-client)
    * [PHP](http://github.com/couchbase/php-couchbase)
* Community projects using libcouchbase
    * [C++11 wrapper](https://github.com/couchbaselabs/libcouchbase-cxx)
    * [cberl - Couchbase NIF](https://github.com/wcummings/cberl)
    * [Perl client](https://github.com/mnunberg/perl-Couchbase-Client)
    * [Ruby](http://github.com/couchbase/couchbase-ruby-client) (uses the old < 2.6 API)

## Documentation

Documentation is available in guide format (introducing the basic concepts of
Couchbase and the library). It is recommended for first-time users, and can
be accessed at our [Documentation Site](https://developer.couchbase.com/documentation/server/current/sdk/c/start-using-sdk.html).

API documentation is also available and is generated from the library's headers.
It may contain references to more advanced features not found in the guide.

API documentation may be generated by running `doxygen` within the source root
directory. When this is done, you should have a `doc/html/index.html` page which
may be viewed.

Doxygen may be downloaded from the
[doxygen downloads page](http://www.stack.nl/~dimitri/doxygen/download.html). Note
however that most Linux distributions as well as Homebrew contain Doxygen in their
repositories.

```
$ doxygen
$ xdg-open doc/html/index.html # Linux
$ open doc/html/index.html # OS X
```

You may also generate documentation using the `doc/Makefile` which dynamically
inserts version information

```
$ make -f doc/Makefile public # for public documentation
$ make -f doc/Makefile internal # for internal documentation
```

The generated documentation will be in the `doc/public/html` directory for
public documentation, and in the `doc/internal/html` directory for internal
documentation.

## Contributors

The following people contributed to libcouchbase (in alphabetic order)
(last updated Nov. 27 2014)

* Brett Lawson <brett19@gmail.com>
* Dave Rigby <daver@couchbase.com>
* Jan Lehnardt <jan@apache.org>
* Mark Nunberg <mnunberg@haskalah.org>
* Matt Ingenthron <ingenthr@cep.net>
* Patrick Varley <patrick@couchbase.com>
* Paul Farag <pfarag@neuraliq.com>
* Pierre Joye <pierre.php@gmail.com>
* Sebastian <sebastian@chango.com>
* Sergey Avseyev <sergey.avseyev@gmail.com>
* Subhashni Balakrishnan <b.subhashni@gmail.com>
* Sundar Sridharan <sundar.sridharan@gmail.com>
* Trond Norbye <trond.norbye@gmail.com>
* Volker Mische <vmx@couchbase.com>
* William Bowers <wbowers@neuraliq.com>
* Yura Sokolov <funny.falcon@gmail.com>
* Yury Alioshinov <haster2010@gmail.com>

## License

libcouchbase is licensed under the Apache 2.0 License. See `LICENSE` file for
details.
