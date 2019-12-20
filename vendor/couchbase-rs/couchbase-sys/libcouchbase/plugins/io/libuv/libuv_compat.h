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

#ifndef LIBUV_COMPAT_H
#define LIBUV_COMPAT_H

#ifdef UV_VERSION_MAJOR
#ifndef UV_VERSION_PATCH
#define UV_VERSION_PATCH 0
#endif
#define UV_VERSION ((UV_VERSION_MAJOR << 16) | (UV_VERSION_MINOR << 8) | (UV_VERSION_PATCH))
#else
#define UV_VERSION 0x000b00
#endif

#if defined(_WIN32) && defined(LIBCOUCHBASE_INTERNAL)
#include <libcouchbase/plugins/io/wsaerr.h>
#endif

#ifndef UNKNOWN
#define UNKNOWN -1
#endif

#ifndef EAIFAMNOSUPPORT
#define EAIFAMNOSUPPORT EAI_FAMILY
#endif

#ifndef EAISERVICE
#define EAISERVICE EAI_SERVICE
#endif

#ifndef EAI_SYSTEM
#define EAI_SYSTEM -11
#endif
#ifndef EADDRINFO
#define EADDRINFO EAI_SYSTEM
#endif

#ifndef EAISOCKTYPE
#define EAISOCKTYPE EAI_SOCKTYPE
#endif

#ifndef ECHARSET
#define ECHARSET 0
#endif

#ifndef EOF
#define EOF -1
#endif

#ifndef ENONET
#define ENONET ENETDOWN
#endif

#ifndef ESHUTDOWN
#define ESHUTDOWN WSAESHUTDOWN
#endif

#ifndef EHOSTDOWN
/* missing only on Windows */
#define EHOSTDOWN WSAEHOSTDOWN
#endif

/* Not all systems have these error codes */
#ifndef EAI_FAIL
#define EAI_FAIL (-1)
#endif
#ifndef EAI_CANCELED
#define EAI_CANCELED -101
#endif
#ifndef EAI_ADDRFAMILY
#define EAI_ADDRFAMILY -9
#endif
#ifndef EREMOTEIO
#define EREMOTEIO -121
#endif
#ifndef EAI_BADHINTS
#define EAI_BADHINTS EAI_FAIL
#endif
#ifndef EAI_NODATA
#define EAI_NODATA EAI_FAIL
#endif
#ifndef EAI_PROTOCOL
#define EAI_PROTOCOL EAI_FAIL
#endif
#ifndef EAI_AGAIN
#define EAI_AGAIN EAI_FAIL
#endif
#ifndef EAI_BADFLAGS
#define EAI_BADFLAGS EAI_FAIL
#endif
#ifndef EAI_MEMORY
#define EAI_MEMORY EAI_FAIL
#endif
#ifndef EAI_OVERFLOW
#define EAI_OVERFLOW EAI_FAIL
#endif
#ifndef EFTYPE
#define EFTYPE EAI_FAIL
#endif

#define OK 0

#if UV_VERSION < 0x000900
#define UVC_RUN_ONCE(l) uv_run_once(l)
#define UVC_RUN_DEFAULT(l) uv_run(l)
#else
#define UVC_RUN_ONCE(l) uv_run(l, UV_RUN_ONCE)
#define UVC_RUN_DEFAULT(l) uv_run(l, UV_RUN_DEFAULT)
#endif

#if UV_VERSION < 0x000b00

#define UVC_TCP_CONNECT(req, handle, addr, cb) uv_tcp_connect(req, handle, *(struct sockaddr_in *)addr, cb);

#define UVC_TCP_CONNECT6(req, handle, addr, cb) uv_tcp_connect6(req, handle, *(struct sockaddr_in6 *)addr, cb);

#define UVC_ALLOC_CB(func) uv_buf_t func(uv_handle_t *handle, size_t suggested_size)

#define UVC_ALLOC_CB_VARS()                                                                                            \
    uv_buf_t _buf;                                                                                                     \
    uv_buf_t *buf = &_buf;

#define UVC_ALLOC_CB_RETURN() return _buf;

#define UVC_READ_CB(func) void func(uv_stream_t *stream, ssize_t nread, const uv_buf_t _buf)

#define UVC_READ_CB_VARS() const uv_buf_t *buf = &_buf;

#define UVC_TIMER_CB(func) void func(uv_timer_t *timer, int status)

static int uvc_uv2syserr(int status)
{
#define X(errnum, errname, errdesc)                                                                                    \
    if (status == UV_##errname) {                                                                                      \
        return errname;                                                                                                \
    }
    UV_ERRNO_MAP(X);
#undef X
    return 0;
}

static int uvc_is_eof(uv_loop_t *loop, int error)
{
    error = uv_last_error(loop).code;
    return error == UV_EOF;
}

static int uvc_last_errno(uv_loop_t *loop, int error)
{
    int uverr = 0;

    if (!error) {
        return 0;
    }

    uverr = uv_last_error(loop).code;
    return uvc_uv2syserr(uverr);
}

#else

#define UVC_TCP_CONNECT(req, handle, addr, cb) uv_tcp_connect(req, handle, addr, cb);

#define UVC_TCP_CONNECT6(req, handle, addr, cb) uv_tcp_connect(req, handle, addr, cb);

#define UVC_ALLOC_CB(func) void func(uv_handle_t *handle, size_t suggested_size, uv_buf_t *buf)

#define UVC_ALLOC_CB_VARS()

#define UVC_ALLOC_CB_RETURN()

#define UVC_READ_CB(func) void func(uv_stream_t *stream, ssize_t nread, const uv_buf_t *buf)

#define UVC_READ_CB_VARS()

#define UVC_TIMER_CB(func) void func(uv_timer_t *timer)

static int uv_uv2syserr(int status)
{
#define X(name, desc)                                                                                                  \
    if (status == UV_##name) {                                                                                         \
        return name;                                                                                                   \
    }
    UV_ERRNO_MAP(X)
#undef X
    return 0;
}

static int uvc_last_errno(uv_loop_t *loop, int error)
{
    return uv_uv2syserr(error);
}

static int uvc_is_eof(uv_loop_t *loop, int error)
{
    (void)loop;
    return error == UV_EOF;
}

#endif
#endif
