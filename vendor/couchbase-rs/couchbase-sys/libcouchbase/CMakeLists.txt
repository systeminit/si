# -*- mode: cmake; cmake-tab-width: 4; indent-tabs-mode: nil -*-
#
# libcouchbase
# Copyright (C) 2013 Couchbase, Inc
# All rights reserved.
#
#   Licensed under the Apache License, Version 2.0 (the "License");
#   you may not use this file except in compliance with the License.
#   You may obtain a copy of the License at
#
#       http://www.apache.org/licenses/LICENSE-2.0
#
#   Unless required by applicable law or agreed to in writing, software
#   distributed under the License is distributed on an "AS IS" BASIS,
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#   See the License for the specific language governing permissions and
#   limitations under the License.

CMAKE_MINIMUM_REQUIRED(VERSION 2.8.9)
# Currently this file is only used for Windows

### BEGIN CONFIGURABLES ###
# These variables can be modified as needed

# Couchbase mock path to download
SET(COUCHBASE_MOCK_VERSION 1.5.22)
SET(COUCHBASE_MOCK_URL "https://github.com/couchbase/CouchbaseMock/releases/download/${COUCHBASE_MOCK_VERSION}/CouchbaseMock-${COUCHBASE_MOCK_VERSION}.jar")
project(libcouchbase)

if (NOT CMAKE_VERSION VERSION_LESS "3.13")
    # CMP0077: option() honors normal variables
    # https://cmake.org/cmake/help/latest/policy/CMP0077.html
    cmake_policy(SET CMP0077 NEW)
endif()

OPTION(LCB_NO_TESTS "Disable building of tests" OFF)
OPTION(LCB_NO_TOOLS "Disable building of additional tools" OFF)
OPTION(LCB_NO_PLUGINS "Disable the building of IO plugins for external libs" OFF)
OPTION(LCB_BUILD_LIBEVENT "Build the libevent plugin" ON)
OPTION(LCB_BUILD_LIBEV "Build the libev plugin (if available)" ON)
OPTION(LCB_BUILD_LIBUV "Build the libuv plugin (if available)" ON)
OPTION(LCB_MAINTAINER_MODE "Enables maintainer mode" OFF)
OPTION(LCB_NO_SSL "Do not compile SSL support" OFF)
OPTION(LCB_USE_ASAN "Use AddressSanitizer support (Requires Clang)" OFF)
OPTION(LCB_USE_COVERAGE "Build with code coverage support" OFF)
OPTION(LCB_USE_ARCHLIBDIR "Use architecture-prefixed library installation directory, if possible" OFF)
OPTION(LCB_BUILD_EXAMPLES "Build example applications" OFF)
OPTION(LCB_NO_MOCK "Don't run tests which depend on the mock" OFF)
OPTION(LCB_BUILD_DTRACE "Build DTrace instrumentation, if available on platform" ON)
OPTION(LCB_EMBED_PLUGIN_LIBEVENT "Embed the libevent plugin" OFF)
OPTION(LCB_STATIC_LIBEVENT "Link static libevent (only applicable if EMBED_PLUGIN_LIBEVENT is ON" OFF)
OPTION(LCB_USE_HDR_HISTOGRAM "Use HdrHistogram for statistics recording" OFF)
OPTION(LCB_INSTALL_HEADERS "Install header files" ON)
OPTION(LCB_INSTALL_PKGCONFIG "Install pkgconfig/libcouchbase.pc" ON)
OPTION(LCB_DUMP_PACKETS "Enable dumping network packets on TRACE log level" OFF)
OPTION(LCB_USE_PROFILER "Build with profiler support (from gperftools)" OFF)
OPTION(LCB_SKIP_GIT_VERSION "Skip version detection using git" OFF)

STRING(TIMESTAMP LCB_BUILD_TIMESTAMP "%Y-%m-%d %H:%M:%S" UTC)

### END CONFIGURABLES ###
IF(MSVC)
    SET(CMAKE_DEBUG_POSTFIX "_d")
