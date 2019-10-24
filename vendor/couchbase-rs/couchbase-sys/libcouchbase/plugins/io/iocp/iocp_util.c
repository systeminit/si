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
 * New-Style v2 plugin for Windows, Using IOCP.
 * This file contains various utility functions used by the plugin
 * @author Mark Nunberg
 */

#include "iocp_iops.h"
#include <sys/types.h>
#include <sys/timeb.h>
#include <time.h>
#include "config.h"
#include <libcouchbase/plugins/io/wsaerr-inl.c>

#if defined(__MINGW32__) && !defined(_ftime_s)
#define _ftime_s _ftime /** Mingw doens't have the _s variant */
#endif

int iocp_w32err_2errno(DWORD error)
{
    return wsaerr_map_impl(error);
}

DWORD iocp_set_last_error(lcb_io_opt_t io, SOCKET sock)
{
    int werr = GetLastError();
    io->v.v2.error = iocp_w32err_2errno(werr);
    return werr;
}

lcb_uint32_t iocp_micros(void)
{
    return (lcb_uint32_t)(gethrtime() / 1000);
}

LPFN_CONNECTEX iocp_initialize_connectex(SOCKET sock)
{
    LPFN_CONNECTEX ret = NULL;
    DWORD dwBytes;
    GUID ce_guid = WSAID_CONNECTEX;

    WSAIoctl(sock, SIO_GET_EXTENSION_FUNCTION_POINTER, &ce_guid, sizeof(ce_guid), &ret, sizeof(&ret), &dwBytes, NULL,
             NULL);

    return ret;
}

int iocp_just_scheduled(iocp_t *io, iocp_overlapped_t *ol, int status)
{
    DWORD err = GetLastError();
    IOCP_LOG(IOCP_TRACE, "Pending count: %d", io->n_iopending);
    if ((status != 0 && err == WSA_IO_PENDING) || status == 0) {
        io->n_iopending++;
        ol->sd->refcount++;
        return 0;
    }

    /**
     * Otherwise, there's something wrong
     */
    IOCP_LOG(IOCP_ERR, "Got non-harmless error for %p: %d", ol, (int)err);
    io->base.v.v2.error = iocp_w32err_2errno(err);
    return -1;
}

void iocp_socket_decref(iocp_t *io, iocp_sockdata_t *sd)
{
    if (--sd->refcount) {
        return;
    }

    if (sd->sSocket != INVALID_SOCKET) {
        closesocket(sd->sSocket);
    }

    lcb_list_delete(&sd->list);

    (void)io;
    free(sd);
}

void iocp_on_dequeued(iocp_t *io, iocp_sockdata_t *sd, int action)
{
    IOCP_LOG(IOCP_TRACE, "Dequeing. A=%d, Pending=%d", action, io->n_iopending);
    iocp_socket_decref(io, sd);
}

/**This following function was copied from libuv.
 * See http://github.com/joyent/libuv for more details */
int iocp_overlapped_status(OVERLAPPED *lpOverlapped)
{
    NTSTATUS status = (NTSTATUS)lpOverlapped->Internal;
    switch (status) {
        case 0:
            return ERROR_SUCCESS;

        case STATUS_PENDING:
            return ERROR_IO_PENDING;

        case STATUS_INVALID_HANDLE:
        case STATUS_OBJECT_TYPE_MISMATCH:
            return WSAENOTSOCK;

        case STATUS_INSUFFICIENT_RESOURCES:
        case STATUS_PAGEFILE_QUOTA:
        case STATUS_COMMITMENT_LIMIT:
        case STATUS_WORKING_SET_QUOTA:
        case STATUS_NO_MEMORY:
        case STATUS_CONFLICTING_ADDRESSES:
        case STATUS_QUOTA_EXCEEDED:
        case STATUS_TOO_MANY_PAGING_FILES:
        case STATUS_REMOTE_RESOURCES:
        case STATUS_TOO_MANY_ADDRESSES:
            return WSAENOBUFS;

        case STATUS_SHARING_VIOLATION:
        case STATUS_ADDRESS_ALREADY_EXISTS:
            return WSAEADDRINUSE;

        case STATUS_LINK_TIMEOUT:
        case STATUS_IO_TIMEOUT:
        case STATUS_TIMEOUT:
            return WSAETIMEDOUT;

        case STATUS_GRACEFUL_DISCONNECT:
            return WSAEDISCON;

        case STATUS_REMOTE_DISCONNECT:
        case STATUS_CONNECTION_RESET:
        case STATUS_LINK_FAILED:
        case STATUS_CONNECTION_DISCONNECTED:
        case STATUS_PORT_UNREACHABLE:
        case STATUS_HOPLIMIT_EXCEEDED:
            return WSAECONNRESET;

        case STATUS_LOCAL_DISCONNECT:
        case STATUS_TRANSACTION_ABORTED:
        case STATUS_CONNECTION_ABORTED:
            return WSAECONNABORTED;

        case STATUS_BAD_NETWORK_PATH:
        case STATUS_NETWORK_UNREACHABLE:
        case STATUS_PROTOCOL_UNREACHABLE:
            return WSAENETUNREACH;

        case STATUS_HOST_UNREACHABLE:
            return WSAEHOSTUNREACH;

        case STATUS_CANCELLED:
        case STATUS_REQUEST_ABORTED:
            return WSAEINTR;

        case STATUS_BUFFER_OVERFLOW:
        case STATUS_INVALID_BUFFER_SIZE:
            return WSAEMSGSIZE;

        case STATUS_BUFFER_TOO_SMALL:
        case STATUS_ACCESS_VIOLATION:
            return WSAEFAULT;

        case STATUS_DEVICE_NOT_READY:
        case STATUS_REQUEST_NOT_ACCEPTED:
            return WSAEWOULDBLOCK;

        case STATUS_INVALID_NETWORK_RESPONSE:
        case STATUS_NETWORK_BUSY:
        case STATUS_NO_SUCH_DEVICE:
        case STATUS_NO_SUCH_FILE:
        case STATUS_OBJECT_PATH_NOT_FOUND:
        case STATUS_OBJECT_NAME_NOT_FOUND:
        case STATUS_UNEXPECTED_NETWORK_ERROR:
            return WSAENETDOWN;

        case STATUS_INVALID_CONNECTION:
            return WSAENOTCONN;

        case STATUS_REMOTE_NOT_LISTENING:
        case STATUS_CONNECTION_REFUSED:
            return WSAECONNREFUSED;

        case STATUS_PIPE_DISCONNECTED:
            return WSAESHUTDOWN;

        case STATUS_INVALID_ADDRESS:
        case STATUS_INVALID_ADDRESS_COMPONENT:
            return WSAEADDRNOTAVAIL;

        case STATUS_NOT_SUPPORTED:
        case STATUS_NOT_IMPLEMENTED:
            return WSAEOPNOTSUPP;

        case STATUS_ACCESS_DENIED:
            return WSAEACCES;

        default:
            if ((status & (FACILITY_NTWIN32 << 16)) == (FACILITY_NTWIN32 << 16) &&
                (status & (ERROR_SEVERITY_ERROR | ERROR_SEVERITY_WARNING))) {
                /* It's a windows error that has been previously mapped to an */
                /* ntstatus code. */
                return (DWORD)(status & 0xffff);
            } else {
                /* The default fallback for unmappable ntstatus codes. */
                return WSAEINVAL;
            }
    }
}
