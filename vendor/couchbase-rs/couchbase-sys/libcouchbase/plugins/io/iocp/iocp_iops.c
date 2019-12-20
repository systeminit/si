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

/**
 * New-Style v1 plugin for Windows, Using IOCP
 * @author Mark Nunberg
 */

#include "iocp_iops.h"
#include <stdio.h>
#include <stdlib.h>
static int start_write(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, struct lcb_iovec_st *iov, lcb_size_t niov,
                       void *uarg, lcb_ioC_write2_callback callback)
{
    iocp_t *io = (iocp_t *)iobase;
    iocp_write_t *w;
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sockbase;
    int rv;
    DWORD dwNbytes;

    /** Figure out which w we should use */
    if (sd->w_info.state == IOCP_WRITEBUF_AVAILABLE) {
        w = &sd->w_info;
        w->state = IOCP_WRITEBUF_INUSE;
        memset(&w->ol_write.base, 0, sizeof(w->ol_write.base));
    } else {
        w = calloc(1, sizeof(*w));
        lcb_assert(w);
        if (!w) {
            iobase->v.v2.error = WSA_NOT_ENOUGH_MEMORY;
            return -1;
        }

        w->state = IOCP_WRITEBUF_ALLOCATED;
        w->ol_write.action = LCBIOCP_ACTION_WRITE;
        w->ol_write.sd = sd;
    }

    w->cb = callback;
    w->uarg = uarg;

    /* nbytes is ignored here, but mandatory for WSASend() */
    rv = WSASend(sd->sSocket, (WSABUF *)iov, niov, &dwNbytes, 0 /* Flags */, (OVERLAPPED *)&w->ol_write,
                 NULL /* IOCP Callback */);
    rv = iocp_just_scheduled(io, &w->ol_write, rv);
    /* TODO: if write completes immediately, maybe return a special code */
    return rv;
}

static int start_read(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, lcb_IOV *iov, lcb_size_t niov, void *uarg,
                      lcb_ioC_read2_callback callback)
{
    int rv;
    DWORD flags = 0, dwNbytes;
    iocp_t *io = (iocp_t *)iobase;
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sockbase;
    struct lcb_buf_info *bi = &sockbase->read_buffer;

    IOCP_LOG(IOCP_DEBUG, "Read Requested..");
    sd->ol_read.action = LCBIOCP_ACTION_READ;
    sd->rdcb = callback;
    sd->rdarg = uarg;

    /** Remove the leftover bits */
    ZeroMemory(&sd->ol_read.base, sizeof(OVERLAPPED));

    /* nbytes and flags, required as an argument, but unused in our code */
    rv = WSARecv(sd->sSocket, (WSABUF *)iov, niov, &dwNbytes, &flags, (OVERLAPPED *)&sd->ol_read, NULL);

    return iocp_just_scheduled(io, &sd->ol_read, rv);
}

