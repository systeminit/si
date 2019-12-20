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

#define LCB_IOPS_V12_NO_DEPRECATE

#include "internal.h"
#include "select_io_opts.h"
#include <libcouchbase/plugins/io/bsdio-inl.c>

#if defined(_WIN32) && !defined(usleep)
#define usleep(n) Sleep((n) / 1000)
#endif

typedef struct sel_EVENT sel_EVENT;
struct sel_EVENT {
    lcb_list_t list;
    lcb_socket_t sock;
    short flags;
    short eflags; /* effective flags */
    void *cb_data;
    lcb_ioE_callback handler;
    sel_EVENT *next; /* for chaining active events */
};

typedef struct sel_TIMER sel_TIMER;
struct sel_TIMER {
    lcb_list_t list;
    int active;
    hrtime_t exptime;
    void *cb_data;
    lcb_ioE_callback handler;
};

typedef struct {
    sel_EVENT events;
    lcb_list_t timers;
    int event_loop;
} sel_LOOP;

static int timer_cmp_asc(lcb_list_t *a, lcb_list_t *b)
{
    sel_TIMER *ta = LCB_LIST_ITEM(a, sel_TIMER, list);
    sel_TIMER *tb = LCB_LIST_ITEM(b, sel_TIMER, list);
    if (ta->exptime > tb->exptime) {
        return 1;
    } else if (ta->exptime < tb->exptime) {
        return -1;
    } else {
        return 0;
    }
}

static void *sel_event_new(lcb_io_opt_t iops)
{
    sel_LOOP *io = iops->v.v2.cookie;
    sel_EVENT *ret = calloc(1, sizeof(sel_EVENT));
    if (ret != NULL) {
        lcb_list_append(&io->events.list, &ret->list);
    }
    return ret;
}

static int sel_event_update(lcb_io_opt_t iops, lcb_socket_t sock, void *event, short flags, void *cb_data,
                            lcb_ioE_callback handler)
{
    sel_EVENT *ev = event;
    ev->sock = sock;
    ev->handler = handler;
    ev->cb_data = cb_data;
    ev->flags = flags;
    (void)iops;
    return 0;
}

static void sel_event_free(lcb_io_opt_t iops, void *event)
{
    sel_EVENT *ev = event;
    lcb_list_delete(&ev->list);
    free(ev);
    (void)iops;
}

static void sel_event_cancel(lcb_io_opt_t iops, lcb_socket_t sock, void *event)
{
    sel_EVENT *ev = event;
    ev->flags = 0;
    ev->cb_data = NULL;
    ev->handler = NULL;
    (void)iops;
    (void)sock;
}

static void *sel_timer_new(lcb_io_opt_t iops)
{
    sel_TIMER *ret = calloc(1, sizeof(sel_TIMER));
    (void)iops;
    return ret;
}

static void sel_timer_cancel(lcb_io_opt_t iops, void *timer)
{
    sel_TIMER *tm = timer;
    if (tm->active) {
        tm->active = 0;
        lcb_list_delete(&tm->list);
    }
    (void)iops;
}

static void sel_timer_free(lcb_io_opt_t iops, void *timer)
{
    sel_timer_cancel(iops, timer);
    free(timer);
    (void)iops;
}

static int sel_timer_schedule(lcb_io_opt_t iops, void *timer, lcb_U32 usec, void *cb_data, lcb_ioE_callback handler)
{
    sel_TIMER *tm = timer;
    sel_LOOP *cookie = iops->v.v2.cookie;
    lcb_assert(!tm->active);
    tm->exptime = gethrtime() + (usec * (hrtime_t)1000);
    tm->cb_data = cb_data;
    tm->handler = handler;
    tm->active = 1;
    lcb_list_add_sorted(&cookie->timers, &tm->list, timer_cmp_asc);

    (void)iops;
    return 0;
}

static void sel_stop_loop(struct lcb_io_opt_st *iops)
{
    sel_LOOP *io = iops->v.v2.cookie;
    io->event_loop = 0;
}

static sel_TIMER *pop_next_timer(sel_LOOP *cookie, hrtime_t now)
{
    sel_TIMER *ret;

    if (LCB_LIST_IS_EMPTY(&cookie->timers)) {
        return NULL;
    }

    ret = LCB_LIST_ITEM(cookie->timers.next, sel_TIMER, list);
    if (ret->exptime > now) {
        return NULL;
    }
    lcb_list_shift(&cookie->timers);
    ret->active = 0;
    return ret;
}

