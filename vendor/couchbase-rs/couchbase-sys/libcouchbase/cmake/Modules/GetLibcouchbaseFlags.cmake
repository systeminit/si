# Common flags for libcouchbase modules. This defines the specific flags
# required for various compilation modes
# Exports:
#   LCB_CORE_CFLAGS:
#       C flags to be used by our "Core" modules. This contains
#       many warnings.
#   LCB_CORE_CXXFLAGS:
#       Like LCB_CORE_CFLAGS, but for C++
#
#   LCB_BASIC_CFLAGS
#       Basic C flags without extra warnings
#   LCB_BASIC_CXXFLAGS
#       Basic C++ flags without extra warnings.
#
# Note that global flags will still be modified for debug settings and the
# like.

MACRO(list2args VAR)
    STRING(REPLACE ";" " " _tmp "${${VAR}}")
    SET("${VAR}" "${_tmp}")
ENDMACRO(list2args)

LIST(APPEND LCB_GNUC_CPP_WARNINGS
    -Wall -pedantic -Wshadow -fdiagnostics-show-option -Wformat
    -Wno-strict-aliasing -Wextra -Winit-self -Wno-missing-field-initializers
    -Wno-variadic-macros)

IF(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    LIST(APPEND LCB_GNUC_CPP_WARNINGS -Wno-cast-align -Wno-dollar-in-identifier-extension)
ENDIF()

IF(LCB_USE_ASAN)
    SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -fsanitize=address")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fno-omit-frame-pointer -fsanitize=address")
    SET(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -undefined dynamic_lookup -fsanitize=address")
    SET(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=address")
ENDIF()

IF(LCB_USE_COVERAGE)
    SET(_covflags "-fprofile-arcs -ftest-coverage")
    IF(CMAKE_COMPILER_IS_GNUCC)
        SET(_covflags "--coverage ${_covflags}")
    ENDIF()

    LIST(APPEND LCB_GNUC_CPP_WARNINGS ${_covflags})
    SET(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} ${_covflags}")
    SET(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} ${_covflags}")
ENDIF()

list2args(LCB_GNUC_CPP_WARNINGS)

LIST(APPEND LCB_GNUC_C_WARNINGS
    ${LCB_GNUC_CPP_WARNINGS}
    -std=gnu99
    -Wundef -Wstrict-prototypes -Wmissing-prototypes -Wredundant-decls
    -Wmissing-declarations)
list2args(LCB_GNUC_C_WARNINGS)

LIST(APPEND LCB_GNUC_CXX_WARNINGS
    ${LCB_GNUC_CPP_WARNINGS}
    -Woverloaded-virtual -Wnon-virtual-dtor -Wctor-dtor-privacy
    -Wno-long-long -Wredundant-decls)

INCLUDE(CheckCXXCompilerFlag)
CHECK_CXX_COMPILER_FLAG("-std=c++11" COMPILER_SUPPORTS_CXX11)
IF(COMPILER_SUPPORTS_CXX11)
    LIST(APPEND LCB_GNUC_CXX_WARNINGS -std=c++11)
ENDIF()

list2args(LCB_GNUC_CXX_WARNINGS)

#MSVC-specific flags for C/C++
LIST(APPEND LCB_CL_CPPFLAGS /nologo /W3 /MP /EHsc)
LIST(APPEND LCB_CL_CPPFLAGS /wd4800 /wd4244 /wd4267)
list2args(LCB_CL_CPPFLAGS)

# Common flags for DEBUG
LIST(APPEND LCB_CL_CPPFLAGS_DEBUG /RTC1)
list2args( LCB_CL_CPPFLAGS_DEBUG)

# Common flags for RELEASE
LIST(APPEND LCB_CL_CPPFLAGS_REL /O2)
list2args(LCB_CL_CPPFLAGS_REL)

MACRO(SET_ALL_FLAGS extra_flags)
    FOREACH(variant C CXX)
        FOREACH(config RELEASE DEBUG RELWITHDEBINFO)
            SET(varname "CMAKE_${variant}_FLAGS_${config}")
            SET(existing ${${varname}})
            SET(${varname} "${existing} ${extra_flags}")
        ENDFOREACH()
        SET(CMAKE_${variant}_FLAGS "${CMAKE_${variant}_FLAGS} ${extra_flags}")
    ENDFOREACH()
ENDMACRO()

IF(MSVC)
    ADD_DEFINITIONS(-D_CRT_SECURE_NO_WARNINGS)
    # Don't warn about "deprecated POSIX names"
    ADD_DEFINITIONS(-D_CRT_NONSTDC_NO_DEPRECATE)

    # Need this for VS 2012 for googletest and C++
    IF(MSVC_VERSION EQUAL 1700 OR MSVC_VERSION GREATER 1700)
        ADD_DEFINITIONS(-D_VARIADIC_MAX=10)
    ENDIF()
    SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} /TC ${LCB_CL_CPPFLAGS}")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${LCB_CL_CPPFLAGS}")
    SET(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} ${LCB_CL_CPPFLAGS_DEBUG}")
    SET(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} ${LCB_CL_CPPFLAGS_DEBUG}")
    SET(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} ${LCB_CL_CPPFLAGS_REL}")
    SET(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} ${LCB_CL_CPPFLAGS_REL}")

    # put debug info into release build and revert /OPT defaults after
    # /DEBUG so that it won't degrade performance and size
    # http://msdn.microsoft.com/en-us/library/xe4t6fc1(v=vs.80).aspx
    # Since CMake for some odd reason strips 'incremental' and 'INCREMENTAL', we'll
    # use weird casing here
    SET(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} /OPT:REF /OPT:ICF /IncReMenTal:no")
    SET(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} /OPT:REF /OPT:ICF /InCreMenTal:no")
    SET(LCB_CORE_CXXFLAGS "")
    SET(LCB_CORE_CFLAGS "")
    SET(LCB_BASIC_CFLAGS "")
    SET(LCB_BASIC_CXXFLAGS "")

ELSE()
    # GCC
    SET_ALL_FLAGS("-fno-strict-aliasing -ggdb3")
    IF(WIN32)
        SET(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -gstabs")
        SET(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -gstabs")
        SET(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static-libgcc -static-libstdc++")
        SET(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -static-libgcc -static-libstdc++")
    ELSE()
        SET_ALL_FLAGS("-pthread")
    ENDIF()
    SET(LCB_CORE_CFLAGS "${LCB_GNUC_C_WARNINGS} -DHAVE_VISIBILITY -fvisibility=hidden")
    SET(LCB_CORE_CXXFLAGS "${LCB_GNUC_CXX_WARNINGS} -DHAVE_VISIBILITY -fvisibility=hidden")
ENDIF()

IF(LCB_UNIVERSAL_BINARY AND (${CMAKE_SYSTEM_NAME} MATCHES "Darwin"))
    SET(CMAKE_C_FLAGS
        "${CMAKE_C_FLAGS} -force_cpusubtype_ALL -arch i386 -arch x86_64")
ENDIF()
