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

/**
 * Inline routines for common 'BSD'-style I/O for plugins.
 *
 * Include this file in your plugin and then call wire_lcb_bsd_impl2 on the
 * plugin instance.
 */

#ifndef _WIN32
#include <netinet/in.h>
#include <netinet/tcp.h>
#endif

static void wire_lcb_bsd_impl2(lcb_bsd_procs*,int);

#ifdef _WIN32
#include "wsaerr-inl.c"
static int
get_wserr(lcb_socket_t sock)
{
    DWORD error = WSAGetLastError();
    int ext = 0;
    int len = sizeof(ext);

    /* Retrieves extended error status and clear */
    getsockopt(sock, SOL_SOCKET, SO_ERROR, (char *)&ext, &len);
    return wsaerr_map_impl(error);
}

static lcb_ssize_t
recvv_impl(lcb_io_opt_t iops, lcb_socket_t sock,
           struct lcb_iovec_st *iov, lcb_size_t niov)
{
    DWORD flags = 0, nr;
    WSABUF *bufptr = (WSABUF *)iov;

    if (WSARecv(sock, bufptr, niov, &nr, &flags, NULL, NULL) == SOCKET_ERROR) {
        LCB_IOPS_ERRNO(iops) = get_wserr(sock);
        if (LCB_IOPS_ERRNO(iops) == ECONNRESET) {
            return 0;
        }
        return -1;
    }

    (void)iops;
    return (lcb_ssize_t)nr;
}

static lcb_ssize_t
recv_impl(lcb_io_opt_t iops, lcb_socket_t sock, void *buf, lcb_size_t nbuf,
          int fl_unused)
{
    WSABUF iov;
    iov.len = nbuf;
    iov.buf = buf;
    (void)fl_unused;
    return recvv_impl(iops, sock, (struct lcb_iovec_st *)&iov, 1);
}

static lcb_ssize_t
sendv_impl(lcb_io_opt_t iops, lcb_socket_t sock, struct lcb_iovec_st *iov,
           lcb_size_t niov)
{
    DWORD nw, fl = 0;
    WSABUF *bufptr = (WSABUF *)iov;
    if (WSASend(sock, bufptr, niov, &nw, fl, NULL, NULL) == SOCKET_ERROR) {
        LCB_IOPS_ERRNO(iops) = get_wserr(sock);
        return -1;
    }
    return (lcb_ssize_t)nw;
}

static lcb_ssize_t
send_impl(lcb_io_opt_t iops, lcb_socket_t sock, const void *buf, lcb_size_t nbuf,
          int flags)
{
    WSABUF iov;
    iov.buf = (void *)buf;
    iov.len = nbuf;
    (void)flags;
    return sendv_impl(iops, sock, (struct lcb_iovec_st *)&iov, 1);
}

#else
static lcb_ssize_t
recvv_impl(lcb_io_opt_t iops, lcb_socket_t sock, struct lcb_iovec_st *iov,
           lcb_size_t niov)
{
    struct msghdr mh;
    lcb_ssize_t ret;

    memset(&mh, 0, sizeof(mh));
    mh.msg_iov = (struct iovec *)iov;
    mh.msg_iovlen = niov;
    ret = recvmsg(sock, &mh, 0);
    if (ret < 0) {
        LCB_IOPS_ERRNO(iops) = errno;
    }
    return ret;
}

static lcb_ssize_t
recv_impl(lcb_io_opt_t iops, lcb_socket_t sock, void *buf, lcb_size_t nbuf,
          int flags)
{
    lcb_ssize_t ret = recv(sock, buf, nbuf, flags);
    if (ret < 0) {
        LCB_IOPS_ERRNO(iops) = errno;
    }
    return ret;
}

static lcb_ssize_t
sendv_impl(lcb_io_opt_t iops, lcb_socket_t sock, struct lcb_iovec_st *iov,
           lcb_size_t niov)
{
    struct msghdr mh;
    lcb_ssize_t ret;

    memset(&mh, 0, sizeof(mh));
    mh.msg_iov = (struct iovec *)iov;
    mh.msg_iovlen = niov;
    ret = sendmsg(sock, &mh, 0);
    if (ret < 0) {
        LCB_IOPS_ERRNO(iops) = errno;
    }
    return ret;
}

static lcb_ssize_t
send_impl(lcb_io_opt_t iops, lcb_socket_t sock, const void *buf, lcb_size_t nbuf,
          int flags)
{
    lcb_ssize_t ret = send(sock, buf, nbuf, flags);
    if (ret < 0) {
        LCB_IOPS_ERRNO(iops) = errno;
    }
    return ret;
}

#endif

static int make_socket_nonblocking(lcb_socket_t sock)
{
#ifdef _WIN32
    u_long nonblocking = 1;
    if (ioctlsocket(sock, FIONBIO, &nonblocking) == SOCKET_ERROR) {
        return -1;
    }
#else
    int flags;
    if ((flags = fcntl(sock, F_GETFL, NULL)) < 0) {
        return -1;
    }
    if (fcntl(sock, F_SETFL, flags | O_NONBLOCK) == -1) {
        return -1;
    }
#endif
    return 0;
}