static int get_next_timeout(sel_LOOP *cookie, struct timeval *tmo, hrtime_t now)
{
    sel_TIMER *first;
    hrtime_t delta;

    if (LCB_LIST_IS_EMPTY(&cookie->timers)) {
        tmo->tv_sec = 0;
        tmo->tv_usec = 0;
        return 0;
    }

    first = LCB_LIST_ITEM(cookie->timers.next, sel_TIMER, list);
    if (now < first->exptime) {
        delta = first->exptime - now;
    } else {
        delta = 0;
    }

    if (delta) {
        delta /= 1000;
        tmo->tv_sec = (long)(delta / 1000000);
        tmo->tv_usec = delta % 1000000;
    } else {
        tmo->tv_sec = 0;
        tmo->tv_usec = 0;
    }
    return 1;
}

static void run_loop(sel_LOOP *io, int is_tick)
{
    sel_EVENT *ev;
    lcb_list_t *ii;

    fd_set readfds, writefds, exceptfds;

    io->event_loop = !is_tick;
    do {
        struct timeval tmo, *t;
        int ret;
        int nevents = 0;
        int has_timers;
        lcb_socket_t fdmax = 0;
        hrtime_t now;

        t = NULL;
        now = gethrtime();

        FD_ZERO(&readfds);
        FD_ZERO(&writefds);
        FD_ZERO(&exceptfds);

        LCB_LIST_FOR(ii, &io->events.list)
        {
            ev = LCB_LIST_ITEM(ii, sel_EVENT, list);
            if (ev->flags != 0) {
                if (ev->flags & LCB_READ_EVENT) {
                    FD_SET(ev->sock, &readfds);
                }

                if (ev->flags & LCB_WRITE_EVENT) {
                    FD_SET(ev->sock, &writefds);
                }

                FD_SET(ev->sock, &exceptfds);
                if (ev->sock > fdmax) {
                    fdmax = ev->sock;
                }
                ++nevents;
            }
        }

        has_timers = get_next_timeout(io, &tmo, now);
        if (has_timers) {
            t = &tmo;
        } else if (is_tick) {
            /* do not wait forever on tick */
            tmo.tv_sec = 0;
            tmo.tv_usec = LCB_MS2US(100);
            t = &tmo;
        }

        if (nevents == 0 && has_timers == 0) {
            io->event_loop = 0;
            return;
        }

        if (nevents) {
            ret = select(fdmax + 1, &readfds, &writefds, &exceptfds, t);
            if (ret == SOCKET_ERROR) {
                return;
            }
        } else {
            ret = 0;
            if (!is_tick) {
                usleep((t->tv_sec * 1000000) + t->tv_usec);
            }
        }

        /** Always invoke the pending timers */
        if (has_timers) {
            sel_TIMER *tm;
            now = gethrtime();

            while ((tm = pop_next_timer(io, now))) {
                tm->handler(-1, 0, tm->cb_data);
            }
        }

        /* To be completely safe, we need to copy active events
         * before handing them. Iterating over the list of
         * registered events isn't safe, because one callback can
         * cancel all registered events before iteration will end
         */

        if (ret && nevents) {
            sel_EVENT *active = NULL;
            LCB_LIST_FOR(ii, &io->events.list)
            {
                ev = LCB_LIST_ITEM(ii, sel_EVENT, list);
                if (ev->flags != 0) {
                    ev->eflags = 0;
                    if (FD_ISSET(ev->sock, &readfds)) {
                        ev->eflags |= LCB_READ_EVENT;
                    }
                    if (FD_ISSET(ev->sock, &writefds)) {
                        ev->eflags |= LCB_WRITE_EVENT;
                    }
                    if (FD_ISSET(ev->sock, &exceptfds)) {
                        ev->eflags = LCB_ERROR_EVENT | LCB_RW_EVENT; /** It should error */
                    }
                    if (ev->eflags != 0) {
                        ev->next = active;
                        active = ev;
                    }
                }
            }
            ev = active;
            while (ev) {
                sel_EVENT *p = ev->next;
                ev->handler(ev->sock, ev->eflags, ev->cb_data);
                ev = p;
            }
        }
    } while (io->event_loop);
}

