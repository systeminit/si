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

#ifndef LCB_SSL_IOTCOMMON
#define LCB_SSL_IOTCOMMON

#include <lcbio/lcbio.h>
#include <lcbio/iotable.h>
#include <lcbio/timer-ng.h>
#include <stddef.h>
#include <errno.h>
#include <openssl/ssl.h>
#include <lcbio/ssl.h>

#define IOTSSL_COMMON_FIELDS                                                                                           \
    lcbio_TABLE base_;        /**< Base table structure to export */                                                   \
    lcbio_pTABLE orig;        /**< Table pointer we are wrapping */                                                    \
    SSL *ssl;                 /**< SSL object */                                                                       \
    BIO *wbio;                /**< BIO used for writing data to network */                                             \
    BIO *rbio;                /**< BIO used for reading data from network */                                           \
    lcb_io_opt_t iops_dummy_; /**< Dummy IOPS structure which is exposed to LCB */                                     \
    int error;                /**< Internal error flag set once a fatal error is detect */                             \
    lcb_STATUS errcode;       /**< The error, converted into libcouchbase */

/**
 * @brief
 * This is the 'base' class for the lcbio_TABLE with SSL. This contains the
 * core BIO and SSL structures as well as some other boilerplate needed to
 * expose a complete lcbio_TABLE interface to the rest of the library.
 *
 * This is 'subclassed' as lcbio_CSSL and lcbio_ESSL for Completion and Event
 * based I/O models respectively.
 */
typedef struct {
    IOTSSL_COMMON_FIELDS
} lcbio_XSSL;

/**
 * @brief Get the associated lcbio_XSSL from an iops pointer
 * @param iops the IOPS structure
 * @return the lcbio_XSSL pointer
 */
#define IOTSSL_FROM_IOPS(iops) (iops)->v.v0.cookie

/**
 * @brief Access the iops `error` field which is exposed to the rest of LCBIO.
 * The rest of the code will inspect this variable when a call fails to retrieve
 * the errno.
 * @param xs the lcbio_XSSL base pointer
 * @return An lvaue to the error field
 */
#define IOTSSL_ERRNO(xs) (xs)->iops_dummy_->v.v0.error

/**
 * @brief Check and handle SSL errors
 *
 * This function inspects the error state of the current `SSL` object. If a
 * fatal error is detected, the internal error flag is set and pending errors
 * are logged.
 *
 * @param xs The XSSL context
 * @param rv The return code received from an `SSL_read()` or `SSL_write()`
 * @return nonzero if a fatal error has occurred, 0 if the error is transient
 * and is `SSL_ERROR_WANT_READ` or `SSL_ERROR_WANT_WRITE`.
 *
 * @note Do not call this function if `rv` is `>0`.
 */
int iotssl_maybe_error(lcbio_XSSL *xs, int rv);

/**
 * Flush errors from the internal error queue. Call this whenever an error
 * has taken place
 * @param xs
 */
void iotssl_log_errors(lcbio_XSSL *xs);

/**
 * This function acts as the 'base' constructor for lcbio_XSSL. It will
 * initialize and proxy the various timer and run/stop routines to the underlying
 * iops plugin. The 'subclass' is still expected to implement the actual send,
 * recv, close, and event routines.
 *
 * @param xs The lcbio_XSSL pointer to initialize (usually a pointer to a field
 * within the real structure)
 * @param orig The original lcbio_TABLE containing the actual socket I/O routines.
 * @param ctx the `SSL_CTX*` which will be used to create the `SSL*` pointer
 */
void iotssl_init_common(lcbio_XSSL *xs, lcbio_TABLE *orig, SSL_CTX *ctx);

/**
 * This function acts as the base destructor for lcbio_XSSL
 * @param xs the lcbio_XSSL to clean up.
 * After this function has been called, none of the base fields should be
 * considered valid (unless a refcounted item is specifically kept alive).
 */
void iotssl_destroy_common(lcbio_XSSL *xs);

#if LCB_CAN_OPTIMIZE_SSL_BIO
/**
 * Reserve a specified amount of bytes for reading into a `BUF_MEM*` structure.
 * Currently the amount reserved is hard coded.
 *
 * Use this function to retrievw a pointer to the unused (but allocated) portion
 * of the `BUF_MEM` structure rather than doing an explicit BIO_write which
 * will result in needless copying of memory. Unfortunately OpenSSL does not have
 * a clean way of growing this buffer but it is possible.
 *
 * @param bm The `BUF_MEM*` structure.
 *
 * @code{.c}
 * BUF_MEM *bm;
 * iotssl_bm_reserve(bm);
 * recv(fd, bm->data, bm->max-mb->length, 0);
 * @endcode
 */
void iotssl_bm_reserve(BUF_MEM *bm);
#endif

/**
 * Prepare the SSL structure so that a subsequent call to SSL_pending will
 * actually determine if there's any data available for read
 * @param ssl the SSL object
 * @return
 */
#define IOTSSL_PENDING_PRECHECK(ssl)                                                                                   \
    do {                                                                                                               \
        char iotssl__dummy;                                                                                            \
        SSL_peek(ssl, &iotssl__dummy, 1);                                                                              \
    } while (0);

/**
 * Wrapper for SSL_pending. In order to work around another bug of the well
 * designed OpenSSL library, which will strangely place "undefined" errors
 * into the queue unless this check is done beforehand.
 *
 * See: https://groups.google.com/forum/#!msg/mailing.openssl.users/so242GuI6Yo/2Jp3Qoo_gsgJ
 * See: http://stackoverflow.com/questions/22753221/openssl-read-write-handshake-data-with-memory-bio
 * See: http://www.opensubscriber.com/message/openssl-users@openssl.org/8638179.html
 */
#define IOTSSL_IS_PENDING(ssl) (SSL_get_ssl_method(ssl) != SSLv23_client_method()) && SSL_pending(ssl)
/**
 * Create and return a pointer to an lcbio_TABLE with an underlying
 * completion-based I/O model
 * @param orig The original table
 * @param sd The socket descriptor which is already connected
 * @param sctx
 * @return NULL on error
 */
lcbio_pTABLE lcbio_Cssl_new(lcbio_pTABLE orig, lcb_sockdata_t *sd, SSL_CTX *sctx);

/**
 * Create and return a pointer to an lcbio_TABLE with an underlying event
 * based I/O model
 * @param orig The original pointer
 * @param fd Socket descriptor
 * @param sctx
 * @return NULL on error.
 */
lcbio_pTABLE lcbio_Essl_new(lcbio_pTABLE orig, lcb_socket_t fd, SSL_CTX *sctx);

#endif
