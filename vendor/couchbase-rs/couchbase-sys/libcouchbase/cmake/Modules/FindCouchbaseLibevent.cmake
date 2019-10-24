# Locate libevent library
# This module defines
#  HAVE_LIBEVENT, if false, do not try to link with libevent
#  LIBEVENT_LIBRARIES, Library path and libs
#  LIBEVENT_INCLUDE_DIR, where to find the ICU headers
#  HAVE_LIBEVENT
#  HAVE_LIBEVENT2

FIND_PATH(LIBEVENT_INCLUDE_DIR evutil.h
          HINTS
               ${LIBEVENT_ROOT}
               ENV LIBEVENT_DIR
          PATH_SUFFIXES include
          PATHS
               ${DEPS_INCLUDE_DIR}
               ~/Library/Frameworks
               /Library/Frameworks
               /opt/local
               /opt/csw
               /opt/libevent
               /opt)

FIND_LIBRARY(LIBEVENT_LIBRARIES
             NAMES event_core libevent_core
             HINTS
                 ${LIBEVENT_ROOT}
                 ENV LIBEVENT_DIR
             PATHS
                 ${DEPS_LIB_DIR}
                 ~/Library/Frameworks
                 /Library/Frameworks
                 /opt/local
                 /opt/csw
                 /opt/libevent
                 /opt)

INCLUDE(CMakePushCheckState)
INCLUDE(CheckFunctionExists)

IF (LIBEVENT_LIBRARIES AND LIBEVENT_INCLUDE_DIR)
  SET(HAVE_LIBEVENT true)
  MESSAGE(STATUS "Found libevent in ${LIBEVENT_INCLUDE_DIR} : ${LIBEVENT_LIBRARIES}")
  CMAKE_PUSH_CHECK_STATE()
  SET(CMAKE_REQUIRED_FLAGS "-I${LIBEVENT_INCLUDE_DIR}")
  SET(CMAKE_REQUIRED_INCLUDES "event2/event.h")
  SET(CMAKE_REQUIRED_LIBRARIES ${LIBEVENT_LIBRARIES})
  CHECK_FUNCTION_EXISTS(event_new HAVE_LIBEVENT2)
  CMAKE_POP_CHECK_STATE()

ELSE (LIBEVENT_LIBRARIES)
  SET(HAVE_LIBEVENT false)
ENDIF (LIBEVENT_LIBRARIES AND LIBEVENT_INCLUDE_DIR)

MARK_AS_ADVANCED(HAVE_LIBEVENT LIBEVENT_INCLUDE_DIR LIBEVENT_LIBRARIES)
