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

#ifndef LCBIO_CTXEASY_H
#define LCBIO_CTXEASY_H
#include "connect.h"
#include "rdb/rope.h"
#include "ringbuffer.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file
 * This file contains routines for reading and writing from and to a socket
 */

/**
 * @ingroup lcbio
 * @defgroup lcbio-ctx Reading/Writing Routines
 *
 * @details
 *
 * # Attaching
 *
 * A context is first _attached_ to a socket and a _user-defined_ data object.
 * The idea is that the context is the broker which schedules I/O on behalf
 * of the application to the socket, and then receives events from the socket,
 * passing it along to user-defined data.
 *
 * To create a new context, invoke the lcbio_ctx_new() function. When you are done
 * with it call the lcbio_ctx_close() function.
 *
 * # Reading
 *
 * Reading data is done through first requesting an amount of data to read
 * and then reading the data from the buffers when the lcbio_CTXPROCS#cb_read
 * handler is invoked.
 *
 * Requesting an amount of data to be read may be dependent on some current
 * parsing context. In cases where the expected message size is known the pattern
 * is to initially request the size of the header; once the header has been
 * delivered to the application, the application should request the full
 * header+body size, and so on.
 *
 * For streaming based protocols where the amount of data is not known ahead of
 * time, requesting a single byte may be sufficient. Note that typically the
 * read callback will be invoked with _more_ bytes than requested.
 *
 * Data is read from the network as one or more chunks to improve performance
 * and increase flexibility. Because of this model, you must iterate over the
 * data read _or_ employ the underlying <rdb/rope.h> API. For streaming data
 * sockets where data may simply be copied to another buffer, use the iterator
 * API presented here.
 *
 * @note The RDB interface requires you to explicitly advance the read
 * cursor in addition to actually obtaining the data. This is handled within
 * the iterator interface declared in this file (but of course must be done
 * manually if employing RDB directly).
 *
 *
 * # Writing
 *
 * Writing can be done through a simple lcbio_ctx_put() which simply copies data
 * to an output buffer, or can be done through more efficient but complex means -
 * see the lcbio_ctx_wwant() function.
 *
 * # Scheduling
 *
 * Any I/O **must always be scheduled**. For maximal efficiency the various
 * read/write functions only set internal flags within the contexts to act
 * as hints that the application intends to read and write data. However in
 * order to actually perform these operations, kernel/OS calls are needed.
 *
 * To indicate that no subsequent I/O operations will be requested until the next
 * event loop iteration, _and_ to apply/schedule any existing I/O within the
 * current iteration, the lcbio_ctx_schedule() function should be called.
 *
 *
 * @addtogroup lcbio-ctx
 * @{ */

typedef struct lcbio_CTX *lcbio_pCTX;

/**
 * @brief Handlers for I/O events
 */
typedef struct {
    /** Error handler invoked with the context and the received error */
    void (*cb_err)(lcbio_pCTX, lcb_STATUS);

    /** Read handler invoked with the context and the number of bytes read */
    void (*cb_read)(lcbio_pCTX, unsigned total);

    /** Triggered by lcbio_ctx_wwant() */
    void (*cb_flush_ready)(lcbio_pCTX);

    /** Triggered when data has been flushed from lcbio_ctx_put_ex() */
    void (*cb_flush_done)(lcbio_pCTX, unsigned requested, unsigned nflushed);
} lcbio_CTXPROCS;

/**
 * Container buffer handle containing a backref to the original context.
 * @private
 */
typedef struct {
    ringbuffer_t rb;
    lcbio_pCTX parent;
} lcbio__EASYRB;

/**
 * @brief Context for socket I/O
 *
 * The CTX structure represents an ownership of a socket. It provides routines
 * for reading and writing from and to the socket, as well as associating
 * application data with the socket.
 */
typedef struct lcbio_CTX {
    lcbio_SOCKET *sock;    /**< Socket resource */
    lcbio_pTABLE io;       /**< Cached IO table */
    void *data;            /**< Associative pointer */
    void *event;           /**< event pointer for E-model I/O */
    lcb_sockdata_t *sd;    /**< cached SD for C-model I/O */
    lcbio__EASYRB *output; /**< for lcbio_ctx_put() */
    lcb_socket_t fd;       /**< cached FD for E-model I/O */
    char evactive;         /**< watcher is active for E-model I/O */
    char wwant;            /**< flag for lcbio_ctx_put_ex */
    char state;            /**< internal state */
    char entered;          /**< inside event handler */
    unsigned npending;     /**< reference count on pending I/O */
    unsigned rdwant;       /**< number of remaining bytes to read */
    lcb_STATUS err;        /**< pending error */
    rdb_IOROPE ior;        /**< for reads */
    lcbio_pASYNC as_err;   /**< async error handler */
    lcbio_CTXPROCS procs;  /**< callbacks */
    const char *subsys;    /**< Informational description of connection */
} lcbio_CTX;

