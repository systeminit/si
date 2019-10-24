/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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

#ifndef LIBCOUCHBASE_COUCHBASE_H
#error "include libcouchbase/couchbase.h first"
#endif

#ifndef LCB_IOPS_H
#define LCB_IOPS_H

/**
 * @file
 * @brief Public I/O integration interface
 * @details
 *
 * This file provides the public I/O interface for integrating with external
 * event loops.
 */

/**
 * @ingroup lcbio lcb-public-api
 * @defgroup lcb-io-plugin-api Network I/O
 * @details
 *
 * I/O Integration comes in two flavors:
 *
 * @par (E)vent/Poll Based Integration
 * This system is based upon the interfaces exposed by the `poll(2)` and
 * `select(2)` calls found in POSIX-based systems and are wrapped by systems
 * such as _libevent_ and _libev_. At their core is the notion that a socket
 * may be polled for readiness (either readiness for reading or readiness
 * for writing). When a socket is deemed ready, a callback is invoked indicating
 * which events took place.
 *
 *
 * @par (C)ompletion/Operation/Buffer Based Integration
 * This system is based upon the interfaces exposed in the Win32 API where
 * I/O is done in terms of operations which are awaiting _completion_. As such
 * buffers are passed into the core, and the application is notified when the
 * operation on those buffers (either read into a buffer, or write from a buffer)
 * has been completed.
 *
 *
 * @addtogroup lcb-io-plugin-api
 * @{
 */