/* Declare */
static void close_impl(lcb_io_opt_t,lcb_socket_t);

static lcb_socket_t
socket_impl(lcb_io_opt_t iops, int domain, int type, int protocol)
{
    lcb_socket_t sock;
#ifdef _WIN32
    sock = (lcb_socket_t)WSASocket(domain, type, protocol, NULL, 0, 0);
#else
    sock = socket(domain, type, protocol);
#endif
    if (sock == INVALID_SOCKET) {
        LCB_IOPS_ERRNO(iops) = errno;
    } else {
        if (make_socket_nonblocking(sock) != 0) {
#ifdef _WIN32
            LCB_IOPS_ERRNO(iops) = get_wserr(sock);
#else
            LCB_IOPS_ERRNO(iops) = errno;
#endif
            close_impl(iops, sock);
            sock = INVALID_SOCKET;
        }
    }
    return sock;
}

static void
close_impl(lcb_io_opt_t iops, lcb_socket_t sock)
{
    (void)iops;
#ifdef _WIN32
    closesocket(sock);
#else
    close(sock);
#endif
}

static int
connect_impl(lcb_io_opt_t iops, lcb_socket_t sock, const struct sockaddr *name,
             unsigned int namelen)
{
    int ret;

#ifdef _WIN32
    ret = WSAConnect(sock, name, (int)namelen, NULL, NULL, NULL, NULL);
    if (ret == SOCKET_ERROR) {
        LCB_IOPS_ERRNO(iops) = get_wserr(sock);
    }
#else
    ret = connect(sock, name, (socklen_t)namelen);
    if (ret < 0) {
        LCB_IOPS_ERRNO(iops) = errno;
    }
#endif
    return ret;
}

#if LCB_IOPROCS_VERSION >= 3

static int
chkclosed_impl(lcb_io_opt_t iops, lcb_socket_t sock, int flags)
{
    char buf = 0;
    int rv = 0;

    (void)iops;

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
#endif /* LCB_IOPROCS_VERSION >= 3 */

#if LCB_IOPROCS_VERSION >= 4
static int
cntl_getset_impl(lcb_io_opt_t io, lcb_socket_t sock, int mode, int oslevel,
    int osopt, int optsize, void *optval)
{
    int rv;
    #ifndef _WIN32
    socklen_t dummy = optsize;
    #else
    char dummy = optsize;
    #endif

    if (mode == LCB_IO_CNTL_GET) {
        rv = getsockopt(sock, oslevel, osopt, &dummy, (socklen_t *)optval);
    } else {
        rv = setsockopt(sock, oslevel, osopt, (const char *)optval, (socklen_t)optsize);
    }
    if (rv == 0) {
        return 0;
    } else {
        int lasterr;
        #ifdef _WIN32
        lasterr = get_wserr(sock);
        #else
        lasterr = errno;
        #endif
        LCB_IOPS_ERRNO(io) = lasterr;
        return -1;
    }
}

static int
cntl_impl(lcb_io_opt_t io, lcb_socket_t sock, int mode, int option, void *arg)
{
    switch (option) {
    case LCB_IO_CNTL_TCP_NODELAY:
        return cntl_getset_impl(io,
            sock, mode, IPPROTO_TCP, TCP_NODELAY, sizeof(int), arg);
    case LCB_IO_CNTL_TCP_KEEPALIVE:
        return cntl_getset_impl(io,
            sock, mode, SOL_SOCKET, SO_KEEPALIVE, sizeof(int), arg);
    default:
        LCB_IOPS_ERRNO(io) = ENOTSUP;
        return -1;
    }
}
#endif

#if !defined(LIBCOUCHBASE_INTERNAL) || defined(LCB_IOPS_V12_NO_DEPRECATE)
static void
wire_lcb_bsd_impl(lcb_io_opt_t io)
{
    io->v.v0.recv = recv_impl;
    io->v.v0.recvv = recvv_impl;
    io->v.v0.send = send_impl;
    io->v.v0.sendv = sendv_impl;
    io->v.v0.socket = socket_impl;
    io->v.v0.connect = connect_impl;
    io->v.v0.close = close_impl;

    /* Avoid annoying 'unused' warnings */
    if (0) { wire_lcb_bsd_impl2(NULL,0); }
}
#define lcb__wire0_nowarn() if (0) { wire_lcb_bsd_impl(NULL); }
#else
#define lcb__wire0_nowarn()
#endif

/** For plugins which use v2 or higher */
static void
wire_lcb_bsd_impl2(lcb_bsd_procs *procs, int version)
{
    procs->recv = recv_impl;
    procs->recvv = recvv_impl;
    procs->send = send_impl;
    procs->sendv = sendv_impl;
    procs->socket0 = socket_impl;
    procs->connect0 = connect_impl;
    procs->close = close_impl;

    /* Check that this field exists at compile-time */
    #if LCB_IOPROCS_VERSION >= 3
    if (version >= 3) {
        procs->is_closed = chkclosed_impl;
    }
    #endif
    #if LCB_IOPROCS_VERSION >= 4
    if (version >= 4) {
        procs->cntl = cntl_impl;
    }
    #endif
    lcb__wire0_nowarn();
}
