IF(LCB_INSTALL_HEADERS)
  INSTALL(
      FILES
      plugin-libuv.c
      plugin-internal.h
      libuv_compat.h
      DESTINATION
      include/libcouchbase/plugins/io/libuv)

  INSTALL(
      FILES
      libuv_io_opts.h
      DESTINATION
      include/libcouchbase)
ENDIF(LCB_INSTALL_HEADERS)

IF(NOT HAVE_LIBUV OR LCB_NO_PLUGINS OR NOT LCB_BUILD_LIBUV)
    RETURN()
ENDIF()

ADD_LIBRARY(couchbase_libuv SHARED plugin-libuv.c)

# The macros abstracting UV versions dynamically inject special signatures. We
# can't have the compiler whine about that.

# Unfortunately, Older GCC (EL5) barfs at this option.
#IF(CMAKE_COMPILER_IS_GNUCC)
#    SET(_lcbuv_cflags "-Wno-unused-parameters")
#ELSE()
#    SET(_lcbuv_cflags "")
#ENDIF()
SET_TARGET_PROPERTIES(couchbase_libuv
    PROPERTIES
    PREFIX "lib"
    IMPORT_PREFIX "lib"
    COMPILE_FLAGS "${CMAKE_C_FLAGS} ${LCB_CORE_FLAGS} ${_lcbuv_cflags}")

TARGET_LINK_LIBRARIES(couchbase_libuv ${LIBUV_LIBRARIES} couchbase)
INCLUDE_DIRECTORIES(AFTER ${LIBUV_INCLUDE_DIR})
ADD_DEFINITIONS(-DLIBCOUCHBASE_INTERNAL=1)
INSTALL(TARGETS
    couchbase_libuv
    LIBRARY DESTINATION "${CMAKE_INSTALL_LIBDIR}/libcouchbase"
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
