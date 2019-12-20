/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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

#include "connect.h"
#include "iotable.h"
#include "timer-ng.h"

#define TMR_IS_DESTROYED(timer) ((timer)->state & LCBIO_TIMER_S_DESTROYED)
#define TMR_IS_ARMED(timer) ((timer)->state & LCBIO_TIMER_S_ARMED)

static void destroy_timer(lcbio_TIMER *timer)
{
    if (timer->event) {
        timer->io->timer.destroy(timer->io->p, timer->event);
    }
    lcbio_table_unref(timer->io);
    free(timer);
}

static void timer_callback(lcb_socket_t sock, short which, void *arg)
{
    lcbio_TIMER *timer = arg;

    lcb_assert(TMR_IS_ARMED(timer));
    lcb_assert(!TMR_IS_DESTROYED(timer));
    timer->state |= LCBIO_TIMER_S_ENTERED;

    lcbio_timer_disarm(timer);
    timer->callback(timer->data);

    if (TMR_IS_DESTROYED(timer)) {
        destroy_timer(timer);
    } else {
        timer->state &= ~LCBIO_TIMER_S_ENTERED;
    }

    (void)sock;
    (void)which;
}

lcbio_TIMER *lcbio_timer_new(lcbio_TABLE *io, void *data, lcbio_TIMER_cb callback)
{
    lcbio_TIMER *ret = calloc(1, sizeof(*ret));

    if (!ret) {
        return NULL;
    }

    ret->callback = callback;
    ret->data = data;
    ret->io = io;
    ret->event = io->timer.create(IOT_ARG(io));
    lcbio_table_ref(io);
    return ret;
}

void lcbio_timer_destroy(lcbio_TIMER *timer)
{
    lcbio_timer_disarm(timer);
    if (timer->state & LCBIO_TIMER_S_ENTERED) {
        timer->state |= LCBIO_TIMER_S_DESTROYED;
    } else {
        destroy_timer(timer);
    }
}

void lcbio_timer_disarm(lcbio_TIMER *timer)
{
    if (!TMR_IS_ARMED(timer)) {
        return;
    }

    timer->state &= ~LCBIO_TIMER_S_ARMED;
    timer->io->timer.cancel(timer->io->p, timer->event);
}

void lcbio_timer_rearm(lcbio_TIMER *timer, uint32_t usec)
{
    if (TMR_IS_ARMED(timer)) {
        lcbio_timer_disarm(timer);
    }

    timer->usec_ = usec;
    timer->io->timer.schedule(timer->io->p, timer->event, usec, timer, timer_callback);
    timer->state |= LCBIO_TIMER_S_ARMED;
}

void lcbio_async_signal(lcbio_TIMER *timer)
{
    lcbio_timer_rearm(timer, 0);
}

void lcbio_async_cancel(lcbio_TIMER *timer)
{
    lcbio_timer_disarm(timer);
}

void lcbio_timer_dump(lcbio_TIMER *timer, FILE *fp)
{
    fprintf(fp, "~~ DUMP TIMER BEGIN ~~\n");
    fprintf(fp, "TIMER=%p\n", (void *)timer);
    fprintf(fp, "INNER PTR=%p\n", timer->event);
    fprintf(fp, "USERDATA=%p\n", timer->data);
    fprintf(fp, "ACTIVE: %s\n", (timer->state & LCBIO_TIMER_S_ARMED) ? "YES" : "NO");
    fprintf(fp, "INTERVAL: %lu\n", (unsigned long)timer->usec_);
    fprintf(fp, "~~ DUMP TIMER END ~~\n");
}