static void sel_run_loop(struct lcb_io_opt_st *iops)
{
    run_loop(iops->v.v2.cookie, 0);
}
static void sel_tick_loop(struct lcb_io_opt_st *iops)
{
    run_loop(iops->v.v2.cookie, 1);
}

static void sel_destroy_iops(struct lcb_io_opt_st *iops)
{
    sel_LOOP *io = iops->v.v2.cookie;
    lcb_list_t *nn, *ii;
    sel_EVENT *ev;
    sel_TIMER *tm;

    if (io->event_loop != 0) {
        fprintf(stderr, "WARN: libcouchbase(plugin-select): the event loop might be still active, but it still try to "
                        "free resources\n");
    }
    LCB_LIST_SAFE_FOR(ii, nn, &io->events.list)
    {
        ev = LCB_LIST_ITEM(ii, sel_EVENT, list);
        sel_event_free(iops, ev);
    }
    lcb_assert(LCB_LIST_IS_EMPTY(&io->events.list));
    LCB_LIST_SAFE_FOR(ii, nn, &io->timers)
    {
        tm = LCB_LIST_ITEM(ii, sel_TIMER, list);
        sel_timer_free(iops, tm);
    }
    lcb_assert(LCB_LIST_IS_EMPTY(&io->timers));
    free(io);
    free(iops);
}

static lcb_socket_t sel_socket_wrap(lcb_io_opt_t io, int domain, int type, int protocol)
{
    lcb_socket_t res = socket_impl(io, domain, type, protocol);
#ifndef _WIN32

    /* This only works on non-Windows where FD_SETSIZE is in effect wrt the
     * actual FD number. On Windows, FD_SETSIZE is the cap on the _total_
     * number of sockets to be used in select; not necessarily what their
     * FD values are.
     *
     * TODO: Just use poll() on POSIX in the future.
     */
    if (res != INVALID_SOCKET && res > FD_SETSIZE) {
        close_impl(io, res);
        fprintf(stderr, "COUCHBASE: too many FDs. Cannot have socket > FD_SETSIZE. Use other I/O plugin\n");
        io->v.v3.error = EINVAL;
        res = INVALID_SOCKET;
    }
#endif
    return res;
}

static void procs2_sel_callback(int version, lcb_loop_procs *loop_procs, lcb_timer_procs *timer_procs,
                                lcb_bsd_procs *bsd_procs, lcb_ev_procs *ev_procs,
                                lcb_completion_procs *completion_procs, lcb_iomodel_t *iomodel)
{
    ev_procs->create = sel_event_new;
    ev_procs->destroy = sel_event_free;
    ev_procs->watch = sel_event_update;
    ev_procs->cancel = sel_event_cancel;

    timer_procs->create = sel_timer_new;
    timer_procs->destroy = sel_timer_free;
    timer_procs->schedule = sel_timer_schedule;
    timer_procs->cancel = sel_timer_cancel;

    loop_procs->start = sel_run_loop;
    loop_procs->stop = sel_stop_loop;
    loop_procs->tick = sel_tick_loop;

    *iomodel = LCB_IOMODEL_EVENT;
    wire_lcb_bsd_impl2(bsd_procs, version);

    /* Override */
    bsd_procs->socket0 = sel_socket_wrap;
    (void)completion_procs;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_create_select_io_opts(int version, lcb_io_opt_t *io, void *arg)
{
    lcb_io_opt_t ret;
    sel_LOOP *cookie;

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
    lcb_list_init(&cookie->events.list);
    lcb_list_init(&cookie->timers);

    /* setup io iops! */
    ret->version = 3;
    ret->dlhandle = NULL;
    ret->destructor = sel_destroy_iops;

    /* consider that struct isn't allocated by the library,
     * `need_cleanup' flag might be set in lcb_create() */
    ret->v.v3.need_cleanup = 0;
    ret->v.v3.get_procs = procs2_sel_callback;
    ret->v.v3.cookie = cookie;

    /* For backwards compatibility */
    wire_lcb_bsd_impl(ret);

    *io = ret;
    (void)arg;
    return LCB_SUCCESS;
}
