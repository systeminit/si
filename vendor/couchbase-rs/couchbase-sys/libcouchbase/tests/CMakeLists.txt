# For Windows+GCC we need to link statically
IF (NOT (WIN32 AND CMAKE_COMPILER_IS_GNUCC))
    SET(gtest_force_shared_crt ON CACHE BOOL "" )
    SET(BUILD_SHARED_LIBS ON)
ENDIF()
ADD_SUBDIRECTORY(${PROJECT_SOURCE_DIR}/contrib/gtest-1.8.1 gtest)
INCLUDE_DIRECTORIES(${CMAKE_CURRENT_SOURCE_DIR})

ADD_EXECUTABLE(check-all EXCLUDE_FROM_ALL
    check-all.cc $<TARGET_OBJECTS:cliopts> mocksupport/procutil.c)

SET_PROPERTY(TARGET check-all
    APPEND PROPERTY INCLUDE_DIRECTORIES
    ${lcb_plat_includes}
    ${SOURCE_ROOT})
TARGET_LINK_LIBRARIES(check-all couchbaseS)


FILE(GLOB T_BASIC_SRC basic/*.cc)
FILE(GLOB T_MC_SRC mc/*.cc)
FILE(GLOB T_RDB_SRC rdb/*.cc)
FILE(GLOB T_SOCK_SRC socktests/*.cc)
FILE(GLOB T_IOSERVER_SRC ioserver/*.cc)
FILE(GLOB T_MOCKSUPPORT_SRC mocksupport/*.c mocksupport/*.cc)
FILE(GLOB T_VBTEST_SRC vbucket/*.cc)

ADD_LIBRARY(ioserver OBJECT EXCLUDE_FROM_ALL ${T_IOSERVER_SRC})
IF(NOT LCB_NO_SSL)
    GET_TARGET_PROPERTY(_ioserver_includes ioserver INCLUDE_DIRECTORIES)
    SET_TARGET_PROPERTIES(ioserver PROPERTIES INCLUDE_DIRECTORIES "${_ioserver_includes};${OPENSSL_INCLUDE_DIR}")
ENDIF()


ADD_EXECUTABLE(nonio-tests EXCLUDE_FROM_ALL nonio_tests.cc ${T_BASIC_SRC})

ADD_EXECUTABLE(mc-tests EXCLUDE_FROM_ALL nonio_tests.cc ${T_MC_SRC}
    $<TARGET_OBJECTS:mcreq> $<TARGET_OBJECTS:mcreq-cxx> $<TARGET_OBJECTS:netbuf> $<TARGET_OBJECTS:vbucket-lcb>)

ADD_EXECUTABLE(mc-malloc-tests EXCLUDE_FROM_ALL nonio_tests.cc ${T_MC_SRC}
    $<TARGET_OBJECTS:mcreq> $<TARGET_OBJECTS:mcreq-cxx> $<TARGET_OBJECTS:netbuf-malloc> $<TARGET_OBJECTS:vbucket-lcb>)

ADD_EXECUTABLE(netbuf-tests
    EXCLUDE_FROM_ALL nonio_tests.cc basic/t_netbuf.cc $<TARGET_OBJECTS:netbuf>)

ADD_EXECUTABLE(rdb-tests EXCLUDE_FROM_ALL nonio_tests.cc
    ${T_RDB_SRC} $<TARGET_OBJECTS:rdb> ${SOURCE_ROOT}/src/list.c)

ADD_EXECUTABLE(sock-tests EXCLUDE_FROM_ALL nonio_tests.cc
    ${T_SOCK_SRC} $<TARGET_OBJECTS:ioserver>)

ADD_EXECUTABLE(vbucket-tests EXCLUDE_FROM_ALL nonio_tests.cc ${T_VBTEST_SRC})
ADD_EXECUTABLE(htparse-tests EXCLUDE_FROM_ALL nonio_tests.cc htparse/t_basic.cc)

FILE(GLOB T_IO_SRC iotests/*.cc)
IF(LCB_NO_MOCK)
    ADD_EXECUTABLE(unit-tests EXCLUDE_FROM_ALL unit_tests.cc)
    SET_TARGET_PROPERTIES(unit-tests PROPERTIES COMPILE_DEFINITIONS NO_COUCHBASEMOCK)
ELSE()
    ADD_LIBRARY(mocksupport OBJECT EXCLUDE_FROM_ALL ${T_MOCKSUPPORT_SRC})
    ADD_EXECUTABLE(unit-tests EXCLUDE_FROM_ALL ${T_IO_SRC} unit_tests.cc $<TARGET_OBJECTS:mocksupport>)
ENDIF()

IF(CMAKE_COMPILER_IS_GNUCXX)
    SET_TARGET_PROPERTIES(unit-tests
        PROPERTIES
        COMPILE_FLAGS "-Wno-sign-compare -Wno-missing-field-initializers")
ENDIF(CMAKE_COMPILER_IS_GNUCXX)

IF(NOT LCB_NO_MOCK)
    IF(NOT EXISTS ${PROJECT_SOURCE_DIR}/tests/CouchbaseMock.jar)
        DOWNLOAD_LCB_DEP("${COUCHBASE_MOCK_URL}" "${PROJECT_SOURCE_DIR}/tests/CouchbaseMock.jar")
    ENDIF()
    FIND_PROGRAM(JAVA java)
    EXECUTE_PROCESS(COMMAND "${JAVA}" -jar "${PROJECT_SOURCE_DIR}/tests/CouchbaseMock.jar" --version
                    RESULT_VARIABLE RET OUTPUT_VARIABLE VER OUTPUT_STRIP_TRAILING_WHITESPACE)
    IF(RET EQUAL 0)
        MESSAGE(STATUS "Using ${VER}")
    ELSE()
        MESSAGE(FATAL_ERROR "Unable to use ${PROJECT_SOURCE_DIR}/tests/CouchbaseMock.jar as CouchbaseMock")
    ENDIF()
ENDIF()

# We're a library using the shared CRT. Don't use the static one
INCLUDE_DIRECTORIES(${gtest_SOURCE_DIR}/include ${gtest_SOURCE_DIR})
TARGET_LINK_LIBRARIES(unit-tests couchbaseS gtest)
TARGET_LINK_LIBRARIES(nonio-tests couchbaseS gtest)
TARGET_LINK_LIBRARIES(mc-tests couchbaseS gtest)
TARGET_LINK_LIBRARIES(mc-malloc-tests couchbaseS gtest)
TARGET_LINK_LIBRARIES(netbuf-tests gtest)
TARGET_LINK_LIBRARIES(rdb-tests gtest)
TARGET_LINK_LIBRARIES(sock-tests couchbaseS gtest)
TARGET_LINK_LIBRARIES(vbucket-tests gtest couchbaseS)
TARGET_LINK_LIBRARIES(htparse-tests gtest couchbaseS)

IF(WIN32)
    TARGET_LINK_LIBRARIES(mc-tests ws2_32.lib)
    TARGET_LINK_LIBRARIES(mc-malloc-tests ws2_32.lib)
ENDIF()

ADD_CUSTOM_TARGET(alltests DEPENDS check-all unit-tests nonio-tests
    rdb-tests sock-tests vbucket-tests mc-tests htparse-tests)


ADD_TEST(NAME BUILD-TESTS COMMAND ${CMAKE_COMMAND} --build "${PROJECT_BINARY_DIR}" --target alltests)

MACRO(DEFINE_MOCKTEST plugin test)
    ADD_TEST(
        NAME
            check-${plugin}-${test}
        COMMAND
            $<TARGET_FILE:check-all>
            --srcdir "${PROJECT_SOURCE_DIR}"
            --testdir "$<TARGET_FILE_DIR:check-all>"
            --libdir "$<TARGET_FILE_DIR:couchbase>"
            --plugins ${plugin}
            --tests ${test}
            --verbose
            --
            --gtest_print_time=1
            --gtest_output=xml:"${PROJECT_BINARY_DIR}/REPORT_${plugin}_${test}.xml")
ENDMACRO()

# Since we need a plugin name, we'll use 'select'. However none of these
# actually do any I/O
DEFINE_MOCKTEST("select" "nonio-tests")
DEFINE_MOCKTEST("select" "rdb-tests")
DEFINE_MOCKTEST("select" "vbucket-tests")
DEFINE_MOCKTEST("select" "mc-tests")
DEFINE_MOCKTEST("select" "htparse-tests")


DEFINE_MOCKTEST("select" "unit-tests")
DEFINE_MOCKTEST("select" "sock-tests")
IF(WIN32)
    DEFINE_MOCKTEST("iocp" "unit-tests")
    DEFINE_MOCKTEST("iocp" "sock-tests")
ENDIF()
IF(HAVE_LIBEVENT AND LCB_BUILD_LIBEVENT)
    DEFINE_MOCKTEST("libevent" "unit-tests")
    DEFINE_MOCKTEST("libevent" "sock-tests")
ENDIF()
IF(HAVE_LIBEV AND LCB_BUILD_LIBEV)
    DEFINE_MOCKTEST("libev" "unit-tests")
    DEFINE_MOCKTEST("libev" "sock-tests")
ENDIF()
IF(HAVE_LIBUV AND LCB_BUILD_LIBUV)
    DEFINE_MOCKTEST("libuv" "unit-tests")
    DEFINE_MOCKTEST("libuv" "sock-tests")
ENDIF()
