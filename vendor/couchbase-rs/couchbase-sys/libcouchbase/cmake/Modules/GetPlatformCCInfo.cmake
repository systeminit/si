# Gets string/platform information about the specific compiler
# Defines:
#  LCB_CC_STRING for the C compiler string (i.e. "msvc", "mingw")
#  LCB_ARCH_STRING for the target architecture, e.g. "x86"

# Figure out how we're building!
IF(MSVC)
    IF(CMAKE_CL_64)
        SET(LCB_ARCH_STRING "amd64")
    ELSE()
        SET(LCB_ARCH_STRING "x86")
    ENDIF(CMAKE_CL_64)

    IF(MSVC80)
        SET(LCB_CC_STRING "vs8")
    ELSEIF(MSVC90)
        SET(LCB_CC_STRING "vc9")
    ELSEIF(MSVC_VERSION EQUAL 1600)
        SET(LCB_CC_STRING "vc10")
    ELSEIF(MSVC_VERSION EQUAL 1700)
        SET(LCB_CC_STRING "vc11")
    ELSEIF(MSVC_VERSION EQUAL 1800)
        SET(LCB_CC_STRING "vc12")
    ELSEIF(MSVC_VERSION EQUAL 1900)
        SET(LCB_CC_STRING "vc14")
    ELSEIF(MSVC_VERSION GREATER_EQUAL 1910 AND MSVC_VERSION LESS_EQUAL 1919)
        SET(LCB_CC_STRING "vc15")
    ENDIF()
ELSE()
    IF(UNIX)
        SET(LCB_CC_STRING "gcc")
        EXECUTE_PROCESS(
            COMMAND
                uname -m
            COMMAND
                tr -d '\n'
            OUTPUT_VARIABLE
                LCB_ARCH_STRING)
    ELSE()
        IF(CMAKE_SIZEOF_VOID_P EQUAL 8)
            SET(LCB_ARCH_STRING "amd64")
        ELSE()
            SET(LCB_ARCH_STRING "x86")
        ENDIF()
        SET(LCB_CC_STRING "mingw")
    ENDIF()
ENDIF()