static int start_connect(lcb_io_opt_t iobase, lcb_sockdata_t *sdbase, const struct sockaddr *name, unsigned int namelen,
                         lcb_io_connect_cb callback)
{
    /* In order to use ConnectEx(), the socket must be bound. */
    union {
        struct sockaddr_in in4;
        struct sockaddr_in6 in6;
    } u_addr;
    BOOL result;
    LPFN_CONNECTEX pConnectEx;
    iocp_t *io = (iocp_t *)iobase;
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sdbase;
    iocp_connect_t *conn;

    conn = calloc(1, sizeof(*conn));

    lcb_assert(conn);
    if (conn == NULL) {
        iobase->v.v2.error = WSA_NOT_ENOUGH_MEMORY;
        return -1;
    }

    conn->cb = callback;
    conn->ol_conn.sd = sd;
    conn->ol_conn.action = LCBIOCP_ACTION_CONNECT;
    IOCP_LOG(IOCP_INFO, "Connnection OL=%p", &conn->ol_conn);

    memset(&u_addr, 0, sizeof(u_addr));
    if (namelen == sizeof(u_addr.in4)) {
        u_addr.in4.sin_family = AF_INET;
    } else if (namelen == sizeof(u_addr.in6)) {
        u_addr.in6.sin6_family = AF_INET6;
    } else {
        free(conn);
        iobase->v.v2.error = WSAEINVAL;
        return -1;
    }

    if (bind(sd->sSocket, (const struct sockaddr *)&u_addr, namelen) != 0) {
        iocp_set_last_error(iobase, sd->sSocket);
        free(conn);
        return -1;
    }

    pConnectEx = iocp_initialize_connectex(sd->sSocket);
    if (!pConnectEx) {
        iocp_set_last_error(iobase, INVALID_SOCKET);
        free(conn);
        return -1;
    }

    result = pConnectEx(sd->sSocket, name, namelen, NULL, 0,
                        NULL, /* Optional buffer and length to send when connected. (unused)*/
                        (OVERLAPPED *)conn);

    /** Other functions return 0 to indicate success. Here it's the opposite */
    return iocp_just_scheduled(io, &conn->ol_conn, result == TRUE ? 0 : -1);
}

static lcb_sockdata_t *create_socket(lcb_io_opt_t iobase, int domain, int type, int protocol)
{
    iocp_t *io = (iocp_t *)iobase;
    HANDLE hResult;
    SOCKET s;
    iocp_sockdata_t *sd;

    sd = calloc(1, sizeof(*sd));
    if (sd == NULL) {
        return NULL;
    }

    /* We need to use WSASocket to set the WSA_FLAG_OVERLAPPED option */
    s = WSASocket(domain, type, protocol, NULL /* protocol info */, 0 /* "Group" */, WSA_FLAG_OVERLAPPED);

    if (s == INVALID_SOCKET) {
        iocp_set_last_error(iobase, s);
        free(sd);
        return NULL;
    }

    /**
     * Disable the send buffer. This ensures a callback is invoked.
     * If this is not set, the send operation may complete immediately
     * and our operations may never be queued. The KB article says we should
     * ensure multiple write requests are batched into a single operation, which
     * is something we already do :)
     *
     * See http://support.microsoft.com/kb/181611 for details.
     * Disabled currently until we can get actual benchmark numbers
     */
    if (0) {
        int optval = 0, rv;
        rv = setsockopt(s, SOL_SOCKET, SO_SNDBUF, (const char *)&optval, sizeof optval);
    }

    hResult = CreateIoCompletionPort((HANDLE)s, io->hCompletionPort, (ULONG_PTR)sd, 0 /* nthreads */);

    if (hResult == NULL) {
        iocp_set_last_error(iobase, s);
        closesocket(s);
        free(sd);
        return NULL;
    }

    sd->ol_read.sd = sd;
    sd->refcount = 1;
    sd->sSocket = s;
    sd->sd_base.socket = s; /* Informational, used in tests */

    /** Initialize the write structure */
    sd->w_info.ol_write.sd = sd;
    sd->w_info.state = IOCP_WRITEBUF_AVAILABLE;
    sd->w_info.ol_write.action = LCBIOCP_ACTION_WRITE;

    lcb_list_append(&io->sockets, &sd->list);

    return &sd->sd_base;
}

static unsigned int close_socket(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase)
{
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sockbase;

    if (sd->sSocket != INVALID_SOCKET) {
        closesocket(sd->sSocket);
        sd->sSocket = INVALID_SOCKET;
    }
    iocp_socket_decref((iocp_t *)iobase, sd);
    return 0;
}

static int sock_nameinfo(lcb_io_opt_t iobase, lcb_sockdata_t *sockbase, struct lcb_nameinfo_st *ni)
{
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sockbase;
    getsockname(sd->sSocket, ni->local.name, ni->local.len);
    getpeername(sd->sSocket, ni->remote.name, ni->remote.len);
    return 0;
}

