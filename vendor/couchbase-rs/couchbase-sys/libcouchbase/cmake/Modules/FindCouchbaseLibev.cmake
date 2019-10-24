# Locate libev library
# This module defines
#  HAVE_LIBEV, if false, do not try to link with libev
#  LIBEV_LIBRARIES, Library path and libs
#  LIBEV_INCLUDE_DIR, where to find the libev headers

FIND_PATH(LIBEV_INCLUDE_DIR ev.h
    PATHS ${LIBEV_ROOT}
    PATH_SUFFIXES include libev
    NO_DEFAULT_PATH)

FIND_LIBRARY(LIBEV_LIBRARIES
    NAMES ev libev
    PATHS ${LIBEV_ROOT}
    PATH_SUFFIXES lib libev
    NO_DEFAULT_PATH)

FIND_PATH(LIBEV_INCLUDE_DIR ev.h
    HINTS
        ENV LIBEV_DIR
    PATH_SUFFIXES include include/libev
    PATHS
        ${DEPS_INCLUDE_DIR}
        ~/Library/Frameworks
        /Library/Frameworks
        /opt/local
        /opt/csw
        /opt/libev
        /opt)

FIND_LIBRARY(LIBEV_LIBRARIES
    NAMES ev libev
    HINTS
        ENV LIBEV_DIR
    PATH_SUFFIXES lib libev
    PATHS
        ${DEPS_LIB_DIR}
        ~/Library/Frameworks
        /Library/Frameworks
        /opt/local
        /opt/csw
        /opt/libev
        /opt)

IF (LIBEV_LIBRARIES AND LIBEV_INCLUDE_DIR)
    SET(HAVE_LIBEV true)
    MESSAGE(STATUS "Found libev in ${LIBEV_INCLUDE_DIR} : ${LIBEV_LIBRARIES}")
ELSE (LIBEV_LIBRARIES)
    SET(HAVE_LIBEV false)
ENDIF (LIBEV_LIBRARIES AND LIBEV_INCLUDE_DIR)

INCLUDE(CMakePushCheckState)
INCLUDE(CheckFunctionExists)
IF(HAVE_LIBEV)
    CMAKE_PUSH_CHECK_STATE()
    SET(CMAKE_REQUIRED_FLAGS "-I${LIBEV_INCLUDE_DIR}")
    SET(CMAKE_REQUIRED_LIBRARIES ${LIBEV_LIBRARIES})
    SET(CMAKE_REQUIRED_INCLUDES "ev.h")
    CHECK_FUNCTION_EXISTS(ev_loop HAVE_LIBEV3)
    IF(NOT HAVE_LIBEV3)
        CHECK_FUNCTION_EXISTS(ev_run HAVE_LIBEV4)
    ENDIF()
    CMAKE_POP_CHECK_STATE()
    IF(HAVE_LIBEV3)
        MESSAGE(STATUS "libev3 found")
    ELSEIF(HAVE_LIBEV4)
        MESSAGE(STATUS "libev4 found")
    ELSE()
        MESSAGE(STATUS "libev not found")
    ENDIF()
ENDIF()

MARK_AS_ADVANCED(HAVE_LIBEV LIBEV_INCLUDE_DIR LIBEV_LIBRARIES)
