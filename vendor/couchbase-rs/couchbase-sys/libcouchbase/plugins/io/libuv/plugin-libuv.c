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

#include "plugin-internal.h"
#include <libcouchbase/plugins/io/bsdio-inl.c>

static my_uvreq_t *alloc_uvreq(my_sockdata_t *sock, generic_callback_t callback);
static void set_last_error(my_iops_t *io, int error);
static void socket_closed_callback(uv_handle_t *handle);

static void wire_iops2(int version, lcb_loop_procs *loop, lcb_timer_procs *timer, lcb_bsd_procs *bsd, lcb_ev_procs *ev,
                       lcb_completion_procs *iocp, lcb_iomodel_t *model);

static void decref_iops(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;
    lcb_assert(io->iops_refcount);
    if (--io->iops_refcount) {
        return;
    }

    memset(io, 0xff, sizeof(*io));
    free(io);
}

static void iops_lcb_dtor(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;
    if (io->startstop_noop) {
        decref_iops(iobase);
        return;
    }

    while (io->iops_refcount > 1) {
        UVC_RUN_ONCE(io->loop);
    }

    if (io->external_loop == 0) {
        uv_loop_delete(io->loop);
    }

    decref_iops(iobase);
}

/******************************************************************************
 ******************************************************************************
 ** Event Loop Functions                                                     **
 ******************************************************************************
 ******************************************************************************/

#if UV_VERSION < 0x000900
static void do_run_loop(my_iops_t *io)
{
    while (uv_run_once(io->loop) && io->do_stop == 0) {
        /* nothing */
    }
    io->do_stop = 0;
}
static void do_stop_loop(my_iops_t *io)
{
    io->do_stop = 1;
}
#else
static void do_run_loop(my_iops_t *io)
{
    uv_run(io->loop, UV_RUN_DEFAULT);
}
static void do_stop_loop(my_iops_t *io)
{
    uv_stop(io->loop);
}
#endif

static void run_event_loop(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;

    if (!io->startstop_noop) {
        do_run_loop(io);
    }
}

static void tick_event_loop(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;
    if (!io->startstop_noop) {
#if UV_VERSION < 0x000900
        uv_run_once(io->loop);
        io->do_stop = 0;
#else
        uv_run(io->loop, UV_RUN_NOWAIT);
#endif
    }
}

static void stop_event_loop(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;
    if (!io->startstop_noop) {
        do_stop_loop(io);
    }
}

LCBUV_API
lcb_STATUS lcb_create_libuv_io_opts(int version, lcb_io_opt_t *io, lcbuv_options_t *options)
{
    lcb_io_opt_t iop;
    uv_loop_t *loop = NULL;
    my_iops_t *ret;

    if (version != 0) {
        return LCB_PLUGIN_VERSION_MISMATCH;
    }

#ifdef _WIN32
    {
        /** UV unloading on Windows doesn't work well */
        HMODULE module;
        /* We need to provide a symbol */
        static int dummy;
        BOOL result;
        result = GetModuleHandleEx(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                                   (LPCSTR)&dummy, &module);
        if (!result) {
            return LCB_EINTERNAL;
        }
    }
#endif

    ret = (my_iops_t *)calloc(1, sizeof(*ret));

    if (!ret) {
        return LCB_CLIENT_ENOMEM;
    }

    iop = &ret->base;
    iop->version = 2;
    iop->destructor = iops_lcb_dtor;
    iop->v.v2.get_procs = wire_iops2;

    ret->iops_refcount = 1;

    *io = iop;
    if (options) {
        if (options->v.v0.loop) {
            ret->external_loop = 1;
            loop = options->v.v0.loop;
        }
        ret->startstop_noop = options->v.v0.startsop_noop;
    }

    if (!loop) {
        loop = uv_loop_new();
    }

    ret->loop = loop;

    return LCB_SUCCESS;
}

#define SOCK_INCR_PENDING(s, fld) (s)->pending.fld++
#define SOCK_DECR_PENDING(s, fld) (s)->pending.fld--

#ifdef DEBUG
static void sock_dump_pending(my_sockdata_t *sock)
{
    printf("Socket %p:\n", (void *)sock);
    printf("\tRead: %d\n", sock->pending.read);
    printf("\tWrite: %d\n", sock->pending.write);
}
#endif

static void sock_do_uv_close(my_sockdata_t *sock)
{
    if (!sock->uv_close_called) {
        sock->uv_close_called = 1;
        uv_close((uv_handle_t *)&sock->tcp, socket_closed_callback);
    }
}

static void decref_sock(my_sockdata_t *sock)
{
    lcb_assert(sock->refcount);

    if (--sock->refcount) {
        return;
    }
    sock_do_uv_close(sock);
}

#define incref_sock(sd) (sd)->refcount++

