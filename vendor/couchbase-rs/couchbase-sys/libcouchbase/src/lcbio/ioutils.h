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

#ifndef LCBIO_UTILS_H
#define LCBIO_UTILS_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file
 * @brief Various I/O-related utilities
 */

typedef enum {
    LCBIO_CSERR_BUSY,     /* request pending */
    LCBIO_CSERR_INTR,     /* eintr */
    LCBIO_CSERR_EINVAL,   /* einval */
    LCBIO_CSERR_EFAIL,    /* hard failure */
    LCBIO_CSERR_CONNECTED /* connection established */
} lcbio_CSERR;

/**
 * Convert the system errno (indicated by 'syserr')
 * @param syserr system error code
 * @return a status code simplifying the error
 */
lcbio_CSERR lcbio_mkcserr(int syserr);

/**
 * Assigns the target error code if it indicates a 'fatal' or 'relevant' error
 * code.
 *
 * @param in Error code to inspect
 * @param[out] out target pointer
 */
void lcbio_mksyserr(lcbio_OSERR in, lcbio_OSERR *out);

/**
 * Convert a system error code into one suitable for returning to the user
 * @param in The code received. This can be 0, for graceful shutdown
 * @param settings The settings for the library
 * @return An error code.
 */
lcb_STATUS lcbio_mklcberr(lcbio_OSERR in, const lcb_settings *settings);

/**
 * Traverse the addrinfo structure and return a socket.
 * @param io the iotable structure used to create the socket
 * @param[in,out] ai an addrinfo structure
 * @param[out] connerr an error if a socket could not be established.
 * @return a new socket, or INVALID_SOCKET
 *
 * The ai structure should be considered as an opaque iterator. This function
 * will look at the first entry in the list and attempt to create a socket.
 * It will traverse through each entry and break when either a socket has
 * been successfully created, or no more addrinfo entries remain.
 */
lcb_socket_t lcbio_E_ai2sock(lcbio_pTABLE io, struct addrinfo **ai, int *connerr);

lcb_sockdata_t *lcbio_C_ai2sock(lcbio_pTABLE io, struct addrinfo **ai, int *conerr);

struct lcbio_NAMEINFO {
    char local[NI_MAXHOST + NI_MAXSERV + 2];
    char remote[NI_MAXHOST + NI_MAXSERV + 2];
};

int lcbio_get_nameinfo(lcbio_SOCKET *sock, struct lcbio_NAMEINFO *nistrs);

/** Basic wrapper around the @ref lcb_ioE_chkclosed_fn family */
int lcbio_is_netclosed(lcbio_SOCKET *sock, int flags);

/**
 * Enable an option on a socket
 * @param sock The socket
 * @param cntl The option (LCB_IO_CNTL_xxx)
 * @return
 */
lcb_STATUS lcbio_enable_sockopt(lcbio_SOCKET *sock, int cntl);

const char *lcbio_strsockopt(int cntl);

void lcbio__load_socknames(lcbio_SOCKET *sock);

#ifdef _WIN32
#define lcbio_syserrno GetLastError()
#else
#define lcbio_syserrno errno
#endif

#ifdef __cplusplus
}

std::string lcbio__inet_ntop(sockaddr_storage *ss);

namespace lcb
{
namespace io
{

/**
 * This interface defines a pending connection request. It may be
 * cancelled.
 */
class ConnectionRequest
{
  public:
    virtual void cancel() = 0;
    virtual ~ConnectionRequest() {}
    static void cancel(ConnectionRequest **pp)
    {
        if (*pp) {
            (*pp)->cancel();
            *pp = NULL;
        }
    }
};

} // namespace io
} // namespace lcb

#endif
#endif