static void *create_timer(lcb_io_opt_t iobase)
{
    (void)iobase;
    return calloc(1, sizeof(iocp_timer_t));
}

static void delete_timer(lcb_io_opt_t iobase, void *opaque)
{
    iocp_timer_t *tmr = (iocp_timer_t *)opaque;
    iocp_t *io = (iocp_t *)iobase;
    if (tmr->is_active) {
        tmr->is_active = 0;
        iocp_tmq_del(&io->timer_queue.list, tmr);
    }
}

static int update_timer(lcb_io_opt_t iobase, void *opaque, lcb_U32 usec, void *arg, lcb_ioE_callback cb)
{
    iocp_t *io = (iocp_t *)iobase;
    iocp_timer_t *tmr = (iocp_timer_t *)opaque;
    lcb_U64 now;

    if (tmr->is_active) {
        iocp_tmq_del(&io->timer_queue.list, tmr);
    }

    tmr->cb = cb;
    tmr->arg = arg;
    tmr->is_active = 1;
    now = iocp_millis();
    tmr->ms = now + (usec / 1000);
    iocp_tmq_add(&io->timer_queue.list, tmr);
    return 0;
}

static void destroy_timer(lcb_io_opt_t iobase, void *opaque)
{
    free(opaque);
    (void)iobase;
}

static int set_nbio(SOCKET s, u_long mode)
{
    int rv;
    rv = ioctlsocket(s, FIONBIO, &mode);
    if (rv == 0) {
        return 0;
    } else {
        fprintf(stderr, "libcouchbase: ioctlsocket => %lu\n", WSAGetLastError());
        return -1;
    }
}

static int check_closed(lcb_io_opt_t io, lcb_sockdata_t *sockbase, int flags)
{
    int rv, err;
    char buf;
    iocp_sockdata_t *sd = (iocp_sockdata_t *)sockbase;
    WSABUF iov;
    DWORD dwReceived, dwFlags = MSG_PEEK;

    /* Currently don't know if IOCP lets use use MSG_PEEK.
     * On the one hand: "This flag is valid only for nonoverlapped sockets".
     * On the other hand:
        > If both lpOverlapped and lpCompletionRoutine are NULL, the socket
        > in this function will be treated as a nonoverlapped socket.
     *
     * Source: http://msdn.microsoft.com/en-us/library/windows/desktop/ms741688(v=vs.85).aspx

     * As a workaround for now, let's just disable this check if we are
     * expecting unsolicited data. It is apparently legal to mix overlapped
     * and non-overlapped calls.
     */

    if ((flags & LCB_IO_SOCKCHECK_PEND_IS_ERROR) == 0) {
        return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
    }

    if (set_nbio(sd->sSocket, 1) != 0) {
        return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
    }

    iov.buf = &buf;
    iov.len = 1;
    rv = WSARecv(sd->sSocket, &iov, 1, &dwReceived, &dwFlags, NULL, NULL);
    err = WSAGetLastError();

    if (set_nbio(sd->sSocket, 0) != 0) {
        return LCB_IO_SOCKCHECK_STATUS_CLOSED;
    }

    if (rv == 0) {
        return LCB_IO_SOCKCHECK_STATUS_CLOSED;
    } else if (err == WSAEWOULDBLOCK) {
        return LCB_IO_SOCKCHECK_STATUS_OK;
    } else {
        return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
    }
}