/******************************************************************************
 ******************************************************************************
 ** Socket Functions                                                         **
 ******************************************************************************
 ******************************************************************************/
static lcb_sockdata_t *create_socket(lcb_io_opt_t iobase, int domain, int type, int protocol)
{
    my_sockdata_t *ret;
    my_iops_t *io = (my_iops_t *)iobase;

    ret = (my_sockdata_t *)calloc(1, sizeof(*ret));
    if (!ret) {
        return NULL;
    }

    uv_tcp_init(io->loop, &ret->tcp.t);
    ret->base.socket = INVALID_SOCKET;

    incref_iops(io);
    incref_sock(ret);

    set_last_error(io, 0);

    (void)domain;
    (void)type;
    (void)protocol;

    return (lcb_sockdata_t *)ret;
}

/**
 * This one is called from uv_close
 */
static void socket_closed_callback(uv_handle_t *handle)
{
    my_sockdata_t *sock = PTR_FROM_FIELD(my_sockdata_t, handle, tcp);
    my_iops_t *io = (my_iops_t *)sock->base.parent;

    if (sock->pending.read) {
        CbREQ (&sock->tcp)(&sock->base, -1, sock->rdarg);
    }

    memset(sock, 0xEE, sizeof(*sock));
    free(sock);

    decref_iops(&io->base);
}

static unsigned int close_socket(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase)
{
    my_sockdata_t *sock = (my_sockdata_t *)sockbase;
    sock->uv_close_called = 1;
    uv_close((uv_handle_t *)&sock->tcp, socket_closed_callback);
    (void)iobase;
    return 0;
}

static int cntl_socket(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, int mode, int option, void *arg)
{
    my_sockdata_t *sd = (my_sockdata_t *)sockbase;
    int rv;

    switch (option) {
        case LCB_IO_CNTL_TCP_NODELAY:
            if (mode == LCB_IO_CNTL_SET) {
                rv = uv_tcp_nodelay(&sd->tcp.t, *(int *)arg);
                if (rv != 0) {
                    set_last_error((my_iops_t *)iobase, rv);
                }
                return rv;
            } else {
                LCB_IOPS_ERRNO(iobase) = ENOTSUP;
                return -1;
            }
        default:
            LCB_IOPS_ERRNO(iobase) = ENOTSUP;
            return -1;
    }
}

/******************************************************************************
 ******************************************************************************
 ** Connection Functions                                                     **
 ******************************************************************************
 ******************************************************************************/
static void connect_callback(uv_connect_t *req, int status)
{
    my_uvreq_t *uvr = (my_uvreq_t *)req;

    set_last_error((my_iops_t *)uvr->socket->base.parent, status);

    if (uvr->cb.conn) {
        uvr->cb.conn(&uvr->socket->base, status);
    }

    decref_sock(uvr->socket);
    free(uvr);
}

static int start_connect(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, const struct sockaddr *name,
                         unsigned int namelen, lcb_io_connect_cb callback)
{
    my_sockdata_t *sock = (my_sockdata_t *)sockbase;
    my_iops_t *io = (my_iops_t *)iobase;
    my_uvreq_t *uvr;
    int ret;
    int err_is_set = 0;

    uvr = alloc_uvreq(sock, (generic_callback_t)callback);
    if (!uvr) {
        return -1;
    }

    if (namelen == sizeof(struct sockaddr_in)) {
        ret = UVC_TCP_CONNECT(&uvr->uvreq.conn, &sock->tcp.t, name, connect_callback);

    } else if (namelen == sizeof(struct sockaddr_in6)) {
        ret = UVC_TCP_CONNECT6(&uvr->uvreq.conn, &sock->tcp.t, name, connect_callback);

    } else {
        io->base.v.v1.error = EINVAL;
        ret = -1;
        err_is_set = 1;
    }

    if (ret) {
        if (!err_is_set) {
            set_last_error(io, ret);
        }

        free(uvr);

    } else {
        incref_sock(sock);
    }

#if UV_VERSION_HEX >= 0x010000
    {
        uv_os_fd_t fd = (uv_os_fd_t)INVALID_SOCKET;
        /* Fetch socket descriptor for internal usage.
         * For example to detect dead sockets. */
        ret = uv_fileno((uv_handle_t *)&sock->tcp, &fd);
        if (ret == 0) {
            sock->base.socket = (lcb_socket_t)fd;
        }
    }
#endif

    return ret;
}

/******************************************************************************
 ******************************************************************************
 ** Write Functions                                                          **
 ******************************************************************************
 ******************************************************************************/
static void write2_callback(uv_write_t *req, int status)
{
    my_write_t *mw = (my_write_t *)req;
    my_sockdata_t *sock = mw->sock;

    if (status != 0) {
        set_last_error((my_iops_t *)sock->base.parent, status);
    }

    mw->callback(&sock->base, status, mw->w.data);
    free(mw);
}

