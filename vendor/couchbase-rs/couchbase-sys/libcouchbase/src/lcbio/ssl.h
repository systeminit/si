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

#ifndef LCB_SSL_H
#define LCB_SSL_H
#include <lcbio/connect.h>
#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file
 * @brief SSL Socket Routines
 */
#ifndef LCB_NO_SSL
#if defined(OPENSSL_VERSION_NUMBER) && OPENSSL_VERSION_NUMBER < 0x10100000L
// OpenSSL 1.1 has changed behavior of BIO_get_mem_ptr behavior, so we cannot
// apply reduce-memory-copy optimization, and fallback to BIO_write
// Reference: https://github.com/openssl/openssl/commit/9fe9d0461ea
#define LCB_CAN_OPTIMIZE_SSL_BIO 1
#else
#define LCB_CAN_OPTIMIZE_SSL_BIO 0
#endif
#endif

/**
 * @ingroup lcbio
 * @defgroup lcbio-ssl SSL Routines
 *
 * @details
 * This file contains the higher level API interfacing with _LCBIO_. It provides
 * APIs to "patch" a socket with SSL as well as establish settings for SSL
 * encryption.
 *
 * @addtogroup lcbio-ssl
 * @{
 */

/** @brief Wrapper around OpenSSL's `SSL_CTX` */
typedef struct lcbio_SSLCTX *lcbio_pSSLCTX;

/**
 * @brief Determine if SSL is supported in the current build
 * @return true if supported, false otherwise
 */
int lcbio_ssl_supported(void);

lcbio_pSSLCTX lcbio_ssl_new__fallback(const char *, const char *, const char *, int, lcb_STATUS *, lcb_settings *);

#ifndef LCB_NO_SSL
/**
 * Create a new SSL context to be used to establish SSL policy.
 * @param tsfile Path to trusted store file
 * @param cafile Optional path to CA file
 * @param keyfile Path to private key file
 * @param noverify To not attempt to verify server's certificate
 * @param errp a pointer to contain the error code if initialization failed
 * @param settings settings structure, used for logging.
 *
 * @return A new SSL context, or NULL on error.
 */
lcbio_pSSLCTX lcbio_ssl_new(const char *tsfile, const char *cafile, const char *keyfile, int noverify, lcb_STATUS *errp,
                            lcb_settings *settings);
#else
#define lcbio_ssl_new lcbio_ssl_new__fallback
#endif

/**
 * Free the SSL context. This should be done when libcouchbase has nothing else
 * to do with the certificate
 * @param ctx
 */
void lcbio_ssl_free(lcbio_pSSLCTX ctx);

/**
 * Apply the SSL settings to a given socket.
 *
 * The socket must be newly connected and must not have already been initialized
 * with SSL (i.e. lcbio_ssl_check() returns false).
 *
 * @param sock The socket to which SSL should be applied
 * @param sctx The context returned by lcbio_ssl_new()
 * @return
 */
lcb_STATUS lcbio_ssl_apply(lcbio_SOCKET *sock, lcbio_pSSLCTX sctx);

/**
 * Checks whether the given socket is using SSL
 * @param sock The socket to check
 * @return true if using SSL, false if plain (or not yet applied)
 */
LCB_INTERNAL_API
int lcbio_ssl_check(lcbio_SOCKET *sock);

/**
 * Retrieves the internal error code from the SSL object within the socket.
 * Should only be called if lcbio_ssl_check() is true.
 *
 * @param sock
 * @return An error code (if present), or LCB_SUCCESS if there is no internal
 * error code.
 */
LCB_INTERNAL_API
lcb_STATUS lcbio_ssl_get_error(lcbio_SOCKET *sock);

/**
 * @brief
 * Initialize any application-level globals needed for SSL support
 * @todo There is currently nothing checking if this hasn't been called more
 * than once.
 */
void lcbio_ssl_global_init(void);

struct lcb_settings_st;

/**
 * Apply SSL to the socket if the socket should use SSL and is not already
 * an SSL socket. This is a convenience function that:
 *
 * 1. Checks the settings to see if SSL is enabled
 * 2. Checks to see if the socket already has SSL (lcbio_ssl_check())
 * 3. Calls lcbio_ssl_apply if (1) and (2) are true.
 *
 * @param sock The socket to SSLify
 * @param settings The settings structure from whence the context and policy are
 * derived.
 * @return
 */
lcb_STATUS lcbio_sslify_if_needed(lcbio_SOCKET *sock, struct lcb_settings_st *settings);

/**@}*/

#ifdef __cplusplus
}
#endif
#endif