static void iops_dtor(lcb_io_opt_t iobase)
{
    iocp_t *io = (iocp_t *)iobase;
    lcb_list_t *cur;

    /** Close all sockets first so we can get events for them */
    LCB_LIST_FOR(cur, &io->sockets)
    {
        iocp_sockdata_t *sd;
        sd = LCB_LIST_ITEM(cur, iocp_sockdata_t, list);
        if (sd->sSocket != INVALID_SOCKET) {
            closesocket(sd->sSocket);
            sd->sSocket = INVALID_SOCKET;
        }
    }
    /** Drain the queue. This should not block */
    while (1) {
        DWORD nbytes;
        ULONG_PTR completionKey;
        LPOVERLAPPED pOl;
        iocp_sockdata_t *sd;
        iocp_overlapped_t *ol;

        GetQueuedCompletionStatus(io->hCompletionPort, &nbytes, &completionKey, &pOl, 0 /* Timeout */);
        sd = (iocp_sockdata_t *)completionKey;
        ol = (iocp_overlapped_t *)pOl;

        if (!ol) {
            break;
        }

        if (ol->action == LCBIOCP_ACTION_CONNECT) {
            free(ol);
        } else if (ol->action == LCBIOCP_ACTION_WRITE) {
            iocp_write_done(io, IOCP_WRITEOBJ_FROM_OVERLAPPED(ol), -1);
        } else if (ol->action == LCBIOCP_ACTION_READ) {
            io->base.v.v2.error = WSAECONNRESET;
            sd->rdcb(&sd->sd_base, -1, sd->rdarg);
        }
        iocp_socket_decref(io, sd);
    }

    /* Destroy all remaining sockets */
    LCB_LIST_FOR(cur, &io->sockets)
    {
        iocp_sockdata_t *sd = LCB_LIST_ITEM(cur, iocp_sockdata_t, list);

        IOCP_LOG(IOCP_WARN, "Leak detected in socket %p (%lu). Refcount=%d", sd, sd->sSocket, sd->refcount);
        if (sd->sSocket != INVALID_SOCKET) {
            closesocket(sd->sSocket);
            sd->sSocket = INVALID_SOCKET;
        }
    }

    if (io->hCompletionPort && CloseHandle(io->hCompletionPort)) {
        IOCP_LOG(IOCP_ERR, "Couldn't CloseHandle: %d", GetLastError());
    }
    free(io);
}

static void get_procs(int version, lcb_loop_procs *loop, lcb_timer_procs *timer, lcb_bsd_procs *bsd, lcb_ev_procs *ev,
                      lcb_completion_procs *iocp, lcb_iomodel_t *model)
{
    *model = LCB_IOMODEL_COMPLETION;

    loop->start = iocp_run;
    loop->stop = iocp_stop;

    iocp->connect = start_connect;
    iocp->read2 = start_read;
    iocp->write2 = start_write;
    iocp->socket = create_socket;
    iocp->close = close_socket;
    iocp->nameinfo = sock_nameinfo;
    iocp->is_closed = check_closed;

    timer->create = create_timer;
    timer->cancel = delete_timer;
    timer->schedule = update_timer;
    timer->destroy = destroy_timer;
    (void)ev;
    (void)bsd;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_iocp_new_iops(int version, lcb_io_opt_t *ioret, void *arg)
{
    iocp_t *io;
    lcb_io_opt_t tbl;

    io = calloc(1, sizeof(*io));
    if (!io) {
        return LCB_CLIENT_ENOMEM;
    }

    /** These functions check if they were called more than once using atomic ops */
    iocp_initialize_loop_globals();
    lcb_list_init(&io->timer_queue.list);
    lcb_list_init(&io->sockets);

    tbl = &io->base;
    *ioret = tbl;

    io->breakout = TRUE;

    /** Create IOCP */
    io->hCompletionPort = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);

    if (!io->hCompletionPort) {
        return LCB_EINTERNAL;
    }

    tbl->destructor = iops_dtor;
    tbl->version = 2;
    tbl->v.v2.get_procs = get_procs;

    (void)version;
    (void)arg;

    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
struct lcb_io_opt_st *lcb_create_iocp_io_opts(void)
{
    struct lcb_io_opt_st *ret;
    lcb_iocp_new_iops(0, &ret, NULL);
    return ret;
}
