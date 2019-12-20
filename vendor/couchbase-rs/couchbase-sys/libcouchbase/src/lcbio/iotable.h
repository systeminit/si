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

#ifndef LCB_IOTABLE_H
#define LCB_IOTABLE_H

#include <libcouchbase/couchbase.h>

/**
 * @file
 * @brief Internal I/O Table routines
 *
 * @details
 * Include this file if you are actually manipulating the I/O system (i.e.
 * creating timers, starting/stoping loops, or writing to/from a socket).
 *
 * This file defines the iotable layout as well as various macros associated
 * with its use. The actual "Public" API (i.e. just for passing it around) can
 * be found in <lcbio/connect.h>
 */

#ifdef __cplusplus
extern "C" {
#endif

/** Whether the underlying model is event-based */
#define IOT_IS_EVENT(iot) ((iot)->model == LCB_IOMODEL_EVENT)

/** Returns an lcb_ev_procs structure for event-based I/O */
#define IOT_V0EV(iot) (iot)->u_io.v0.ev

/** Returns an lcb_bsd_procs structure for event-based I/O */
#define IOT_V0IO(iot) (iot)->u_io.v0.io

/** Returns an lcb_completion_procs structure for completion-based I/O */
#define IOT_V1(iot) (iot)->u_io.completion

/** Error code of last I/O operation */
#define IOT_ERRNO(iot) (iot)->p->v.v0.error

/** Start the loop */
#define IOT_START(iot) (iot)->loop.start((iot)->p)

/** Stop the loop */
#define IOT_STOP(iot) (iot)->loop.stop((iot)->p)

/** First argument to IO Table */
#define IOT_ARG(iot) (iot)->p

typedef struct lcbio_TABLE {
    lcb_io_opt_t p;
    lcb_iomodel_t model;
    lcb_timer_procs timer;
    lcb_loop_procs loop;

    union {
        struct {
            lcb_ev_procs ev;
            lcb_bsd_procs io;
        } v0;
        lcb_completion_procs completion;
    } u_io;
    unsigned refcount;
    void (*dtor)(void *);

#ifdef __cplusplus
    bool is_E() const
    {
        return IOT_IS_EVENT(this);
    }
    bool is_C() const
    {
        return !is_E();
    }
    int get_errno() const
    {
        return IOT_ERRNO(this);
    }

    void run_loop()
    {
        IOT_START(this);
    }
    void stop_loop()
    {
        IOT_STOP(this);
    }

    int E_connect(lcb_socket_t sock, const sockaddr *saddr, unsigned addrlen)
    {
        return IOT_V0IO(this).connect0(p, sock, saddr, addrlen);
    }

    lcb_socket_t E_socket(int domain, int type, int protocol)
    {
        return IOT_V0IO(this).socket0(p, domain, type, protocol);
    }

    lcb_socket_t E_socket(const addrinfo *ai)
    {
        return E_socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
    }

    void E_close(lcb_socket_t sock)
    {
        IOT_V0IO(this).close(p, sock);
    }

    void *E_event_create()
    {
        return IOT_V0EV(this).create(p);
    }

    void E_event_watch(lcb_socket_t fd, void *event, short mask, void *arg, lcb_ioE_callback cb)
    {
        IOT_V0EV(this).watch(p, fd, event, mask, arg, cb);
    }

    void E_event_destroy(void *event)
    {
        IOT_V0EV(this).destroy(p, event);
    }

    void E_event_cancel(lcb_socket_t fd, void *event)
    {
        IOT_V0EV(this).cancel(p, fd, event);
    }

    int E_check_closed(lcb_socket_t s, int flags)
    {
        return IOT_V0IO(this).is_closed(p, s, flags);
    }

    int E_cntl(lcb_socket_t s, int mode, int opt, void *val)
    {
        return IOT_V0IO(this).cntl(p, s, mode, opt, val);
    }

    void C_close(lcb_sockdata_t *sd)
    {
        IOT_V1(this).close(p, sd);
    }

    int C_connect(lcb_sockdata_t *sd, const sockaddr *addr, unsigned addrlen, lcb_io_connect_cb callback)
    {
        return IOT_V1(this).connect(p, sd, addr, addrlen, callback);
    }

    lcb_sockdata_t *C_socket(int domain, int type, int protocol)
    {
        return IOT_V1(this).socket(p, domain, type, protocol);
    }

    lcb_sockdata_t *C_socket(const addrinfo *ai)
    {
        return C_socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
    }

    int C_check_closed(lcb_sockdata_t *sock, int flags)
    {
        return IOT_V1(this).is_closed(p, sock, flags);
    }

    int C_cntl(lcb_sockdata_t *sd, int mode, int opt, void *val)
    {
        return IOT_V1(this).cntl(p, sd, mode, opt, val) == 0;
    }

    bool has_cntl()
    {
        if (is_E()) {
            return IOT_V0IO(this).cntl != NULL;
        } else {
            return IOT_V1(this).cntl != NULL;
        }
    }
#endif

} lcbio_TABLE;

#ifdef __cplusplus
}
#endif
#endif