static int start_write2(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, struct lcb_iovec_st *iov, lcb_size_t niov,
                        void *uarg, lcb_ioC_write2_callback callback)
{
    my_write_t *w;
    my_sockdata_t *sd = (my_sockdata_t *)sockbase;
    int ret;

    w = (my_write_t *)calloc(1, sizeof(*w));
    w->w.data = uarg;
    w->callback = callback;
    w->sock = sd;

    ret = uv_write(&w->w, (uv_stream_t *)&sd->tcp, (uv_buf_t *)iov, niov, write2_callback);

    if (ret != 0) {
        free(w);
        set_last_error((my_iops_t *)iobase, -1);
    }

    return ret;
}

/******************************************************************************
 ******************************************************************************
 ** Read Functions                                                           **
 ******************************************************************************
 ******************************************************************************/

/**
 * Currently we support a single IOV. In theory while we could support
 * multiple IOVs, two problems arise:
 *
 * (1) Because UV does not guarantee that it'll utilize the first IOV completely
 *     we may end up having a gap of unused space between IOVs. This may be
 *     resolved by keeping an offset into the last-returned IOV and then
 *     determining how much of this data was actually populated by UV itself.
 *
 * (2) In the event of an error, UV gives us "Undefined" behavior if we try
 *     to utilize the socket again. The IOPS policy dictates that we deliver
 *     any outstanding data to libcouchbase and _then_ deliver the pending
 *     error. If we are forced to do this all in a single go, we'd be forced
 *     to set up an 'async handle' to deliver the pending error, complicating
 *     our code paths.
 */

static UVC_ALLOC_CB(alloc_cb)
{
    UVC_ALLOC_CB_VARS()

    my_sockdata_t *sock = PTR_FROM_FIELD(my_sockdata_t, handle, tcp);
    buf->base = (char *)sock->iov.iov_base;
    buf->len = sock->iov.iov_len;

    (void)suggested_size;
    UVC_ALLOC_CB_RETURN();
}

static UVC_READ_CB(read_cb)
{
    UVC_READ_CB_VARS()

    my_tcp_t *mt = (my_tcp_t *)stream;
    my_sockdata_t *sock = PTR_FROM_FIELD(my_sockdata_t, mt, tcp);
    my_iops_t *io = (my_iops_t *)sock->base.parent;
    lcb_ioC_read2_callback callback = CbREQ(mt);

    if (nread == 0) {
        /* we have a fixed IOV between requests, so just retry again */
        return;
    }

    /**
     * XXX:
     * For multi-IOV support, we would require a counter to determine if this
     * EAGAIN is spurious (i.e. no previous data in buffer), or actual. In
     * the case of the former, we'd retry -- but in the latter it is a signal
     * that there is no more pending data within the socket buffer AND we have
     * outstanding data to deliver back to the caller.
     */
    SOCK_DECR_PENDING(sock, read);
    uv_read_stop(stream);
    CbREQ(mt) = NULL;

    if (nread < 0) {
        set_last_error(io, uvc_last_errno(io->loop, nread));
        if (uvc_is_eof(io->loop, nread)) {
            nread = 0;
        }
    }
    callback(&sock->base, nread, sock->rdarg);
    decref_sock(sock);
    (void)buf;
}

static int start_read(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, lcb_IOV *iov, lcb_size_t niov, void *uarg,
                      lcb_ioC_read2_callback callback)
{
    my_sockdata_t *sock = (my_sockdata_t *)sockbase;
    my_iops_t *io = (my_iops_t *)iobase;
    int ret;

    sock->iov = *iov;
    sock->rdarg = uarg;
    sock->tcp.callback = callback;

    ret = uv_read_start((uv_stream_t *)&sock->tcp.t, alloc_cb, read_cb);
    set_last_error(io, ret);

    if (ret == 0) {
        SOCK_INCR_PENDING(sock, read);
        incref_sock(sock);
    }
    return ret;
}

static int get_nameinfo(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, struct lcb_nameinfo_st *ni)
{
    my_sockdata_t *sock = (my_sockdata_t *)sockbase;
    my_iops_t *io = (my_iops_t *)iobase;
    uv_tcp_getpeername(&sock->tcp.t, ni->remote.name, ni->remote.len);
    uv_tcp_getsockname(&sock->tcp.t, ni->local.name, ni->local.len);

    (void)io;
    return 0;
}

/******************************************************************************
 ******************************************************************************
 ** Timer Functions                                                          **
 ** There are just copied from the old couchnode I/O code                    **
 ******************************************************************************
 ******************************************************************************/
static UVC_TIMER_CB(timer_cb)
{
    my_timer_t *mytimer = (my_timer_t *)timer;
    if (mytimer->callback) {
        mytimer->callback(-1, 0, mytimer->cb_arg);
    }
}

