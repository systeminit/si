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

/**
 * This file contains IO operations that use libev
 *
 * @author Sergey Avseyev
 */
#define LCB_IOPS_V12_NO_DEPRECATE
#include "config.h"
#ifdef HAVE_LIBEV_EV_H
#include <libev/ev.h>
#else
#include <ev.h>
#endif
#include "libev_io_opts.h"
#include <errno.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <libcouchbase/plugins/io/bsdio-inl.c>

struct libev_cookie {
    struct ev_loop *loop;
    int allocated;
    int suspended;
};

struct libev_event {
    union {
        struct ev_io io;
        struct ev_timer timer;
    } ev;
    void *data;
    void (*handler)(lcb_socket_t sock, short which, void *cb_data);
};

static void handler_thunk(struct ev_loop *loop, ev_io *io, int events)
{
    struct libev_event *evt = (struct libev_event *)io;
    int which = 0;

    if (events & EV_READ) {
        which |= LCB_READ_EVENT;
    }
    if (events & EV_WRITE) {
        which |= LCB_WRITE_EVENT;
    }
    evt->handler(io->fd, which, evt->data);

    (void)loop;
}

static void timer_thunk(struct ev_loop *loop, ev_timer *timer, int events)
{
    struct libev_event *evt = (struct libev_event *)timer;
    evt->handler(0, 0, evt->data);
    (void)events;
    (void)loop;
}

static void *lcb_io_create_event(struct lcb_io_opt_st *iops)
{
    struct libev_event *event = calloc(1, sizeof(*event));
    (void)iops;
    return event;
}

static int lcb_io_update_event(struct lcb_io_opt_st *iops, lcb_socket_t sock, void *event, short flags, void *cb_data,
                               void (*handler)(lcb_socket_t sock, short which, void *cb_data))
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    struct libev_event *evt = event;
    int events = EV_NONE;

    if (flags & LCB_READ_EVENT) {
        events |= EV_READ;
    }
    if (flags & LCB_WRITE_EVENT) {
        events |= EV_WRITE;
    }

    if (events == evt->ev.io.events && handler == evt->handler) {
        /* no change! */
        return 0;
    }

    ev_io_stop(io_cookie->loop, &evt->ev.io);
    evt->data = cb_data;
    evt->handler = handler;
    ev_init(&evt->ev.io, handler_thunk);
    ev_io_set(&evt->ev.io, sock, events);
    ev_io_stop(io_cookie->loop, &evt->ev.io);
    ev_io_start(io_cookie->loop, &evt->ev.io);

    return 0;
}
static void lcb_io_delete_event(struct lcb_io_opt_st *iops, lcb_socket_t sock, void *event)
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    struct libev_event *evt = event;
    ev_io_stop(io_cookie->loop, &evt->ev.io);
    ev_io_init(&evt->ev.io, NULL, 0, 0);
    (void)sock;
}

static void lcb_io_destroy_event(struct lcb_io_opt_st *iops, void *event)
{
    lcb_io_delete_event(iops, -1, event);
    free(event);
}

static int lcb_io_update_timer(struct lcb_io_opt_st *iops, void *timer, lcb_uint32_t usec, void *cb_data,
                               void (*handler)(lcb_socket_t sock, short which, void *cb_data))
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    struct libev_event *evt = timer;
    ev_tstamp start;
    evt->data = cb_data;
    evt->handler = handler;
    start = usec / (ev_tstamp)1000000;
    ev_timer_stop(io_cookie->loop, &evt->ev.timer);
    ev_timer_init(&evt->ev.timer, timer_thunk, start, 0);
    ev_timer_start(io_cookie->loop, &evt->ev.timer);
    return 0;
}

static void lcb_io_delete_timer(struct lcb_io_opt_st *iops, void *event)
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    struct libev_event *evt = event;
    ev_timer_stop(io_cookie->loop, &evt->ev.timer);
}

