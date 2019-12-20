/* -*- Mode: C; tab-width: 3; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018-2019 Couchbase, Inc.
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

#ifndef LIBCOUCHBASE_couchbase_api4_h__
#define LIBCOUCHBASE_couchbase_api4_h__

typedef struct lcb_INSTANCE lcb_INSTANCE;
typedef struct lcb_CMDCREATE lcb_CMDCREATE;
typedef struct lcb_LOGGER lcb_LOGGER;
typedef struct lcb_IOOPTS lcb_IOOPTS;
typedef struct lcb_AUTHENTICATOR lcb_AUTHENTICATOR;
typedef struct lcb_HTTP_HANDLE_ lcb_HTTP_HANDLE;

typedef struct lcb_CMDSTATS lcb_CMDSTATS;
typedef struct lcb_CMDFLUSH lcb_CMDFLUSH;
typedef struct lcb_CMDNOOP lcb_CMDNOOP;

typedef enum {
    LCB_LOG_TRACE = 0,
    LCB_LOG_DEBUG,
    LCB_LOG_INFO,
    LCB_LOG_WARN,
    LCB_LOG_ERROR,
    LCB_LOG_FATAL,
    LCB_LOG_MAX
} lcb_LOG_SEVERITY;

typedef void (*lcb_LOGGER_CALLBACK)(lcb_LOGGER *logger, unsigned int iid, const char *subsys, lcb_LOG_SEVERITY severity,
                                    const char *srcfile, int srcline, const char *fmt, va_list ap);
typedef void (*lcb_AUTHENTICATOR_CALLBACK)(lcb_AUTHENTICATOR *auth, const char *host, const char *port,
                                           const char *bucket, char **username, size_t *username_len, char **password,
                                           size_t *password_len);
typedef void (*lcb_RESPONSE_CALLBACK)(lcb_INSTANCE instance, lcb_CALLBACK_TYPE type, const lcb_RESPBASE *resp);

LIBCOUCHBASE_API lcb_STATUS lcb_logger_create(lcb_LOGGER **logger);
LIBCOUCHBASE_API lcb_STATUS lcb_logger_destroy(lcb_LOGGER *logger);
LIBCOUCHBASE_API lcb_STATUS lcb_logger_callback(lcb_LOGGER *logger, lcb_LOGGER_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_logger_level(lcb_LOGGER *logger, lcb_LOG_SEVERITY level);

LIBCOUCHBASE_API lcb_STATUS lcb_ioopts_create(lcb_IOOPTS **ioopts);
LIBCOUCHBASE_API lcb_STATUS lcb_ioopts_destroy(lcb_IOOPTS *ioopts);

LIBCOUCHBASE_API lcb_STATUS lcb_authenticator_create(lcb_AUTHENTICATOR **auth, const char *username,
                                                     size_t username_len, const char *password, size_t password_len);
LIBCOUCHBASE_API lcb_STATUS lcb_authenticator_new_dynamic(lcb_AUTHENTICATOR **auth,
                                                          lcb_AUTHENTICATOR_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_authenticator_destroy(lcb_AUTHENTICATOR *auth);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_create(lcb_CMDCREATE **options);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_destroy(lcb_CMDCREATE *options);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_type(lcb_CMDCREATE *options, lcb_INSTANCE_TYPE type);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_connstr(lcb_CMDCREATE *options, const char *connstr, size_t connstr_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_logger(lcb_CMDCREATE *options, const lcb_LOGGER *logger);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_authenticator(lcb_CMDCREATE *options, lcb_AUTHENTICATOR *auth);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcreate_ioopts(lcb_CMDCREATE *options, const lcb_IOOPTS *io);

LIBCOUCHBASE_API lcb_STATUS lcb_create(lcb_INSTANCE **instance, lcb_CMDCREATE *options);
LIBCOUCHBASE_API lcb_STATUS lcb_destroy(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_connect(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_wait(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_tick_nowait(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_is_waiting(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_breakout(lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_set_cookie(lcb_INSTANCE *instance, const void *cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_get_cookie(lcb_INSTANCE *instance, const void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_refresh_config(lcb_INSTANCE *instance);

LIBCOUCHBASE_API lcb_STATUS lcb_install_callback(lcb_INSTANCE *instance, lcb_RESPONSE_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_get_callback(lcb_INSTANCE *instance, lcb_RESPONSE_CALLBACK *callback);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstats_create(lcb_CMDSTATS **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstats_destroy(lcb_CMDSTATS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstats_cookie(lcb_CMDSTATS *cmd, const void *cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstats_parent_span(lcb_CMDSTATS *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstats_key(lcb_CMDSTATS *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_stats(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDSTATS *cmd);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdflush_create(lcb_CMDFLUSH **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdflush_destroy(lcb_CMDFLUSH *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdflush_cookie(lcb_CMDFLUSH *cmd, const void *cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdflush_parent_span(lcb_CMDFLUSH *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_flush(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDFLUSH *cmd);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdnoop_create(lcb_CMDNOOP **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdnoop_destroy(lcb_CMDNOOP *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdnoop_cookie(lcb_CMDNOOP *cmd, const void *cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdnoop_parent_span(lcb_CMDNOOP *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_noop(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDNOOP *cmd);

#endif
