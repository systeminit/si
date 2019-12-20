/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2013-2019 Couchbase, Inc.
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

#ifndef LCBUV_PLUGIN_INTERNAL_H
#define LCBUV_PLUGIN_INTERNAL_H

#include <libcouchbase/couchbase.h>
#include <uv.h>

#include <errno.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#ifndef _WIN32
#include <unistd.h>
#endif

#include "libuv_compat.h"

#ifdef LCBUV_EMBEDDED_SOURCE
#include <libcouchbase/libuv_io_opts.h>
#else
/** Load definitions from inside */
#include "libuv_io_opts.h"
#endif

typedef void (*v0_callback_t)(lcb_socket_t, short, void *);
typedef void (*generic_callback_t)(void);

/**
 * These structures come about the limitation that with -Werror -Wextra
 * compilation fails because strictly speaking, a function pointer isn't
 * convertible to a normal pointer.
 */

/**
 * Macro - sometimes we might want to use the ->data field?
 */
#define CbREQ(mr) (mr)->callback
typedef struct {
    uv_tcp_t t;
    lcb_ioC_read2_callback callback;
} my_tcp_t;

/**
 * Wrapper for lcb_sockdata_t
 */
typedef struct {
    lcb_sockdata_t base;

    /**
     * UV tcp handle. This is also a uv_stream_t.
     * ->data field contains the read callback
     */
    my_tcp_t tcp;

    /** Reference count */
    unsigned int refcount;

    /** Flag indicating whether uv_close has already been called  on the handle */
    unsigned char uv_close_called;

    lcb_IOV iov;
    void *rdarg;

    struct {
        int read;
        int write;
    } pending;

} my_sockdata_t;

typedef struct {
    uv_write_t w;
    lcb_ioC_write2_callback callback;
    my_sockdata_t *sock;
} my_write_t;

typedef struct {
    struct lcb_io_opt_st base;
    uv_loop_t *loop;

    /** Refcount. When this hits zero we free this */
    unsigned int iops_refcount;

    /** Whether using a user-initiated loop */
    int external_loop;

    /** whether start/stop are noops */
    int startstop_noop;

    /** for 0.8 only, whether to stop */
    int do_stop;
} my_iops_t;

typedef struct {
    uv_timer_t uvt;
    v0_callback_t callback;
    void *cb_arg;
    my_iops_t *parent;
} my_timer_t;

typedef struct {
    union {
        uv_connect_t conn;
        uv_idle_t idle;
    } uvreq;

    union {
        lcb_io_connect_cb conn;
        generic_callback_t cb_;
    } cb;

    my_sockdata_t *socket;
} my_uvreq_t;

/******************************************************************************
 ******************************************************************************
 ** Common Macros                                                            **
 ******************************************************************************
 ******************************************************************************/
#define PTR_FROM_FIELD(t, p, fld) ((t *)(void *)((char *)p - (offsetof(t, fld))))

#define incref_iops(io) (io)->iops_refcount++

#ifdef _WIN32
typedef ULONG lcb_uvbuf_len_t;
#else
typedef size_t lcb_uvbuf_len_t;
#endif

#ifndef INVALID_SOCKET
#define INVALID_SOCKET -1
#endif

#endif
