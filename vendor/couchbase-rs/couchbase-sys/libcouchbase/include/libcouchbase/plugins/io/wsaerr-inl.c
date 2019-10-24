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

#include "wsaerr.h"

static int
wsaerr_map_impl(DWORD in)
{
    switch (in) {
    case WSAECONNRESET:
        return ECONNRESET;

    case WSAECONNABORTED:
    case WSA_OPERATION_ABORTED:
        return ECONNABORTED;

    case WSA_NOT_ENOUGH_MEMORY:
        return ENOMEM;

    case WSAEWOULDBLOCK:
    case WSA_IO_PENDING:
        return EWOULDBLOCK;

    case WSAEINVAL:
        return EINVAL;

    case WSAEINPROGRESS:
        return EINPROGRESS;

    case WSAEALREADY:
        return EALREADY;

    case WSAEISCONN:
        return EISCONN;

    case WSAENOTCONN:
    case WSAESHUTDOWN:
        return ENOTCONN;

    case WSAECONNREFUSED:
        return ECONNREFUSED;

    case WSAEINTR:
        return EINTR;


    case WSAENETDOWN:
    case WSAENETUNREACH:
    case WSAEHOSTUNREACH:
    case WSAEHOSTDOWN:
        return ENETUNREACH;

    case WSAETIMEDOUT:
        return ETIMEDOUT;

    case WSAENOTSOCK:
        return ENOTSOCK;

    default:
        return EINVAL;
    }
}