static void *create_timer(lcb_io_opt_t iobase)
{
    my_iops_t *io = (my_iops_t *)iobase;
    my_timer_t *timer = (my_timer_t *)calloc(1, sizeof(*timer));
    if (!timer) {
        return NULL;
    }

    timer->parent = io;
    incref_iops(io);
    uv_timer_init(io->loop, &timer->uvt);

    return timer;
}

static int update_timer(lcb_io_opt_t iobase, void *timer_opaque, lcb_uint32_t usec, void *cbdata,
                        v0_callback_t callback)
{
    my_timer_t *timer = (my_timer_t *)timer_opaque;

    timer->callback = callback;
    timer->cb_arg = cbdata;

    (void)iobase;

    return uv_timer_start(&timer->uvt, timer_cb, usec / 1000, 0);
}

static void delete_timer(lcb_io_opt_t iobase, void *timer_opaque)
{
    my_timer_t *timer = (my_timer_t *)timer_opaque;

    uv_timer_stop(&timer->uvt);
    timer->callback = NULL;

    (void)iobase;
}

static void timer_close_cb(uv_handle_t *handle)
{
    my_timer_t *timer = (my_timer_t *)handle;
    decref_iops(&timer->parent->base);
    memset(timer, 0xff, sizeof(*timer));
    free(timer);
}

static void destroy_timer(lcb_io_opt_t io, void *timer_opaque)
{
    delete_timer(io, timer_opaque);
    uv_close((uv_handle_t *)timer_opaque, timer_close_cb);
}

static my_uvreq_t *alloc_uvreq(my_sockdata_t *sock, generic_callback_t callback)
{
    my_uvreq_t *ret = (my_uvreq_t *)calloc(1, sizeof(*ret));
    if (!ret) {
        sock->base.parent->v.v1.error = ENOMEM;
        return NULL;
    }
    ret->socket = sock;
    ret->cb.cb_ = callback;
    return ret;
}

static void set_last_error(my_iops_t *io, int error)
{
    io->base.v.v1.error = uvc_last_errno(io->loop, error);
}

#if UV_VERSION_HEX >= 0x010000
static int check_closed(lcb_io_opt_t io, lcb_sockdata_t *sockbase, int flags)
{
    my_sockdata_t *sd = (my_sockdata_t *)sockbase;

    char buf = 0;
    int rv = 0;
    lcb_socket_t sock = sd->base.socket;

    if (sock == INVALID_SOCKET) {
        return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
    }

GT_RETRY:
    /* We can ignore flags for now, since both Windows and POSIX support MSG_PEEK */
    rv = recv(sock, &buf, 1, MSG_PEEK);
    if (rv == 1) {
        if (flags & LCB_IO_SOCKCHECK_PEND_IS_ERROR) {
            return LCB_IO_SOCKCHECK_STATUS_CLOSED;
        } else {
            return LCB_IO_SOCKCHECK_STATUS_OK;
        }
    } else if (rv == 0) {
        /* Really closed! */
        return LCB_IO_SOCKCHECK_STATUS_CLOSED;
    } else {
        int last_err;
#ifdef _WIN32
        last_err = get_wserr(sock);
#else
        last_err = errno;
#endif

        if (last_err == EINTR) {
            goto GT_RETRY;
        } else if (last_err == EWOULDBLOCK || last_err == EAGAIN) {
            return LCB_IO_SOCKCHECK_STATUS_OK; /* Nothing to report. So we're good */
        } else {
            return LCB_IO_SOCKCHECK_STATUS_CLOSED;
        }
    }
}
#endif

static void wire_iops2(int version, lcb_loop_procs *loop, lcb_timer_procs *timer, lcb_bsd_procs *bsd, lcb_ev_procs *ev,
                       lcb_completion_procs *iocp, lcb_iomodel_t *model)
{
    *model = LCB_IOMODEL_COMPLETION;
    loop->start = run_event_loop;
    loop->stop = stop_event_loop;
    loop->tick = tick_event_loop;

    timer->create = create_timer;
    timer->cancel = delete_timer;
    timer->schedule = update_timer;
    timer->destroy = destroy_timer;

    iocp->close = close_socket;
    iocp->socket = create_socket;
    iocp->connect = start_connect;
    iocp->nameinfo = get_nameinfo;
    iocp->read2 = start_read;
    iocp->write2 = start_write2;
    iocp->cntl = cntl_socket;
#if UV_VERSION_HEX >= 0x010000
    iocp->is_closed = check_closed;
#endif

    /** Stuff we don't use */
    iocp->write = NULL;
    iocp->wballoc = NULL;
    iocp->wbfree = NULL;
    iocp->serve = NULL;

    (void)bsd;
    (void)version;
    (void)ev;
}