#ifdef __cplusplus
extern "C" {
#endif

/** @brief Type representing the native socket type of the operating system */
#ifdef _WIN32
typedef SOCKET lcb_socket_t;
#else
typedef int lcb_socket_t;
#endif

struct sockaddr;

#ifndef _WIN32
/** Defined if the lcb_IOV structure conforms to `struct iovec` */
#define LCB_IOV_LAYOUT_UIO
typedef struct lcb_iovec_st {
    void *iov_base;
    size_t iov_len;
} lcb_IOV;
#else
/** Defined if the lcb_IOV structure conforms to `WSABUF` */
#define LCB_IOV_LAYOUT_WSABUF
typedef struct lcb_iovec_st {
    ULONG iov_len;
    void *iov_base;
} lcb_IOV;
#endif

#if defined(LIBCOUCHBASE_INTERNAL) && !defined(LCB_IOPS_V12_NO_DEPRECATE)
#define LCB__IOPS_CONCAT2(X, Y) X##Y
#define LCB__IOPS_CONCAT(X, Y) LCB__IOPS_CONCAT2(X, Y)
#define LCB_IOPS_DEPRECATED(X) void (*LCB__IOPS_CONCAT(lcb__iops__dummy, __LINE__))(void)
#else
#define LCB_IOPS_DEPRECATED(X) X
#endif

/** @brief structure describing a connected socket's endpoints */
struct lcb_nameinfo_st {
    struct {
        struct sockaddr *name;
        int *len;
    } local;

    struct {
        struct sockaddr *name;
        int *len;
    } remote;
};

/**
 * @struct lcb_IOV
 * @brief structure indicating a buffer and its size
 *
 * @details
 * This is compatible with a `struct iovec` on Unix and a `WSABUF` structure
 * on Windows. It has an `iov_base` field which is the base pointer and an
 * `iov_len` field which is the length of the buffer.
 */

typedef struct lcb_io_opt_st *lcb_io_opt_t;

/**
 * @brief Callback invoked for all poll-like events
 *
 * @param sock the socket associated with the event
 * @param events the events which activated this callback. This is set of bits
 *        comprising of LCB_READ_EVENT, LCB_WRITE_EVENT, and LCB_ERROR_EVENT
 * @param uarg a user-defined pointer passed to the
 *        lcb_ioE_event_watch_fn routine.
 */
typedef void (*lcb_ioE_callback)(lcb_socket_t sock, short events, void *uarg);

/**@name Timer Callbacks
 *@{*/

/**
 * @brief Create a new timer object.
 *
 * @param iops the io structure
 * @return an opaque timer handle. The timer shall remain inactive and shall
 *         be destroyed via the lcb_io_timer_destroy_fn routine.
 */
typedef void *(*lcb_io_timer_create_fn)(lcb_io_opt_t iops);

/**
 * @brief Destroy a timer handler
 *
 * Destroy a timer previously created with lcb_io_timer_create_fn
 * @param iops the io structure
 * @param timer the opaque handle
 * The timer must have already been cancelled via lcb_io_timer_cancel_fn
 */
typedef void (*lcb_io_timer_destroy_fn)(lcb_io_opt_t iops, void *timer);

/**
 * @brief Cancel a pending timer callback
 *
 * Cancel and unregister a pending timer. If the timer has already
 * fired, this does nothing. If the timer has not yet fired, the callback
 * shall not be delivered.
 *
 * @param iops the I/O structure
 * @param timer the timer to cancel.
 */
typedef void (*lcb_io_timer_cancel_fn)(lcb_io_opt_t iops, void *timer);

/**
 * @brief Schedule a callback to be invoked within a given interval.
 *
 * Schedule a timer to be fired within usec microseconds from now
 * @param iops the I/O structure
 * @param timer a timer previously created with timer_create
 * @param usecs the timer interval
 * @param uarg the user-defined pointer to be passed in the callback
 * @param callback the callback to invoke
 */
typedef int (*lcb_io_timer_schedule_fn)(lcb_io_opt_t iops, void *timer, lcb_U32 usecs, void *uarg,
                                        lcb_ioE_callback callback);

/**@}*/

/**@name Event Handle Callbacks
 * @{*/

/**
 * @brief Create a new event handle.
 *
 * An event object may be used to monitor a socket for given I/O readiness events
 * @param iops the I/O structure.
 * @return a new event handle.
 * The handle may then be associated with a
 * socket and watched (via lcb_ioE_event_watch_fn) for I/O readiness.
 */
typedef void *(*lcb_ioE_event_create_fn)(lcb_io_opt_t iops);

/**
 * @brief Destroy an event handle
 *
 * Destroy an event object. The object must not be active.
 * @param iops the I/O structure
 * @param event the event to free
 */
typedef void (*lcb_ioE_event_destroy_fn)(lcb_io_opt_t iops, void *event);

/**
 * @deprecated lcb_ioE_event_watch_fn() should be used with `0` for events
 * @brief Cancel pending callbacks and unwatch a handle.
 *
 * @param iops the I/O structure
 * @param sock the socket associated with the event
 * @param event the opaque event object
 *
 * This function may be called multiple times and shall not fail even if the
 * event is already inactive.
 */
typedef void (*lcb_ioE_event_cancel_fn)(lcb_io_opt_t iops, lcb_socket_t sock, void *event);

/** Data is available for reading */
#define LCB_READ_EVENT 0x02
/** Data can be written */
#define LCB_WRITE_EVENT 0x04
/** Exceptional condition ocurred on socket */
#define LCB_ERROR_EVENT 0x08
#define LCB_RW_EVENT (LCB_READ_EVENT | LCB_WRITE_EVENT)

/**
 * Associate an event with a socket, requesting notification when one of
 * the events specified in 'flags' becomes available on the socket.
 *
 * @param iops the IO context
 * @param socket the socket to watch
 * @param event the event to associate with the socket. If this parameter is
 * @param evflags a bitflag of events to watch. This is one of LCB_READ_EVENT,
 * LCB_WRITE_EVENT, or LCB_RW_EVENT.
 * If this value is `0` then existing events shall be cancelled on the
 * socket.
 *
 * Note that the callback may _also_ receive LCB_ERROR_EVENT but this cannot
 * be requested as an event to watch for.
 *
 * @param uarg a user defined pointer to be passed to the callback
 * @param callback the callback to invoke when one of the events becomes
 * ready.
 *
 * @attention
 * It shall be legal to call this routine multiple times without having to call
 * the lcb_ioE_event_cancel_fn(). The cancel function should in fact be implemented
 * via passing a `0` to the `evflags` parameter, effectively clearing the
 * event.
 */
typedef int (*lcb_ioE_event_watch_fn)(lcb_io_opt_t iops, lcb_socket_t socket, void *event, short evflags, void *uarg,
                                      lcb_ioE_callback callback);

/**@}*/

/**@name BSD-API I/O Routines
 * @{*/

/**
 * @brief Receive data into a single buffer
 * @see `recv(2)` socket API call.
 */
typedef lcb_SSIZE (*lcb_ioE_recv_fn)(lcb_io_opt_t iops, lcb_socket_t sock, void *target_buf, lcb_SIZE buflen,
                                     int _unused_flags);

/** @brief Send data from a single buffer.
 * @see `send(2)` on POSIX
 */
typedef lcb_SSIZE (*lcb_ioE_send_fn)(lcb_io_opt_t iops, lcb_socket_t sock, const void *srcbuf, lcb_SIZE buflen,
                                     int _ignored);

/**@brief Read data into a series of buffers.
 * @see the `recvmsg(2)` function on POSIX */
typedef lcb_SSIZE (*lcb_ioE_recvv_fn)(lcb_io_opt_t iops, lcb_socket_t sock, lcb_IOV *iov, lcb_SIZE niov);

/**@brief Write data from multiple buffers.
 * @see the `sendmsg(2)` function on POSIX */
typedef lcb_SSIZE (*lcb_ioE_sendv_fn)(lcb_io_opt_t iops, lcb_socket_t sock, lcb_IOV *iov, lcb_SIZE niov);

/**@brief Create a new socket.
 * @see `socket(2)` on POSIX */
typedef lcb_socket_t (*lcb_ioE_socket_fn)(lcb_io_opt_t iops, int domain, int type, int protocol);

/**@brief Connect a created socket
 * @see `connect(2)` on POSIX */
typedef int (*lcb_ioE_connect_fn)(lcb_io_opt_t iops, lcb_socket_t sock, const struct sockaddr *dst,
                                  unsigned int addrlen);

/** @internal */
typedef int (*lcb_ioE_bind_fn)(lcb_io_opt_t iops, lcb_socket_t sock, const struct sockaddr *srcaddr,
                               unsigned int addrlen);

/** @internal */
typedef int (*lcb_ioE_listen_fn)(lcb_io_opt_t iops, lcb_socket_t bound_sock, unsigned int queuelen);

/** @internal */
typedef lcb_socket_t (*lcb_ioE_accept_fn)(lcb_io_opt_t iops, lcb_socket_t lsnsock);

/** @brief Close a socket
 * @see `close(2)` and `shutdown(2)` */
typedef void (*lcb_ioE_close_fn)(lcb_io_opt_t iops, lcb_socket_t sock);

/**
 * While checking the socket, treat pending data as an _error_.
 * This flag will be _missing_ if the socket participates in a protocol
 * where unsolicited data is possible.
 *
 * Currently Couchbase does not provide such a protocol (at least not one where
 * sockets are placed in a pool), but it may in the future.
 *
 * This may be passed as a `flags` option to lcb_ioE_chkclosed_fn
 */
#define LCB_IO_SOCKCHECK_PEND_IS_ERROR 1

#define LCB_IO_SOCKCHECK_STATUS_CLOSED 1
#define LCB_IO_SOCKCHECK_STATUS_OK 0
#define LCB_IO_SOCKCHECK_STATUS_UNKNOWN -1

/**@brief Check if a socket has been closed or not. This is used to check
 * a socket's state after a period of inactivity.
 *
 *
 * @param iops The iops
 * @param sock The socket to check
 * @param flags A bit set of options.
 * @return A value greater than 0 if the socket _is_ closed, 0 if the socket
 * has not been closed, or a negative number, if the status could not be
 * determined within the given constraints (for example, if `flags` did not
 * specify `LCB_IO_SOCKCHECK_PEND_IS_ERROR`, and the implementation does not
 * have a way to check status otherwise.
 *
 * @since 2.4.4
 */
typedef int (*lcb_ioE_chkclosed_fn)(lcb_io_opt_t iops, lcb_socket_t sock, int flags);

/** For use with `io{E,C}_cntl_fn`, indicates the setting should be retrieved */
#define LCB_IO_CNTL_GET 0
/** For use with lcb_io{E,C}_cntl_fn`, indicates the setting should be modified */
#define LCB_IO_CNTL_SET 1

/** Disable Nagle's algorithm (use an int) */
#define LCB_IO_CNTL_TCP_NODELAY 1

/** Enable/Disable TCP Keepalive */
#define LCB_IO_CNTL_TCP_KEEPALIVE 2

/**
 * @brief Execute a specificied operation on a socket.
 * @param iops The iops
 * @param sock The socket
 * @param mode The mode, can be @ref LCB_IO_CNTL_GET or @ref LCB_IO_CNTL_SET
 * @param option The option to access
 * @param[in,out] arg the argument for the option
 * @return zero on success, nonzero on failure.
 */
typedef int (*lcb_ioE_cntl_fn)(lcb_io_opt_t iops, lcb_socket_t sock, int mode, int option, void *arg);
/**@}*/

struct ringbuffer_st;
struct lcb_connection_st;
struct lcbio_SOCKET;

/** @deprecated Ringbuffers are no longer used this way by the library for I/O */
struct lcb_buf_info {
    char *root;
    lcb_SIZE size;
    struct ringbuffer_st *ringbuffer;
    struct lcb_iovec_st iov[2];
};

/**
 * @brief Socket handle for completion-based I/O
 *
 * The sockdata structure is analoguous to an `lcb_socket_t` returned by
 * the E-model I/O.
 */
typedef struct lcb_sockdata_st {
    lcb_socket_t socket;             /**< System socket, for informational purposes */
    lcb_io_opt_t parent;             /**< Parent I/O context */
    struct lcbio_SOCKET *lcbconn;    /**< Internal socket equivalent */
    int closed;                      /**< @deprecated No longer used by the library */
    int is_reading;                  /**< Internally used by lcbio */
    struct lcb_buf_info read_buffer; /**< @deprecated No longer used by the library */
} lcb_sockdata_t;

/** @deprecated */
typedef struct lcb_io_writebuf_st {
    struct lcb_io_opt_st *parent;
    struct lcb_buf_info buffer;
} lcb_io_writebuf_t;

/**@name Completion Routines
 * @{*/

/**
 * @brief Create a completion socket handle
 *
 * Create an opaque socket handle
 * @param iops the IO context
 * @param domain socket address family, e.g. AF_INET
 * @param type the transport type, e.g. SOCK_STREAM
 * @param protocol the IP protocol, e.g. IPPROTO_TCP
 * @return a socket pointer or NULL on failure.
 */
typedef lcb_sockdata_t *(*lcb_ioC_socket_fn)(lcb_io_opt_t iops, int domain, int type, int protocol);

/**
 * @brief Callback to be invoked upon a connection result.
 * Callback invoked for a connection result.
 * @param socket the socket which is being connected
 * @param status the status. 0 for success, nonzero on failure
 */
typedef void (*lcb_io_connect_cb)(lcb_sockdata_t *socket, int status);

/**
 * @brief Request a connection for a socket
 * @param iops the IO context
 * @param sd the socket pointer
 * @param dst the address to connect to
 * @param naddr the size of the address len, e.g. sizeof(struct sockaddr_in)
 * @param callback the callback to invoke when the connection status is determined
 * @return 0 on success, nonzero if a connection could not be scheduled.
 */
typedef int (*lcb_ioC_connect_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd, const struct sockaddr *dst, unsigned int naddr,
                                  lcb_io_connect_cb callback);

/**
 * @brief Callback invoked when a new client connection has been established
 * @param sd_server the server listen socket
 * @param sd_client the new client socket
 * @param status if there was an error accepting (in this case, sd_client is NULL
 */
typedef void(lcb_ioC_serve_callback)(lcb_sockdata_t *sd_server, lcb_sockdata_t *sd_client, int status);

/**
 * Specify that the socket start accepting connections. This should be called
 * on a newly created non-connected socket
 * @param iops the I/O context
 * @param server_socket the socket used to listen with
 * @param sockaddr the local address for listening
 * @param callback the callback to invoke for each new connection
 */
typedef int (*lcb_ioC_serve_fn)(lcb_io_opt_t iops, lcb_sockdata_t *server_socket, const struct sockaddr *listen_addr,
                                lcb_ioC_serve_callback callback);

/**
 * @brief Request address information on a connected socket
 * @param iops the I/O context
 * @param sock the socket from which to retrieve information
 * @param ni a nameinfo structure to populate with the relevant details
 */
typedef int (*lcb_ioC_nameinfo_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sock, struct lcb_nameinfo_st *ni);

/**@deprecated*/
typedef void (*lcb_ioC_read_callback)(lcb_sockdata_t *sd, lcb_SSIZE nread);
#define lcb_io_read_cb lcb_ioC_read_callback
/**@deprecated See lcb_ioC_read2_fn(). Wrapped if not implemented*/
typedef int (*lcb_ioC_read_fn)(lcb_io_opt_t, lcb_sockdata_t *, lcb_ioC_read_callback);
/**@deprecated See lcb_ioC_write2_fn(). Wrapped if not implemented*/
typedef lcb_io_writebuf_t *(*lcb_ioC_wballoc_fn)(lcb_io_opt_t, lcb_sockdata_t *);
/**@deprecated See lcb_ioC_write2_fn(). Wrapped if not implemented */
typedef void (*lcb_ioC_wbfree_fn)(lcb_io_opt_t, lcb_sockdata_t *, lcb_io_writebuf_t *);
/**@deprecated See lcb_ioC_write2_fn(). This will be wrapped if not implemented */
typedef void (*lcb_ioC_write_callback)(lcb_sockdata_t *, lcb_io_writebuf_t *, int);
#define lcb_io_write_cb lcb_ioC_write_callback

/**@deprecated*/
typedef int (*lcb_ioC_write_fn)(lcb_io_opt_t, lcb_sockdata_t *, lcb_io_writebuf_t *, lcb_ioC_write_callback);

/**
 * @brief Callback received when a buffer has been flushed
 * @param sd the socket
 * @param status nonzero on error
 * @param arg the opaque handle passed in the write2 call
 */
typedef void (*lcb_ioC_write2_callback)(lcb_sockdata_t *sd, int status, void *arg);

/**
 * @brief Schedule a flush of a series of buffers to the network
 *
 * @param iops the I/O context
 * @param sd the socket on which to send
 * @param iov an array of IOV structures.
 *        The buffers pointed to by the IOVs themselves (i.e. `iov->iov_len`)
 *        **must** not be freed or modified until the callback has been invoked.
 *        The storage for the IOVs themselves (i.e. the array passed in `iov`)
 *        is copied internally to the implementation.
 *
 * @param niov the number of IOV structures within the array
 * @param uarg an opaque pointer to be passed in the callback
 * @param callback the callback to invoke. This will be called when the buffers
 *        passed have either been completely flushed (and are no longer required)
 *        or when an error has taken place.
 */
typedef int (*lcb_ioC_write2_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd, lcb_IOV *iov, lcb_SIZE niov, void *uarg,
                                 lcb_ioC_write2_callback callback);

/**
 * @brief Callback invoked when a read has been completed
 * @param sd the socket
 * @param nread number of bytes read, or -1 on error
 * @param arg user provided argument for callback.
 */
typedef void (*lcb_ioC_read2_callback)(lcb_sockdata_t *sd, lcb_SSIZE nread, void *arg);
/**
 * @brief Schedule a read from the network
 * @param iops the I/O context
 * @param sd the socket on which to read
 * @param iov an array of IOV structures
 * @param niov the number of IOV structures within the array
 * @param uarg a pointer passed to the callback
 * @param callback the callback to invoke
 * @return 0 on success, nonzero on error
 *
 * The IOV array itself shall copied (if needed) into the I/O implementation
 * and thus does not need to be kept in memory after the function has been
 * called. Note that the underlying buffers _do_ need to remain valid until
 * the callback is received.
 */
typedef int (*lcb_ioC_read2_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd, lcb_IOV *iov, lcb_SIZE niov, void *uarg,
                                lcb_ioC_read2_callback callback);

/**
 * @brief Asynchronously shutdown the socket.
 *
 * Request an asynchronous close for the specified socket. This merely releases
 * control from the library over to the plugin for the specified socket and
 * does _not_ actually imply that the resources have been closed.
 *
 * Notable, callbacks for read and write operations will _still_ be invoked
 * in order to maintain proper resource deallocation. However the socket's
 * closed field will be set to true.
 *
 * @param iops the I/O context
 * @param sd the socket structure
 */
typedef unsigned int (*lcb_ioC_close_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd);

/**
 * This is the completion variant of @ref lcb_ioE_chkclosed_fn. See that
 * function for details
 *
 * @param iops
 * @param sd
 * @param flags
 * @return
 */
typedef int (*lcb_ioC_chkclosed_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd, int flags);

/**
 * @see lcb_ioE_cntl_fn.
 *
 * @param iops
 * @param sd
 * @param mode
 * @param option
 * @param arg
 * @return
 */
typedef int (*lcb_ioC_cntl_fn)(lcb_io_opt_t iops, lcb_sockdata_t *sd, int mode, int option, void *arg);

/**@}*/

/**
 * @brief Start the event loop
 * @param iops The I/O context
 *
 * This should start polling for socket events on all registered watchers
 * and scheduled events. This function should return either when there are
 * no more timers or events pending, or when lcb_io_stop_fn() has been invoked.
 */
typedef void (*lcb_io_start_fn)(lcb_io_opt_t iops);

/**
 * @brief Run a single iteration of the event loop without blocking. This
 * is intended to be an optimization to allow scheduled I/O operations to
 * complete without blocking the main thread
 */
typedef void (*lcb_io_tick_fn)(lcb_io_opt_t iops);

/**
 * @brief Pause the event loop
 * @param iops The I/O Context
 *
 * This function shall suspend the event loop, causing a current invocation
 * to lcb_io_start_fn() to return as soon as possible
 */
typedef void (*lcb_io_stop_fn)(lcb_io_opt_t iops);

LCB_DEPRECATED(typedef void (*lcb_io_error_cb)(lcb_sockdata_t *socket));

#define LCB_IOPS_BASE_FIELDS                                                                                           \
    void *cookie;                                                                                                      \
    int error;                                                                                                         \
    int need_cleanup;

struct lcb_iops_evented_st {
    LCB_IOPS_BASE_FIELDS
    LCB_IOPS_DEPRECATED(lcb_ioE_socket_fn socket);
    LCB_IOPS_DEPRECATED(lcb_ioE_connect_fn connect);
    LCB_IOPS_DEPRECATED(lcb_ioE_recv_fn recv);
    LCB_IOPS_DEPRECATED(lcb_ioE_send_fn send);
    LCB_IOPS_DEPRECATED(lcb_ioE_recvv_fn recvv);
    LCB_IOPS_DEPRECATED(lcb_ioE_sendv_fn sendv);
    LCB_IOPS_DEPRECATED(lcb_ioE_close_fn close);
    LCB_IOPS_DEPRECATED(lcb_io_timer_create_fn create_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_destroy_fn destroy_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_cancel_fn delete_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_schedule_fn update_timer);
    LCB_IOPS_DEPRECATED(lcb_ioE_event_create_fn create_event);
    LCB_IOPS_DEPRECATED(lcb_ioE_event_destroy_fn destroy_event);
    LCB_IOPS_DEPRECATED(lcb_ioE_event_watch_fn update_event);
    LCB_IOPS_DEPRECATED(lcb_ioE_event_cancel_fn delete_event);
    LCB_IOPS_DEPRECATED(lcb_io_stop_fn stop_event_loop);
    LCB_IOPS_DEPRECATED(lcb_io_start_fn run_event_loop);
};

struct lcb_iops_completion_st {
    LCB_IOPS_BASE_FIELDS
    LCB_IOPS_DEPRECATED(lcb_ioC_socket_fn create_socket);
    LCB_IOPS_DEPRECATED(lcb_ioC_connect_fn start_connect);
    LCB_IOPS_DEPRECATED(lcb_ioC_wballoc_fn create_writebuf);
    LCB_IOPS_DEPRECATED(lcb_ioC_wbfree_fn release_writebuf);
    LCB_IOPS_DEPRECATED(lcb_ioC_write_fn start_write);
    LCB_IOPS_DEPRECATED(lcb_ioC_read_fn start_read);
    LCB_IOPS_DEPRECATED(lcb_ioC_close_fn close_socket);
    LCB_IOPS_DEPRECATED(lcb_io_timer_create_fn create_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_destroy_fn destroy_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_cancel_fn delete_timer);
    LCB_IOPS_DEPRECATED(lcb_io_timer_schedule_fn update_timer);
    LCB_IOPS_DEPRECATED(lcb_ioC_nameinfo_fn get_nameinfo);
    void (*pad1)(void);
    void (*pad2)(void);
    LCB_IOPS_DEPRECATED(void (*send_error)(struct lcb_io_opt_st *, lcb_sockdata_t *, void (*)(lcb_sockdata_t *)));
    LCB_IOPS_DEPRECATED(lcb_io_stop_fn stop_event_loop);
    LCB_IOPS_DEPRECATED(lcb_io_start_fn run_event_loop);
};

/** @brief Common functions for starting and stopping timers */
typedef struct lcb_timerprocs_st {
    lcb_io_timer_create_fn create;
    lcb_io_timer_destroy_fn destroy;
    lcb_io_timer_cancel_fn cancel;
    lcb_io_timer_schedule_fn schedule;
} lcb_timer_procs;

/** @brief Common functions for starting and stopping the event loop */
typedef struct lcb_loopprocs_st {
    lcb_io_start_fn start;
    lcb_io_stop_fn stop;
    lcb_io_tick_fn tick;
} lcb_loop_procs;

/** @brief Functions wrapping the Berkeley Socket API */
typedef struct lcb_bsdprocs_st {
    lcb_ioE_socket_fn socket0;
    lcb_ioE_connect_fn connect0;
    lcb_ioE_recv_fn recv;
    lcb_ioE_recvv_fn recvv;
    lcb_ioE_send_fn send;
    lcb_ioE_sendv_fn sendv;
    lcb_ioE_close_fn close;
    lcb_ioE_bind_fn bind;
    lcb_ioE_listen_fn listen;
    lcb_ioE_accept_fn accept;
    lcb_ioE_chkclosed_fn is_closed;
    lcb_ioE_cntl_fn cntl;
} lcb_bsd_procs;

/** @brief Functions handling socket watcher events */
typedef struct lcb_evprocs_st {
    lcb_ioE_event_create_fn create;
    lcb_ioE_event_destroy_fn destroy;
    lcb_ioE_event_cancel_fn cancel;
    lcb_ioE_event_watch_fn watch;
} lcb_ev_procs;

/** @brief Functions for completion-based I/O */
typedef struct {
    lcb_ioC_socket_fn socket;
    lcb_ioC_close_fn close;
    lcb_ioC_read_fn read;
    lcb_ioC_connect_fn connect;
    lcb_ioC_wballoc_fn wballoc;
    lcb_ioC_wbfree_fn wbfree;
    lcb_ioC_write_fn write;
    lcb_ioC_write2_fn write2;
    lcb_ioC_read2_fn read2;
    lcb_ioC_serve_fn serve;
    lcb_ioC_nameinfo_fn nameinfo;
    lcb_ioC_chkclosed_fn is_closed;
    lcb_ioC_cntl_fn cntl;
} lcb_completion_procs;

/**
 * Enumeration defining the I/O model
 */
typedef enum {
    LCB_IOMODEL_EVENT,     /**< Event/Poll style */
    LCB_IOMODEL_COMPLETION /**< IOCP/Completion style */
} lcb_iomodel_t;

/**
 * @param version the ABI/API version for the proc structures. Note that
 *  ABI is forward compatible for all proc structures, meaning that newer
 *  versions will always extend new fields and never replace existing ones.
 *  However in order to avoid a situation where a newer version of a plugin
 *  is loaded against an older version of the library (in which case the plugin
 *  will assume the proc table size is actually bigger than it is) the version
 *  serves as an indicator for this. The version actually passed is defined
 *  in `LCB_IOPROCS_VERSION`
 *
 * @param loop_procs a table to be set to basic loop control routines
 * @param timer_procs a table to be set to the timer routines
 * @param bsd_procs a table to be set to BSD socket API routines
 * @param ev_procs a table to be set to event watcher routines
 * @param completion_procs a table to be set to completion routines
 * @param iomodel the I/O model to be used. If this is `LCB_IOMODEL_COMPLETION`
 * then the contents of `bsd_procs` will be ignored and `completion_procs` must
 * be populated. If the mode is `LCB_IOMODEL_EVENT` then the `bsd_procs` must be
 * populated and `completion_procs` is ignored.
 *
 * Important to note that internally the `ev`, `bsd`, and `completion` field are
 * defined as a union, thus
 * @code{.c}
 * union {
 *     struct {
 *         lcb_bsd_procs;
 *         lcb_ev_procs;
 *     } event;
 *     struct lcb_completion_procs completion;
 * }
 * @endcode
 * thus setting both fields will actually clobber.
 *
 * @attention
 * Note that the library takes ownership of the passed tables and it should
 * not be controlled or accessed by the plugin.
 *
 * @attention
 * This function may not have any side effects as it may be called
 * multiple times.
 *
 * As opposed to the v0 and v1 IOPS structures that require a table to be
 * populated and returned, the v2 IOPS works differently. Specifically, the
 * IOPS population happens at multiple stages:
 *
 * 1. The base structure is returned, i.e. `lcb_create_NAME_iops` where _NAME_
 *    is the name of the plugin
 *
 * 2. Once the structure is returned, LCB shall invoke the `v.v2.get_procs()`
 *    function. The callback is responsible for populating the relevant fields.
 *
 * Note that the old `v0` and `v1` fields are now proxied via this mechanism.
 * It _is_ possible to still monkey-patch the IO routines, but ensure the
 * monkey patching takes place _before_ the instance is created (as the
 * instance will initialize its own IO Table); thus, e.g.
 * @code{.c}
 * static void monkey_proc_fn(...) {
 *     //
 * }
 *
 * static void monkey_patch_io(lcb_io_opt_t io) {
 *     io->v.v0.get_procs = monkey_proc_fn;
 * }
 *
 * int main(void) {
 *     lcb_create_st options;
 *     lcb_INSTANCE instance;
 *     lcb_io_opt_t io;
 *     lcb_create_iops(&io, NULL);
 *     monkey_patch_io(io);
 *     options.v.v0.io = io;
 *     lcb_create(&instance, &options);
 *     // ...
 * }
 * @endcode
 *
 * Typically the `get_procs` function will only be called once, and this will
 * happen from within lcb_create(). Thus in order to monkey patch you must
 * ensure that initially the `get_procs` function itself is first supplanted
 * and then return your customized I/O routines from your own `get_procs` (in
 * this example, `monkey_proc_fn()`)
 *
 */
typedef void (*lcb_io_procs_fn)(int version, lcb_loop_procs *loop_procs, lcb_timer_procs *timer_procs,
                                lcb_bsd_procs *bsd_procs, lcb_ev_procs *ev_procs,
                                lcb_completion_procs *completion_procs, lcb_iomodel_t *iomodel);

struct lcbio_TABLE;
struct lcb_iops2_st {
    LCB_IOPS_BASE_FIELDS
    lcb_io_procs_fn get_procs;
    struct lcbio_TABLE *iot;
};

/* This is here to provide backwards compatibility with older (broken) clients
 * which attempt to 'subclass' the select plugin, or similar. In this case we
 * provide 17 callback fields (unused here) which the plugin implementation
 * may set, so that the older code can continue to function without upgrading
 * the client to a newer version. This should not be used except by internal
 * plugins; specifically the ABI layout of this field is subject to change
 * (for example, additional fields may be added or existing fields may be
 * renamed/removed) without notice.
 */
typedef void (*lcb__iops3fndummy)(void);
struct lcb_iops3_st {
    LCB_IOPS_BASE_FIELDS
    lcb__iops3fndummy pads[17];
    lcb_io_procs_fn get_procs;
    struct lcbio_TABLE *iot;
};

/**
 * This number is bumped up each time a new field is added to any of the
 * function tables. This number is backwards compatible (i.e. version 3 contains
 * all the fields of version 2, and some additional ones)
 */
#define LCB_IOPROCS_VERSION 4

#define LCB_IOPS_BASEFLD(iops, fld) ((iops)->v.base).fld
#define LCB_IOPS_ERRNO(iops) LCB_IOPS_BASEFLD(iops, error)

struct lcb_io_opt_st {
    int version;
    void *dlhandle;
    void (*destructor)(struct lcb_io_opt_st *iops);
    union {
        struct {
            LCB_IOPS_BASE_FIELDS
        } base;

        /** These two names are deprecated internally */
        struct lcb_iops_evented_st v0;
        struct lcb_iops_completion_st v1;
        struct lcb_iops2_st v2;
        struct lcb_iops3_st v3;
    } v;
};

/**
 * @brief Signature for a loadable plugin's IOPS initializer
 *
 * @param version the plugin init API version. This will be 0 for this function
 * @param io a pointer to be set to the I/O table
 * @param cookie a user-defined argument passed to the I/O initializer
 * @return LCB_SUCCESS on success, an error on failure
 */
typedef lcb_STATUS (*lcb_io_create_fn)(int version, lcb_io_opt_t *io, void *cookie);

/**
 * @volatile
 *
 * This is an alternative to copying the 'bsdio-inl.c' file around. It is
 * designed specifically for the @ref lcb_io_procs_fn function and will do the
 * job of applying the current _runtime_ version of the default event-based I/O
 * implementation.
 *
 * e.g.
 * @code{.c}
 * static void getprocs_impl(int version, lcb_loop_procs *loop_procs,
 *      lcb_timer_procs *timer_procs, lcb_bsd_procs *bsd_procs,
 *      lcb_ev_procs *ev_procs, lcb_completion_procs *completion_procs,
 *      lcb_iomodel_t *iomodel) {
 *
 *      // do stuff normally
 *      // ..
 *      // install the default I/O handlers:
 *      lcb_iops_wire_bsd_impl2(bsd_procs, version);
 * @endcode
 *
 * Use this function with care, and understand the implications between using
 * this API call and embedding the `bsdio-inl.c` source file. Specifically:
 *
 * - If your application is using an _older_ version of the library, this
 *   implementation may contain bugs not present in the version you compiled
 *   against (and an embedded version may be newer)
 * - If your application is using a _newer_ version, there may be some additional
 *   I/O functions which you may wish to wrap or rather not implement at all,
 *   but will be implemented if you call this function.
 */
LIBCOUCHBASE_API
void lcb_iops_wire_bsd_impl2(lcb_bsd_procs *procs, int version);

/******************************************************************************
 ******************************************************************************
 ** IO CREATION                                                              **
 ******************************************************************************
 ******************************************************************************/

/**
 * @brief Built-in I/O plugins
 * @committed
 */
typedef enum {
    LCB_IO_OPS_INVALID = 0x00, /**< @internal */
    LCB_IO_OPS_DEFAULT = 0x01, /**< @internal */

    /** Integrate with the libevent loop. See lcb_create_libevent_io_opts() */
    LCB_IO_OPS_LIBEVENT = 0x02,
    LCB_IO_OPS_WINSOCK = 0x03, /**< @internal */
    LCB_IO_OPS_LIBEV = 0x04,
    LCB_IO_OPS_SELECT = 0x05,
    LCB_IO_OPS_WINIOCP = 0x06,
    LCB_IO_OPS_LIBUV = 0x07
} lcb_io_ops_type_t;

/** @brief IO Creation for builtin plugins */
typedef struct {
    lcb_io_ops_type_t type; /**< The predefined type you want to create */
    void *cookie;           /**< Plugin-specific argument */
} lcb_IOCREATEOPTS_BUILTIN;

#ifndef __LCB_DOXYGEN__
/* These are mostly internal structures which may be in use by older applications.*/
typedef struct {
    const char *sofile;
    const char *symbol;
    void *cookie;
} lcb_IOCREATEOPTS_DSO;
typedef struct {
    lcb_io_create_fn create;
    void *cookie;
} lcb_IOCREATEOPS_FUNCTIONPOINTER;
#endif

/** @uncommitted */
struct lcb_create_io_ops_st {
    int version;
    union {
        lcb_IOCREATEOPTS_BUILTIN v0;
        lcb_IOCREATEOPTS_DSO v1;
        lcb_IOCREATEOPS_FUNCTIONPOINTER v2;
    } v;
};

/**
 * Create a new instance of one of the library-supplied io ops types.
 *
 * This function should only be used if you wish to override/customize the
 * default I/O plugin behavior; for example to select a specific implementation
 * (e.g. always for the _select_ plugin) and/or to integrate
 * a builtin plugin with your own application (e.g. pass an existing `event_base`
 * structure to the _libevent_ plugin).
 *
 * If you _do_ use this function, then you must call lcb_destroy_io_ops() on
 * the plugin handle once it is no longer required (and no instance is using
 * it).
 *
 * Whether a single `lcb_io_opt_t` may be used by multiple instances at once
 * is dependent on the specific implementation, but as a general rule it should
 * be assumed to be unsafe.
 *
 * @param[out] op The newly created io ops structure
 * @param options How to create the io ops structure
 * @return @ref LCB_SUCCESS on success
 * @uncommitted
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_create_io_ops(lcb_io_opt_t *op, const struct lcb_create_io_ops_st *options);

/**
 * Destroy the plugin handle created by lcb_create_io_ops()
 * @param op ops structure
 * @return LCB_SUCCESS on success
 * @uncommitted
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_destroy_io_ops(lcb_io_opt_t op);

#ifdef __cplusplus
}
#endif

/**@}*/

#endif