ELSE()
    IF(APPLE)
        SET(CMAKE_MACOSX_RPATH ON)
    ENDIF()
    SET(CMAKE_INSTALL_RPATH_USE_LINK_PATH TRUE)
    SET(CMAKE_INSTALL_RPATH_USE_LINK_PATH TRUE)
    LIST(FIND CMAKE_PLATFORM_IMPLICIT_LINK_DIRECTORIES "${CMAKE_INSTALL_PREFIX}/lib" isSystemDir)
    IF("${isSystemDir}" STREQUAL "-1")
        SET(CMAKE_INSTALL_RPATH "${CMAKE_INSTALL_PREFIX}/lib")
    ENDIF("${isSystemDir}" STREQUAL "-1")
ENDIF()

# If building from a configured autotools tree, bail.
IF(EXISTS ${PROJECT_SOURCE_DIR}/src/config.h)
    MESSAGE(FATAL_ERROR "config.h found in src/. This will break the CMake build. Remove it manually then rerun")
ENDIF()

SET(LCB_GENSRCDIR ${PROJECT_BINARY_DIR}/generated)
INCLUDE_DIRECTORIES(${LCB_GENSRCDIR})
SET(LCB_GENINFODIR ${PROJECT_SOURCE_DIR}/packaging/distinfo)

INCLUDE(cmake/Modules/GetLibcouchbaseFlags.cmake)
INCLUDE(cmake/Modules/FindCouchbaseLibevent.cmake)
IF (LCB_BUILD_LIBEV)
    INCLUDE(cmake/Modules/FindCouchbaseLibev.cmake)
ENDIF()
IF (LCB_BUILD_LIBUV)
    INCLUDE(cmake/Modules/FindCouchbaseLibuv.cmake)
ENDIF()
INCLUDE(cmake/Modules/GetPlatformCCInfo.cmake)
INCLUDE(cmake/Modules/GetVersionInfo.cmake)
IF (MSVC)
    INCLUDE(cmake/Modules/CopyPDB.cmake)
ENDIF ()
INCLUDE(cmake/Modules/DownloadLcbDep.cmake)
INCLUDE(CheckIncludeFiles)
INCLUDE(cmake/source_files.cmake)

IF(LCB_USE_PROFILER)
    INCLUDE(cmake/Modules/FindProfiler.cmake)
ENDIF()
IF(LCB_USE_HDR_HISTOGRAM)
    INCLUDE(cmake/Modules/FindCouchbaseHdrHistogram.cmake)
ENDIF()
IF(HDR_HISTOGRAM_FOUND)
    INCLUDE_DIRECTORIES(${HDR_HISTOGRAM_INCLUDES})
    SET(LCB_HDR_HISTOGRAM_LINK "${HDR_HISTOGRAM_LIBRARIES}")
    LIST(APPEND LCB_CORE_SRC "src/hdr_timings.c")
ELSE()
    SET(LCB_HDR_HISTOGRAM_LINK "")
    LIST(APPEND LCB_CORE_SRC "src/timings.c")
ENDIF()

IF(LIB_INSTALL_DIR)
    SET(CMAKE_INSTALL_LIBDIR "${LIB_INSTALL_DIR}")
ENDIF()
IF(NOT LCB_USE_ARCHLIBDIR AND NOT CMAKE_INSTALL_LIBDIR)
    SET(CMAKE_INSTALL_LIBDIR "lib")
ENDIF()
INCLUDE(GNUInstallDirs)


SET(CPACK_PACKAGE_VERSION_MAJOR "${LCB_VERSION_MAJOR}")
SET(CPACK_PACKAGE_VERSION_MINOR "${LCB_VERSION_MINOR}")
SET(CPACK_PACKAGE_VERSION_PATCH "${LCB_VERSION_PATCH}")
IF(NOT CMAKE_BUILD_TYPE)
    SET(CPACK_INSTALL_SCRIPT
        "${PROJECT_SOURCE_DIR}/cmake/Modules/DistScript.cmake")
ENDIF()
SET(lcb_package_name "libcouchbase-${LCB_VERSION}_${LCB_CC_STRING}_${LCB_ARCH_STRING}")
SET(CPACK_PACKAGE_FILE_NAME ${lcb_package_name})

IF(WIN32)
    SET(CPACK_GENERATOR "ZIP")
ELSE()
    SET(CPACK_GENERATOR "TGZ")
ENDIF()

