IF(WIN32)
    INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/contrib/win32-defs)
    ADD_LIBRARY(couchbase_iocp OBJECT
        iocp_iops.c
        iocp_loop.c
        iocp_timer.c
        iocp_util.c)
    ADD_DEFINITIONS(-DLIBCOUCHBASE_INTERNAL=1)
ENDIF(WIN32)
