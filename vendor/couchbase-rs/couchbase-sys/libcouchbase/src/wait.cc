/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#include <lcbio/iotable.h>
#include <lcbio/timer-ng.h>

static bool
has_pending(lcb_INSTANCE *instance)
{

    if (!instance->retryq->empty(!LCBT_SETTING(instance, wait_for_config))) {
        return true;
    }

    if (lcb_aspend_pending(&instance->pendops)) {
        return true;
    }

    for (size_t ii = 0; ii < LCBT_NSERVERS(instance); ii++) {
        if (instance->get_server(ii)->has_pending()) {
            return true;
        }
    }
    return false;
}

static void
maybe_reset_timeouts(lcb_INSTANCE *instance)
{

    if (!LCBT_SETTING(instance, readj_ts_wait)) {
        return;
    }

    uint64_t now = lcb_nstime();
    for (size_t ii = 0; ii < LCBT_NSERVERS(instance); ++ii) {
        mcreq_reset_timeouts(instance->get_server(ii), now);
    }
    instance->retryq->reset_timeouts(now);
}

void
lcb_maybe_breakout(lcb_INSTANCE *instance)
{
    if (!instance->wait) {
        return;
    }
    if (has_pending(instance)) {
        return;
    }

    instance->wait = 0;
    instance->iotable->loop.stop(IOT_ARG(instance->iotable));
}

/**
 * Returns non zero if the event loop is running now
 *
 * @param instance the instance to run the event loop for.
 */
LIBCOUCHBASE_API
int lcb_is_waiting(lcb_INSTANCE *instance)
{
    return instance->wait != 0;
}

/**
 * Run the event loop until we've got a response for all of our spooled
 * commands. You should not call this function from within your callbacks.
 *
 * @param instance the instance to run the event loop for.
 *
 * @author Trond Norbye
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_wait(lcb_INSTANCE *instance)
{
    if (instance->wait != 0) {
        return instance->last_error;
    }

    if (!has_pending(instance)) {
        return LCB_SUCCESS;
    }

    maybe_reset_timeouts(instance);
    instance->last_error = LCB_SUCCESS;
    instance->wait = 1;
    IOT_START(instance->iotable);
    instance->wait = 0;

    if (LCBT_VBCONFIG(instance)) {
        return LCB_SUCCESS;
    }

    return instance->last_error;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_tick_nowait(lcb_INSTANCE *instance)
{
    lcb_io_tick_fn tick = instance->iotable->loop.tick;
    if (!tick) {
        return LCB_CLIENT_FEATURE_UNAVAILABLE;
    } else {
        maybe_reset_timeouts(instance);
        tick(IOT_ARG(instance->iotable));
        return LCB_SUCCESS;
    }
}

LIBCOUCHBASE_API
void lcb_wait3(lcb_INSTANCE *instance, lcb_WAITFLAGS flags)
{
    if (flags == LCB_WAIT_DEFAULT) {
        if (instance->wait) {
            return;
        }
        if (has_pending(instance)) {
            return;
        }
    }

    maybe_reset_timeouts(instance);
    instance->wait = 1;
    IOT_START(instance->iotable);
    instance->wait = 0;
}

/**
 * Stop event loop
 *
 * @param instance the instance to run the event loop for.
 */
LIBCOUCHBASE_API
void lcb_breakout(lcb_INSTANCE *instance)
{
    if (instance->wait) {
        IOT_STOP(instance->iotable);
        instance->wait = 0;
    }
}