INCLUDE(CPack)

CONFIGURE_FILE(
    ${PROJECT_SOURCE_DIR}/include/libcouchbase/configuration.h.in
    ${LCB_GENSRCDIR}/libcouchbase/configuration.h
    @ONLY)

CONFIGURE_FILE(
    ${PROJECT_SOURCE_DIR}/cmake/defs.mk.in
    ${PROJECT_BINARY_DIR}/defs.mk)

CONFIGURE_FILE(
    ${PROJECT_SOURCE_DIR}/packaging/dllversion.rc.in
    ${PROJECT_BINARY_DIR}/dllversion.rc)

SET(NT_REQUIRED_VERSION 0x0600)

ADD_DEFINITIONS(-DLIBCOUCHBASE_INTERNAL=1)

SET(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${PROJECT_BINARY_DIR}/bin)
SET(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${PROJECT_BINARY_DIR}/lib)
IF(WIN32)
    SET(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${PROJECT_BINARY_DIR}/bin)
ELSE()
    SET(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${PROJECT_BINARY_DIR}/lib)
ENDIF()

SET(SOURCE_ROOT ${PROJECT_SOURCE_DIR})

IF(WIN32)
    ADD_DEFINITIONS(-DWINVER=${NT_REQUIRED_VERSION})
    ADD_DEFINITIONS(-D_WIN32_WINNT=${NT_REQUIRED_VERSION})
    SET(lcb_plat_includes "${SOURCE_ROOT}/contrib/win32-defs")
    SET(lcb_plat_libs ws2_32.lib dnsapi.lib)
    SET(lcb_plat_objs $<TARGET_OBJECTS:couchbase_iocp>)
ELSE()
    SET(lcb_plat_libs m)
    IF(NOT CMAKE_SYSTEM_NAME STREQUAL "FreeBSD")
        SET(lcb_plat_libs ${lcb_plat_libs} dl resolv)
    ELSE()
        # BSD _and_ DTrace
        IF(LCB_BUILD_DTRACE)
            SET(lcb_plat_libs ${lcb_plat_libs} elf)
        ENDIF()
    ENDIF()
    IF(CMAKE_SYSTEM_NAME STREQUAL "SunOS")
        SET(lcb_plat_libs ${lcb_plat_libs} nsl socket)
    ENDIF()
    IF(LCB_EMBED_PLUGIN_LIBEVENT)
        SET(lcb_plat_objs ${lcb_plat_objs} $<TARGET_OBJECTS:couchbase_libevent>)
        SET(lcb_plat_libs ${lcb_plat_libs} ${LIBEVENT_LIBRARIES})
        ADD_DEFINITIONS(-DLCB_EMBED_PLUGIN_LIBEVENT)
    ENDIF()
ENDIF()

INCLUDE_DIRECTORIES(BEFORE ${SOURCE_ROOT}/include
                           ${SOURCE_ROOT}/contrib
                           ${SOURCE_ROOT}/contrib/cbsasl/include
                           ${SOURCE_ROOT}/src
                           ${lcb_plat_includes}
                           ${SOURCE_ROOT})

# These files are bundled in a static library.
# They export no symbols of their own and may
# be considered as 'static'

ADD_LIBRARY(couchbase_utils OBJECT ${LCB_UTILS_SRC})

MACRO(LCB_UTIL tgt)
    SET_TARGET_PROPERTIES(${tgt}
        PROPERTIES
        COMPILE_FLAGS "${LCB_CORE_CFLAGS}"
        POSITION_INDEPENDENT_CODE TRUE)
ENDMACRO()

MACRO(LCB_CXXUTIL tgt)
    SET_TARGET_PROPERTIES(${tgt}
        PROPERTIES
        COMPILE_FLAGS "${LCB_CORE_CXXFLAGS}"
        POSITION_INDEPENDENT_CODE TRUE)
ENDMACRO()

