# These define the various source file listings of various modules
# within the library.
# This is included by the top-level CMakeLists.txt

# couchbase_utils
SET(LCB_UTILS_SRC
    contrib/genhash/genhash.c
    src/strcodecs/base64.c
    src/gethrtime.c
    src/hashtable.c
    src/list.c
    src/logging.c
    src/ringbuffer.c)

# lcbio
FILE(GLOB LCB_IO_SRC src/lcbio/*.c)
FILE(GLOB LCB_IO_CXXSRC src/lcbio/*.cc)

# common memcached operations
FILE(GLOB LCB_OP_SRC src/operations/*.c)

# memcached packets
FILE(GLOB LCB_MC_SRC src/mc/*.c)
FILE(GLOB LCB_MC_CXXSRC src/mc/*.cc)

# read buffer management
FILE(GLOB LCB_RDB_SRC src/rdb/*.c)

# send buffer management
FILE(GLOB LCB_NETBUF_SRC src/netbuf/*.c)

# HTTP protocol management
LIST(APPEND LCB_HT_SRC "contrib/http_parser/http_parser.c")

# bucket config ("confmon")
FILE(GLOB LCB_BCONF_SRC src/bucketconfig/*.c)

SET(LCB_CORE_SRC
    ${LCB_OP_SRC}
    ${LCB_BCONF_SRC}
    ${LCB_N1QL_SRC}
    src/callbacks.c
    src/iofactory.c
    src/settings.c
    src/utilities.c)

FILE(GLOB LCB_TRACING_SRC src/tracing/*.cc)

SET(LCB_CORE_CXXSRC
    src/instance.cc
    src/analytics/analytics.cc
    src/auth.cc
    src/bootstrap.cc
    src/bucketconfig/bc_cccp.cc
    src/bucketconfig/bc_http.cc
    src/bucketconfig/bc_file.cc
    src/bucketconfig/bc_static.cc
    src/bucketconfig/confmon.cc
    src/collections.cc
    src/connspec.cc
    src/crypto.cc
    src/dns-srv.cc
    src/dump.cc
    src/errmap.cc
    src/getconfig.cc
    src/nodeinfo.cc
    src/handler.cc
    src/hostlist.cc
    src/http/http.cc
    src/http/http_io.cc
    src/lcbht/lcbht.cc
    src/newconfig.cc
    src/n1ql/n1ql.cc
    src/n1ql/ixmgmt.cc
    src/cbft.cc
    src/operations/cbflush.cc
    src/operations/counter.cc
    src/operations/durability.cc
    src/operations/durability-cas.cc
    src/operations/durability-seqno.cc
    src/operations/get.cc
    src/operations/observe.cc
    src/operations/observe-seqno.cc
    src/operations/pktfwd.cc
    src/operations/remove.cc
    src/operations/stats.cc
    src/operations/store.cc
    src/operations/subdoc.cc
    src/operations/touch.cc
    src/operations/ping.cc
    src/operations/exists.cc
    src/mcserver/mcserver.cc
    src/mcserver/negotiate.cc
    src/metrics.cc
    src/retrychk.cc
    src/retryq.cc
    src/rnd.cc
    src/docreq/docreq.cc
    src/views/viewreq.cc
    src/cntl.cc
    src/wait.cc
    ${LCB_TRACING_SRC}
    )
