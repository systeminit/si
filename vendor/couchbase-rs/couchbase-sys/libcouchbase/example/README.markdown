Examples
========

An example says more than a 1000 words (and its way easier to write an
example than trying to explain something). In this directory you'll
find a varity of examples that will show you different use patterns of
libcoucbase.

libeventdirect
--------------

This example programs shows you how you may integrate libcouchbase
into your own event loop (in this example libevent).

syncmode
--------

This example shows you how you may use the synchronous interface of
libcouchbase.

yajl
----

This is an example that shows you how to work with views and json

pillowfight
-----------

This is an example that implements a small test program to show
you some of the functionalities in libcouchbase.

minimal
-------

This is an minimal single-file example which works both unix-like and windows OS.
It accepts three arguments: "host:port", "bucket" and "password".

Build:

     gcc -lcouchbase -o minimal minimal.c
     cl /DWIN32 /Iinclude lib\libcouchbase.lib minimal.c

Execute:

     valgrind -v --tool=memcheck  --leak-check=full --show-reachable=yes ./minimal
     ./minimal <host:port> <bucket> <passwd>
     mininal.exe <host:port> <bucket> <passwd>