ADD_LIBRARY(netbuf OBJECT ${LCB_NETBUF_SRC})
ADD_LIBRARY(netbuf-malloc OBJECT ${LCB_NETBUF_SRC})
ADD_LIBRARY(mcreq OBJECT ${LCB_MC_SRC})
ADD_LIBRARY(mcreq-cxx OBJECT ${LCB_MC_CXXSRC})
ADD_LIBRARY(rdb OBJECT ${LCB_RDB_SRC})
ADD_LIBRARY(lcbio OBJECT ${LCB_IO_SRC})
ADD_LIBRARY(lcbio-cxx OBJECT ${LCB_IO_CXXSRC})
ADD_LIBRARY(lcbht OBJECT ${LCB_HT_SRC})
ADD_LIBRARY(lcbcore OBJECT ${LCB_CORE_SRC})
ADD_LIBRARY(lcbcore-cxx OBJECT ${LCB_CORE_CXXSRC})

SET_TARGET_PROPERTIES(netbuf-malloc PROPERTIES COMPILE_DEFINITIONS NETBUF_LIBC_PROXY=1)
LCB_UTIL(netbuf-malloc)
LCB_UTIL(netbuf)
LCB_UTIL(rdb)
LCB_UTIL(lcbio)
LCB_CXXUTIL(lcbio-cxx)
LCB_UTIL(couchbase_utils)
LCB_UTIL(mcreq)
LCB_CXXUTIL(mcreq-cxx)
LCB_UTIL(lcbht)
LCB_UTIL(lcbcore)
LCB_CXXUTIL(lcbcore-cxx)

IF(LCB_NO_SSL)
    MESSAGE(STATUS "SSL support will be disabled")
    ADD_DEFINITIONS(-DLCB_NO_SSL=1)
ELSE()
    IF(NOT OPENSSL_FOUND)
        # When building under Couchbase Server we've already detected
        # and set the appropriate OpenSSL variables
        IF(APPLE)
            IF(NOT OpenSSL_DIR)
                EXECUTE_PROCESS(COMMAND brew --prefix openssl
                    OUTPUT_VARIABLE OPENSSL_ROOT_DIR
                    OUTPUT_STRIP_TRAILING_WHITESPACE)
                MESSAGE(STATUS "Found OpenSSL Prefix: ${OPENSSL_ROOT_DIR}")
            ENDIF()
        ENDIF()
        FIND_PACKAGE(OpenSSL)
    ENDIF(NOT OPENSSL_FOUND)

    IF(OPENSSL_FOUND)
        MESSAGE(STATUS "SSL Found")
    ELSE()
        MESSAGE(STATUS "SSL Not Found. SSL support will be disabled")
        ADD_DEFINITIONS(-DLCB_NO_SSL=1)
    ENDIF()
ENDIF()

ADD_SUBDIRECTORY(src/vbucket)
ADD_SUBDIRECTORY(contrib/cbsasl)
ADD_SUBDIRECTORY(contrib/cliopts)
ADD_SUBDIRECTORY(src/ssl)
ADD_SUBDIRECTORY(contrib/lcb-jsoncpp)
ADD_SUBDIRECTORY(contrib/snappy)
IF(LCB_BUILD_EXAMPLES)
    ADD_SUBDIRECTORY(example)
ENDIF()

IF(LCB_BUILD_STATIC)
    SET(_lcb_linkspec STATIC)
ELSE()
    SET(_lcb_linkspec SHARED)
ENDIF()

# CMake configuration
IF(NOT WIN32 AND LCB_BUILD_DTRACE)
    INCLUDE(cmake/Modules/ConfigureDtrace.cmake)
ENDIF()

