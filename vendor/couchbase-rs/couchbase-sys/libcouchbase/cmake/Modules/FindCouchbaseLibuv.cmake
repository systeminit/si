# Locate libuvent library
# This module defines
#  HAVE_LIBUV, if false, do not try to link with libuvent
#  LIBUV_LIBRARIES, Library path and libs
#  LIBUV_INCLUDE_DIR, where to find the ICU headers

FIND_PATH(LIBUV_INCLUDE_DIR uv.h
          HINTS
               ${LIBUV_ROOT}
          PATH_SUFFIXES include
          PATHS
               ~/Library/Frameworks
               /Library/Frameworks
               /opt/local
               /opt/csw
               /opt/libuv
               /opt)
MESSAGE(STATUS "Found UV include dir: ${LIBUV_INCLUDE_DIR}")

IF(WIN32)
    FIND_LIBRARY(LIBUV_LIBRARIES_DEBUG
        NAMES uv libuv
        HINTS ${LIBUV_ROOT}
        PATH_SUFFIXES Debug Debug/lib)

    FIND_LIBRARY(LIBUV_LIBRARIES_OPTIMIZED
        NAMES uv libuv
        HINTS ${LIBUV_ROOT}
        PATH_SUFFIXES Release Release/lib)

    SET(LIBUV_LIBRARIES
        OPTIMIZED ${LIBUV_LIBRARIES_OPTIMIZED}
        DEBUG ${LIBUV_LIBRARIES_DEBUG})

ELSE()
    FIND_LIBRARY(LIBUV_LIBRARIES
        NAMES uv
        HINTS ${LIBUV_ROOT}
        PATH_SUFFIXES lib
        PATHS
            ~/Library/Frameworks
            /Library/Frameworks
            /opt/local
            /opt/csw
            /opt/libuv
            /opt)
ENDIF(WIN32)


IF (LIBUV_LIBRARIES)
  SET(HAVE_LIBUV true)
  MESSAGE(STATUS "Found libuv in ${LIBUV_INCLUDE_DIR} : ${LIBUV_LIBRARIES}")
ELSE (LIBUV_LIBRARIES)
  SET(HAVE_LIBUV false)
ENDIF (LIBUV_LIBRARIES)

MARK_AS_ADVANCED(HAVE_LIBUV LIBUV_INCLUDE_DIR LIBUV_LIBRARIES LIBUV_ROOT)
