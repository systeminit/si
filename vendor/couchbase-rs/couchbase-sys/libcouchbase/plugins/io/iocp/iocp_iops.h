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

#ifndef LCB_IOCP_H
#define LCB_IOCP_H

#include <libcouchbase/assert.h>

#define WIN32_NO_STATUS
#include <libcouchbase/couchbase.h>
#include "list.h"

#undef WIN32_NO_STATUS

#ifdef __MINGW32__
#include "mingwdefs.h"
#else
#include <ntstatus.h>
#endif

#include "iocpdefs.h"

/**
 * These macros provide atomic operations for one-time initialization
 * functions. They take as their parameter a 'LONG'.
 * The initial value of the variable should be 0. If the initialization
 * function has not run yet, it is set to 1. Subsequent functions will
 * return 1.
 */
#define IOCP_SYNCTYPE volatile LONG
#define IOCP_INITONCE(syncvar) InterlockedCompareExchange(&syncvar, 1, 0) ? 0 : 1

#ifdef __cplusplus
extern "C" {
#endif

#define LCB_IOCP_VISTA_API
#if _WIN32_WINNT < 0x0600
#undef LCB_IOCP_VISTA_API
#endif

/** @see iocp_overlapped_t */
enum { LCBIOCP_ACTION_NONE = 100, LCBIOCP_ACTION_READ, LCBIOCP_ACTION_WRITE, LCBIOCP_ACTION_CONNECT };

struct iocp_sockdata_st;

/**
 * This structure is our 'overlapped' subclass. It in itself does not
 * contain any data, but rather determines how to read the
 * CompletionKey passed along with GetQueuedCompletionStatus
 * This information is determined from the ::action field */
typedef struct {
    OVERLAPPED base;
    struct iocp_sockdata_st *sd;
    unsigned char action;
} iocp_overlapped_t;

typedef enum { IOCP_WRITEBUF_AVAILABLE = 0, IOCP_WRITEBUF_INUSE, IOCP_WRITEBUF_ALLOCATED } iocp_wbuf_state_t;

typedef struct {
    iocp_overlapped_t ol_write;
    lcb_ioC_write2_callback cb;
    iocp_wbuf_state_t state;
    void *uarg;
} iocp_write_t;

#define IOCP_WRITEOBJ_FROM_OVERLAPPED(ol) (iocp_write_t *)(((char *)ol) - offsetof(iocp_write_t, ol_write))

typedef struct iocp_sockdata_st {
    lcb_sockdata_t sd_base;
    /**OVERLAPPED subclass used for read operations. Since no more than one read
     * per socket may be active at any given time, this is attached to the socket
     * structure */
    iocp_overlapped_t ol_read;
    /** Write structure allocated as a single chunk */
    iocp_write_t w_info;
    /** Reference count. Mainly managed by iocp_just_scheduled() and
     * iocp_on_dequeued(). This value is set to 1 for a new socket. */
    unsigned int refcount;
    /** Actual socket descriptor */
    SOCKET sSocket;
    /** Callback for read operations */
    lcb_ioC_read2_callback rdcb;
    /** Argument for read callback */
    void *rdarg;
    /** Node in linked list of sockets */
    lcb_list_t list;
} iocp_sockdata_t;

typedef struct {
    iocp_overlapped_t ol_conn;
    lcb_io_connect_cb cb;
} iocp_connect_t;

typedef struct iocp_timer_st {
    lcb_list_t list;
    char is_active;
    lcb_U64 ms;
    lcb_ioE_callback cb;
    void *arg;
} iocp_timer_t;

typedef struct iocp_st {
    struct lcb_io_opt_st base; /**< Base table */
    HANDLE hCompletionPort;    /**< Completion port */
    iocp_timer_t timer_queue;  /**< Pending timers */
    lcb_list_t sockets;        /**< List of all sockets */
    unsigned int n_iopending;  /**< Count of outstanding I/O operations */
    BOOL breakout;             /**< Flag unset during lcb_wait() and set during lcb_breakout() */
} iocp_t;

/**
 * @brief Call this function when an I/O operation has been scheduled.
 * @param io the io operations structure
 * @param ol the overlapped upon which the operation was associated with
 * @param status the return code of the scheduling API.
 * @return an error code suitable for propagating up to the library.
 *
 * Note that this function does more than convert an error code. If an operation
 * has been successfuly scheduled, the relevant socket's reference count will
 * also be incremented.
 */
int iocp_just_scheduled(iocp_t *io, iocp_overlapped_t *ol, int status);

/**
 * @brief Call this function when an I/O operation has been completed. This
 * is the logical end block of the iocp_just_scheduled() API
 * @param io the I/O operations structure
 * @param sd the socket associated with the operation
 * @param action the type of operation performed
 */
void iocp_on_dequeued(iocp_t *io, iocp_sockdata_t *sd, int action);
void iocp_socket_decref(iocp_t *io, iocp_sockdata_t *sd);

int iocp_w32err_2errno(DWORD error);
DWORD iocp_set_last_error(lcb_io_opt_t io, SOCKET sock);

/** Get current timestamp in microseconds */
lcb_U32 iocp_micros(void);

/** Get current timestamp in milliseconds */
#define iocp_millis() (iocp_micros() / 1000)

/** Initialize any globals needed by the plugin. This may be called more than
 * once, and will only initialize once. */
void iocp_initialize_loop_globals(void);

/** Call this on a new socket to retrieve its `ConnectEx` function pointer */
LPFN_CONNECTEX iocp_initialize_connectex(SOCKET s);

/** This safely invokes and restores the callback */
void iocp_write_done(iocp_t *io, iocp_write_t *w, int status);

/**Extract the actual error code from an OVERLAPPED after an operation
 * has been received on it
 * @param lpOverlapped the overlapped structure
 * @return The actual Winsock error code
 */
int iocp_overlapped_status(OVERLAPPED *lpOverlapped);

void iocp_run(lcb_io_opt_t iobase);
void iocp_stop(lcb_io_opt_t iobase);

/** Timer Functions*/
void iocp_tmq_add(lcb_list_t *list, iocp_timer_t *timer);
void iocp_tmq_del(lcb_list_t *list, iocp_timer_t *timer);
lcb_U64 iocp_tmq_next_timeout(lcb_list_t *list, lcb_U64 now);
iocp_timer_t *iocp_tmq_pop(lcb_list_t *list, lcb_U64 now);

LIBCOUCHBASE_API
lcb_STATUS lcb_iocp_new_iops(int version, lcb_io_opt_t *ioret, void *arg);

enum { IOCP_TRACE, IOCP_DEBUG, IOCP_INFO, IOCP_WARN, IOCP_ERR, IOCP_FATAL };
#ifdef IOCP_LOG_VERBOSE
#include <stdio.h>
#define IOCP_LOG(facil, ...)                                                                                           \
    fprintf(stderr, "[%s] <%s:%d>: ", #facil, __FILE__, __LINE__);                                                     \
    fprintf(stderr, __VA_ARGS__);                                                                                      \
    fprintf(stderr, "\n");
#else
#define IOCP_LOG(...)
#endif

#ifdef __cplusplus
}
#endif

#endif /* LCB_IOCP_H */