static void lcb_io_destroy_timer(struct lcb_io_opt_st *iops, void *event)
{
    lcb_io_delete_timer(iops, event);
    free(event);
}

static void lcb_io_stop_event_loop(struct lcb_io_opt_st *iops)
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
#ifdef HAVE_LIBEV4
    ev_break(io_cookie->loop, EVBREAK_ONE);
#else
    ev_unloop(io_cookie->loop, EVUNLOOP_ONE);
#endif
}

static void run_common(struct lcb_io_opt_st *iops, int is_tick)
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    int flags;

    io_cookie->suspended = 0;
#ifdef HAVE_LIBEV4
    flags = is_tick ? EVRUN_NOWAIT : 0;
    ev_run(io_cookie->loop, flags);
#else
    flags = is_tick ? EVLOOP_NOBLOCK : 0;
    ev_loop(io_cookie->loop, flags);
#endif
    io_cookie->suspended = 1;
}

static void lcb_io_run_event_loop(struct lcb_io_opt_st *iops)
{
    run_common(iops, 0);
}

static void lcb_io_tick_event_loop(struct lcb_io_opt_st *iops)
{
    run_common(iops, 1);
}

static void lcb_destroy_io_opts(struct lcb_io_opt_st *iops)
{
    struct libev_cookie *io_cookie = iops->v.v2.cookie;
    if (io_cookie->allocated) {
        ev_loop_destroy(io_cookie->loop);
    }
    free(io_cookie);
    free(iops);
}

static void procs2_ev_callback(int version, lcb_loop_procs *loop_procs, lcb_timer_procs *timer_procs,
                               lcb_bsd_procs *bsd_procs, lcb_ev_procs *ev_procs, lcb_completion_procs *completion_procs,
                               lcb_iomodel_t *iomodel)
{
    ev_procs->cancel = lcb_io_delete_event;
    ev_procs->create = lcb_io_create_event;
    ev_procs->watch = lcb_io_update_event;
    ev_procs->destroy = lcb_io_destroy_event;

    timer_procs->create = lcb_io_create_event;
    timer_procs->cancel = lcb_io_delete_timer;
    timer_procs->schedule = lcb_io_update_timer;
    timer_procs->destroy = lcb_io_destroy_timer;

    loop_procs->start = lcb_io_run_event_loop;
    loop_procs->stop = lcb_io_stop_event_loop;
    loop_procs->tick = lcb_io_tick_event_loop;

    *iomodel = LCB_IOMODEL_EVENT;
    wire_lcb_bsd_impl2(bsd_procs, version);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_create_libev_io_opts(int version, lcb_io_opt_t *io, void *arg)
{
    struct ev_loop *loop = arg;
    struct lcb_io_opt_st *ret;
    struct libev_cookie *cookie;
    if (version != 0) {
        return LCB_PLUGIN_VERSION_MISMATCH;
    }
    ret = calloc(1, sizeof(*ret));
    cookie = calloc(1, sizeof(*cookie));
    if (ret == NULL || cookie == NULL) {
        free(ret);
        free(cookie);
        return LCB_CLIENT_ENOMEM;
    }

    /* setup io iops! */
    ret->version = 3;
    ret->dlhandle = NULL;
    ret->destructor = lcb_destroy_io_opts;
    ret->v.v3.get_procs = procs2_ev_callback;

    /* consider that struct isn't allocated by the library,
     * `need_cleanup' flag might be set in lcb_create() */
    ret->v.v3.need_cleanup = 0;

    if (loop == NULL) {
        if ((cookie->loop = ev_loop_new(EVFLAG_AUTO | EVFLAG_NOENV)) == NULL) {
            free(ret);
            free(cookie);
            return LCB_CLIENT_ENOMEM;
        }
        cookie->allocated = 1;
    } else {
        cookie->loop = loop;
        cookie->allocated = 0;
    }
    cookie->suspended = 1;
    ret->v.v3.cookie = cookie;

    wire_lcb_bsd_impl(ret);

    *io = ret;
    return LCB_SUCCESS;
}
