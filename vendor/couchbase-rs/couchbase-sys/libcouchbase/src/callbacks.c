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

#include "internal.h"

#define DEFINE_DUMMY_CALLBACK(name, resptype)                                                                          \
    static void name(lcb_INSTANCE *i, const void *c, lcb_STATUS e, const resptype *r)                                  \
    {                                                                                                                  \
        (void)i;                                                                                                       \
        (void)e;                                                                                                       \
        (void)c;                                                                                                       \
        (void)r;                                                                                                       \
    }

static void dummy_bootstrap_callback(lcb_INSTANCE *instance, lcb_STATUS err)
{
    (void)instance;
    (void)err;
}

static void dummy_pktfwd_callback(lcb_INSTANCE *instance, const void *cookie, lcb_STATUS err, lcb_PKTFWDRESP *resp)
{
    (void)instance;
    (void)cookie;
    (void)err;
    (void)resp;
}
static void dummy_pktflushed_callback(lcb_INSTANCE *instance, const void *cookie)
{
    (void)instance;
    (void)cookie;
}

typedef union {
    lcb_RESPBASE base;
    lcb_RESPGET get;
    lcb_RESPSTORE store;
    lcb_RESPUNLOCK unlock;
    lcb_RESPCOUNTER arith;
    lcb_RESPREMOVE del;
    lcb_RESPENDURE endure;
    lcb_RESPUNLOCK unl;
    lcb_RESPSTATS stats;
    lcb_RESPMCVERSION mcversion;
    lcb_RESPVERBOSITY verbosity;
    lcb_RESPOBSERVE observe;
    lcb_RESPHTTP http;
    lcb_RESPGETCID getcid;
} uRESP;

static void nocb_fallback(lcb_INSTANCE *instance, int type, const lcb_RESPBASE *response)
{
    (void)instance;
    (void)type;
    (void)response;
}

void lcb_initialize_packet_handlers(lcb_INSTANCE *instance)
{
    instance->callbacks.errmap = lcb_errmap_default;
    instance->callbacks.bootstrap = dummy_bootstrap_callback;
    instance->callbacks.pktflushed = dummy_pktflushed_callback;
    instance->callbacks.pktfwd = dummy_pktfwd_callback;
    instance->callbacks.v3callbacks[LCB_CALLBACK_DEFAULT] = nocb_fallback;
}

#define CALLBACK_ACCESSOR(name, cbtype, field)                                                                         \
    LIBCOUCHBASE_API                                                                                                   \
    cbtype name(lcb_INSTANCE *instance, cbtype cb)                                                                     \
    {                                                                                                                  \
        cbtype ret = instance->callbacks.field;                                                                        \
        if (cb != NULL) {                                                                                              \
            instance->callbacks.field = cb;                                                                            \
        }                                                                                                              \
        return ret;                                                                                                    \
    }

LIBCOUCHBASE_API
lcb_destroy_callback lcb_set_destroy_callback(lcb_INSTANCE *instance, lcb_destroy_callback cb)
{
    lcb_destroy_callback ret = LCBT_SETTING(instance, dtorcb);
    if (cb) {
        LCBT_SETTING(instance, dtorcb) = cb;
    }
    return ret;
}

CALLBACK_ACCESSOR(lcb_set_errmap_callback, lcb_errmap_callback, errmap)
CALLBACK_ACCESSOR(lcb_set_bootstrap_callback, lcb_bootstrap_callback, bootstrap)
CALLBACK_ACCESSOR(lcb_set_pktfwd_callback, lcb_pktfwd_callback, pktfwd)
CALLBACK_ACCESSOR(lcb_set_pktflushed_callback, lcb_pktflushed_callback, pktflushed)
CALLBACK_ACCESSOR(lcb_set_open_callback, lcb_open_callback, open)

LIBCOUCHBASE_API
lcb_RESPCALLBACK lcb_install_callback3(lcb_INSTANCE *instance, int cbtype, lcb_RESPCALLBACK cb)
{
    lcb_RESPCALLBACK ret;
    if (cbtype >= LCB_CALLBACK__MAX) {
        return NULL;
    }

    ret = instance->callbacks.v3callbacks[cbtype];
    instance->callbacks.v3callbacks[cbtype] = cb;
    return ret;
}

LIBCOUCHBASE_API
lcb_RESPCALLBACK lcb_get_callback3(lcb_INSTANCE *instance, int cbtype)
{
    if (cbtype >= LCB_CALLBACK__MAX) {
        return NULL;
    }
    return instance->callbacks.v3callbacks[cbtype];
}

LIBCOUCHBASE_API
const char *lcb_strcbtype(int cbtype)
{
    switch (cbtype) {
        case LCB_CALLBACK_GET:
            return "GET";
        case LCB_CALLBACK_STORE:
            return "STORE";
        case LCB_CALLBACK_COUNTER:
            return "COUNTER";
        case LCB_CALLBACK_TOUCH:
            return "TOUCH";
        case LCB_CALLBACK_REMOVE:
            return "REMOVE";
        case LCB_CALLBACK_UNLOCK:
            return "UNLOCK";
        case LCB_CALLBACK_STATS:
            return "STATS";
        case LCB_CALLBACK_VERSIONS:
            return "VERSIONS";
        case LCB_CALLBACK_VERBOSITY:
            return "VERBOSITY";
        case LCB_CALLBACK_OBSERVE:
            return "OBSERVE";
        case LCB_CALLBACK_GETREPLICA:
            return "GETREPLICA";
        case LCB_CALLBACK_ENDURE:
            return "ENDURE";
        case LCB_CALLBACK_HTTP:
            return "HTTP";
        case LCB_CALLBACK_CBFLUSH:
            return "CBFLUSH";
        case LCB_CALLBACK_OBSEQNO:
            return "OBSEQNO";
        case LCB_CALLBACK_STOREDUR:
            return "STOREDUR";
        case LCB_CALLBACK_SDMUTATE:
            return "SDMUTATE";
        case LCB_CALLBACK_SDLOOKUP:
            return "SDLOOKUP";
        case LCB_CALLBACK_NOOP:
            return "NOOP";
        case LCB_CALLBACK_EXISTS:
            return "EXISTS";
        default:
            return "UNKNOWN";
    }
}

lcb_RESPCALLBACK lcb_find_callback(lcb_INSTANCE *instance, lcb_CALLBACK_TYPE cbtype)
{
    lcb_RESPCALLBACK ret = instance->callbacks.v3callbacks[cbtype];
    if (!ret) {
        ret = instance->callbacks.v3callbacks[LCB_CALLBACK_DEFAULT];
        if (!ret) {
            ret = nocb_fallback;
        }
    }
    return ret;
}
