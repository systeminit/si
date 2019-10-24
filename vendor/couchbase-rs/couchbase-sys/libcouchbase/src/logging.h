/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *       http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

#ifndef LCB_LOGGING_H
#define LCB_LOGGING_H
#include <stdarg.h>

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

struct lcb_settings_st;
struct lcb_st;
struct lcbvb_CONFIG_st;

/**
 * Default printf logger which is enabled via LCB_LOGLEVEL in the
 * environment
 */
extern struct lcb_logprocs_st *lcb_console_logprocs;

struct lcb_CONSOLELOGGER {
    struct lcb_logprocs_st base;
    FILE *fp;
    int minlevel;
};

/**
 * Log a message via the installed logger. The parameters correlate to the
 * arguments passed to the lcb_logging_callback function.
 *
 * Typically a subsystem may wish to define macros in order to reduce the
 * number of arguments manually passed for each message.
 */
LCB_INTERNAL_API
void lcb_log(const struct lcb_settings_st *settings, const char *subsys, int severity, const char *srcfile, int srcline,
             const char *fmt, ...)

#ifdef __GNUC__
    __attribute__((format(printf, 6, 7)))
#endif
    ;

LCB_INTERNAL_API
void lcb_log_badconfig(const struct lcb_settings_st *settings, const char *subsys, int severity, const char *srcfile,
                       int srcline, const struct lcbvb_CONFIG_st *vbc, const char *origin_txt);

lcb_logprocs *lcb_init_console_logger(void);

#define LCB_LOGS(settings, subsys, severity, msg) lcb_log(settings, subsys, severity, __FILE__, __LINE__, msg)

#define LCB_LOG_EX(settings, subsys, severity, msg) lcb_log(settings, subsys, severity, __FILE__, __LINE__, msg)

#define LCB_LOG_BASIC(settings, msg) lcb_log(settings, "unknown", 0, __FILE__, __LINE__, msg)

/** Macro for overcoming Win32 identifiers */
#define LCB_LOG_ERR LCB_LOG_ERROR
/** Undefine DEBUG macro to fix environments which are defining it */
#undef DEBUG

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* LCB_LOGGING_H */