/**@name Creating and Closing
 *@{*/

/**
 * Creates a new context object.
 *
 * The object remains valid until lcbio_ctx_close() is invoked.
 *
 * @param sock the underlying socket object. This function increments the
 *        socket's reference count.
 * @param data user defined data to associate with context. The data may be
 *        obtained at a later point via lcbio_ctx_data()
 *
 * @param procs callback table
 * @return a new context object.
 */
lcbio_CTX *lcbio_ctx_new(lcbio_SOCKET *sock, void *data, const lcbio_CTXPROCS *procs);

/**
 * Callback invoked when the connection is about to be release
 * @param sock the socket being released.
 * @param releasable whether the socket may be reused
 * @param arg an argument passed to the close() function.
 *
 * If you wish to reuse the socket (and reusable is true) then the socket's
 * reference count should be incremented via lcbio_ref().
 */
typedef void (*lcbio_CTXCLOSE_cb)(lcbio_SOCKET *sock, int releasable, void *arg);

/**
 * @brief Close the context object.
 *
 * This will invalidate any pending I/O operations
 * and subsequent callbacks on the context will not be received. After calling
 * this function, the pointer will be deemed invalid.
 *
 * @param ctx
 * @param callback a callback to invoke (see above)
 * @param arg argument passed to the callback
 */
void lcbio_ctx_close(lcbio_CTX *ctx, lcbio_CTXCLOSE_cb callback, void *arg);

typedef void (*lcbio_CTXDTOR_cb)(lcbio_pCTX);
void lcbio_ctx_close_ex(lcbio_CTX *ctx, lcbio_CTXCLOSE_cb cb, void *cbarg, lcbio_CTXDTOR_cb dtor, void *dtor_arg);
/**@}*/

/**@name Informational Routines
 * @{*/

/**
 * Get the data associated with the context. This is the pointer specified
 * during the constructor
 */
#define lcbio_ctx_data(ctx) (ctx)->data

/** Get the associated lcbio_SOCKET object */
#define lcbio_ctx_sock(ctx) (ctx)->sock

/** Dump a textual representation of the context to the screen */
void lcbio_ctx_dump(lcbio_CTX *ctx, FILE *fp);

/**@}*/

/** Asynchronously trigger the error callback */
void lcbio_ctx_senderr(lcbio_CTX *ctx, lcb_STATUS err);

/**
 * Schedule any pending I/O to be scheduled immediately. If data was requested
 * via lcbio_ctx_rwant() then a request will be sent for reading. If data was
 * requested to be flushed either via lcbio_ctx_put() or lcbio_ctx_wwant()
 * then those will be scheduled as well.
 *
 * This call is a no-op if invoked from within the current handler, as this
 * function is invoked implicitly after the I/O handler itself has returned.
 *
 * It is safe (though typically not efficient) to invoke this function
 * multiple times. Each invocation may potentially involve system calls
 * and buffer allocations, depending on the I/O plugin being used.
 */
void lcbio_ctx_schedule(lcbio_CTX *ctx);

/**
 * @brief Add output data to the write buffer.
 *
 * The data added is copied to an internal buffer and flushed to the network
 * when appropriate. If you wish to have more control over how the data is written
 * then see the lcbio_ctx_wwant() function.
 *
 * @param ctx
 * @param buf the buffer to write
 * @param nbuf the size of the buffer to write
 */
void lcbio_ctx_put(lcbio_CTX *ctx, const void *buf, unsigned nbuf);

