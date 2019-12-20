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

#ifndef LCBIO_CONNECTION_H
#define LCBIO_CONNECTION_H
#include <libcouchbase/couchbase.h>
#include "list.h"
#include "logging.h"
#include "settings.h"
#include "hostlist.h"
#ifdef __cplusplus
namespace lcb
{
namespace io
{
struct Connstart;
struct PoolRequest;
class ConnectionRequest;
} // namespace io
} // namespace lcb
typedef lcb::io::ConnectionRequest *lcbio_pCONNSTART;
typedef lcb::io::ConnectionRequest lcbio_MGRREQ;
extern "C" {
#else
struct lcbio_CONNSTART;
typedef struct lcbio_CONNSTART *lcbio_pCONNSTART;
typedef struct lcbio_MGRREQ lcbio_MGRREQ;
#endif

typedef lcbio_MGRREQ *lcbio_pMGRREQ;

/**
 * @file
 * This provides the core socket routines
 */

/**
 * @ingroup lcbio
 * @defgroup lcbio-core Socket Module
 * @brief Socket routines
 * @addtogroup lcbio-core
 * @{
 */

typedef struct lcbio_TABLE *lcbio_pTABLE;
typedef struct lcbio_TIMER *lcbio_pTIMER, *lcbio_pASYNC;

/**
 * A type representing the underlying operating system's error type. On Unix
 * this is typically an `int`, while on Windows this is a `DWORD`
 */
#ifdef WIN32
typedef DWORD lcbio_OSERR;
#else
typedef int lcbio_OSERR;
#endif

/** @brief Information about a connected socket */
typedef struct {
    unsigned naddr;
    struct sockaddr_storage sa_remote;
    struct sockaddr_storage sa_local;
    lcb_host_t ep;
} lcbio_CONNINFO;

struct lcb_IOMETRICS_st;

/** @brief Subsystem, which utilizes the socket */
typedef enum {
    LCBIO_SERVICE_UNSPEC = 0,
    LCBIO_SERVICE_CFG,
    LCBIO_SERVICE_KV,
    LCBIO_SERVICE_MGMT,
    LCBIO_SERVICE_VIEW,
    LCBIO_SERVICE_N1QL,
    LCBIO_SERVICE_FTS,
    LCBIO_SERVICE_CBAS,
    LCBIO_SERVICE_MAX
} lcbio_SERVICE;

const char *lcbio_svcstr(lcbio_SERVICE service);

/** @brief Core socket structure */
typedef struct lcbio_SOCKET {
    lcbio_pTABLE io;
    lcb_settings *settings;
    void *ctx;
    struct lcb_IOMETRICS_st *metrics;
    lcbio_CONNINFO *info;
    lcbio_OSERR last_error; /**< last OS error */
    unsigned refcount;      /**< refcount on socket */
    union {
        lcb_sockdata_t *sd;
        lcb_socket_t fd;
    } u;
    lcb_list_t protos;
    hrtime_t atime;
    lcbio_SERVICE service;
    lcb_U64 id;
} lcbio_SOCKET;

/**
 * @name Connecting and Destroying a Socket
 * @{
 */

/**
 * Invoked when the connection result is ready
 * @param s the socket to use. You should call lcbio_ref() on it. May be NULL
 *        in the case of an error
 * @param arg user provided argument to the lcbio_connect() function
 * @param err an error code (if connection is NULL)
 * @param syserr the raw errno variable received.
 */
typedef void (*lcbio_CONNDONE_cb)(lcbio_SOCKET *s, void *arg, lcb_STATUS err, lcbio_OSERR syserr);

/**
 * Schedule a new connection to a remote endpoint.
 *
 * @param iot I/O table to use. The socket will increment the reference count of
 *        the table until the socket is destroyed.
 * @param settings Settings structure. Used for logging
 * @param dest the endpoint to connect to
 * @param timeout number of time to wait for connection. The handler will be
 *        invoked with an error of `LCB_ETIMEDOUT` if a successful connection
 *        cannot be established in time.
 * @param handler a handler to invoke with the result. The handler will always
 *        be invoked unless the request has been cancelled. You should inspect
 *        the socket and error code in the handler to see if the connection has
 *        been successful.
 * @param arg the argument passed to the handler
 * @return a request handle. The handle may be cancelled (to stop the pending
 *         connection attempt) before the handler is invoked.
 *         Once the handler is invoked, the returned handle is considered to
 *         be invalid; as such the following idiom should be employed:
 *
 *
 * @code{.c}
 * struct my_ctx {
 *   lcbio_SOCKET *sock;
 *   lcbio_pCONNSTART creq;
 * }
 *
 * static void do_connect(void) {
 *   my_ctx *ctx;
 *   ctx->creq = lcbio_connect(iot, settings, dest, tmo, handler, ctx);
 *   // check errors..
 * }
 *
 *
 * static void handler(lcbio_SOCKET *s, void *arg, lcb_STATUS err) {
 *   my_ctx *ctx = arg;
 *   ctx->creq = NULL;
 *   if (!(ctx->sock = s)) {
 *    ...
 *   }
 * }
 * @endcode
 */
lcbio_pCONNSTART lcbio_connect(lcbio_pTABLE iot, lcb_settings *settings, const lcb_host_t *dest, uint32_t timeout,
                               lcbio_CONNDONE_cb handler, void *arg);

/**
 * Wraps an existing socket descriptor into an lcbio_SOCKET structure
 * @param iot
 * @param settings
 * @param fd The socket descriptor to wrap. This must refer to a _connected_
 * socket (e.g. via `connect(2)` or `socketpair(2)`.
 * @return A new socket object.
 */
lcbio_SOCKET *lcbio_wrap_fd(lcbio_pTABLE iot, lcb_settings *settings, lcb_socket_t fd);

/**
 * Wraps `lcb_connect()` by traversing a list of hosts. This will cycle through
 * each host in the list until a connection has been successful. Currently
 * this will not intercept the handler but will catch any hostname lookup
 * failures.
 *
 * @param iot
 * @param settings
 * @param hl The hostlist to traverse
 * @param rollover If the hostlist position is at the end, this boolean parameter
 *        indicates whether the position should be reset
 * @param timeout
 * @param handler
 * @param arg
 * @see lcbio_connect()
 */
lcbio_pCONNSTART lcbio_connect_hl(lcbio_pTABLE iot, lcb_settings *settings, hostlist_t hl, int rollover,
                                  uint32_t timeout, lcbio_CONNDONE_cb handler, void *arg);

/**
 * Cancel a pending connection attempt. Once the attempt is cancelled the
 * handler will not be invoked and the CONNSTART object will be invalid.
 * @param cs the handle returned from lcbio_connect()
 */
void lcbio_connect_cancel(lcbio_pCONNSTART cs);

/**
 * Cancel any pending I/O on this socket. Outstanding callbacks for I/O (i.e.
 * for completion-based reads and writes) will still be delivered with an error
 * code. Outstanding callbacks for event-based I/O will not be invoked.
 *
 * This function does not modify the reference count of the socket directly
 * but will clear any lcbio_PROTOCTX objects attached to it.
 */
void lcbio_shutdown(lcbio_SOCKET *);

/**
 * Increment the reference count on the socket. When the socket is no longer
 * needed, call lcbio_unref().
 */
#define lcbio_ref(s) (s)->refcount++

/**
 * Decrement the reference count on the socket. When the reference count hits
 * zero, lcbio_shutdown() will be called.
 */
#define lcbio_unref(s)                                                                                                 \
    if (!--(s)->refcount) {                                                                                            \
        lcbio__destroy(s);                                                                                             \
    }

/** @} */

/**
 * @name Protocol Contexts
 * @{
 */

typedef enum {
    LCBIO_PROTOCTX_SESSINFO = 1,
    LCBIO_PROTOCTX_POOL,
    LCBIO_PROTOCTX_HOSTINFO,
    LCBIO_PROTOCTX_SSL,
    LCBIO_PROTOCTX_MAX
} lcbio_PROTOID;

/**
 * @brief Protocol-specific data attached to lcbio_SOCKET.
 *
 * A protocol context is an object which is bound to the actual low level
 * socket connection rather than the logical socket owner. This is used for
 * resources which operate on the TCP state (such as encryption or authentication)
 * or which employ socket reuse (for things such as pooling).
 */
typedef struct lcbio_PROTOCTX {
    lcb_list_t ll;
    lcbio_PROTOID id;
    /** Called when the context is to be removed from the socket */
    void (*dtor)(struct lcbio_PROTOCTX *);
} lcbio_PROTOCTX;

/**
 * Attach an lcbio_PROTOCTX object to the socket. This object will remain
 * part of the socket until lcbio_shutdown() is invoked, or the context itself
 * is removed explicitly.
 *
 * @param socket the socket the context should be added to
 * @param proto the object to be added. The protocol object should have its
 *        `id` and `dtor` fields initialized.
 */
void lcbio_protoctx_add(lcbio_SOCKET *socket, lcbio_PROTOCTX *proto);

/**
 * Retrieve an existing protocol context by its ID
 * @param socket The socket to query
 * @param id The ID of the context
 * @return the context, or NULL if not found
 */
lcbio_PROTOCTX *lcbio_protoctx_get(const lcbio_SOCKET *socket, lcbio_PROTOID id);

/**
 * Remove a protocol context by its ID
 * @param socket socket from which to remove
 * @param id The id of the context to remove
 * @param call_dtor whether the destructor should be invoked
 * @return the returned context, or NULL if not found
 */
lcbio_PROTOCTX *lcbio_protoctx_delid(lcbio_SOCKET *socket, lcbio_PROTOID id, int call_dtor);

/**
 * Delete a protocol context by its pointer.
 * @param socket The socket from which the context should be removed
 * @param ctx The pointer to remove
 * @param call_dtor Whether to invoke the destructor for the lcbio_PROTOCTX
 */
void lcbio_protoctx_delptr(lcbio_SOCKET *socket, lcbio_PROTOCTX *ctx, int call_dtor);

/** @private */
void lcbio__protoctx_delall(lcbio_SOCKET *s);

/** @} */

/**
 * Get the lcb_host_t pointer indicating the endpoint the socket is connected to.
 * @param sock The socket
 * @return a pointer to the host.
 */
#define lcbio_get_host(sock) (&(sock)->info->ep)

/**
 * @private
 * Internal destroy function for when the refcount hits 0
 */
void lcbio__destroy(lcbio_SOCKET *s);

/**
 * @name IO Table Functions
 * @details
 * These functions provide the user-facing API for dealing with the lcbio_TABLE
 * structure. These functions only control its handling as an opaque object.
 * The definition of the structure may be found in <lcbio/iotable.h> and contains
 * more routines for actually using it.
 *
 * @{
 */

/**
 * Create a new table based on the input iops structure. The table itself retains
 * ownership over the structure and will destroy it once the table itself has
 * been destroyed.
 * @param io An IOPS structure. See lcb_create_io_ops()
 * @return A table with a reference count initialized to 1
 */
lcbio_pTABLE lcbio_table_new(lcb_io_opt_t io);

/** Increment the reference count on the lcbio_TABLE */
void lcbio_table_unref(lcbio_pTABLE iot);

/** Decrement the reference count on the lcbio_TABLE */
void lcbio_table_ref(lcbio_pTABLE iot);

/**
 * Set the metrics object for the socket. Various operations will then log
 * the number of bytes written/received on the socket.
 */
#define lcbio_set_metrics(sock, m) (sock)->metrics = m

/** @}*/

/** @name IO Status Codes
 *@{ */
typedef enum {
    LCBIO_COMPLETED = 0, /**< Operation has been completed */
    LCBIO_PENDING,       /**< Operation is partially completed */
    LCBIO__SUCCESS_MAX,  /**< Status codes higher than this value are errors */
    LCBIO_IOERR,         /**< An I/O error has been received */
    LCBIO_INTERR,        /**< An internal non-I/O error has been received */
    LCBIO_SHUTDOWN       /**< Socket was gracefully closed */
} lcbio_IOSTATUS;

#define LCBIO_WFLUSHED LCBIO_COMPLETED
#define LCBIO_CANREAD LCBIO_COMPLETED
#define LCBIO_IS_OK(s) ((s) < LCBIO__SUCCESS_MAX)
/** @} */

#ifdef __cplusplus
}
#endif
#endif

/**
 * @}
 */