FILE(GLOB LCB_JSPARSE_SRC src/jsparse/*.cc)
ADD_LIBRARY(lcb_jsparse OBJECT ${LCB_JSPARSE_SRC})
LCB_CXXUTIL(lcb_jsparse)

INCLUDE_DIRECTORIES(contrib/snappy)

SET(LCB_CORE_OBJS
    $<TARGET_OBJECTS:couchbase_select>
    $<TARGET_OBJECTS:couchbase_utils>
    $<TARGET_OBJECTS:vbucket-lcb>
    $<TARGET_OBJECTS:mcreq>
    $<TARGET_OBJECTS:mcreq-cxx>
    $<TARGET_OBJECTS:netbuf>
    $<TARGET_OBJECTS:cbsasl-lcb>
    $<TARGET_OBJECTS:lcbio>
    $<TARGET_OBJECTS:lcbio-cxx>
    $<TARGET_OBJECTS:rdb>
    $<TARGET_OBJECTS:lcbht>
    $<TARGET_OBJECTS:lcbcore>
    $<TARGET_OBJECTS:lcbcore-cxx>
    $<TARGET_OBJECTS:lcb_jsparse>
    $<TARGET_OBJECTS:lcb_jsoncpp>
    $<TARGET_OBJECTS:lcb_snappy>
    ${LCB_DTRACE_OBJECT}
    ${lcb_plat_objs}
    ${lcb_ssl_objs})

ADD_LIBRARY(couchbaseS STATIC ${LCB_CORE_OBJS})
ADD_LIBRARY(couchbase ${_lcb_linkspec} ${LCB_CORE_OBJS} ${PROJECT_BINARY_DIR}/dllversion.rc)

# For DTrace implementations which need to gain access to all the *.o files first
# we need to hook the linker command to a custom perl script which will intercept
# the object files passed to the linker, run dtrace on them, and inject the generated
# object into the linker commandline. This is a bit fragile but we don't officially
# support any of these platforms anyway
IF(LCB_DTRACE_INSTRO)
    SET(_lcb_linkhook "${PROJECT_SOURCE_DIR}/cmake/dtrace-instr-link.pl ${LCB_DTRACE_SRC}")
    SET_TARGET_PROPERTIES(couchbase PROPERTIES RULE_LAUNCH_LINK ${_lcb_linkhook})
    SET_TARGET_PROPERTIES(couchbaseS PROPERTIES RULE_LAUNCH_LINK ${_lcb_linkhook})
ENDIF()

# This is done to be ABI-compatible with Autotools. On OSX, autotools generates
# a library with a Compatibility and Current versions of 3.x despite our library
# actually being versioned at 2.x. To make things worse, autotools also actually
# provided a _symlink_ of libcouchbase.2.dylib. CMake cannot do this manually
# and thus we disable the SOVERSION appending when on OS X, and revert to
# inserting the version information by hand.
IF(NOT APPLE)
    SET_TARGET_PROPERTIES(couchbase PROPERTIES
        SOVERSION "${LCB_SONAME_MAJOR}" VERSION "${LCB_SONAME_FULL}")
ELSE()
    SET_TARGET_PROPERTIES(couchbase PROPERTIES
        LINK_FLAGS "-compatibility_version ${LCB_SONAME_MAJOR} -current_version ${LCB_SONAME_FULL}"
        OUTPUT_NAME couchbase.2)
    IF(NOT LCB_BUILD_STATIC)
        ADD_CUSTOM_COMMAND(TARGET couchbase POST_BUILD
            COMMAND rm -f libcouchbase.dylib
            COMMAND ln libcouchbase.2.dylib libcouchbase.dylib
            WORKING_DIRECTORY ${CMAKE_LIBRARY_OUTPUT_DIRECTORY})
        INSTALL(FILES ${CMAKE_LIBRARY_OUTPUT_DIRECTORY}/libcouchbase.dylib
            DESTINATION ${CMAKE_INSTALL_LIBDIR})
    ENDIF()
ENDIF()

SET_TARGET_PROPERTIES(couchbase PROPERTIES PREFIX "lib")
SET_TARGET_PROPERTIES(couchbase PROPERTIES IMPORT_PREFIX "lib")
SET(LCB_LINK_DEPS ${lcb_plat_libs} ${lcb_ssl_libs} ${LCB_HDR_HISTOGRAM_LINK})
IF(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    SET(LCB_LINK_DEPS ${LCB_LINK_DEPS} rt)
ENDIF()
IF(LIBPROFILER)
    SET(LCB_LINK_DEPS ${LCB_LINK_DEPS} ${LIBPROFILER})
ENDIF()

TARGET_LINK_LIBRARIES(couchbase ${LCB_LINK_DEPS})
TARGET_LINK_LIBRARIES(couchbaseS ${LCB_LINK_DEPS})

ENABLE_TESTING()
IF(NOT LCB_NO_TESTS)
    ADD_SUBDIRECTORY(tests tests)
ELSE()
    ADD_TEST(NAME dummytest COMMAND echo "No Tests!")
ENDIF(NOT LCB_NO_TESTS)

IF(NOT LCB_NO_TOOLS)
    ADD_SUBDIRECTORY(tools tools)
ENDIF(NOT LCB_NO_TOOLS)

# Always add this file, as we need the headers for this!
ADD_SUBDIRECTORY(plugins/io/libuv)
IF(NOT LCB_NO_PLUGINS AND LCB_BUILD_LIBUV)
ENDIF()

IF(NOT LCB_NO_PLUGINS AND NOT WIN32)
    IF(LCB_BUILD_LIBEVENT)
        ADD_SUBDIRECTORY(plugins/io/libevent)
    ENDIF()
    IF(LCB_BUILD_LIBEV)
        ADD_SUBDIRECTORY(plugins/io/libev)
    ENDIF()
    IF(NOT (HAVE_LIBEV OR HAVE_LIBEVENT))
        MESSAGE(FATAL_ERROR
            "libev or libevent development files missing.
            You can disable these dependencies by passing
            -DLCB_NO_PLUGINS=1 to Cmake, or
            --disable-plugins to the configure script")
    ENDIF()
ENDIF()


IF(LCB_MAINTAINER_MODE)
    IF(NOT (HAVE_LIBEV AND HAVE_LIBEVENT AND HAVE_LIBUV))
        MESSAGE(FATAL_ERROR "Maintainer mode requires all plugins to be present")
    ENDIF()
ENDIF()

ADD_SUBDIRECTORY(plugins/io/select)
ADD_SUBDIRECTORY(plugins/io/iocp)
INSTALL(TARGETS couchbase
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})

IF(LCB_INSTALL_HEADERS)
    INSTALL(DIRECTORY include/libcouchbase ${LCB_GENSRCDIR}/libcouchbase
            DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
            FILES_MATCHING PATTERN *.h PATTERN *.c)
ENDIF()

INSTALL(DIRECTORY doc/man/ DESTINATION ${CMAKE_INSTALL_MANDIR}/man1 FILES_MATCHING PATTERN *.1)
INSTALL(DIRECTORY doc/man/ DESTINATION ${CMAKE_INSTALL_MANDIR}/man4  FILES_MATCHING PATTERN *.4)

IF(MSVC)
    INSTALL_PDBS(couchbase)
ENDIF()

SET(_lcb_tarname "libcouchbase-${LCB_VERSION}")
SET(_lcb_manifest "${LCB_GENINFODIR}/MANIFEST")

# pkg-config stuff goes with installation
IF(NOT WIN32 AND LCB_INSTALL_PKGCONFIG)
    CONFIGURE_FILE(packaging/libcouchbase.pc.in packaging/libcouchbase.pc @ONLY)
    INSTALL(FILES ${PROJECT_BINARY_DIR}/packaging/libcouchbase.pc DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)
ENDIF()


ADD_CUSTOM_TARGET(uninstall
    COMMAND xargs rm -vf < ${CMAKE_CURRENT_BINARY_DIR}/install_manifest.txt)

ADD_CUSTOM_TARGET(file_manifest
    COMMAND sh -c 'test -e ${_lcb_manifest} || git ls-files > ${_lcb_manifest}'
    WORKING_DIRECTORY ${PROJECT_SOURCE_DIR})


ADD_CUSTOM_TARGET(dist
    COMMAND rm -rf "${_lcb_tarname}"
    COMMAND mkdir "${_lcb_tarname}"
    COMMAND tar -cf - -C ${PROJECT_SOURCE_DIR} -T ${_lcb_manifest} | tar xf - -C "${_lcb_tarname}"
    COMMAND cp -a "${LCB_GENINFODIR}" "${_lcb_tarname}/packaging"
    COMMAND tar -czf "${_lcb_tarname}.tar.gz" "${_lcb_tarname}"
    COMMAND rm -rf "${_lcb_tarname}"
    DEPENDS file_manifest)

# Generate our configuration file _after_ we've collected everything
INCLUDE(cmake/Modules/GenerateConfigDotH.cmake)

# Build any local tests/scripts
IF (EXISTS ${SOURCE_ROOT}/tests/LOCAL)
    ADD_SUBDIRECTORY(${SOURCE_ROOT}/tests/LOCAL)
ENDIF()