/**
 * Invoke the lcbio_CTXPROCS#cb_flush_ready()
 * callback when a flush may be invoked. Note that the
 * callback may be invoked from within this function itself, or
 * it may be invoked at some point later.
 *
 * In order to ensure that the callback is actually invoked (in
 * cases where it is not invoked immediately), call lcbio_ctx_schedule()
 * before returning to the loop.
 *
 * When the callback is invoked, you should call the lcbio_ctx_put_ex() function
 * to actually enqueue the data to be written. The lcbio_ctx_put_ex() function
 * should be called multiple times until either no write buffers remain
 * or the function itself returns a false value. The lcbio_ctx_put_ex() function
 * may either flush the data immediately or schedule for the data to be flushed
 * depending on the I/O implementation and network conditions.
 *
 * Once the data is actually flushed to the socket's buffers, the
 * lcbio_CTXPROCS#cb_flush_done() callback is invoked.
 * This callback indicates the underlying buffers are no longer required and may
 * be released or reused by the application. Note that the IOV array passed
 * into lcbio_ctx_put_ex() is always _Conceptually_ copied (i.e.
 * this may be a stack-based structure which does not need to remain valid
 * outside the function call to lcbio_ctx_put_ex() itself).
 *
 * Additionally, note that the number of bytes flushed within the
 * lcbio_CTXPROCS#cb_flush_done()
 * callback may not equal the number of bytes initially placed inside the IOVs
 * (i.e. it may be less). In this case the application is expected to update
 * the IOV structures and the origin buffers appropriately.
 *
 * This model allows for efficient handling in both completion and event based
 * environments.
 *
 * ### Implementation Notes
 *
 * For completion-based models, the lcbio_CTXPROCS#cb_flush_ready()
 * callback is invoked immediately from the wwant() function, while the
 * flush_done() is dependent on the actual completion of the write.
 *
 * For event-based models, the wwant flag is set inside the context and is then
 * checked by the lcbio_ctx_schedule() function. When the event handler is invoked, the
 * flush_ready() callback is invoked as well - typically in a loop until an
 * `EWOULDBLOCK` is received on the socket itself.
 */
void lcbio_ctx_wwant(lcbio_CTX *ctx);

/**
 * @brief Flush data from the lcbio_CTXPROCS#cb_flush_ready() callback
 *
 * This function is intended to be called from within the `cb_flush_ready`
 * handler (see lcbio_ctx_wwant()).
 *
 * @param ctx
 * @param iov the IOV array. The IOV array may point to a stack-based array
 * @param niov number of elements in the array
 * @param nb The total number of bytes described by all the elements of the
 * array.
 *
 * @return nonzero if more data can be written (i.e. this function may be
 * invoked again), zero otherwise.
 */
int lcbio_ctx_put_ex(lcbio_CTX *ctx, lcb_IOV *iov, unsigned niov, unsigned nb);

/**
 * Require that the read callback not be invoked until at least `n`
 * bytes are available within the buffer.
 *
 * @param ctx
 * @param n the number of bytes required to be in the buffer before the
 *        callback should be invoked.
 *
 * @note
 * Note that this flag does _not_ maintain state between successive callbacks.
 * You must call this function each time you need more data as it is cleared
 * before the invocation into the callback.
 *
 * @note
 * This function sets the number of **total** bytes
 * which must be available in the buffer before the callback is invoked. Thus
 * you should set this to the total number of bytes needed, and **not** the
 * number of remaining bytes that should be read.
 */
void lcbio_ctx_rwant(lcbio_CTX *ctx, unsigned n);

/** @private */
typedef struct {
    unsigned remaining;
    void *buf;
    unsigned nbuf;
} lcbio_CTXRDITER;

/**
 * Iterate over the read buffers
 *
 * @code{.c}
 * static void read_callback(lcbio_CTX *ctx, void *arg, unsigned nb) {
 *     lcbio_CTXRDITER iter;
 *     LCBIO_CTX_ITERFOR(ctx, &iter, nb) {
 *         void *buf = lcbio_ctx_ribuf(&iter);
 *         unsigned nbuf = lcbio_ctx_risize(&iter);
 *          // do stuff with the buffer
 *     }
 * }
 * @endcode
 *
 * When each iteration is complete, the pointer returned by ctx_ribuf is
 * no longer valid.
 *
 * @param ctx the context which contains the buffer
 * @param[in,out] iter an empty iterator
 * @param[in] nb the number of bytes to iterate over.
 */
#define LCBIO_CTX_ITERFOR(ctx, iter, nb)                                                                               \
    for (lcbio_ctx_ristart(ctx, iter, nb); !lcbio_ctx_ridone(iter); lcbio_ctx_rinext(ctx, iter))

/** Obtains the buffer from the current iterator */
#define lcbio_ctx_ribuf(iter) ((iter)->buf)

/** Obtains the length of the buffer from the current iterator */
#define lcbio_ctx_risize(iter) ((iter)->nbuf)

void lcbio_ctx_ristart(lcbio_CTX *ctx, lcbio_CTXRDITER *iter, unsigned nb);

void lcbio_ctx_rinext(lcbio_CTX *ctx, lcbio_CTXRDITER *iter);

#define lcbio_ctx_ridone(iter) (!(iter)->remaining)

#define LCBIO_CTX_RSCHEDULE(ctx, nb)                                                                                   \
    do {                                                                                                               \
        lcbio_ctx_rwant(ctx, nb);                                                                                      \
        lcbio_ctx_schedule(ctx);                                                                                       \
    } while (0)

#ifdef __cplusplus
}
#endif
#endif

/** @} */
