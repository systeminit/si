/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#define LIBCOUCHBASE_COUCHBASE_H 1

/**
 * @file
 * Main header file for Couchbase
 */

#define LCB_CONFIG_MCD_PORT 11210
#define LCB_CONFIG_MCD_SSL_PORT 11207
#define LCB_CONFIG_HTTP_PORT 8091
#define LCB_CONFIG_HTTP_SSL_PORT 18091
#define LCB_CONFIG_MCCOMPAT_PORT 11211

struct lcb_st;

/**
 * @ingroup lcb-init
 * Library handle representing a connection to a cluster and its data buckets. The contents
 * of this structure are opaque.
 * @see lcb_create
 * @see lcb_destroy
 */
typedef struct lcb_st lcb_INSTANCE;
typedef struct lcb_HTTP_HANDLE_ lcb_HTTP_HANDLE;

#include <stddef.h>
#include <time.h>
#include <stdarg.h>
#include <stdio.h>
#include <libcouchbase/sysdefs.h>
#include <libcouchbase/assert.h>
#include <libcouchbase/visibility.h>
#include <libcouchbase/error.h>
#include <libcouchbase/iops.h>
#include <libcouchbase/configuration.h>
#include <libcouchbase/kvbuf.h>
#include <libcouchbase/auth.h>
#include <libcouchbase/tracing.h>
#include <libcouchbase/cntl.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef lcb_U8 lcb_datatype_t;
typedef lcb_U32 lcb_USECS;

