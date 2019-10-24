FIND_PROGRAM(DTRACE dtrace)
IF(DTRACE)
    SET(LCB_DTRACE_HEADER "${LCB_GENSRCDIR}/probes.h")
    SET(LCB_DTRACE_SRC "${PROJECT_SOURCE_DIR}/src/probes.d")

    # Generate probes.h
    EXECUTE_PROCESS(COMMAND ${DTRACE} -C -h -s ${LCB_DTRACE_SRC} -o ${LCB_DTRACE_HEADER}
        RESULT_VARIABLE _rv)
    IF(NOT ${_rv} EQUAL 0)
        MESSAGE(WARNING "Could not execute DTrace. DTrace support will be disabled!")
        RETURN()
    ENDIF()

    # Fix probes.h on FreeBSD
    IF(CMAKE_SYSTEM_NAME STREQUAL "FreeBSD")
        FIND_PROGRAM(SED sed)
        EXECUTE_PROCESS(COMMAND ${SED} -i.tmp "s/, *char \\*/, const char \\*/g" ${LCB_DTRACE_HEADER}
            RESULT_VARIABLE _rv)
        IF(NOT ${_rv} EQUAL 0)
            MESSAGE(WARNING "Could not execute sed to update dtrace-generated header. DTrace support will be disabled!")
            RETURN()
        ENDIF()
    ENDIF()

    ADD_DEFINITIONS(-DHAVE_DTRACE)
    IF(NOT APPLE)
        SET(LCB_DTRACE_OBJECT "${LCB_GENSRCDIR}/probes.o")
        IF(CMAKE_SYSTEM_NAME STREQUAL "Linux")
            SET(LCB_DTRACE_OPTIONS "-k")
        ENDIF()
        # Generate probes.o
        IF(CMAKE_SYSTEM_NAME STREQUAL "SunOS" OR CMAKE_SYSTEM_NAME STREQUAL "FreeBSD")
            SET(LCB_DTRACE_INSTRO ON)
            UNSET(LCB_DTRACE_OBJECT)
        ELSE()
            ADD_CUSTOM_COMMAND(OUTPUT ${LCB_DTRACE_OBJECT}
                DEPENDS ${LCB_DTRACE_SRC}
                COMMAND ${DTRACE} -C -G ${LCB_DTRACE_OPTIONS} -s ${LCB_DTRACE_SRC} -o ${LCB_DTRACE_OBJECT})
        ENDIF()
    ENDIF()

    FIND_PROGRAM(STAP stap)
    IF(STAP)
      SET(LCB_TAPSET_ROOT ${CMAKE_INSTALL_PREFIX}/share/systemtap/tapset)
      CONFIGURE_FILE(
        ${PROJECT_SOURCE_DIR}/cmake/libcouchbase.stp.in
        ${LCB_GENSRCDIR}/libcouchbase.so.${LCB_SONAME_FULL}.stp)
      INSTALL(
        FILES ${LCB_GENSRCDIR}/libcouchbase.so.${LCB_SONAME_FULL}.stp
        DESTINATION ${LCB_TAPSET_ROOT})
    ENDIF()
ENDIF()
