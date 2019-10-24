FIND_LIBRARY(LIBPROFILER
             NAMES profiler
             HINTS
                 ENV LIBPROFILER_DIR
             PATHS
                 ${DEPS_LIB_DIR}
                 ~/Library/Frameworks
                 /Library/Frameworks
                 /opt/local
                 /opt)

IF(LIBPROFILER)
  MESSAGE(STATUS "Found libprofiler: ${LIBPROFILER}")
ELSE()
  MESSAGE(FATAL_ERROR "Unable to find gperftools libprofiler, Try disabling LCB_USE_PROFILER option of CMake.")
ENDIF()