/******************************************************************************
 ******************************************************************************
 ******************************************************************************
 ** INITIALIZATION                                                           **
 ******************************************************************************
 ******************************************************************************
 ******************************************************************************/

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-init Initialization
 *
 * @details
 *
 * To communicate with a Couchbase cluster, a new library handle instance is
 * created in the form of an lcb_INSTANCE. To create such an object, the lcb_create()
 * function is called, passing it a structure of type lcb_create_st. The structure
 * acts as a container for a union of other structures which are extended
 * as more features are added. This container is forwards and backwards
 * compatible, meaning that if the structure is extended, you code and application
 * will still function if using an older version of the structure. The current
 * sub-field of the lcb_create_st structure is the `v3` field.
 *
 * Connecting to the cluster involes the client knowing the necessary
 * information needed to actually locate its services and connect to it.
 *
 * A connection specification consists of:
 *
 * 1. One or more hosts which comprise the cluster
 * 2. The name of the bucket to access and perform operations on
 * 3. The credentials of the bucket
 *
 * All these options are specified within the form of a URI in the form of
 *
 * `couchbase://$HOSTS/$BUCKET?$OPTIONS`
 *
 * @note
 * If any of the fields (hosts, bucket, options) contain the `/` character then
 * it _must_ be url-encoded; thus a bucket named `foo/bar` would be specified
 * as `couchbase:///foo%2Fbar`
 *
 * @par Hosts
 *
 * In the most typical use case, you would specify a list of several hostnames
 * delimited by a comma (`,`); each host specified should be a member of the
 * cluster. The library will use this list to initially connect to the cluster.
 *
 * Note that it is not necessary to specify _all_ the nodes of the cluster as in
 * a normal situation the library will only initially connect to one of the nodes.
 * Passing multiple nodes increases the chance of a connection succeeding even
 * if some of the nodes are currently down. Once connected to the cluster, the
 * client will update itself with the other nodes actually found within the
 * cluster and discard the list passed to it
 *
 * You can specify multiple hosts like so:
 *
 * `couchbase://foo.com,bar.com,baz.com`
 *
 * Or a single host:
 *
 * `couchbase://localhost`
 *
 * #### Specifying Ports and Protocol Options
 *
 * The default `couchbase://` scheme will assume all hosts and/or ports
 * specify the _memcached_ port. If no port is specified, it is assumed
 * that the port is _11210). For more extended options there are additional
 * schemes available:
 *
 * * `couchbases://` - Will assume all ports refer to the SSL-enabled memcached
 *   ports. This setting implicitly enables SSL on the instance as well. If no
 *   ports are provided for the hosts, the implicit port for each host will be
 *   _11207_.
 *
 * * `http://` - Will assume all ports refer to the HTTP REST API ports used
 *   by Couchbase 2.2 and lower. These are also used when connecting to a
 *   memcached bucket. If no port is specified it will be assumed the port is
 *   _8091_.
 *
 * ### Bucket
 *
 * A bucket may be specified by using the optional _path_ component of the URI
 * For protected buckets a password will still need to be supplied out of band.
 *
 * * `couchbase://1.1.1.1,2.2.2.2,3.3.3.3/users` - Connect to the `users`
 *   bucket.
 *
 * ### Options
 *
 * @warning The key-value options here are considered to be an uncommitted interface
 * as their names may change.
 *
 * Options can be specified as the _query_ part of the connection string,
 * for example:
 *
 * `couchbase://cbnode.net/beer?operation_timeout=10000000`.
 *
 * Options may either be appropriate _key_ parameters for lcb_cntl_string()
 * or one of the following:
 *
 * * `boostrap_on` - specify bootstrap protocols. Values can be `http` to force
 *   old-style bootstrap mode for legacy clusters, `cccp` to force bootstrap
 *   over the memcached port (For clusters 2.5 and above), or `all` to try with
 *   _cccp_ and revert to _http_
 *
 * * `truststorepath` - Specify the path (on the local filesystem) to the server's
 *   SSL certificate truststore. Only applicable if SSL is being used (i.e. the
 *   scheme is `couchbases`). The trust store is optional, and when missing,
 *   the library will use `certpath` as location for verification, and expect
 *   any extra certificates to be concatenated in there.
 *
 * * `certpath` - Specify the path (on the local filesystem) to the server's
 *   SSL certificate. Only applicable if SSL is being used (i.e. the scheme is
 *   `couchbases`)
 *
 * * `keypath` - Specify the path (on the local filesystem) to the client
 *   SSL private key. Only applicable if SSL client certificate authentication
 *   is being used (i.e. the scheme is `couchbases` and `certpath` contains
 *   client certificate). Read more in the server documentation:
 *   https://developer.couchbase.com/documentation/server/5.0/security/security-certs-auth.html
 *
 * ### Bucket Identification and Credentials
 *
 * The most common settings you will wish to modify are the bucket name
 *  and the credentials field (`user` and `passwd`). If a
 * `bucket` is not specified it will revert to the `default` bucket (i.e. the
 * bucket which is created when Couchbase Server is installed).
 *
 * The `user` and `passwd` fields authenticate for the bucket. This is only
 * needed if you have configured your bucket to employ SASL auth. You can tell
 * if the bucket has been configured with SASL auth by
 *
 * 1. Logging into the Couchbase Administration Console
 * 2. Going to the _Data Buckets_ tab
 * 3. Locate the row for your bucket
 * 4. Expand the row into the detailed view (by clicking on the arrow at the
 *    left of the row)
 * 5. Click on _Edit_
 * 6. Inspect the _Access Control_ section in the pop-up
 *
 * The bucket name is specified as the _path_ portion of the URI.
 *
 * For security purposes, the _user_ and _passwd_ cannot be specified within
 * the URI
 *
 *
 * @note
 * You may not change the bucket or credentials after initializing the handle.
 *
 * #### Bootstrap Options
 *
 * The default configuration process will attempt to bootstrap first from
 * the new memcached configuration protocol (CCCP) and if that fails, use
 * the "HTTP" protocol via the REST API.
 *
 * The CCCP configuration will by default attempt to connect to one of
 * the nodes specified on the port 11201. While normally the memcached port
 * is determined by the configuration itself, this is not possible when
 * the configuration has not been attained. You may specify a list of
 * alternate memcached servers by using the 'mchosts' field.
 *
 * If you wish to modify the default bootstrap protocol selection, you
 * can use the `bootstrap_on` option to specify the desired bootstrap
 * specification
 * to use for configuration (note that the ordering of this array is
 * ignored). Using this mechanism, you can disable CCCP or HTTP.
 *
 * To force only "new-style" bootstrap, you may use `bootstrap_on=cccp`.
 * To force only "old-style" bootstrap, use `bootstrap_on=http`. To force the
 * default behavior, use `bootstrap_on=all`
 *
 *
 * @addtogroup lcb-init
 * @{
 */

/** @brief Handle types @see lcb_create_st3::type */
typedef enum {
    LCB_TYPE_BUCKET = 0x00, /**< Handle for data access (default) */
    LCB_TYPE_CLUSTER = 0x01 /**< Handle for administrative access */
} lcb_type_t;

/**
 * @brief Type of the bucket
 *
 * @see https://developer.couchbase.com/documentation/server/current/architecture/core-data-access-buckets.html
 */
typedef enum {
    LCB_BTYPE_UNSPEC = 0x00,    /**< Unknown or unspecified */
    LCB_BTYPE_COUCHBASE = 0x01, /**< Data persisted and replicated */
    LCB_BTYPE_EPHEMERAL = 0x02, /**< Data not persisted, but replicated */
    LCB_BTYPE_MEMCACHED = 0x03  /**< Data not persisted and not replicated */
} lcb_BTYPE;

#ifndef __LCB_DOXYGEN__
/* These are definitions for some of the older fields of the `lcb_create_st`
 * structure. They are here for backwards compatibility and should not be
 * used by new code */
typedef enum {
    LCB_CONFIG_TRANSPORT_LIST_END = 0,
    LCB_CONFIG_TRANSPORT_HTTP = 1,
    LCB_CONFIG_TRANSPORT_CCCP,
    LCB_CONFIG_TRANSPORT_MAX
} lcb_config_transport_t;
#define LCB_CREATE_V0_FIELDS                                                                                           \
    const char *host;                                                                                                  \
    const char *user;                                                                                                  \
    const char *passwd;                                                                                                \
    const char *bucket;                                                                                                \
    struct lcb_io_opt_st *io;
#define LCB_CREATE_V1_FIELDS LCB_CREATE_V0_FIELDS lcb_type_t type;
#define LCB_CREATE_V2_FIELDS                                                                                           \
    LCB_CREATE_V1_FIELDS const char *mchosts;                                                                          \
    const lcb_config_transport_t *transports;
struct lcb_create_st0 {
    LCB_CREATE_V0_FIELDS
};
struct lcb_create_st1 {
    LCB_CREATE_V1_FIELDS
};
struct lcb_create_st2 {
    LCB_CREATE_V2_FIELDS
};
#endif

/**
 * @brief Inner structure V3 for lcb_create().
 */
struct lcb_create_st3 {
    const char *connstr; /**< Connection string */

    /**
     * Username to use for authentication. This should only be set when
     * connecting to a server 5.0 or greater.
     */
    const char *username;

    /**
     * Password for bucket. Can also be password for username on servers >= 5.0
     */
    const char *passwd;

    void *_pad_bucket;        /**< @private */
    struct lcb_io_opt_st *io; /**< IO Options */
    lcb_type_t type;
};

/**
 * @brief Inner structure V4 for lcb_create().
 *
 * Same as V3, but allows to supply logger (@see LCB_CNTL_LOGGER).
 */
struct lcb_create_st4 {
    const char *connstr; /**< Connection string */

    /**
     * Username to use for authentication. This should only be set when
     * connecting to a server 5.0 or greater.
     */
    const char *username;

    /**
     * Password for bucket. Can also be password for username on servers >= 5.0
     */
    const char *passwd;

    lcb_logprocs *logger;     /**< Logger */
    struct lcb_io_opt_st *io; /**< IO Options */
    lcb_type_t type;
};

/**
 * @brief Wrapper structure for lcb_create()
 * @see lcb_create_st3
 */
struct lcb_create_st {
    /** Indicates which field in the @ref lcb_CRST_u union should be used. Set this to `3` */
    int version;

    /**This union contains the set of current and historical options. The
     * The #v3 field should be used. */
    union lcb_CRST_u {
        struct lcb_create_st0 v0;
        struct lcb_create_st1 v1;
        struct lcb_create_st2 v2;
        struct lcb_create_st3 v3; /**< Use this field */
        struct lcb_create_st4 v4;
    } v;
};

/**
 * @brief Create an instance of lcb.
 * @param instance Where the instance should be returned
 * @param options How to create the libcouchbase instance
 * @return LCB_SUCCESS on success
 *
 *
 * ### Examples
 * Create an instance using the default values:
 *
 * @code{.c}
 * lcb_INSTANCE *instance;
 * lcb_STATUS err = lcb_create(&instance, NULL);
 * if (err != LCB_SUCCESS) {
 *    fprintf(stderr, "Failed to create instance: %s\n", lcb_strerror(NULL, err));
 *    exit(EXIT_FAILURE);
 * }
 * @endcode
 *
 * Specify server list
 *
 * @code{.c}
 * struct lcb_create_st options;
 * memset(&options, 0, sizeof(options));
 * options.version = 3;
 * options.v.v3.connstr = "couchbase://host1,host2,host3";
 * err = lcb_create(&instance, &options);
 * @endcode
 *
 *
 * Create a handle for data requests to protected bucket
 *
 * @code{.c}
 * struct lcb_create_st options;
 * memset(&options, 0, sizeof(options));
 * options.version = 3;
 * options.v.v3.host = "couchbase://example.com,example.org/protected"
 * options.v.v3.passwd = "secret";
 * err = lcb_create(&instance, &options);
 * @endcode
 * @committed
 * @see lcb_create_st3
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_create(lcb_INSTANCE **instance, const struct lcb_create_st *options);

/**
 * @brief Schedule the initial connection
 * This function will schedule the initial connection for the handle. This
 * function _must_ be called before any operations can be performed.
 *
 * lcb_set_bootstrap_callback() or lcb_get_bootstrap_status() can be used to
 * determine if the scheduled connection completed successfully.
 *
 * @par Synchronous Usage
 * @code{.c}
 * lcb_STATUS rc = lcb_connect(instance);
 * if (rc != LCB_SUCCESS) {
 *    your_error_handling(rc);
 * }
 * lcb_wait(instance);
 * rc = lcb_get_bootstrap_status(instance);
 * if (rc != LCB_SUCCESS) {
 *    your_error_handler(rc);
 * }
 * @endcode
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_connect(lcb_INSTANCE *instance);

/**
 * Bootstrap callback. Invoked once the instance is ready to perform operations
 * @param instance The instance which was bootstrapped
 * @param err The error code received. If this is not LCB_SUCCESS then the
 * instance is not bootstrapped and must be recreated
 *
 * @attention This callback only receives information during instantiation.
 * @committed
 */
typedef void (*lcb_bootstrap_callback)(lcb_INSTANCE *instance, lcb_STATUS err);

/**
 * @brief Set the callback for notification of success or failure of
 * initial connection.
 *
 * @param instance the instance
 * @param callback the callback to set. If `NULL`, return the existing callback
 * @return The existing (and previous) callback.
 * @see lcb_connect()
 * @see lcb_get_bootstrap_status()
 */
LIBCOUCHBASE_API
lcb_bootstrap_callback lcb_set_bootstrap_callback(lcb_INSTANCE *instance, lcb_bootstrap_callback callback);

/**
 * @brief Gets the initial bootstrap status
 *
 * This is an alternative to using the lcb_bootstrap_callback() and may be used
 * after the initial lcb_connect() and lcb_wait() sequence.
 * @param instance
 * @return LCB_SUCCESS if properly bootstrapped or an error code otherwise.
 *
 * @attention
 * Calling this function only makes sense during instantiation.
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_get_bootstrap_status(lcb_INSTANCE *instance);

/**
 * Sets the authenticator object for the instance. This may be done anytime, but
 * should probably be done before calling `lcb_connect()` for best effect.
 *
 * @param instance the handle
 * @param auth the authenticator object used. The library will increase the
 * refcount on the authenticator object.
 */
LIBCOUCHBASE_API
void lcb_set_auth(lcb_INSTANCE *instance, lcb_AUTHENTICATOR *auth);
/**@}*/

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-kv-api Key/Value
 *
 * @brief Preview APIs for performing commands
 *
 * @details
 * Basic command and structure definitions for public API. This represents the
 * "V3" API of libcouchbase. This API replaces the legacy API (which now wraps
 * this one). It contains common definitions for scheduling, response structures
 * and callback signatures.
 *
 * @addtogroup lcb-kv-api
 * @{
 */

/** @ingroup lcb-mutation-tokens */
typedef struct {
    uint64_t uuid_;  /**< @private */
    uint64_t seqno_; /**< @private */
    uint16_t vbid_;  /**< @private */
} lcb_MUTATION_TOKEN;

LIBCOUCHBASE_API int lcb_mutation_token_is_valid(const lcb_MUTATION_TOKEN *token);

/**
 * @brief Response flags.
 * These provide additional 'meta' information about the response
 * One of more of these values can be set in @ref lcb_RESPBASE::rflags
 */
typedef enum {
    /** No more responses are to be received for this request */
    LCB_RESP_F_FINAL = 0x01,

    /**The response was artificially generated inside the client.
     * This does not contain reply data from the server for the command, but
     * rather contains the basic fields to indicate success or failure and is
     * otherwise empty.
     */
    LCB_RESP_F_CLIENTGEN = 0x02,

    /**The response was a result of a not-my-vbucket error */
    LCB_RESP_F_NMVGEN = 0x04,

    /**The response has additional internal data.
     * Used by lcb_resp_get_mutation_token() */
    LCB_RESP_F_EXTDATA = 0x08,

    /**Flag, only valid for subdoc responses, indicates that the response was
     * processed using the single-operation protocol. */
    LCB_RESP_F_SDSINGLE = 0x10,

    /**The response has extra error information as value (see SDK-RFC-28). */
    LCB_RESP_F_ERRINFO = 0x20
} lcb_RESPFLAGS;

/**
 * The type of response passed to the callback. This is used to install callbacks
 * for the library and to distinguish between responses if a single callback
 * is used for multiple response types.
 *
 * @note These callbacks may conflict with the older version 2 callbacks. The
 * rules are as follows:
 * * If a callback has been installed using lcb_install_callback3(), then
 * the older version 2 callback will not be invoked for that operation. The order
 * of installation does not matter.
 * * If the LCB_CALLBACK_DEFAULT callback is installed, _none_ of the version 2
 * callbacks are invoked.
 */
typedef enum {
    LCB_CALLBACK_DEFAULT = 0, /**< Default callback invoked as a fallback */
    LCB_CALLBACK_GET,         /**< lcb_get3() */
    LCB_CALLBACK_STORE,       /**< lcb_store3() */
    LCB_CALLBACK_COUNTER,     /**< lcb_counter3() */
    LCB_CALLBACK_TOUCH,       /**< lcb_touch3() */
    LCB_CALLBACK_REMOVE,      /**< lcb_remove3() */
    LCB_CALLBACK_UNLOCK,      /**< lcb_unlock3() */
    LCB_CALLBACK_STATS,       /**< lcb_stats3() */
    LCB_CALLBACK_VERSIONS,    /**< lcb_server_versions3() */
    LCB_CALLBACK_VERBOSITY,   /**< lcb_server_verbosity3() */
    LCB_CALLBACK_OBSERVE,     /**< lcb_observe3_ctxnew() */
    LCB_CALLBACK_GETREPLICA,  /**< lcb_rget3() */
    LCB_CALLBACK_ENDURE,      /**< lcb_endure3_ctxnew() */
    LCB_CALLBACK_HTTP,        /**< lcb_http3() */
    LCB_CALLBACK_CBFLUSH,     /**< lcb_cbflush3() */
    LCB_CALLBACK_OBSEQNO,     /**< For lcb_observe_seqno3() */
    LCB_CALLBACK_STOREDUR,    /** <for lcb_storedur3() */
    LCB_CALLBACK_SDLOOKUP,
    LCB_CALLBACK_SDMUTATE,
    LCB_CALLBACK_NOOP,                     /**< lcb_noop3() */
    LCB_CALLBACK_PING,                     /**< lcb_ping3() */
    LCB_CALLBACK_DIAG,                     /**< lcb_diag() */
    LCB_CALLBACK_COLLECTIONS_GET_MANIFEST, /**< lcb_getmanifest() */
    LCB_CALLBACK_GETCID,                   /**< lcb_getcid() */
    LCB_CALLBACK_EXISTS,                   /**< lcb_exists() */
    LCB_CALLBACK__MAX                      /* Number of callbacks */
} lcb_CALLBACK_TYPE;

/* The following callback types cannot be set using lcb_install_callback3(),
 * however, their value is passed along as the second argument of their
 * respective callbacks. This allows you to still use the same callback,
 * differentiating their meaning by the type. */

/** Callback type for views (cannot be used for lcb_install_callback3()) */
#define LCB_CALLBACK_VIEWQUERY -1

/** Callback type for N1QL (cannot be used for lcb_install_callback3()) */
#define LCB_CALLBACK_N1QL -2

/** Callback type for N1QL index management (cannot be used for lcb_install_callback3()) */
#define LCB_CALLBACK_IXMGMT -3

/** Callback type for Analytics (cannot be used for lcb_install_callback3()) */
#define LCB_CALLBACK_ANALYTICS -4

#define LCB_CALLBACK_OPEN -5

/**
 * @uncommitted
 * Durability levels
 */
typedef enum {
    LCB_DURABILITYLEVEL_NONE = 0x00,
    /**
     * Mutation must be replicated to (i.e. held in memory of that node) a
     * majority ((configured_nodes / 2) + 1) of the configured nodes of the
     * bucket.
     */
    LCB_DURABILITYLEVEL_MAJORITY = 0x01,
    /**
     * As majority, but additionally persisted to the active node.
     */
    LCB_DURABILITYLEVEL_MAJORITY_AND_PERSIST_ON_MASTER = 0x02,
    /**
     * Mutation must be persisted to (i.e. written and fsync'd to disk) a
     * majority of the configured nodes of the bucket.
     */
    LCB_DURABILITYLEVEL_PERSIST_TO_MAJORITY = 0x03
} lcb_DURABILITY_LEVEL;

typedef struct lcb_CMDBASE_ lcb_CMDBASE;
typedef struct lcb_RESPBASE_ lcb_RESPBASE;

/**
 * Callback invoked for responses.
 * @param instance The handle
 * @param cbtype The type of callback - or in other words, the type of operation
 * this callback has been invoked for.
 * @param resp The response for the operation. Depending on the operation this
 * response structure should be casted into a more specialized type.
 */
typedef void (*lcb_RESPCALLBACK)(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *resp);

/**
 * @committed
 *
 * Install a new-style callback for an operation. The callback will be invoked
 * with the relevant response structure.
 *
 * @param instance the handle
 * @param cbtype the type of operation for which this callback should be installed.
 *        The value should be one of the lcb_CALLBACK_TYPE constants
 * @param cb the callback to install
 * @return the old callback
 *
 * @note LCB_CALLBACK_DEFAULT is initialized to the default handler which proxies
 * back to the older 2.x callbacks. If you set `cbtype` to LCB_CALLBACK_DEFAULT
 * then your `2.x` callbacks _will not work_.
 *
 * @note The old callback may be `NULL`. It is usually not an error to have a
 * `NULL` callback installed. If the callback is `NULL`, then the default callback
 * invocation pattern will take place (as desribed above). However it is an error
 * to set the default callback to `NULL`.
 */
LIBCOUCHBASE_API
lcb_RESPCALLBACK lcb_install_callback3(lcb_INSTANCE *instance, int cbtype, lcb_RESPCALLBACK cb);

/**
 * @committed
 *
 * Get the current callback installed as `cbtype`. Note that this does not
 * perform any kind of resolution (as described in lcb_install_callback3) and
 * will only return a non-`NULL` value if a callback had specifically been
 * installed via lcb_install_callback3() with the given `cbtype`.
 *
 * @param instance the handle
 * @param cbtype the type of callback to retrieve
 * @return the installed callback for the type.
 */
LIBCOUCHBASE_API
lcb_RESPCALLBACK lcb_get_callback3(lcb_INSTANCE *instance, int cbtype);

/**
 * Returns the type of the callback as a string.
 * This function is helpful for debugging and demonstrative processes.
 * @param cbtype the type of the callback (the second argument to the callback)
 * @return a string describing the callback type
 */
LIBCOUCHBASE_API
const char *lcb_strcbtype(int cbtype);

/**@}*/

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-get Read
 * @brief Retrieve a document from the cluster
 * @addtogroup lcb-get
 * @{
 */

/**@brief Command for retrieving a single item
 *
 * @see lcb_get3()
 * @see lcb_RESPGET
 *
 * @note The #cas member should be set to 0 for this operation. If the #cas is
 * not 0, lcb_get3() will fail with ::LCB_OPTIONS_CONFLICT.
 *
 * ### Use of the `exptime` field
 *
 * <ul>
 * <li>Get And Touch:
 *
 * It is possible to retrieve an item and concurrently modify its expiration
 * time (thus keeping it "alive"). The item's expiry time can be set using
 * the #exptime field.
 * </li>
 *
 * <li>Lock
 * If the #lock field is set to non-zero, the #exptime field indicates the amount
 * of time the lock should be held for
 * </li>
 * </ul>
 */

/**
 * @committed
 *
 * @brief Spool a single get operation
 * @param instance the handle
 * @param cookie a pointer to be associated with the command
 * @param cmd the command structure
 * @return LCB_SUCCESS if successful, an error code otherwise
 *
 * @par Request
 * @code{.c}
 * lcb_CMDGET cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "Hello", 5);
 * lcb_get3(instance, cookie, &cmd);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_GET, get_callback);
 * static void get_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb) {
 *     const lcb_RESPGET *resp = (const lcb_RESPGET*)rb;
 *     printf("Got response for key: %.*s\n", (int)resp->key, resp->nkey);
 *
 *     if (resp->rc != LCB_SUCCESS) {
 *         printf("Couldn't get item: %s\n", lcb_strerror(NULL, resp->rc));
 *     } else {
 *         printf("Got value: %.*s\n", (int)resp->nvalue, resp->value);
 *         printf("Got CAS: 0x%llx\n", resp->cas);
 *         printf("Got item/formatting flags: 0x%x\n", resp->itmflags);
 *     }
 * }
 *
 * @endcode
 *
 * @par Errors
 * @cb_err ::LCB_KEY_ENOENT if the item does not exist in the cluster
 * @cb_err ::LCB_ETMPFAIL if the lcb_CMDGET::lock option was set but the item
 * was already locked. Note that this error may also be returned (as a generic
 * error) if there is a resource constraint within the server itself.
 */

typedef struct lcb_RESPGET_ lcb_RESPGET;

LIBCOUCHBASE_API lcb_STATUS lcb_respget_status(const lcb_RESPGET *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_error_context(const lcb_RESPGET *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_error_ref(const lcb_RESPGET *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_cookie(const lcb_RESPGET *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_cas(const lcb_RESPGET *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_datatype(const lcb_RESPGET *resp, uint8_t *datatype);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_flags(const lcb_RESPGET *resp, uint32_t *flags);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_key(const lcb_RESPGET *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respget_value(const lcb_RESPGET *resp, const char **value, size_t *value_len);

typedef struct lcb_CMDGET_ lcb_CMDGET;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_create(lcb_CMDGET **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_destroy(lcb_CMDGET *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_parent_span(lcb_CMDGET *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_collection(lcb_CMDGET *cmd, const char *scope, size_t scope_len,
                                                  const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_key(lcb_CMDGET *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_expiration(lcb_CMDGET *cmd, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_locktime(lcb_CMDGET *cmd, uint32_t duration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_durability(lcb_CMDGET *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_timeout(lcb_CMDGET *cmd, uint32_t timeout);

LIBCOUCHBASE_API lcb_STATUS lcb_get(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGET *cmd);
/**@}*/

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-get-replica Read (Replica)
 * @brief Retrieve a document from a replica if it cannot be fetched from the
 * primary
 * @addtogroup lcb-get-replica
 * @{
 */

/**@committed
 *
 * @brief Spool a single get-with-replica request
 * @param instance
 * @param cookie
 * @param cmd
 * @return LCB_SUCCESS on success, error code otherwise.
 *
 * When a response is received, the callback installed for ::LCB_CALLBACK_GETREPLICA
 * will be invoked. The response will be an @ref lcb_RESPGET pointer.
 *
 * ### Request
 * @code{.c}
 * lcb_CMDGETREPLICA cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "key", 3);
 * lcb_rget3(instance, cookie, &cmd);
 * @endcode
 *
 * ### Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_GETREPLICA, rget_callback);
 * static void rget_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPGET *resp = (const lcb_RESPGET *)rb;
 *     printf("Got Get-From-Replica response for %.*s\n", (int)resp->key, resp->nkey);
 *     if (resp->rc == LCB_SUCCESS) {
 *         printf("Got response: %.*s\n", (int)resp->value, resp->nvalue);
 *     else {
 *         printf("Couldn't retrieve: %s\n", lcb_strerror(NULL, resp->rc));
 *     }
 * }
 * @endcode
 *
 * @warning As this function queries a replica node for data it is possible
 * that the returned document may not reflect the latest document in the server.
 *
 * @warning This function should only be used in cases where a normal lcb_get3()
 * has failed, or where there is reason to believe it will fail. Because this
 * function may query more than a single replica it may cause additional network
 * and server-side CPU load. Use sparingly and only when necessary.
 *
 * @cb_err ::LCB_KEY_ENOENT if the key is not found on the replica(s),
 * ::LCB_NO_MATCHING_SERVER if there are no replicas (either configured or online),
 * or if the given replica
 * (if lcb_CMDGETREPLICA::strategy is ::LCB_REPLICA_SELECT) is not available or
 * is offline.
 */

typedef enum {
    LCB_REPLICA_MODE_ANY = 0x00,
    LCB_REPLICA_MODE_ALL = 0x01,
    LCB_REPLICA_MODE_IDX0 = 0x02,
    LCB_REPLICA_MODE_IDX1 = 0x03,
    LCB_REPLICA_MODE_IDX2 = 0x04,
    LCB_REPLICA_MODE__MAX
} lcb_REPLICA_MODE;

typedef struct lcb_RESPGETREPLICA_ lcb_RESPGETREPLICA;

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_status(const lcb_RESPGETREPLICA *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_error_context(const lcb_RESPGETREPLICA *resp, const char **ctx,
                                                             size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_error_ref(const lcb_RESPGETREPLICA *resp, const char **ref,
                                                         size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_cookie(const lcb_RESPGETREPLICA *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_cas(const lcb_RESPGETREPLICA *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_datatype(const lcb_RESPGETREPLICA *resp, uint8_t *datatype);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_flags(const lcb_RESPGETREPLICA *resp, uint32_t *flags);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_key(const lcb_RESPGETREPLICA *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_value(const lcb_RESPGETREPLICA *resp, const char **value,
                                                     size_t *value_len);

typedef struct lcb_CMDGETREPLICA_ lcb_CMDGETREPLICA;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_create(lcb_CMDGETREPLICA **cmd, lcb_REPLICA_MODE mode);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_destroy(lcb_CMDGETREPLICA *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_parent_span(lcb_CMDGETREPLICA *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_collection(lcb_CMDGETREPLICA *cmd, const char *scope, size_t scope_len,
                                                         const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_key(lcb_CMDGETREPLICA *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_timeout(lcb_CMDGETREPLICA *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_getreplica(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETREPLICA *cmd);

/**@}*/

typedef struct lcb_RESPEXISTS_ lcb_RESPEXISTS;

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_status(const lcb_RESPEXISTS *resp);
LIBCOUCHBASE_API int lcb_respexists_is_persisted(const lcb_RESPEXISTS *resp);
LIBCOUCHBASE_API int lcb_respexists_is_found(const lcb_RESPEXISTS *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respexists_error_context(const lcb_RESPEXISTS *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respexists_error_ref(const lcb_RESPEXISTS *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respexists_cookie(const lcb_RESPEXISTS *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respexists_cas(const lcb_RESPEXISTS *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respexists_key(const lcb_RESPEXISTS *resp, const char **key, size_t *key_len);

typedef struct lcb_CMDEXISTS_ lcb_CMDEXISTS;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_create(lcb_CMDEXISTS **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_destroy(lcb_CMDEXISTS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_parent_span(lcb_CMDEXISTS *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_collection(lcb_CMDEXISTS *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_key(lcb_CMDEXISTS *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_timeout(lcb_CMDEXISTS *cmd, uint32_t timeout);

LIBCOUCHBASE_API lcb_STATUS lcb_exists(lcb_INSTANCE *instance, void *cookie, const lcb_CMDEXISTS *cmd);

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-store Create/Update
 * @brief Set the value of a document
 * @addtogroup lcb-store
 * @{
 */

/**
 * @brief Values for lcb_CMDSTORE::operation
 *
 * Storing an item in couchbase is only one operation with a different
 * set of attributes / constraints.
 */
typedef enum {
    /**
     * The default storage mode. This constant was added in version 2.6.2 for
     * the sake of maintaining a default storage mode, eliminating the need
     * for simple storage operations to explicitly define
     * lcb_CMDSTORE::operation. Behaviorally it is identical to @ref LCB_STORE_SET
     * in that it will make the server unconditionally store the item, whether
     * it exists or not.
     */
    LCB_STORE_UPSERT = 0x00,

    /**
     * Will cause the operation to fail if the key already exists in the
     * cluster.
     */
    LCB_STORE_ADD = 0x01,

    /**
     * Will cause the operation to fail _unless_ the key already exists in the
     * cluster.
     */
    LCB_STORE_REPLACE = 0x02,

    /** Unconditionally store the item in the cluster */
    LCB_STORE_SET = 0x03,

    /**
     * Rather than setting the contents of the entire document, take the value
     * specified in lcb_CMDSTORE::value and _append_ it to the existing bytes in
     * the value.
     */
    LCB_STORE_APPEND = 0x04,

    /**
     * Like ::LCB_STORE_APPEND, but prepends the new value to the existing value.
     */
    LCB_STORE_PREPEND = 0x05
} lcb_STORE_OPERATION;

/**
 * @committed
 * @brief Schedule a single storage request
 * @param instance the handle
 * @param cookie pointer to associate with the command
 * @param cmd the command structure
 * @return LCB_SUCCESS on success, error code on failure
 *
 * ### Request
 *
 * @code{.c}
 * lcb_CMDSTORE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "Key", 3);
 * LCB_CMD_SET_VALUE(&cmd, "value", 5);
 * cmd.operation = LCB_ADD; // Only create if it does not exist
 * cmd.exptime = 60; // expire in a minute
 * lcb_store3(instance, cookie, &cmd);
 * lcb_wait3(instance, LCB_WAIT_NOCHECK);
 * @endcode
 *
 * ### Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_STORE, store_callback);
 * void store_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     if (rb->rc == LCB_SUCCESS) {
 *         printf("Store success: CAS=%llx\n", rb->cas);
 *     } else {
 *         printf("Store failed: %s\n", lcb_strerror(NULL, rb->rc);
 *     }
 * }
 * @endcode
 *
 * Operation-specific error codes include:
 * @cb_err ::LCB_KEY_ENOENT if ::LCB_REPLACE was used and the key does not exist
 * @cb_err ::LCB_KEY_EEXISTS if ::LCB_ADD was used and the key already exists
 * @cb_err ::LCB_KEY_EEXISTS if the CAS was specified (for an operation other
 *          than ::LCB_ADD) and the item exists on the server with a different
 *          CAS
 * @cb_err ::LCB_KEY_EEXISTS if the item was locked and the CAS supplied did
 * not match the locked item's CAS (or if no CAS was supplied)
 * @cb_err ::LCB_NOT_STORED if an ::LCB_APPEND or ::LCB_PREPEND operation was
 * performed and the item did not exist on the server.
 * @cb_err ::LCB_E2BIG if the size of the value exceeds the cluster per-item
 *         value limit (currently 20MB).
 *
 *
 * @note After a successful store operation you can use lcb_endure3_ctxnew()
 * to wait for the item to be persisted and/or replicated to other nodes.
 */

typedef struct lcb_RESPSTORE_ lcb_RESPSTORE;

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_status(const lcb_RESPSTORE *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_error_context(const lcb_RESPSTORE *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_error_ref(const lcb_RESPSTORE *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_cookie(const lcb_RESPSTORE *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_cas(const lcb_RESPSTORE *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_key(const lcb_RESPSTORE *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_operation(const lcb_RESPSTORE *resp, lcb_STORE_OPERATION *operation);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_mutation_token(const lcb_RESPSTORE *resp, lcb_MUTATION_TOKEN *token);

LIBCOUCHBASE_API int lcb_respstore_observe_attached(const lcb_RESPSTORE *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_stored(const lcb_RESPSTORE *resp, int *store_ok);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_master_exists(const lcb_RESPSTORE *resp, int *master_exists);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_master_persisted(const lcb_RESPSTORE *resp, int *master_persisted);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_responses(const lcb_RESPSTORE *resp, uint16_t *num_responses);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_persisted(const lcb_RESPSTORE *resp, uint16_t *num_persisted);
LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_replicated(const lcb_RESPSTORE *resp, uint16_t *num_replicated);

typedef struct lcb_CMDSTORE_ lcb_CMDSTORE;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_create(lcb_CMDSTORE **cmd, lcb_STORE_OPERATION operation);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_destroy(lcb_CMDSTORE *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_parent_span(lcb_CMDSTORE *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_collection(lcb_CMDSTORE *cmd, const char *scope, size_t scope_len,
                                                    const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_key(lcb_CMDSTORE *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_value(lcb_CMDSTORE *cmd, const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_value_iov(lcb_CMDSTORE *cmd, const lcb_IOV *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_expiration(lcb_CMDSTORE *cmd, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_cas(lcb_CMDSTORE *cmd, uint64_t cas);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_flags(lcb_CMDSTORE *cmd, uint32_t flags);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_datatype(lcb_CMDSTORE *cmd, uint8_t datatype);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_durability(lcb_CMDSTORE *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_durability_observe(lcb_CMDSTORE *cmd, int persist_to, int replicate_to);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_timeout(lcb_CMDSTORE *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_store(lcb_INSTANCE *instance, void *cookie, const lcb_CMDSTORE *cmd);

typedef void (*lcb_open_callback)(lcb_INSTANCE *instance, lcb_STATUS err);
LIBCOUCHBASE_API lcb_open_callback lcb_set_open_callback(lcb_INSTANCE *instance, lcb_open_callback callback);

/**
 * Opens bucket.
 *
 * @param instance
 * @return
 */
LIBCOUCHBASE_API lcb_STATUS lcb_open(lcb_INSTANCE *instance, const char *bucket, size_t bucket_len);

/**@}*/

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-remove Delete
 * @brief Remove documents from the cluster
 * @addtogroup lcb-remove
 * @{
 */

/**@committed
 * @brief Spool a removal of an item
 * @param instance the handle
 * @param cookie pointer to associate with the request
 * @param cmd the command
 * @return LCB_SUCCESS on success, other code on failure
 *
 * ### Request
 * @code{.c}
 * lcb_CMDREMOVE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "deleteme", strlen("deleteme"));
 * lcb_remove3(instance, cookie, &cmd);
 * @endcode
 *
 * ### Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, rm_callback);
 * void rm_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     printf("Key: %.*s...", (int)resp->nkey, resp->key);
 *     if (rb->rc != LCB_SUCCESS) {
 *         printf("Failed to remove item!: %s\n", lcb_strerror(NULL, rb->rc));
 *     } else {
 *         printf("Removed item!\n");
 *     }
 * }
 * @endcode
 *
 * The following operation-specific error codes are returned in the callback
 * @cb_err ::LCB_KEY_ENOENT if the key does not exist
 * @cb_err ::LCB_KEY_EEXISTS if the CAS was specified and it does not match the
 *         CAS on the server
 * @cb_err ::LCB_KEY_EEXISTS if the item was locked and no CAS (or an incorrect
 *         CAS) was specified.
 *
 */

typedef struct lcb_RESPREMOVE_ lcb_RESPREMOVE;

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_status(const lcb_RESPREMOVE *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_error_context(const lcb_RESPREMOVE *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_error_ref(const lcb_RESPREMOVE *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_cookie(const lcb_RESPREMOVE *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_cas(const lcb_RESPREMOVE *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_key(const lcb_RESPREMOVE *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respremove_mutation_token(const lcb_RESPREMOVE *resp, lcb_MUTATION_TOKEN *token);

typedef struct lcb_CMDREMOVE_ lcb_CMDREMOVE;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_create(lcb_CMDREMOVE **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_destroy(lcb_CMDREMOVE *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_parent_span(lcb_CMDREMOVE *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_collection(lcb_CMDREMOVE *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_key(lcb_CMDREMOVE *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_cas(lcb_CMDREMOVE *cmd, uint64_t cas);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_durability(lcb_CMDREMOVE *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_timeout(lcb_CMDREMOVE *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_remove(lcb_INSTANCE *instance, void *cookie, const lcb_CMDREMOVE *cmd);

/**@}*/

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-counter Counters
 * @brief Manipulate the numeric content of a document
 * @details Counter operations treat the document being accessed as a numeric
 * value (the document should contain a parseable integer as its content). This
 * value may then be incremented or decremented.
 *
 * @addtogroup lcb-counter
 * @{
 */

/**@committed
 * @brief Schedule single counter operation
 * @param instance the instance
 * @param cookie the pointer to associate with the request
 * @param cmd the command to use
 * @return LCB_SUCCESS on success, other error on failure
 *
 * @par Request
 * @code{.c}
 * lcb_CMDCOUNTER cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "counter", strlen("counter"));
 * cmd.delta = 1; // Increment by one
 * cmd.initial = 42; // Default value is 42 if it does not exist
 * cmd.exptime = 300; // Expire in 5 minutes
 * lcb_counter3(instance, NULL, &cmd);
 * lcb_wait3(instance, LCB_WAIT_NOCHECK);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACKTYPE_COUNTER, counter_cb);
 * void counter_cb(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPCOUNTER *resp = (const lcb_RESPCOUNTER *)rb;
 *     if (resp->rc == LCB_SUCCESS) {
 *         printf("Incremented counter for %.*s. Current value %llu\n",
 *                (int)resp->nkey, resp->key, resp->value);
 *     }
 * }
 * @endcode
 *
 * @par Callback Errors
 * In addition to generic errors, the following errors may be returned in the
 * callback (via lcb_RESPBASE::rc):
 *
 * @cb_err ::LCB_KEY_ENOENT if the counter doesn't exist
 * (and lcb_CMDCOUNTER::create was not set)
 * @cb_err ::LCB_DELTA_BADVAL if the existing document's content could not
 * be parsed as a number by the server.
 */

typedef struct lcb_RESPCOUNTER_ lcb_RESPCOUNTER;

LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_status(const lcb_RESPCOUNTER *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_error_context(const lcb_RESPCOUNTER *resp, const char **ctx,
                                                          size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_error_ref(const lcb_RESPCOUNTER *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_cookie(const lcb_RESPCOUNTER *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_cas(const lcb_RESPCOUNTER *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_key(const lcb_RESPCOUNTER *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_mutation_token(const lcb_RESPCOUNTER *resp, lcb_MUTATION_TOKEN *token);
LIBCOUCHBASE_API lcb_STATUS lcb_respcounter_value(const lcb_RESPCOUNTER *resp, uint64_t *value);

typedef struct lcb_CMDCOUNTER_ lcb_CMDCOUNTER;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_create(lcb_CMDCOUNTER **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_destroy(lcb_CMDCOUNTER *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_parent_span(lcb_CMDCOUNTER *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_collection(lcb_CMDCOUNTER *cmd, const char *scope, size_t scope_len,
                                                      const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_key(lcb_CMDCOUNTER *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_expiration(lcb_CMDCOUNTER *cmd, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_delta(lcb_CMDCOUNTER *cmd, int64_t number);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_initial(lcb_CMDCOUNTER *cmd, uint64_t number);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_durability(lcb_CMDCOUNTER *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdcounter_timeout(lcb_CMDCOUNTER *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_counter(lcb_INSTANCE *instance, void *cookie, const lcb_CMDCOUNTER *cmd);

/**@} (Group: Counter) */

/**@ingroup lcb-kv-api
 * @defgroup lcb-lock Lock/Unlock
 * @details Documents may be locked and unlocked on the server. While a document
 * is locked, any attempt to modify it (or lock it again) will fail.
 *
 * @note Locks are not persistent across nodes (if a node fails over, the lock
 * is not transferred to a replica).
 * @note The recommended way to manage access and concurrency control for
 * documents in Couchbase is through the CAS (See lcb_CMDBASE::cas and
 * lcb_RESPBASE::cas), which can also be considered a form of opportunistic
 * locking.
 *
 * @par Locking an item
 * There is no exclusive function to lock an item. Locking an item is done
 * using @ref lcb_get3(), by setting the lcb_CMDGET::lock option to true.
 *
 * @addtogroup lcb-lock
 * @{
 */

/**
 * @committed
 * @brief
 * Unlock a previously locked item using lcb_CMDGET::lock
 *
 * @param instance the instance
 * @param cookie the context pointer to associate with the command
 * @param cmd the command containing the information about the locked key
 * @return LCB_SUCCESS if successful, an error code otherwise
 * @see lcb_get3()
 *
 * @par Request
 *
 * @code{.c}
 * void locked_callback(lcb_INSTANCE, lcb_CALLBACK_TYPE, const lcb_RESPBASE *resp) {
 *   lcb_CMDUNLOCK cmd = { 0 };
 *   LCB_CMD_SET_KEY(&cmd, resp->key, resp->nkey);
 *   cmd.cas = resp->cas;
 *   lcb_unlock3(instance, cookie, &cmd);
 * }
 *
 * @endcode
 */

typedef struct lcb_RESPUNLOCK_ lcb_RESPUNLOCK;

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_status(const lcb_RESPUNLOCK *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_error_context(const lcb_RESPUNLOCK *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_error_ref(const lcb_RESPUNLOCK *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_cookie(const lcb_RESPUNLOCK *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_cas(const lcb_RESPUNLOCK *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_key(const lcb_RESPUNLOCK *resp, const char **key, size_t *key_len);

typedef struct lcb_CMDUNLOCK_ lcb_CMDUNLOCK;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_create(lcb_CMDUNLOCK **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_destroy(lcb_CMDUNLOCK *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_parent_span(lcb_CMDUNLOCK *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_collection(lcb_CMDUNLOCK *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_key(lcb_CMDUNLOCK *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_cas(lcb_CMDUNLOCK *cmd, uint64_t cas);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_timeout(lcb_CMDUNLOCK *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_unlock(lcb_INSTANCE *instance, void *cookie, const lcb_CMDUNLOCK *cmd);

/**@} (Group: Unlock) */

/**@ingroup lcb-kv-api
 * @defgroup lcb-touch Touch/Expiry
 * @brief Modify or clear a document's expiration time
 * @details Couchbase allows documents to contain expiration times
 * (see lcb_CMDBASE::exptime). Most operations allow the expiry time to be
 * updated, however lcb_touch3() allows the exclusive update of the expiration
 * time without additional network overhead.
 *
 * @addtogroup lcb-touch
 * @{
 */

/**@committed
 * @brief Spool a touch request
 * @param instance the handle
 * @param cookie the pointer to associate with the request
 * @param cmd the command
 * @return LCB_SUCCESS on success, other error code on failure
 *
 * @par Request
 * @code{.c}
 * lcb_CMDTOUCH cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "keep_me", strlen("keep_me"));
 * cmd.exptime = 0; // Clear the expiration
 * lcb_touch3(instance, cookie, &cmd);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_TOUCH, touch_callback);
 * void touch_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     if (rb->rc == LCB_SUCCESS) {
 *         printf("Touch succeeded\n");
 *     }
 * }
 * @endcode
 */

typedef struct lcb_RESPTOUCH_ lcb_RESPTOUCH;

LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_status(const lcb_RESPTOUCH *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_error_context(const lcb_RESPTOUCH *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_error_ref(const lcb_RESPTOUCH *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_cookie(const lcb_RESPTOUCH *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_cas(const lcb_RESPTOUCH *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_key(const lcb_RESPTOUCH *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_resptouch_mutation_token(const lcb_RESPTOUCH *resp, lcb_MUTATION_TOKEN *token);

typedef struct lcb_CMDTOUCH_ lcb_CMDTOUCH;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_create(lcb_CMDTOUCH **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_destroy(lcb_CMDTOUCH *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_parent_span(lcb_CMDTOUCH *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_collection(lcb_CMDTOUCH *cmd, const char *scope, size_t scope_len,
                                                    const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_key(lcb_CMDTOUCH *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_expiration(lcb_CMDTOUCH *cmd, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_durability(lcb_CMDTOUCH *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdtouch_timeout(lcb_CMDTOUCH *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_touch(lcb_INSTANCE *instance, void *cookie, const lcb_CMDTOUCH *cmd);

/**@} (Group: Touch) */
/**@} (Group: KV API) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-ping PING
 * @brief Broadcast NOOP-like commands to each service in the cluster
 *
 * @addtogroup lcb-ping
 * @{
 */

/**
 * Status of the service
 *
 * @committed
 */
typedef enum {
    LCB_PING_STATUS_OK = 0,
    LCB_PING_STATUS_TIMEOUT,
    LCB_PING_STATUS_ERROR,
    LCB_PING_STATUS_INVALID, /* bad index or argument */
    LCB_PING_STATUS__MAX
} lcb_PING_STATUS;

/**
 * Type of the service. This enumeration is used in PING responses.
 *
 * @committed
 */
typedef enum {
    LCB_PING_SERVICE_KV = 0,
    LCB_PING_SERVICE_VIEWS,
    LCB_PING_SERVICE_N1QL,
    LCB_PING_SERVICE_FTS,
    LCB_PING_SERVICE_ANALYTICS,
    LCB_PING_SERVICE__MAX
} lcb_PING_SERVICE;

typedef struct lcb_RESPPING_ lcb_RESPPING;

LIBCOUCHBASE_API lcb_STATUS lcb_respping_status(const lcb_RESPPING *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_cookie(const lcb_RESPPING *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_value(const lcb_RESPPING *resp, const char **json, size_t *json_len);
LIBCOUCHBASE_API size_t lcb_respping_result_size(const lcb_RESPPING *resp);
LIBCOUCHBASE_API lcb_PING_STATUS lcb_respping_result_status(const lcb_RESPPING *resp, size_t index);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_id(const lcb_RESPPING *resp, size_t index, const char **endpoint_id,
                                                   size_t *endpoint_id_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_service(const lcb_RESPPING *resp, size_t index, lcb_PING_SERVICE *type);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_remote(const lcb_RESPPING *resp, size_t index, const char **address,
                                                       size_t *address_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_local(const lcb_RESPPING *resp, size_t index, const char **address,
                                                      size_t *address_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_latency(const lcb_RESPPING *resp, size_t index, uint64_t *latency);
LIBCOUCHBASE_API lcb_STATUS lcb_respping_result_scope(const lcb_RESPPING *resp, size_t index, const char **name,
                                                      size_t *name_len);

typedef struct lcb_CMDPING_ lcb_CMDPING;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_create(lcb_CMDPING **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_destroy(lcb_CMDPING *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_parent_span(lcb_CMDPING *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_report_id(lcb_CMDPING *cmd, const char *report_id, size_t report_id_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_all(lcb_CMDPING *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_kv(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_n1ql(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_views(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_fts(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_analytics(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_no_metrics(lcb_CMDPING *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdping_encode_json(lcb_CMDPING *cmd, int enable, int pretty, int with_details);
LIBCOUCHBASE_API lcb_STATUS lcb_ping(lcb_INSTANCE *instance, void *cookie, const lcb_CMDPING *cmd);

/**
 * @brief Returns diagnostics report about network connections.
 *
 * @committed
 *
 * @par Request
 * @code{.c}
 * lcb_CMDDIAG cmd = { 0 };
 * lcb_diag(instance, fp, &cmd);
 * lcb_wait(instance);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_DIAG, diag_callback);
 * void diag_callback(lcb_INSTANCE, int, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPDIAG *resp = (const lcb_RESPDIAG *)rb;
 *     if (resp->rc != LCB_SUCCESS) {
 *         fprintf(stderr, "failed: %s\n", lcb_strerror(NULL, resp->rc));
 *     } else {
 *         if (resp->njson) {
 *             fprintf(stderr, "\n%.*s", (int)resp->njson, resp->json);
 *         }
 *     }
 * }
 * @endcode
 *
 * @param instance the library handle
 * @param cookie the cookie passed in the callback
 * @param cmd command structure.
 * @return status code for scheduling.
 */

typedef struct lcb_RESPDIAG_ lcb_RESPDIAG;

LIBCOUCHBASE_API lcb_STATUS lcb_respdiag_status(const lcb_RESPDIAG *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respdiag_cookie(const lcb_RESPDIAG *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respdiag_value(const lcb_RESPDIAG *resp, const char **json, size_t *json_len);

typedef struct lcb_CMDDIAG_ lcb_CMDDIAG;

LIBCOUCHBASE_API lcb_STATUS lcb_cmddiag_create(lcb_CMDDIAG **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmddiag_destroy(lcb_CMDDIAG *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmddiag_report_id(lcb_CMDDIAG *cmd, const char *report_id, size_t report_id_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmddiag_prettify(lcb_CMDDIAG *cmd, int enable);
LIBCOUCHBASE_API lcb_STATUS lcb_diag(lcb_INSTANCE *instance, void *cookie, const lcb_CMDDIAG *cmd);

/**@} (Group: PING) */

/**@ingroup lcb-public-api
 * @defgroup lcb-http HTTP Client
 * @brief Access Couchbase HTTP APIs
 * @details The low-level HTTP client may be used to access various HTTP-based
 * Couchbase APIs.
 *
 * Note that existing higher level APIs can be used for N1QL queries (see
 * @ref lcb-n1ql-api) and MapReduce view queries (see @ref lcb-view-api)
 *
 * @addtogroup lcb-http
 * @{
 */

/**
 * @brief The type of HTTP request to execute
 */
typedef enum {
    /**
     * Execute a request against the bucket. The handle must be of
     * @ref LCB_TYPE_BUCKET and must be connected.
     */
    LCB_HTTP_TYPE_VIEW = 0,

    /**
     * Execute a management API request. The credentials used will match
     * those passed during the instance creation time. Thus is the instance
     * type is @ref LCB_TYPE_BUCKET then only bucket-level credentials will
     * be used.
     */
    LCB_HTTP_TYPE_MANAGEMENT = 1,

    /**
     * Execute an arbitrary request against a host and port
     */
    LCB_HTTP_TYPE_RAW = 2,

    /** Execute an N1QL Query */
    LCB_HTTP_TYPE_N1QL = 3,

    /** Search a fulltext index */
    LCB_HTTP_TYPE_FTS = 4,

    /** Execute an Analytics Query */
    LCB_HTTP_TYPE_CBAS = 5,

    /**
     * Special pseudo-type, for ping endpoints in various services.
     * Behaves like RAW (the lcb_ping3() function will setup custom path),
     * but supports Keep-Alive
     */
    LCB_HTTP_TYPE_PING = 6,

    LCB_HTTP_TYPE_MAX
} lcb_HTTP_TYPE;

/**
 * @brief HTTP Request method enumeration
 * These just enumerate the various types of HTTP request methods supported.
 * Refer to the specific cluster or view API to see which method is appropriate
 * for your request
 */
typedef enum {
    LCB_HTTP_METHOD_GET = 0,
    LCB_HTTP_METHOD_POST = 1,
    LCB_HTTP_METHOD_PUT = 2,
    LCB_HTTP_METHOD_DELETE = 3,
    LCB_HTTP_METHOD_MAX = 4
} lcb_HTTP_METHOD;

/**
 * @committed
 * Issue an HTTP API request.
 * @param instance the library handle
 * @param cookie cookie to be associated with the request
 * @param cmd the command
 * @return LCB_SUCCESS if the request was scheduled successfully.
 *
 *
 * @par Simple Response
 * @code{.c}
 * void http_callback(lcb_INSTANCE, int, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPHTTP *resp = (const lcb_RESPHTTP *)rb;
 *     if (resp->rc != LCB_SUCCESS) {
 *         printf("I/O Error for HTTP: %s\n", lcb_strerror(NULL, resp->rc));
 *         return;
 *     }
 *     printf("Got HTTP Status: %d\n", resp->htstatus);
 *     printf("Got paylod: %.*s\n", (int)resp->nbody, resp->body);
 *     const char **hdrp = resp->headers;
 *     while (*hdrp != NULL) {
 *         printf("%s: %s\n", hdrp[0], hdrp[1]);
 *         hdrp += 2;
 *     }
 * }
 * @endcode
 *
 * @par Streaming Response
 * If the @ref LCB_CMDHTTP_F_STREAM flag is set in lcb_CMDHTTP::cmdflags then the
 * response callback is invoked multiple times as data arrives off the socket.
 * @code{.c}
 * void http_strm_callback(lcb_INSTANCE, int, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPHTTP *resp = (const lcb_RESPHTTP *)resp;
 *     if (resp->rflags & LCB_RESP_F_FINAL) {
 *         if (resp->rc != LCB_SUCCESS) {
 *             // ....
 *         }
 *         const char **hdrp = resp->headers;
 *         // ...
 *     } else {
 *         handle_body(resp->body, resp->nbody);
 *     }
 * }
 * @endcode
 *
 * @par Connection Reuse
 * The library will attempt to reuse connections for frequently contacted hosts.
 * By default the library will keep one idle connection to each host for a maximum
 * of 10 seconds. The number of open idle HTTP connections can be controlled with
 * @ref LCB_CNTL_HTTP_POOLSIZE.
 *
 */

typedef struct lcb_RESPHTTP_ lcb_RESPHTTP;

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_status(const lcb_RESPHTTP *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_cookie(const lcb_RESPHTTP *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_http_status(const lcb_RESPHTTP *resp, uint16_t *status);
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_path(const lcb_RESPHTTP *resp, const char **path, size_t *path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_body(const lcb_RESPHTTP *resp, const char **body, size_t *body_len);
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_handle(const lcb_RESPHTTP *resp, lcb_HTTP_HANDLE **handle);
LIBCOUCHBASE_API int lcb_resphttp_is_final(const lcb_RESPHTTP *resp);
/**
 * List of key-value headers. This field itself may be `NULL`. The list
 * is terminated by a `NULL` pointer to indicate no more headers.
 */
LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_headers(const lcb_RESPHTTP *resp, const char *const **headers);

typedef struct lcb_CMDHTTP_ lcb_CMDHTTP;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_create(lcb_CMDHTTP **cmd, lcb_HTTP_TYPE type);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_destroy(lcb_CMDHTTP *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_parent_span(lcb_CMDHTTP *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_method(lcb_CMDHTTP *cmd, lcb_HTTP_METHOD method);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_path(lcb_CMDHTTP *cmd, const char *path, size_t path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_content_type(lcb_CMDHTTP *cmd, const char *content_type,
                                                     size_t content_type_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_body(lcb_CMDHTTP *cmd, const char *body, size_t body_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_handle(lcb_CMDHTTP *cmd, lcb_HTTP_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_username(lcb_CMDHTTP *cmd, const char *username, size_t username_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_password(lcb_CMDHTTP *cmd, const char *password, size_t password_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_host(lcb_CMDHTTP *cmd, const char *host, size_t host_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_streaming(lcb_CMDHTTP *cmd, int streaming);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_skip_auth_header(lcb_CMDHTTP *cmd, int skip_auth);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_timeout(lcb_CMDHTTP *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_http(lcb_INSTANCE *instance, void *cookie, const lcb_CMDHTTP *cmd);

/**
 * @brief Cancel ongoing HTTP request
 *
 * This API will stop the current request. Any pending callbacks will not be
 * invoked any any pending data will not be delivered. Useful for a long running
 * request which is no longer needed
 *
 * @param instance The handle to lcb
 * @param request The request handle
 *
 * @committed
 *
 * @par Example
 * @code{.c}
 * lcb_CMDHTTP htcmd = { 0 };
 * populate_htcmd(&htcmd); // dummy function
 * lcb_http_request_t reqhandle;
 * htcmd.reqhandle = &reqhandle;
 * lcb_http3(instance, cookie, &htcmd);
 * do_stuff();
 * lcb_cancel_http_request(instance, reqhandle);
 * @endcode
 */
LIBCOUCHBASE_API lcb_STATUS lcb_http_cancel(lcb_INSTANCE *instance, lcb_HTTP_HANDLE *handle);

/**@} (Group: HTTP) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-cookie User Cookies
 * @brief Associate user-defined data with operations
 * @details
 *
 * User-defined pointers may be passed to all operations in the form of a
 * `cookie` parameter. This cookie parameter allows any kind of application
 * context to be accessible via the callback (in lcb_RESPBASE::cookie).
 *
 * The library will not inspect or manage the address or contents of the
 * cookie; it may live on the stack (especially if using the library
 * synchronously), on the heap, or may be NULL altogether.
 *
 * In addition to per-operation cookies, the library allows the instance itself
 * (i.e. the `lcb_INSTANCE` object) to contain its own cookie. This is helpful when
 * there is a wrapper object which needs to be accessed from within the callback
 *
 * @addtogroup lcb-cookie
 * @{
 */

/**
 * Associate a cookie with an instance of lcb. The _cookie_ is a user defined
 * pointer which will remain attached to the specified `lcb_INSTANCE` for its duration.
 * This is the way to associate user data with the `lcb_INSTANCE`.
 *
 * @param instance the instance to associate the cookie to
 * @param cookie the cookie to associate with this instance.
 *
 * @attention
 * There is no destructor for the specified `cookie` stored with the instance;
 * thus you must ensure to manually free resources to the pointer (if it was
 * dynamically allocated) when it is no longer required.
 * @committed
 *
 * @code{.c}
 * typedef struct {
 *   const char *status;
 *   // ....
 * } instance_info;
 *
 * static void bootstrap_callback(lcb_INSTANCE *instance, lcb_STATUS err) {
 *   instance_info *info = (instance_info *)lcb_get_cookie(instance);
 *   if (err == LCB_SUCCESS) {
 *     info->status = "Connected";
 *   } else {
 *     info->status = "Error";
 *   }
 * }
 *
 * static void do_create(void) {
 *   instance_info *info = calloc(1, sizeof(*info));
 *   // info->status is currently NULL
 *   // .. create the instance here
 *   lcb_set_cookie(instance, info);
 *   lcb_set_bootstrap_callback(instance, bootstrap_callback);
 *   lcb_connect(instance);
 *   lcb_wait(instance);
 *   printf("Status of instance is %s\n", info->status);
 * }
 * @endcode
 */
LIBCOUCHBASE_API
void lcb_set_cookie(lcb_INSTANCE *instance, const void *cookie);

/**
 * Retrieve the cookie associated with this instance
 * @param instance the instance of lcb
 * @return The cookie associated with this instance or NULL
 * @see lcb_set_cookie()
 * @committed
 */
LIBCOUCHBASE_API
const void *lcb_get_cookie(lcb_INSTANCE *instance);
/**@} (Group: Cookies) */

/**
 * @defgroup lcb-wait Waiting
 * @brief Functions for synchronous I/O execution
 * @details The lcb_wait() family of functions allow to control when the
 * library sends the operations to the cluster and waits for their execution.
 *
 * It is also possible to use non-blocking I/O with the library
 *
 * @addtogroup lcb-wait
 * @{
 */

/**
 * @brief Wait for the execution of all batched requests
 *
 * A batched request is any request which requires network I/O.
 * This includes most of the APIs. You should _not_ use this API if you are
 * integrating with an asynchronous event loop (i.e. one where your application
 * code is invoked asynchronously via event loops).
 *
 * This function will block the calling thread until either
 *
 * * All operations have been completed
 * * lcb_breakout() is explicitly called
 *
 * @param instance the instance containing the requests
 * @return whether the wait operation failed, or LCB_SUCCESS
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_wait(lcb_INSTANCE *instance);

/**
 * @volatile
 * This function will cause a single "tick" in the underlying event loop,
 * causing operations whose I/O can be executed immediately to be sent to
 * the server.
 *
 * Like lcb_wait(), callbacks for operations may be delivered here, however
 * some operations may be left incomplete if their I/O could not be processed
 * immediately. This function is intended as an optimization for large batches
 * of operations - so that some I/O can be completed during the batching process
 * itself, and only the remainder of those operations (which would have blocked)
 * will be completed with lcb_wait() (which should be invoked after the batch).
 *
 * This function is mainly useful if there is a significant delay in time
 * between each scheduling function, in which I/O may be completed during these
 * gaps (thereby emptying the socket's kernel write buffer, and allowing for
 * new operations to be added after the interval). Calling this function for
 * batches where all data is available up-front may actually make things slower.
 *
 * @warning
 * You must call lcb_wait() at least one after any batch of operations to ensure
 * they have been completed. This function is provided as an optimization only.
 *
 * @return LCB_CLIENT_FEATURE_UNAVAILABLE if the event loop does not support
 * the "tick" mode.
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_tick_nowait(lcb_INSTANCE *instance);

/**@brief Flags for lcb_wait3()*/
typedef enum {
    /**Behave like the old lcb_wait()*/
    LCB_WAIT_DEFAULT = 0x00,

    /**Do not check pending operations before running the event loop. By default
     * lcb_wait() will traverse the server list to check if any operations are
     * pending, and if nothing is pending the function will return without
     * running the event loop. This is usually not necessary for applications
     * which already _only_ call lcb_wait() when they know they have scheduled
     * at least one command.
     */
    LCB_WAIT_NOCHECK = 0x01
} lcb_WAITFLAGS;

/**
 * @committed
 * @brief Wait for completion of scheduled operations.
 * @param instance the instance
 * @param flags flags to modify the behavior of lcb_wait(). Pass 0 to obtain
 * behavior identical to lcb_wait().
 */
LIBCOUCHBASE_API
void lcb_wait3(lcb_INSTANCE *instance, lcb_WAITFLAGS flags);

/**
 * @brief Forcefully break from the event loop.
 *
 * You may call this function from within any callback to signal to the library
 * that it return control to the function calling lcb_wait() as soon as possible.
 * Note that if there are pending functions which have not been processed, you
 * are responsible for calling lcb_wait() a second time.
 *
 * @param instance the instance to run the event loop for.
 * @committed
 */
LIBCOUCHBASE_API
void lcb_breakout(lcb_INSTANCE *instance);

/**
 * @brief Check if instance is blocked in the event loop
 * @param instance the instance to run the event loop for.
 * @return non-zero if nobody is waiting for IO interaction
 * @uncommitted
 */
LIBCOUCHBASE_API
int lcb_is_waiting(lcb_INSTANCE *instance);
/**@} (Group: Wait) */

/**
 * @uncommitted
 *
 * @brief Force the library to refetch the cluster configuration
 *
 * The library by default employs various heuristics to determine if a new
 * configuration is needed from the cluster. However there are some situations
 * in which an application may wish to force a refresh of the configuration:
 *
 * * If a specific node has been failed
 *   over and the library has received a configuration in which there is no
 *   master node for a given key, the library will immediately return the error
 *   `LCB_NO_MATCHING_SERVER` for the given item and will not request a new
 *   configuration. In this state, the client will not perform any network I/O
 *   until a request has been made to it using a key that is mapped to a known
 *   active node.
 *
 * * The library's heuristics may have failed to detect an error warranting
 *   a configuration change, but the application either through its own
 *   heuristics, or through an out-of-band channel knows that the configuration
 *   has changed.
 *
 *
 * This function is provided as an aid to assist in such situations
 *
 * If you wish for your application to block until a new configuration is
 * received, you _must_ call lcb_wait3() with the LCB_WAIT_NO_CHECK flag as
 * this function call is not bound to a specific operation. Additionally there
 * is no status notification as to whether this operation succeeded or failed
 * (the configuration callback via lcb_set_configuration_callback() may
 * provide hints as to whether a configuration was received or not, but by no
 * means should be considered to be part of this function's control flow).
 *
 * In general the use pattern of this function is like so:
 *
 * @code{.c}
 * unsigned retries = 5;
 * lcb_STATUS err;
 * do {
 *   retries--;
 *   err = lcb_get(instance, cookie, ncmds, cmds);
 *   if (err == LCB_NO_MATCHING_SERVER) {
 *     lcb_refresh_config(instance);
 *     usleep(100000);
 *     lcb_wait3(instance, LCB_WAIT_NO_CHECK);
 *   } else {
 *     break;
 *   }
 * } while (retries);
 * if (err == LCB_SUCCESS) {
 *   lcb_wait3(instance, 0); // equivalent to lcb_wait(instance);
 * } else {
 *   printf("Tried multiple times to fetch the key, but its node is down\n");
 * }
 * @endcode
 */
LIBCOUCHBASE_API
void lcb_refresh_config(lcb_INSTANCE *instance);

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-sched Advanced Scheduling
 * @brief Additional functions for scheduling operations
 *
 * @details
 *
 * An application may spool multiple operations into the library with the
 * option of unspooling previously-spooled operations in case one of
 * the operations cannot be spooled. These semantics exist primarily to
 * support "all-or-nothing" scheduling found in the V2 API as well as in
 * some other wrapping SDKs.
 *
 * From version 2.4.0 to version 2.5.5, use of the explicit scheduling
 * API was mandatory to schedule operations. This API is optional since 2.5.6.
 *
 * The following operation APIs are low level entry points which create a
 * single operation. To use these operation APIs you should call the
 * lcb_sched_enter() which creates a virtual scope in which to create operations.
 *
 * For each of these operation APIs, the actual API call will insert the
 * created packet into a "Scheduling Queue" (this is done through
 * mcreq_sched_add() which is in mcreq.h). You may add as many items to this
 * scheduling queue as you would like.
 *
 * Note that an operation is only added to the queue if it was able to be
 * scheduled properly. If a scheduling failure occurred (for example, if a
 * configuration is missing, the command had invalid input, or memory allocation
 * failed) then the command will not be placed into the queue.
 *
 * Once all operations have been scheduled you can call
 * lcb_sched_leave() which will place all commands scheduled into the I/O
 * queue.
 *
 * If you wish to _discard_ all scheduled operations (for example, if one of
 * them errored, and your application cannot handle partial scheduling failures)
 * then you may call lcb_sched_fail() which will release all the resources
 * of the packets placed into the temporary queue.
 *
 * @par Behavior from version 2.5.6
 *
 * Starting from version 2.5.6, use of this API is optional. Scheduling functions
 * will now check if an empty call to lcb_sched_enter() is present. If no call
 * to lcb_sched_enter() is found then the library will implicitly call
 * lcb_sched_leave().
 *
 * @addtogroup lcb-sched
 * @{
 */

/**
 * @brief Enter a scheduling context.
 *
 * @uncommitted
 *
 * A scheduling context is an ephemeral list of
 * commands issued to various servers. Operations (like lcb_get3(), lcb_store3())
 * place packets into the current context.
 *
 * The context mechanism allows you to efficiently pipeline and schedule multiple
 * operations of different types and quantities. The network is not touched
 * and nothing is scheduled until the context is exited.
 *
 * @param instance the instance
 *
 * @code{.c}
 * lcb_sched_enter(instance);
 * lcb_get3(...);
 * lcb_store3(...);
 * lcb_counter3(...);
 * lcb_sched_leave(instance);
 * lcb_wait3(instance, LCB_WAIT_NOCHECK);
 * @endcode
 */
LIBCOUCHBASE_API
void lcb_sched_enter(lcb_INSTANCE *instance);

/**
 * @uncommitted
 *
 * @brief Leave the current scheduling context, scheduling the commands within the
 * context to be flushed to the network.
 *
 * @details This will initiate a network-level flush (depending on the I/O system)
 * to the network. For completion-based I/O systems this typically means
 * allocating a temporary write context to contain the buffer. If using a
 * completion-based I/O module (for example, Windows or libuv) then it is
 * recommended to limit the number of calls to one per loop iteration. If
 * limiting the number of calls to this function is not possible (for example,
 * if the legacy API is being used, or you wish to use implicit scheduling) then
 * the flushing may be decoupled from this function - see the documentation for
 * lcb_sched_flush().
 *
 * @param instance the instance
 */
LIBCOUCHBASE_API
void lcb_sched_leave(lcb_INSTANCE *instance);

/**
 * @uncommitted
 * @brief Fail all commands in the current scheduling context.
 *
 * The commands placed within the current
 * scheduling context are released and are never flushed to the network.
 * @param instance
 *
 * @warning
 * This function only affects commands which have a direct correspondence
 * to memcached packets. Currently these are commands scheduled by:
 *
 * * lcb_get3()
 * * lcb_rget3()
 * * lcb_unlock3()
 * * lcb_touch3()
 * * lcb_store3()
 * * lcb_counter3()
 * * lcb_remove3()
 * * lcb_stats3()
 * * lcb_observe3_ctxnew()
 * * lcb_observe_seqno3()
 *
 * Other commands are _compound_ commands and thus should be in their own
 * scheduling context.
 */
LIBCOUCHBASE_API
void lcb_sched_fail(lcb_INSTANCE *instance);

/**
 * @committed
 * @brief Request commands to be flushed to the network
 *
 * By default, the library will implicitly request a flush to the network upon
 * every call to lcb_sched_leave().
 *
 * [ Note, this does not mean the items are flushed
 * and I/O is performed, but it means the relevant event loop watchers are
 * activated to perform the operations on the next iteration ]. If
 * @ref LCB_CNTL_SCHED_IMPLICIT_FLUSH
 * is disabled then this behavior is disabled and the
 * application must explicitly call lcb_sched_flush(). This may be considered
 * more performant in the cases where multiple discreet operations are scheduled
 * in an lcb_sched_enter()/lcb_sched_leave() pair. With implicit flush enabled,
 * each call to lcb_sched_leave() will possibly invoke system repeatedly.
 */
LIBCOUCHBASE_API
void lcb_sched_flush(lcb_INSTANCE *instance);

/**@} (Group: Adanced Scheduling) */

/**@ingroup lcb-public-api
 * @defgroup lcb-destroy Destroying
 * @brief Library destruction routines
 * @addtogroup lcb-destroy
 * @{
 */
/**
 * Destroy (and release all allocated resources) an instance of lcb.
 * Using instance after calling destroy will most likely cause your
 * application to crash.
 *
 * Note that any pending operations will not have their callbacks invoked.
 *
 * @param instance the instance to destroy.
 * @committed
 */
LIBCOUCHBASE_API
void lcb_destroy(lcb_INSTANCE *instance);

/**
 * @brief Callback received when instance is about to be destroyed
 * @param cookie cookie passed to lcb_destroy_async()
 */
typedef void (*lcb_destroy_callback)(const void *cookie);

/**
 * @brief Set the callback to be invoked when the instance is destroyed
 * asynchronously.
 * @return the previous callback.
 */
LIBCOUCHBASE_API
lcb_destroy_callback lcb_set_destroy_callback(lcb_INSTANCE *instance, lcb_destroy_callback);
/**
 * @brief Asynchronously schedule the destruction of an instance.
 *
 * This function provides a safe way for asynchronous environments to destroy
 * the lcb_INSTANCE *handle without worrying about reentrancy issues.
 *
 * @param instance
 * @param arg a pointer passed to the callback.
 *
 * While the callback and cookie are optional, they are very much recommended
 * for testing scenarios where you wish to ensure that all resources allocated
 * by the instance have been closed. Specifically when the callback is invoked,
 * all timers (save for the one actually triggering the destruction) and sockets
 * will have been closed.
 *
 * As with lcb_destroy() you may call this function only once. You may not
 * call this function together with lcb_destroy as the two are mutually
 * exclusive.
 *
 * If for whatever reason this function is being called in a synchronous
 * flow, lcb_wait() must be invoked in order for the destruction to take effect
 *
 * @see lcb_set_destroy_callback
 *
 * @committed
 */
LIBCOUCHBASE_API
void lcb_destroy_async(lcb_INSTANCE *instance, const void *arg);
/**@} (Group: Destroy) */

/**@}*/

/** @internal */
#define LCB_DATATYPE_JSON 0x01

/** @internal */
typedef enum { LCB_VALUE_RAW = 0x00, LCB_VALUE_F_JSON = 0x01, LCB_VALUE_F_SNAPPYCOMP = 0x02 } lcb_VALUEFLAGS;

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-cluster-status Cluster Information
 * @brief These functions return status information about the handle, the current
 * connection, and the number of nodes found within the cluster.
 *
 * @see lcb_cntl() for more functions to retrieve status info
 *
 * @addtogroup lcb-cluster-status
 * @{
 */

/**@brief
 * Type of node to retrieve for the lcb_get_node() function
 */
typedef enum {
    /** Get an HTTP configuration (Rest API) node */
    LCB_NODE_HTCONFIG = 0x01,
    /** Get a data (memcached) node */
    LCB_NODE_DATA = 0x02,
    /** Get a view (CAPI) node */
    LCB_NODE_VIEWS = 0x04,
    /** Only return a node which is connected, or a node which is known to be up */
    LCB_NODE_CONNECTED = 0x08,

    /** Specifying this flag adds additional semantics which instruct the library
     * to search additional resources to return a host, and finally,
     * if no host can be found, return the string
     * constant @ref LCB_GETNODE_UNAVAILABLE. */
    LCB_NODE_NEVERNULL = 0x10,

    /** Equivalent to `LCB_NODE_HTCONFIG|LCB_NODE_CONNECTED` */
    LCB_NODE_HTCONFIG_CONNECTED = 0x09,

    /**Equivalent to `LCB_NODE_HTCONFIG|LCB_NODE_NEVERNULL`.
     * When this is passed, some additional attempts may be made by the library
     * to return any kind of host, including searching the initial list of hosts
     * passed to the lcb_create() function. */
    LCB_NODE_HTCONFIG_ANY = 0x11
} lcb_GETNODETYPE;

/** String constant returned by lcb_get_node() when the @ref LCB_NODE_NEVERNULL
 * flag is specified, and no node can be returned */
#define LCB_GETNODE_UNAVAILABLE "invalid_host:0"

/**
 * @brief Return a string of `host:port` for a node of the given type.
 *
 * @param instance the instance from which to retrieve the node
 * @param type the type of node to return
 * @param index the node number if index is out of bounds it will be wrapped
 * around, thus there is never an invalid value for this parameter
 *
 * @return a string in the form of `host:port`. If LCB_NODE_NEVERNULL was specified
 * as an option in `type` then the string constant LCB_GETNODE_UNAVAILABLE is
 * returned. Otherwise `NULL` is returned if the type is unrecognized or the
 * LCB_NODE_CONNECTED option was specified and no connected node could be found
 * or a memory allocation failed.
 *
 * @note The index parameter is _ignored_ if `type` is
 * LCB_NODE_HTCONFIG|LCB_NODE_CONNECTED as there will always be only a single
 * HTTP bootstrap node.
 *
 * @code{.c}
 * const char *viewnode = lcb_get_node(instance, LCB_NODE_VIEWS, 0);
 * // Get the connected REST endpoint:
 * const char *restnode = lcb_get_node(instance, LCB_NODE_HTCONFIG|LCB_NODE_CONNECTED, 0);
 * if (!restnode) {
 *   printf("Instance not connected via HTTP!\n");
 * }
 * @endcode
 *
 * Iterate over all the data nodes:
 * @code{.c}
 * unsigned ii;
 * for (ii = 0; ii < lcb_get_num_servers(instance); ii++) {
 *   const char *kvnode = lcb_get_node(instance, LCB_NODE_DATA, ii);
 *   if (kvnode) {
 *     printf("KV node %s exists at index %u\n", kvnode, ii);
 *   } else {
 *     printf("No node for index %u\n", ii);
 *   }
 * }
 * @endcode
 *
 * @committed
 */
LIBCOUCHBASE_API
const char *lcb_get_node(lcb_INSTANCE *instance, lcb_GETNODETYPE type, unsigned index);

/**
 * @committed
 *
 * @brief Get the target server for a given key.
 *
 * This is a convenience function wrapping around the vBucket API which allows
 * you to retrieve the target node (the node which will be contacted) when
 * performing KV operations involving the key.
 *
 * @param instance the instance
 * @param key the key to use
 * @param nkey the length of the key
 * @return a string containing the hostname, or NULL on error.
 *
 * Since this is a convenience function, error details are not contained here
 * in favor of brevity. Use the full vBucket API for more powerful functions.
 */
LIBCOUCHBASE_API
const char *lcb_get_keynode(lcb_INSTANCE *instance, const void *key, size_t nkey);

/**
 * @brief Get the number of the replicas in the cluster
 *
 * @param instance The handle to lcb
 * @return -1 if the cluster wasn't configured yet, and number of replicas
 * otherwise. This may be `0` if there are no replicas.
 * @committed
 */
LIBCOUCHBASE_API
lcb_S32 lcb_get_num_replicas(lcb_INSTANCE *instance);

/**
 * @brief Get the number of the nodes in the cluster
 * @param instance The handle to lcb
 * @return -1 if the cluster wasn't configured yet, and number of nodes otherwise.
 * @committed
 */
LIBCOUCHBASE_API
lcb_S32 lcb_get_num_nodes(lcb_INSTANCE *instance);

/**
 * @brief Get a list of nodes in the cluster
 *
 * @return a NULL-terminated list of 0-terminated strings consisting of
 * node hostnames:admin_ports for the entire cluster.
 * The storage duration of this list is only valid until the
 * next call to a libcouchbase function and/or when returning control to
 * libcouchbase' event loop.
 *
 * @code{.c}
 * const char * const * curp = lcb_get_server_list(instance);
 * for (; *curp; curp++) {
 *   printf("Have node %s\n", *curp);
 * }
 * @endcode
 * @committed
 */
LIBCOUCHBASE_API
const char *const *lcb_get_server_list(lcb_INSTANCE *instance);

/**@} (Group: Cluster Info) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-cntl Settings
 * @brief Get/Set Library Options
 *
 * @details
 *
 * The lcb_cntl() function and its various helpers are the means by which to
 * modify settings within the library
 *
 * @addtogroup lcb-cntl
 * @see <cntl.h>
 * @{
 */

/**
 * This function exposes an ioctl/fcntl-like interface to read and write
 * various configuration properties to and from an lcb_INSTANCE *handle.
 *
 * @param instance The instance to modify
 *
 * @param mode One of LCB_CNTL_GET (to retrieve a setting) or LCB_CNTL_SET
 *      (to modify a setting). Note that not all configuration properties
 *      support SET.
 *
 * @param cmd The specific command/property to modify. This is one of the
 *      LCB_CNTL_* constants defined in this file. Note that it is safe
 *      (and even recommanded) to use the raw numeric value (i.e.
 *      to be backwards and forwards compatible with libcouchbase
 *      versions), as they are not subject to change.
 *
 *      Using the actual value may be useful in ensuring your application
 *      will still compile with an older libcouchbase version (though
 *      you may get a runtime error (see return) if the command is not
 *      supported
 *
 * @param arg The argument passed to the configuration handler.
 *      The actual type of this pointer is dependent on the
 *      command in question.  Typically for GET operations, the
 *      value of 'arg' is set to the current configuration value;
 *      and for SET operations, the current configuration is
 *      updated with the contents of *arg.
 *
 * @return ::LCB_NOT_SUPPORTED if the code is unrecognized
 * @return ::LCB_EINVAL if there was a problem with the argument
 *         (typically for LCB_CNTL_SET) other error codes depending on the command.
 *
 * The following error codes are returned if the ::LCB_CNTL_DETAILED_ERRCODES
 * are enabled.
 *
 * @return ::LCB_ECTL_UNKNOWN if the code is unrecognized
 * @return ::LCB_ECTL_UNSUPPMODE An invalid _mode_ was passed
 * @return ::LCB_ECTL_BADARG if the value was invalid
 *
 * @committed
 *
 * @see lcb_cntl_setu32()
 * @see lcb_cntl_string()
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_cntl(lcb_INSTANCE *instance, int mode, int cmd, void *arg);

/**
 * Alternatively one may change configuration settings by passing a string key
 * and value. This may be used to provide a simple interface from a command
 * line or higher level language to allow the setting of specific key-value
 * pairs.
 *
 * The format for the value is dependent on the option passed, the following
 * value types exist:
 *
 * - **Timeval**. A _timeval_ value can either be specified as fractional
 *   seconds (`"1.5"` for 1.5 seconds), or in microseconds (`"1500000"`). In
 *   releases prior to libcouchbase 2.8, this was called _timeout_.
 * - **Number**. This is any valid numerical value. This may be signed or
 *   unsigned depending on the setting.
 * - **Boolean**. This specifies a boolean. A true value is either a positive
 *   numeric value (i.e. `"1"`) or the string `"true"`. A false value
 *   is a zero (i.e. `"0"`) or the string `"false"`.
 * - **Float**. This is like a _Number_, but also allows fractional specification,
 *   e.g. `"2.4"`.
 * - **String**. Arbitrary string as `char *`, e.g. for client identification
 *   string.
 * - **Path**. File path.
 * - **FILE*, Path**. Set file stream pointer (lcb_cntl() style) or file path
 *   (lcb_cntl_string() style).
 *
 * | Code                                    | Name                      | Type              |
 * |-----------------------------------------|---------------------------|-------------------|
 * |@ref LCB_CNTL_OP_TIMEOUT                 | `"operation_timeout"`     | Timeval           |
 * |@ref LCB_CNTL_VIEW_TIMEOUT               | `"view_timeout"`          | Timeval           |
 * |@ref LCB_CNTL_N1QL_TIMEOUT               | `"n1ql_timeout"`          | Timeval           |
 * |@ref LCB_CNTL_HTTP_TIMEOUT               | `"http_timeout"`          | Timeval           |
 * |@ref LCB_CNTL_CONFIG_POLL_INTERVAL       | `"config_poll_interval"`  | Timeval           |
 * |@ref LCB_CNTL_CONFERRTHRESH              | `"error_thresh_count"`    | Number (Positive) |
 * |@ref LCB_CNTL_CONFIGURATION_TIMEOUT      | `"config_total_timeout"`  | Timeval           |
 * |@ref LCB_CNTL_CONFIG_NODE_TIMEOUT        | `"config_node_timeout"`   | Timeval           |
 * |@ref LCB_CNTL_CONFDELAY_THRESH           | `"error_thresh_delay"`    | Timeval           |
 * |@ref LCB_CNTL_DURABILITY_TIMEOUT         | `"durability_timeout"`    | Timeval           |
 * |@ref LCB_CNTL_DURABILITY_INTERVAL        | `"durability_interval"`   | Timeval           |
 * |@ref LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS  | `"randomize_nodes"`       | Boolean           |
 * |@ref LCB_CNTL_CONFIGCACHE                | `"config_cache"`          | Path              |
 * |@ref LCB_CNTL_DETAILED_ERRCODES          | `"detailed_errcodes"`     | Boolean           |
 * |@ref LCB_CNTL_HTCONFIG_URLTYPE           | `"http_urlmode"`          | Number (enum #lcb_HTCONFIG_URLTYPE) |
 * |@ref LCB_CNTL_RETRY_INTERVAL             | `"retry_interval"`        | Timeval           |
 * |@ref LCB_CNTL_HTTP_POOLSIZE              | `"http_poolsize"`         | Number            |
 * |@ref LCB_CNTL_VBGUESS_PERSIST            | `"vbguess_persist"`       | Boolean           |
 * |@ref LCB_CNTL_CONLOGGER_LEVEL            | `"console_log_level"`     | Number (enum #lcb_log_severity_t) |
 * |@ref LCB_CNTL_FETCH_MUTATION_TOKENS      | `"fetch_mutation_tokens"` | Boolean           |
 * |@ref LCB_CNTL_DURABILITY_MUTATION_TOKENS | `"dur_mutation_tokens"`   | Boolean           |
 * |@ref LCB_CNTL_TCP_NODELAY                | `"tcp_nodelay"`           | Boolean           |
 * |@ref LCB_CNTL_CONLOGGER_FP               | `"console_log_file"`      | FILE*, Path       |
 * |@ref LCB_CNTL_CLIENT_STRING              | `"client_string"`         | String            |
 * |@ref LCB_CNTL_TCP_KEEPALIVE              | `"tcp_keepalive"`         | Boolean           |
 * |@ref LCB_CNTL_CONFIG_POLL_INTERVAL       | `"config_poll_interval"`  | Timeval           |
 * |@ref LCB_CNTL_IP6POLICY                  | `"ipv6"`                  | String ("disabled", "only", "allow") |
 *
 * @committed - Note, the actual API call is considered committed and will
 * not disappear, however the existence of the various string settings are
 * dependendent on the actual settings they map to. It is recommended that
 * applications use the numerical lcb_cntl() as the string names are
 * subject to change.
 *
 * @see lcb_cntl()
 * @see lcb-cntl-settings
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_cntl_string(lcb_INSTANCE *instance, const char *key, const char *value);

/**
 * @brief Convenience function to set a value as an lcb_U32
 * @param instance
 * @param cmd setting to modify
 * @param arg the new value
 * @return see lcb_cntl() for details
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_cntl_setu32(lcb_INSTANCE *instance, int cmd, lcb_U32 arg);

/**
 * @brief Retrieve an lcb_U32 setting
 * @param instance
 * @param cmd setting to retrieve
 * @return the value.
 * @warning This function does not return an error code. Ensure that the cntl is
 * correct for this version, or use lcb_cntl() directly.
 * @committed
 */
LIBCOUCHBASE_API
lcb_U32 lcb_cntl_getu32(lcb_INSTANCE *instance, int cmd);

/**
 * Determine if a specific control code exists
 * @param ctl the code to check for
 * @return 0 if it does not exist, nonzero if it exists.
 */
LIBCOUCHBASE_API
int lcb_cntl_exists(int ctl);
/**@}*/ /* settings */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-build-info Build Information
 * @brief Get library version and supported features
 * @details
 * These functions and macros may be used to conditionally compile features
 * depending on the version of the library being used. They may also be used
 * to employ various features at runtime and to retrieve the version for
 * informational purposes.
 * @addtogroup lcb-build-info
 * @{
 */

#if !defined(LCB_VERSION_STRING) || defined(__LCB_DOXYGEN__)
/** @brief libcouchbase version string */
#define LCB_VERSION_STRING "unknown"
#endif

#if !defined(LCB_VERSION) || defined(__LCB_DOXYGEN__)
/**@brief libcouchbase hex version
 *
 * This number contains the hexadecimal representation of the library version.
 * It is in a format of `0xXXYYZZ` where `XX` is the two digit major version
 * (e.g. `02`), `YY` is the minor version (e.g. `05`) and `ZZ` is the patch
 * version (e.g. `24`).
 *
 * For example:
 *
 * String   |Hex
 * ---------|---------
 * 2.0.0    | 0x020000
 * 2.1.3    | 0x020103
 * 3.0.15   | 0x030015
 */
#define LCB_VERSION 0x000000
#endif

#if !defined(LCB_VERSION_CHANGESET) || defined(__LCB_DOXYGEN__)
/**@brief The SCM revision ID. @see LCB_CNTL_CHANGESET */
#define LCB_VERSION_CHANGESET "0xdeadbeef"
#endif

/**
 * Get the version of the library.
 *
 * @param version where to store the numeric representation of the
 *         version (or NULL if you don't care)
 *
 * @return the textual description of the version ('\0'
 *          terminated). Do <b>not</b> try to release this string.
 *
 */
LIBCOUCHBASE_API
const char *lcb_get_version(lcb_U32 *version);

/** Global/extern variable containing the version of the library */
LIBCOUCHBASE_API LCB_EXTERN_VAR const lcb_U32 lcb_version_g;

/**@brief Whether the library has SSL support*/
#define LCB_SUPPORTS_SSL 1
/**@brief Whether the library has experimental compression support */
#define LCB_SUPPORTS_SNAPPY 2
/**@brief Whether the library has experimental tracing support */
#define LCB_SUPPORTS_TRACING 3

/**
 * @committed
 * Determine if this version has support for a particularl feature
 * @param n the feature ID to check for
 * @return 0 if not supported, nonzero if supported.
 */
LIBCOUCHBASE_API
int lcb_supports_feature(int n);

/**@} (Group: Build Info) */

/**
 * Functions to allocate and free memory related to libcouchbase. This is
 * mainly for use on Windows where it is possible that the DLL and EXE
 * are using two different CRTs
 */
LIBCOUCHBASE_API
void *lcb_mem_alloc(lcb_SIZE size);

/** Use this to free memory allocated with lcb_mem_alloc */
LIBCOUCHBASE_API
void lcb_mem_free(void *ptr);

/**
 * @internal
 *
 * These two functions unconditionally start and stop the event loop. These
 * should be used _only_ when necessary. Use lcb_wait and lcb_breakout
 * for safer variants.
 *
 * Internally these proxy to the run_event_loop/stop_event_loop calls
 */
LCB_INTERNAL_API
void lcb_run_loop(lcb_INSTANCE *instance);

/** @internal */
LCB_INTERNAL_API
void lcb_stop_loop(lcb_INSTANCE *instance);

/** @internal */
/* This returns the library's idea of time */
LCB_INTERNAL_API
lcb_U64 lcb_nstime(void);

/**
 * @volatile
 * Returns whether the library redacting logs for this connection instance.
 *
 * @return non-zero if the logs are being redacted for this instance.
 */
LIBCOUCHBASE_API
int lcb_is_redacting_logs(lcb_INSTANCE *instance);

/**
 * @addtogroup lcb-analytics-api
 * @{
 */

typedef struct lcb_ANALYTICS_HANDLE_ lcb_ANALYTICS_HANDLE;
typedef struct lcb_DEFERRED_HANDLE_ lcb_DEFERRED_HANDLE;
typedef struct lcb_RESPANALYTICS_ lcb_RESPANALYTICS;
typedef void (*lcb_ANALYTICS_CALLBACK)(lcb_INSTANCE *, int, const lcb_RESPANALYTICS *);

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_status(const lcb_RESPANALYTICS *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_cookie(const lcb_RESPANALYTICS *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_row(const lcb_RESPANALYTICS *resp, const char **row, size_t *row_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_http_response(const lcb_RESPANALYTICS *resp, const lcb_RESPHTTP **http);
LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_handle(const lcb_RESPANALYTICS *resp, lcb_ANALYTICS_HANDLE **handle);
LIBCOUCHBASE_API int lcb_respanalytics_is_final(const lcb_RESPANALYTICS *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_deferred_handle_extract(const lcb_RESPANALYTICS *resp,
                                                                      lcb_DEFERRED_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_destroy(lcb_DEFERRED_HANDLE *handle);
LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_status(lcb_DEFERRED_HANDLE *handle, const char **status,
                                                       size_t *status_len);
LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_callback(lcb_DEFERRED_HANDLE *handle, lcb_ANALYTICS_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_poll(lcb_INSTANCE *instance, void *cookie, lcb_DEFERRED_HANDLE *handle);

typedef struct lcb_CMDANALYTICS_ lcb_CMDANALYTICS;

typedef struct lcb_INGEST_OPTIONS_ lcb_INGEST_OPTIONS;
typedef enum {
    LCB_INGEST_METHOD_NONE = 0,
    LCB_INGEST_METHOD_UPSERT,
    LCB_INGEST_METHOD_INSERT,
    LCB_INGEST_METHOD_REPLACE,
    LCB_INGEST_METHOD__MAX
} lcb_INGEST_METHOD;

typedef enum { LCB_INGEST_STATUS_OK = 0, LCB_INGEST_STATUS_IGNORE, LCB_INGEST_STATUS__MAX } lcb_INGEST_STATUS;

typedef struct lcb_INGEST_PARAM_ lcb_INGEST_PARAM;

typedef lcb_INGEST_STATUS (*lcb_INGEST_DATACONVERTER_CALLBACK)(lcb_INSTANCE *instance, lcb_INGEST_PARAM *param);

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_create(lcb_INGEST_OPTIONS **options);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_destroy(lcb_INGEST_OPTIONS *options);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_method(lcb_INGEST_OPTIONS *options, lcb_INGEST_METHOD method);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_expiration(lcb_INGEST_OPTIONS *options, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_ignore_error(lcb_INGEST_OPTIONS *options, int flag);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_data_converter(lcb_INGEST_OPTIONS *options,
                                                              lcb_INGEST_DATACONVERTER_CALLBACK callback);

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_cookie(lcb_INGEST_PARAM *param, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_row(lcb_INGEST_PARAM *param, const char **row,
                                                               size_t *row_len);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_method(lcb_INGEST_PARAM *param, lcb_INGEST_METHOD *method);
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_set_id(lcb_INGEST_PARAM *param, const char *id,
                                                                  size_t id_len, void (*id_dtor)(const char *));
LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_set_out(lcb_INGEST_PARAM *param, const char *out,
                                                                   size_t out_len, void (*out_dtor)(const char *));

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_create(lcb_CMDANALYTICS **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_destroy(lcb_CMDANALYTICS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_reset(lcb_CMDANALYTICS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_parent_span(lcb_CMDANALYTICS *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_callback(lcb_CMDANALYTICS *cmd, lcb_ANALYTICS_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_query(lcb_CMDANALYTICS *cmd, const char *query, size_t query_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_statement(lcb_CMDANALYTICS *cmd, const char *statement,
                                                       size_t statement_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_named_param(lcb_CMDANALYTICS *cmd, const char *name, size_t name_len,
                                                         const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_positional_param(lcb_CMDANALYTICS *cmd, const char *value,
                                                              size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_ingest_options(lcb_CMDANALYTICS *cmd, lcb_INGEST_OPTIONS *options);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_deferred(lcb_CMDANALYTICS *cmd, int deferred);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_option(lcb_CMDANALYTICS *cmd, const char *name, size_t name_len,
                                                    const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_handle(lcb_CMDANALYTICS *cmd, lcb_ANALYTICS_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_timeout(lcb_CMDANALYTICS *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_analytics(lcb_INSTANCE *instance, void *cookie, const lcb_CMDANALYTICS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_analytics_cancel(lcb_INSTANCE *instance, lcb_ANALYTICS_HANDLE *handle);

/** @} */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-cbft-api Full Text Search
 * @brief Search for strings in documents and more
 */

/**
 * @addtogroup lcb-cbft-api
 * @{
 */
typedef struct lcb_FTS_HANDLE_ lcb_FTS_HANDLE;
typedef struct lcb_RESPFTS_ lcb_RESPFTS;

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_status(const lcb_RESPFTS *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respfts_cookie(const lcb_RESPFTS *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respfts_row(const lcb_RESPFTS *resp, const char **row, size_t *row_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respfts_http_response(const lcb_RESPFTS *resp, const lcb_RESPHTTP **http);
LIBCOUCHBASE_API lcb_STATUS lcb_respfts_handle(const lcb_RESPFTS *resp, lcb_FTS_HANDLE **handle);
LIBCOUCHBASE_API int lcb_respfts_is_final(const lcb_RESPFTS *resp);

typedef struct lcb_CMDFTS_ lcb_CMDFTS;
typedef void (*lcb_FTS_CALLBACK)(lcb_INSTANCE *, int, const lcb_RESPFTS *);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_create(lcb_CMDFTS **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_destroy(lcb_CMDFTS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_parent_span(lcb_CMDFTS *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_callback(lcb_CMDFTS *cmd, lcb_FTS_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_query(lcb_CMDFTS *cmd, const char *query, size_t query_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_handle(lcb_CMDFTS *cmd, lcb_FTS_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_timeout(lcb_CMDFTS *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_fts(lcb_INSTANCE *instance, void *cookie, const lcb_CMDFTS *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_fts_cancel(lcb_INSTANCE *instance, lcb_FTS_HANDLE *handle);
/** @} */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-n1ql-api N1QL/Analytics
 * @brief Execute N1QL/Analytics queries.
 *
 * Query language based on SQL, but designed for structured and flexible JSON
 * documents. Querying can solve typical programming tasks such as finding a
 * user profile by email address, performing aggregations etc.
 *
 * @code{.c}
 * const char *query = "{\"statement\":\"SELECT * FROM breweries LIMIT 10\"}";
 * lcb_CMDN1QL cmd = {0};
 * int idx = 0;
 * // NOTE: with this flag, the request will be issued to Analytics service
 * cmd.cmdflags = LCB_CMDN1QL_F_ANALYTICSQUERY;
 * cmd.callback = row_callback;
 * cmd.query = query;
 * cmd.nquery = strlen(query);
 * lcb_n1ql_query(instance, &idx, &cmd);
 * lcb_wait(instance);
 * @endcode
 *
 * Where row_callback might be implemented like this:
 *
 * @code{.c}
 * static void row_callback(lcb_INSTANCE *instance, int type, const lcb_RESPN1QL *resp)
 * {
 *     int *idx = (int *)resp->cookie;
 *     if (resp->rc != LCB_SUCCESS) {
 *         printf("failed to execute query: %s\n", lcb_strerror_short(resp->rc));
 *         exit(EXIT_FAILURE);
 *     }
 *     if (resp->rflags & LCB_RESP_F_FINAL) {
 *         printf("META: ");
 *     } else {
 *         printf("ROW #%d: ", (*idx)++);
 *     }
 *     printf("%.*s\n", (int)resp->nrow, (char *)resp->row);
 * }
 * @endcode
 *
 * @see more details on @ref lcb_n1ql_query and @ref lcb_CMDN1QL.
 *
 * Also there is a query builder available for N1QL queries: @ref lcb_n1p_new/@ref lcb_n1p_mkcmd.
 */

/**
 * @addtogroup lcb-n1ql-api
 * @{
 */
typedef struct lcb_RESPN1QL_ lcb_RESPN1QL;
typedef struct lcb_CMDN1QL_ lcb_CMDN1QL;
/**
 * Pointer for request instance
 */
typedef struct lcb_N1QL_HANDLE_ lcb_N1QL_HANDLE;

/**
 * Callback to be invoked for each row
 * @param The instance
 * @param Callback type. This is set to @ref LCB_CALLBACK_N1QL
 * @param The response.
 */
typedef void (*lcb_N1QL_CALLBACK)(lcb_INSTANCE *, int, const lcb_RESPN1QL *);

typedef enum {
    /** No consistency constraints */
    LCB_N1QL_CONSISTENCY_NONE = 0,

    /**
     * This is implicitly set by the lcb_n1p_synctok() family of functions. This
     * will ensure that mutations up to the vector indicated by the mutation token
     * passed to lcb_n1p_synctok() are used.
     */
    LCB_N1QL_CONSISTENCY_RYOW = 1,

    /** Refresh the snapshot for each request */
    LCB_N1QL_CONSISTENCY_REQUEST = 2,

    /** Refresh the snapshot for each statement */
    LCB_N1QL_CONSISTENCY_STATEMENT = 3
} lcb_N1QL_CONSISTENCY;

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_status(const lcb_RESPN1QL *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_cookie(const lcb_RESPN1QL *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_row(const lcb_RESPN1QL *resp, const char **row, size_t *row_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_http_response(const lcb_RESPN1QL *resp, const lcb_RESPHTTP **http);
LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_handle(const lcb_RESPN1QL *resp, lcb_N1QL_HANDLE **handle);
LIBCOUCHBASE_API int lcb_respn1ql_is_final(const lcb_RESPN1QL *resp);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_create(lcb_CMDN1QL **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_destroy(lcb_CMDN1QL *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_reset(lcb_CMDN1QL *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_payload(lcb_CMDN1QL *cmd, const char **payload, size_t *payload_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_parent_span(lcb_CMDN1QL *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_callback(lcb_CMDN1QL *cmd, lcb_N1QL_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_query(lcb_CMDN1QL *cmd, const char *query, size_t query_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_statement(lcb_CMDN1QL *cmd, const char *statement, size_t statement_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_named_param(lcb_CMDN1QL *cmd, const char *name, size_t name_len,
                                                    const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_positional_param(lcb_CMDN1QL *cmd, const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_adhoc(lcb_CMDN1QL *cmd, int adhoc);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_client_context_id(lcb_CMDN1QL *cmd, const char* value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pretty(lcb_CMDN1QL *cmd, int pretty);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_readonly(lcb_CMDN1QL *cmd, int readonly);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_scan_cap(lcb_CMDN1QL *cmd, int value);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pipeline_cap(lcb_CMDN1QL *cmd, int value);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pipeline_batch(lcb_CMDN1QL *cmd, int value);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency(lcb_CMDN1QL *cmd, lcb_N1QL_CONSISTENCY mode);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency_token_for_keyspace(lcb_CMDN1QL *cmd, const char *keyspace,
                                                                       size_t keyspace_len, lcb_MUTATION_TOKEN *token);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency_tokens(lcb_CMDN1QL *cmd, lcb_INSTANCE *instance);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_option(lcb_CMDN1QL *cmd, const char *name, size_t name_len, const char *value,
                                               size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_handle(lcb_CMDN1QL *cmd, lcb_N1QL_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_timeout(lcb_CMDN1QL *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_n1ql(lcb_INSTANCE *instance, void *cookie, const lcb_CMDN1QL *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_n1ql_cancel(lcb_INSTANCE *instance, lcb_N1QL_HANDLE *handle);
/** @} */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-view-api Views (Map-Reduce)
 * @brief Higher level API which splits view results into rows
 */

/**
 * @addtogroup lcb-view-api
 * @{
 */

typedef struct lcb_RESPVIEW_ lcb_RESPVIEW;
typedef struct lcb_CMDVIEW_ lcb_CMDVIEW;

/**
 * Pointer for request instance
 */
typedef struct lcb_VIEW_HANDLE_ lcb_VIEW_HANDLE;

/**
 * Callback function invoked for each row returned from the view
 * @param instance the library handle
 * @param cbtype the callback type. This is set to @ref LCB_CALLBACK_VIEWQUERY
 * @param row Information about the current row
 *
 * Note that this callback's `row->rflags` will contain the @ref LCB_RESP_F_FINAL
 * flag set after all rows have been returned. Applications should check for
 * the presence of this flag. If this flag is present, the row itself will
 * contain the raw response metadata in its lcb_RESPVIEWQUERY::value field.
 */
typedef void (*lcb_VIEW_CALLBACK)(lcb_INSTANCE *instance, int cbtype, const lcb_RESPVIEW *row);

LIBCOUCHBASE_API lcb_STATUS lcb_respview_status(const lcb_RESPVIEW *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_cookie(const lcb_RESPVIEW *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_key(const lcb_RESPVIEW *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_doc_id(const lcb_RESPVIEW *resp, const char **doc_id, size_t *doc_id_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_row(const lcb_RESPVIEW *resp, const char **row, size_t *row_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_document(const lcb_RESPVIEW *resp, const lcb_RESPGET **doc);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_http_response(const lcb_RESPVIEW *resp, const lcb_RESPHTTP **http);
LIBCOUCHBASE_API lcb_STATUS lcb_respview_handle(const lcb_RESPVIEW *resp, lcb_VIEW_HANDLE **handle);
LIBCOUCHBASE_API int lcb_respview_is_final(const lcb_RESPVIEW *resp);

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_create(lcb_CMDVIEW **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_destroy(lcb_CMDVIEW *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_parent_span(lcb_CMDVIEW *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_callback(lcb_CMDVIEW *cmd, lcb_VIEW_CALLBACK callback);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_design_document(lcb_CMDVIEW *cmd, const char *ddoc, size_t ddoc_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_view_name(lcb_CMDVIEW *cmd, const char *view, size_t view_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_option_string(lcb_CMDVIEW *cmd, const char *optstr, size_t optstr_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_post_data(lcb_CMDVIEW *cmd, const char *data, size_t data_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_include_docs(lcb_CMDVIEW *cmd, int include_docs);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_max_concurrent_docs(lcb_CMDVIEW *cmd, uint32_t num);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_no_row_parse(lcb_CMDVIEW *cmd, int flag);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_handle(lcb_CMDVIEW *cmd, lcb_VIEW_HANDLE **handle);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_timeout(lcb_CMDVIEW *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_view(lcb_INSTANCE *instance, void *cookie, const lcb_CMDVIEW *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_view_cancel(lcb_INSTANCE *instance, lcb_VIEW_HANDLE *handle);
/** @} */

/**@ingroup lcb-public-api
 * @defgroup lcb-subdoc Sub-Document API
 * @brief Experimental in-document API access
 * @details The sub-document API uses features from the upcoming Couchbase
 * 4.5 release which allows access to parts of the document. These parts are
 * called _sub-documents_ and can be accessed using the sub-document API
 *
 * @addtogroup lcb-subdoc
 * @{
 */

typedef struct lcb_RESPSUBDOC_ lcb_RESPSUBDOC;

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_status(const lcb_RESPSUBDOC *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_error_context(const lcb_RESPSUBDOC *resp, const char **ctx, size_t *ctx_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_error_ref(const lcb_RESPSUBDOC *resp, const char **ref, size_t *ref_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_cookie(const lcb_RESPSUBDOC *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_cas(const lcb_RESPSUBDOC *resp, uint64_t *cas);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_key(const lcb_RESPSUBDOC *resp, const char **key, size_t *key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_mutation_token(const lcb_RESPSUBDOC *resp, lcb_MUTATION_TOKEN *token);

LIBCOUCHBASE_API size_t lcb_respsubdoc_result_size(const lcb_RESPSUBDOC *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_result_status(const lcb_RESPSUBDOC *resp, size_t index);
LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_result_value(const lcb_RESPSUBDOC *resp, size_t index, const char **value,
                                                        size_t *value_len);

typedef struct lcb_SUBDOCOPS_ lcb_SUBDOCOPS;

/** Create intermediate paths */
#define LCB_SUBDOCOPS_F_MKINTERMEDIATES (1 << 16)

/** Access document XATTR path */
#define LCB_SUBDOCOPS_F_XATTRPATH (1 << 18)

/** Access document virtual/materialized path. Implies F_XATTRPATH */
#define LCB_SUBDOCOPS_F_XATTR_MACROVALUES (1 << 19)

/** Access Xattrs of deleted documents */
#define LCB_SUBDOCOPS_F_XATTR_DELETED_OK (1 << 20)

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_create(lcb_SUBDOCOPS **operations, size_t capacity);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_destroy(lcb_SUBDOCOPS *operations);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_get(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags, const char *path,
                                              size_t path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_exists(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                 const char *path, size_t path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_replace(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                  const char *path, size_t path_len, const char *value,
                                                  size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_dict_add(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                   const char *path, size_t path_len, const char *value,
                                                   size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_dict_upsert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                      const char *path, size_t path_len, const char *value,
                                                      size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_first(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                          const char *path, size_t path_len, const char *value,
                                                          size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_last(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                         const char *path, size_t path_len, const char *value,
                                                         size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_unique(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                           const char *path, size_t path_len, const char *value,
                                                           size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_insert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                       const char *path, size_t path_len, const char *value,
                                                       size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_counter(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                  const char *path, size_t path_len, int64_t delta);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_remove(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                 const char *path, size_t path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_get_count(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                    const char *path, size_t path_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_get(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_add(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                      const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_upsert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                         const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_replace(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                          const char *value, size_t value_len);
LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_remove(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags);

typedef struct lcb_CMDSUBDOC_ lcb_CMDSUBDOC;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_create(lcb_CMDSUBDOC **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_destroy(lcb_CMDSUBDOC *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_parent_span(lcb_CMDSUBDOC *cmd, lcbtrace_SPAN *span);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_collection(lcb_CMDSUBDOC *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_key(lcb_CMDSUBDOC *cmd, const char *key, size_t key_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_cas(lcb_CMDSUBDOC *cmd, uint64_t cas);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_operations(lcb_CMDSUBDOC *cmd, const lcb_SUBDOCOPS *operations);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_expiration(lcb_CMDSUBDOC *cmd, uint32_t expiration);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_durability(lcb_CMDSUBDOC *cmd, lcb_DURABILITY_LEVEL level);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_create_if_missing(lcb_CMDSUBDOC *cmd, int flag);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_timeout(lcb_CMDSUBDOC *cmd, uint32_t timeout);

LIBCOUCHBASE_API lcb_STATUS lcb_subdoc(lcb_INSTANCE *instance, void *cookie, const lcb_CMDSUBDOC *cmd);
/** @} */

/* Post-include some other headers */
#ifdef __cplusplus
}
#endif /* __cplusplus */
#endif /* LIBCOUCHBASE_COUCHBASE_H */
