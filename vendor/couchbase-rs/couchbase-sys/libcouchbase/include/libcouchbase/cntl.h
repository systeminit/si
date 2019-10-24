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

/**
 * @file
 * @brief Command codes for libcouchbase.
 *
 * @details
 * These codes may be passed to 'lcb_cntl'.
 *
 * Note that the constant values are also public API; thus allowing forwards
 * and backwards compatibility.
 */

#ifndef LCB_CNTL_H
#define LCB_CNTL_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @ingroup lcb-cntl
 * @defgroup lcb-cntl-settings Setting List
 * @brief Adjust tunables for the client
 * @details
 *
 * The constants in this file are used to control the behavior of the library.
 * All of the operations above may be passed as the `cmd` parameter to the
 * lcb_cntl() function, thus:
 *
 * @code{.c}
 * char something;
 * lcb_STATUS rv;
 * rv = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_FOO, &something);
 * @endcode
 *
 * will retrieve the setting of `LCB_CNTL_FOO` into `something`.
 *
 * You may also use the lcb_cntl_string() function, which operates on
 * strings and can set various configuration properties fairly simply. Note
 * however that string names are subject to change, and not all configuration
 * directives have a string alias:
 *
 * @code{.c}
 * rv = lcb_cntl_string("operation_timeout", "5.0");
 * @endcode
 *
 * Of the commands listed below, some will be read-only (i.e. you may only
 * _read_ the setting using the @ref LCB_CNTL_GET `mode`), some will be write-only
 * (i.e. you may only _modify_ the setting, and use @ref LCB_CNTL_SET for the `mode`)
 * and some will be both readable and writable.
 *
 * Along the documentation of each specific command, there is a table displaying
 * the modes supported and the expected pointer type to be passed as the `arg`
 * value into lcb_cntl(). Note that some read-write commands require different
 * pointer types depending on whether the `mode` is retrieval or storage.
 *
 *
 * @section lcb-time-info Timeout and Time Value Settings
 *
 * There are various settings on the library that control behavior with
 * respect to wall clock time.
 *
 * Timeout settings control how long the library will wait for a certain event
 * before proceeding to the next course of action (which may either be to try
 * a different operation or fail the current one, depending on the specific
 * timeout).
 *
 * Other settings may configure how often the library proactively polls for
 * a configuration update, retries various interally retried operations and
 * so forth.
 *
 * Time values are specified in _microseconds_ stored within an `lcb_U32`.
 *
 * When specified as an argument to lcb_cntl_string() or through the connection
 * string, it will be parsed from a string float value where the integer-part
 * is in seconds and the fractional-part is in fractions of a second.
 *
 * Note that timeouts in libcouchbase are implemented via an event loop
 * scheduler. As such their accuracy and promptness is limited by how
 * often the event loop is invoked and how much wall time is spent in
 * each of their handlers. Specifically if you issue long running blocking
 * calls within any of the handlers (and this means any of the library's
 * callbacks) then the timeout accuracy will be impacted.
 *
 * Further behavior is dependent on the event loop plugin itself and how
 * it schedules timeouts.
 *
 *
 * @par Configuration Stability Attributes
 * Configuration parameters are still subject to the API classification used
 * in @ref lcb_attributes. For _deprecated_ control commands, lcb_cntl() will
 * either perform the operation, _or_ consider it a no-op, _or_ return an error
 * code.
 */

/**
 * @addtogroup lcb-cntl-settings
 * @{
 */

/**
 * @name Modes
 * Modes for the lcb_cntl() `mode` argument
 * @{
 */
#define LCB_CNTL_SET 0x01 /**< @brief Modify a setting */
#define LCB_CNTL_GET 0x00 /**< @brief Retrieve a setting */
/**@}*/

/**
 * @brief Operation Timeout
 *
 * The operation timeout is the maximum amount of time the library will wait
 * for an operation to receive a response before invoking its callback with
 * a failure status.
 *
 * An operation may timeout if:
 *
 * * A server is taking too long to respond
 * * An updated cluster configuration has not been promptly received
 *
 * @code{.c}
 * lcb_U32 tmo = 3500000;
 * lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_OP_TIMEOUT, &tmo);
 * @endcode
 *
 * @cntl_arg_both{lcbU32*}
 * @committed
 * @see lcb-time-info
 */
#define LCB_CNTL_OP_TIMEOUT 0x00

/**
 * @brief Views Timeout
 * This is the I/O timeout for HTTP requests issues with LCB_HTTP_TYPE_VIEWS
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_VIEW_TIMEOUT 0x01

/**
 * @brief N1QL Timeout
 * This is the I/O timeout for N1QL queries, issued via lcb_n1ql_query()
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_N1QL_TIMEOUT 0x3D

/**
 * @brief Get the name of the bucket
 * This returns the name of the bucket this instance is connected to, or `NULL`
 * if not yet connected to a bucket
 *
 * @cntl_arg_getonly{`const char*`}
 * @committed
 */
#define LCB_CNTL_BUCKETNAME 0x30

/**
 * @brief Get the bucket type.
 * This returns the bucket type - which is either of the following:
 *
 *  * LCB_BTYPE_UNSPEC
 *  * LCB_BTYPE_COUCHBASE
 *  * LCB_BTYPE_EPHEMERAL
 *  * LCB_BTYPE_MEMCACHED
 *
 * @see https://developer.couchbase.com/documentation/server/current/architecture/core-data-access-buckets.html
 *
 * @cntl_arg_getonly{lcb_BTYPE*}
 */
#define LCB_CNTL_BUCKETTYPE 0x48

/**
 * @brief Get the handle type.
 * This returns the handle type - which is either LCB_TYPE_CLUSTER or
 * LCB_TYPE_BUCKET
 *
 * @cntl_arg_getonly{lcb_type_t*}
 */
#define LCB_CNTL_HANDLETYPE 0x04

/**@brief Get the vBucket handle.
 * Obtains the current cluster configuration from the client.
 *
 * @cntl_arg_getonly{lcbvb_CONFIG**}
 */
#define LCB_CNTL_VBCONFIG 0x05

/**@brief Get the iops implementation instance
 *
 * @cntl_arg_getonly{lcb_io_opt_t*}
 * @uncommitted
 */
#define LCB_CNTL_IOPS 0x06

/** @brief Structure containing mapping information for a key */
typedef struct lcb_cntl_vbinfo_st {
    int version;

    union {
        /** v0 */
        struct {
            const void *key;  /**< **Input** Key */
            lcb_SIZE nkey;    /**< **Input** Length of key */
            int vbucket;      /**< **Output** Mapped vBucket */
            int server_index; /**< **Output** Server index for vBucket */
        } v0;
    } v;
} lcb_cntl_vbinfo_t;

/**
 * @brief Get the vBucket ID for a given key, based on the current configuration
 *
 * @cntl_arg_getonly{lcb_cntl_vbinfo_t*}
 * @committed
 */
#define LCB_CNTL_VBMAP 0x07

/**
 * Modes for handling IPv6 in the IO layer.
 */
typedef enum {
    LCB_IPV6_DISABLED = 0x00, /**< disable IPv6 */
    LCB_IPV6_ONLY = 0x1,      /**< enforce only IPv6 */
    LCB_IPV6_ALLOW = 0x02     /**< use both IPv6 and IPv4 */
} lcb_ipv6_t;

/**
 * @brief IPv4/IPv6 selection policy
 *
 * Setting which controls whether hostname lookups should prefer IPv4 or IPv6
 *
 * Use `ipv6` in the connection string (e.g. "ipv6=allow" or "ipv6=only")
 *
 * @cntl_arg_both{lcb_ipv6_t*}
 * @committed
 */
#define LCB_CNTL_IP6POLICY 0x0b

/**
 * @brief Configuration error threshold.
 *
 * This number indicates how many
 * network/mapping/not-my-vbucket errors are received before a configuration
 * update is requested again.
 *
 * @cntl_arg_both{lcb_SIZE*}
 */
#define LCB_CNTL_CONFERRTHRESH 0x0c

/**
 * @brief Default timeout for lcb_durability_poll()
 * @ingroup lcb-time-info
 *
 * This is the time the client will
 * spend sending repeated probes to a given key's vBucket masters and replicas
 * before they are deemed not to have satisfied the durability requirements
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_DURABILITY_TIMEOUT 0x0d

/**
 * @brief Polling grace interval for lcb_durability_poll()
 *
 * This is the time the client will wait between repeated probes to
 * a given server.
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_DURABILITY_INTERVAL 0x0e

/**
 * @brief Timeout for otherwise unspecified HTTP requests
 *
 * Examples of these kinds of HTTP requests might be cluster management,
 * user management, etc.
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_HTTP_TIMEOUT 0x0f

/**
 * @brief Print verbose plugin load information to console
 *
 * This modifies a static, global setting regarding whether to
 * print verbose information when trying to dynamically load an IO plugin.
 * The information printed can be useful in determining why a plugin failed
 * to load. This setting can also be controlled via the
 * "LIBCOUCHBASE_DLOPEN_DEBUG" environment variable (and if enabled from the
 * environment, will override the setting mentioned here).
 *
 * @cntl_arg_both{int*}
 *
 * @note Pass NULL to lcb_cntl for the 'instance' parameter.
 * @volatile
 */
#define LCB_CNTL_IOPS_DLOPEN_DEBUG 0x11

/**@brief Initial bootstrap timeout.
 * This is how long the client will wait to obtain the initial configuration.
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed*/
#define LCB_CNTL_CONFIGURATION_TIMEOUT 0x12

/**
 * @brief Randomize order of bootstrap nodes.
 *
 * This controls whether the connection attempts for configuration retrievals
 * should be done in the supplied order or whether they should be randomized.
 *
 * For the initial connection the supplied order is the list of hosts provided
 * in the lcb_create_st structure. For subsequent connections this is the
 * order of nodes as received by the server.
 *
 * @cntl_arg_both{int*}
 * @committed
 */
#define LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS 0x14

/**
 * @brief Determine if file-based configuration has been loaded
 *
 * If the configuration cache is in use, the argument pointer
 * will be set to a true value. If the configuration cache was not used,
 * the argument pointer will be set to false.
 *
 * A false value may indicates that the client will need to load the
 * configuration from the network. This may be caused by the following:
 * - The configuration cache did not exist or was empty
 * - The configuration cache contained stale information
 *
 * @cntl_arg_getonly{int*}
 * @uncommitted
 */
#define LCB_CNTL_CONFIG_CACHE_LOADED 0x15

/**
 * @brief Force a specific SASL mechanism
 *
 * Force a specific SASL mechanism to use for authentication. This
 * can allow a user to ensure a certain level of security and have the
 * connection fail if the desired mechanism is not available.
 *
 * When setting this value, the arg parameter shall be a
 * `NUL`-terminated string or a `NULL` pointer (to unset). When retrieving
 * this value, the parameter shall be set to a `char **`. Note that this
 * value (in LCB_CNTL_GET) is valid only until the next call to a
 * libcouchbase API, after which it may have been freed.
 *
 * @cntl_arg_get_and_set{char**, char*}
 */
#define LCB_CNTL_FORCE_SASL_MECH 0x16

/**
 * @brief Maximum number of HTTP redirects to follow
 * Set how many redirects the library should follow for the single request.
 * Set to -1 to remove limit at all.
 *
 * @cntl_arg_both{int*}
 * @uncommitted
 */
#define LCB_CNTL_MAX_REDIRECTS 0x17

/**
 * @name Logging
 *
 * Verbose logging may be enabled by default using the environment variable
 * `LCB_LOGLEVEL` and setting it to a number > 1; higher values produce more
 * verbose output. The maximum level is `5`.
 *
 * You may also install your own logger using lcb_cntl() and the
 * @ref LCB_CNTL_LOGGER constant. Note that
 * the logger functions will not be called rapidly from within hot paths.
 * @{
 */

/**
 * @brief Logging Levels
 * @committed
 */
typedef enum {
    LCB_LOG_TRACE = 0, /**< the most verbose level */
    LCB_LOG_DEBUG,     /**< diagnostic information, required to investigate problems */
    LCB_LOG_INFO,      /**< useful notices, not often */
    LCB_LOG_WARN,      /**< error notifications */
    LCB_LOG_ERROR,     /**< error messages, usually the library have to re-initialize connection instance */
    LCB_LOG_FATAL,     /**< fatal errors, the library cannot proceed */
    LCB_LOG_MAX        /**< internal value for total number of levels */
} lcb_log_severity_t;

struct lcb_logprocs_st;

/**
 * @brief Logger callback
 *
 *
 * This callback is invoked for each logging message emitted
 * @param procs the logging structure provided
 * @param iid instance id
 * @param subsys a string describing the module which emitted the message
 * @param severity one of the LCB_LOG_* severity constants.
 * @param srcfile the source file which emitted this message
 * @param srcline the line of the file for the message
 * @param fmt a printf format string
 * @param ap a va_list for vprintf
 */
typedef void (*lcb_logging_callback)(struct lcb_logprocs_st *procs, unsigned int iid, const char *subsys, int severity,
                                     const char *srcfile, int srcline, const char *fmt, va_list ap);

/**
 * @brief Logging context
 * @volatile
 *
 * This structure defines the logging handlers. Currently there is only
 * a single field defined which is the default callback for the loggers.
 * This API may change.
 */
typedef struct lcb_logprocs_st {
    int version;
    union {
        struct {
            lcb_logging_callback callback;
        } v0;
    } v;
} lcb_logprocs;

/**
 * @brief Access the lcb_logprocs structure
 * @uncommitted
 *
 * The lcb_logoprocs structure passed must not be freed until the instance
 * is completely destroyed. This will only happen once the destruction
 * callback is called (see lcb_set_destroy_callback()).
 *
 * @cntl_arg_get_and_set{lcb_logprocs**,lcb_logprocs*}*/
#define LCB_CNTL_LOGGER 0x18

/**
 * Helper to express printf spec for sensitive data. Usage:
 *
 *   printf("Logged as " LCB_LOG_SPEC("%s") " user", LCB_LOG_UD(instance, doc->username));
 */
#define LCB_LOG_SPEC(fmt) "%s" fmt "%s"

#define LCB_LOG_UD_OTAG "<ud>"
#define LCB_LOG_UD_CTAG "</ud>"
/**
 * User data is data that is stored into Couchbase by the application user account.
 *
 * - Key and value pairs in JSON documents, or the key exclusively
 * - Application/Admin usernames that identify the human person
 * - Names and email addresses asked during product registration and alerting
 * - Usernames
 * - Document xattrs
 * - Query statements included in the log file collected by support that leak
 *   the document fields (Select floor_price from stock).
 */
#define LCB_LOG_UD(instance, val)                                                                                      \
    lcb_is_redacting_logs(instance) ? LCB_LOG_UD_OTAG : "", val, lcb_is_redacting_logs(instance) ? LCB_LOG_UD_CTAG : ""

#define LCB_LOG_MD_OTAG "<md>"
#define LCB_LOG_MD_CTAG "</md>"
/**
 * Metadata is logical data needed by Couchbase to store and process user data.
 *
 * - Cluster name
 * - Bucket names
 * - DDoc/view names
 * - View code
 * - Index names
 * - Mapreduce Design Doc Name and Definition (IP)
 * - XDCR Replication Stream Names
 * - And other couchbase resource specific meta data
 */
#define LCB_LOG_MD(instance, val)                                                                                      \
    lcb_is_redacting_logs(instance) ? LCB_LOG_MD_OTAG : "", val, lcb_is_redacting_logs(instance) ? LCB_LOG_MD_CTAG : ""

#define LCB_LOG_SD_OTAG "<sd>"
#define LCB_LOG_SD_CTAG "</sd>"
/**
 * System data is data from other parts of the system Couchbase interacts with over the network.
 *
 * - IP addresses
 * - IP tables
 * - Hosts names
 * - Ports
 * - DNS topology
 */
#define LCB_LOG_SD(instance, val)                                                                                      \
    lcb_is_redacting_logs(instance) ? LCB_LOG_SD_OTAG : "", val, lcb_is_redacting_logs(instance) ? LCB_LOG_SD_CTAG : ""
/**@}*/

/**
 * @brief Refresh Throttling
 *
 * Modify the amount of time (in microseconds) before the
 * @ref LCB_CNTL_CONFERRTHRESH will forcefully be set to its maximum
 * number forcing a configuration refresh.
 *
 * Note that if you expect a high number of timeouts in your operations, you
 * should set this to a high number (along with `CONFERRTHRESH`). If you
 * are using the default timeout setting, then this value is likely optimal.
 *
 * @cntl_arg_both{lcb_U32*}
 * @see LCB_CNTL_CONFERRTHRESH
 */
#define LCB_CNTL_CONFDELAY_THRESH 0x19

/**
 * @brief Get the transport used to fetch cluster configuration.
 * @cntl_arg_getonly{lcb_config_transport_t*}
 * @uncommitted
 */
#define LCB_CNTL_CONFIG_TRANSPORT 0x1A

/**
 * @brief Per-node configuration timeout.
 *
 * The per-node configuration timeout sets the amount of time to wait
 * for each node within the bootstrap/configuration process. This interval
 * is a subset of the @ref LCB_CNTL_CONFIGURATION_TIMEOUT
 * option mentioned above and is intended
 * to ensure that the bootstrap process does not wait too long for a given
 * node. Nodes that are physically offline may never respond and it may take
 * a long time until they are detected as being offline.
 * See CCBC-261 and CCBC-313 for more reasons.
 *
 * @note the `CONFIGURATION_TIMEOUT` should be higher than this number.
 * No check is made to ensure that this is the case, however.
 *
 * @cntl_arg_both{lcb_U32*}
 * @see LCB_CNTL_CONFIGURATION_TIMEOUT
 * @committed
 */
#define LCB_CNTL_CONFIG_NODE_TIMEOUT 0x1B

/**
 * @brief Idling/Persistence for HTTP bootstrap
 *
 * By default the behavior of the library for HTTP bootstrap is to keep the
 * stream open at all times (opening a new stream on a different host if the
 * existing one is broken) in order to proactively receive configuration
 * updates.
 *
 * The default value for this setting is -1. Changing this to another number
 * invokes the following semantics:
 *
 * - The configuration stream is not kept alive indefinitely. It is kept open
 *   for the number of seconds specified in this setting. The socket is closed
 *   after a period of inactivity (indicated by this setting).
 *
 * - If the stream is broken (and no current refresh was requested by the
 *   client) then a new stream is not opened.
 *
 * @cntl_arg_both{lcb_U32*}
 * @volatile
 */
#define LCB_CNTL_HTCONFIG_IDLE_TIMEOUT 0x1C

/**
 * @brief Get the current SCM changeset for the library binary
 * @cntl_arg_getonly{char**}
 */
#define LCB_CNTL_CHANGESET 0x1F

/**
 * @brief File used for the configuration cache.
 *
 * The configuration
 * cache allows to bootstrap from a cluster without using the initial
 * bootstrap connection, considerably reducing latency. If the file passed
 * does not exist, the normal bootstrap process is performed and the file
 * is written to with the current information.  File will be updated as
 * the configuration in the cluster changes.  Multiple instances may race
 * to update the file, and that is the intended behavior.
 *
 * @note The leading directories for the file must exist, otherwise the file
 * will never be created.
 *
 * @note Configuration cache is not supported for memcached buckets
 * @cntl_arg_get_and_set{char**, char*}
 * @uncommitted
 * @see LCB_CNTL_CONFIG_CACHE_LOADED
 */
#define LCB_CNTL_CONFIGCACHE 0x21

/**
 * @brief File used for read-only configuration cache
 *
 * This is identical to the @ref LCB_CNTL_CONFIGCACHE directive, except that
 * it guarantees that the library will never overwrite or otherwise modify
 * the path specified.
 *
 * @see LCB_CNTL_CONFIGCACHE
 */
#define LCB_CNTL_CONFIGCACHE_RO 0x36

/**
 * SSL options
 *
 * @committed
 */
typedef enum {
    LCB_SSL_ENABLED = 1 << 0,     /**< Use SSL */
    LCB_SSL_NOVERIFY = 1 << 1,    /**< Don't verify certificates */
    LCB_SSL_NOGLOBALINIT = 1 << 2 /**< Do not call SSL's global init functions */
} lcb_SSLOPTS;

/**
 * @brief Get SSL Mode
 *
 * Retrieve the SSL mode currently in use by the library. This is a read-only
 * setting. To set the SSL mode at the library, specify the appropriate values
 * within the connection string. See @ref lcb_create_st3 for details.
 *
 * @cntl_arg_getonly{`int*` (value is one of @ref lcb_SSLOPTS)}
 * @committed
 */
#define LCB_CNTL_SSL_MODE 0x22

/**
 * @brief Get SSL Certificate path
 *
 * Retrieve the path to the CA certificate (if any) being used.
 *
 * @cntl_arg_getonly{`char**`}
 * @see LCB_CNTL_SSL_MODE
 * @committed
 */
#define LCB_CNTL_SSL_CERT 0x23

/**
 * @brief Get SSL private key path
 *
 * Retrieve the path to the private key (if any) being used.
 * When key specified, the library will use it to authenticate on the services,
 * skipping all other authentication mechanisms (SASL, HTTP Basic auth, etc)
 *
 * @cntl_arg_getonly{`char**`}
 * @see LCB_CNTL_SSL_MODE
 * @see https://developer.couchbase.com/documentation/server/5.0/security/security-certs-auth.html
 * @committed
 */
#define LCB_CNTL_SSL_KEY 0x4b

/**
 * @brief Get SSL trust store path
 *
 * Trust store might be NULL, in this case the library expects it to be concatenated with certificate.
 *
 * @cntl_arg_getonly{`char**`}
 * @see LCB_CNTL_SSL_MODE
 * @see https://developer.couchbase.com/documentation/server/5.0/security/security-certs-auth.html
 * @committed
 */
#define LCB_CNTL_SSL_TRUSTSTORE 0x4d

/**
 * Alias for @ref LCB_CNTL_SSL_CERT for backward compatibility.
 * @deprecated
 */
#define LCB_CNTL_SSL_CACERT LCB_CNTL_SSL_CERT

/**
 * @brief Select retry mode to manipulate
 */
typedef enum {
    LCB_RETRY_ON_TOPOCHANGE = 0, /**< Select retry for topology */
    LCB_RETRY_ON_SOCKERR,        /**< Select retry for network errors */
    LCB_RETRY_ON_VBMAPERR,       /**< Select retry for NOT_MY_VBUCKET responses */

    /** Retry when there is no node for the item. This case is special as the
     * `cmd` setting is treated as a boolean rather than a bitmask*/
    LCB_RETRY_ON_MISSINGNODE,
    LCB_RETRY_ON_MAX /**<< maximum index */
} lcb_RETRYMODEOPTS;

typedef enum {
    /**Don't retry any commands. A command which has been forwarded to
     * a server and a not-my-vbucket has been received in response for it
     * will result in a failure.*/
    LCB_RETRY_CMDS_NONE = 0,

    /**Only retry simple retrieval operations (excludes touch,
     * get-and-touch, and get-locked) which may be retried many numbers of times
     * without risking unintended data manipulation. */
    LCB_RETRY_CMDS_GET = 0x01,

    /**Retry operations which may potentially fail because they have been
     * accepted by a previous server, but will not silently corrupt data.
     * Such commands include mutation operations containing a CAS.*/
    LCB_RETRY_CMDS_SAFE = 0x03, /* Includes 'GET', plus a new flag (e.g. 0x02|0x01) */

    /**Retry all commands, disregarding any potential unintended receipt of
     * errors or data mutation.*/
    LCB_RETRY_CMDS_ALL = 0x07 /* e.g. 0x01|0x03| NEW FLAG: 0x04 */
} lcb_RETRYCMDOPTS;

/**@brief Create a retry setting value
 * @param mode the mode to set (@see lcb_RETRYMODEOPTS)
 * @param policy the policy determining which commands should be retried
 * (@see lcb_RETRYCMDOPTS)
 * @return a value which can be assigned to an `lcb_U32` and passed to
 * the @ref LCB_CNTL_RETRYMODE setting
 */
#define LCB_RETRYOPT_CREATE(mode, policy) (((mode) << 16) | policy)

/** Get mode from retry setting value */
#define LCB_RETRYOPT_GETMODE(u) ((u) >> 16)
/** Get policy from retry setting value */
#define LCB_RETRYOPT_GETPOLICY(u) ((u)&0xffff)

/**
 * @volatile
 *
 * @brief Set retry policies
 *
 * This function sets the retry behavior. The retry behavior is the action the
 * library should take when a command has failed because of a failure which
 * may be a result of environmental and/or topology issues. In such cases it
 * may be possible to retry the command internally and have it succeed a second
 * time without propagating an error back to the application.
 *
 * The behavior consists of a _mode_ and _command_ selectors. The _command_
 * selector indicates which commands should be retried (and which should be
 * propagated up to the user) whereas the _mode_ indicates under which
 * circumstances should the _command_ policy be used.
 *
 * Disable retries anywhere:
 * @code{.c}
 * for (int ii = 0; ii < LCB_RETRY_ON_MAX; ++ii) {
 *   lcb_U32 val = LCB_RETRYOPT_CREATE(ii, LCB_RETRY_CMDS_NONE);
 *   lcb_STATUS err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_RETRYMODE, &val);
 * }
 * @endcode
 *
 * Only retry simple GET operations when retry is needed because of topology
 * changes:
 * @code{.c}
 * lcb_U32 val = LCB_RETRYOPT_CREATE(LCB_RETRY_ON_TOPOCHANGE, LCB_RETRY_CMDS_GET);
 * lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_RETRYMODE, &val);
 * @endcode
 *
 * Determine the behavior of the library when a `NOT_MY_VBUCKET` is received:
 * @code{.c}
 * lcb_U32 val = LCB_RETRYOPT_CREATE(LCB_RETRY_ON_VBMAPERR, 0);
 * lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_RETRYMODE, &val);
 * lcb_U32 policy = LCB_RETRYOPT_GETPOLICY(val);
 * @endcode
 *
 * @cntl_arg_both{`lcb_U32 *`}
 */
#define LCB_CNTL_RETRYMODE 0x24

/**
 * @brief Enumeration representing various URL forms to use for the configuration
 * stream */
typedef enum {
    /** `/pools/default/b[s]/$bucket`: Introduced in Couchbase Server 2.5 */
    LCB_HTCONFIG_URLTYPE_25PLUS = 0x01,

    /** `/pools/default/buckets[Streaming]/$bucket`. */
    LCB_HTCONFIG_URLTYPE_COMPAT = 0x02,

    /** Try `25PLUS` first and fallback to `COMPAT` */
    LCB_HTCONFIG_URLTYPE_TRYALL = 0x03
} lcb_HTCONFIG_URLTYPE;

/**
 * @brief Set the URL selection mode.
 *
 * The URL type can be a mask of the #lcb_HTCONFIG_URLTYPE constants which
 * indicate which URLs the HTTP provider should use.
 *
 * The default is to use the `25PLUS` URI first, and fallback on the compat uri
 * if the terse one fails with an HTTP 404 (Not Found). The new-style URI is
 * considered more efficient on cluster resources and can help the cluster
 * maintain many more streaming connections than the compat version, however
 * it is only available in Couchbase Server 2.5 and greater.
 *
 * This setting is only used when CCCP is disabled. This will typically be for
 * older clusters or for memcached buckets.
 * @cntl_arg_both{`int*` (value is one of @ref lcb_HTCONFIG_URLTYPE)}

 * @volatile Primarily here to support tests and buggy HTTP servers/proxies
 * which do not like to maintain a connection upon receipt of a 404.
 */
#define LCB_CNTL_HTCONFIG_URLTYPE 0x25

/**
 * Determines whether to run the event loop internally within lcb_destroy()
 * until no more I/O resources remain for the library. This is usually only
 * necessary if you are creating a lot of instances and/or are using memory
 * leak analysis tools.
 *
 * @cntl_arg_both{`int*` (as a boolean)}
 * @see lcb_destroy_async() and lcb_set_destroy_callback()
 * @volatile
 */
#define LCB_CNTL_SYNCDESTROY 0x28

/**
 * Sets the logging level for the console logger. If a logger is already
 * initialized (either from the environment, or via lcb_cntl_logger() then
 * this operation does nothing.
 *
 * This is mainly useful for applications which want to proxy the built in
 * logging options via command line options and the like, rather than setting
 * it from the environment.
 *
 * The argument passed to lcb_cntl() is an integer of 0 until
 * `LCB_LOG_MAX`, though the actual type is of `lcb_U32` rather than
 * an enum type #lcb_log_severity_t.
 *
 * @cntl_arg_setonly{const lcb_U32 *}
 * @see LCB_CNTL_LOGGER
 */
#define LCB_CNTL_CONLOGGER_LEVEL 0x29

/**
 *
 * Sets the output file (as a `FILE*`) for the console logger. Note that
 * any existing file pointer will be cleared (but not `fclose()`d.
 *
 * If used with lcb_cntl_string(), (using the `console_log_file` parameter),
 * the third argument is taken as the _name_ of a file. Note that the user
 * is responsible for closing the file.
 *
 * This setting does not require a library handle and therefore the first
 * argument to lcb_cntl() should be `NULL`.
 *
 *
 * @cntl_arg_get_and_set{`FILE**`, `FILE*`}
 * @see LCB_CNTL_LOGGER
 * @see LCB_CNTL_CONLOGGER_LEVEL
 */
#define LCB_CNTL_CONLOGGER_FP 0x3B

/**
 * Sets the behavior for reporting network errors. By default network errors
 * are returned as `LCB_NETWORK_ERROR` return codes for compatibility reasons.
 * More detailed error codes may be available by enabling this option which will
 * return appropriate error codes which have a category of
 * @ref LCB_ERRTYPE_NETWORK
 *
 * Using this option means your programming model is centered around the various
 * LCB_EIF* macros (see <libcouchbase/error.h>) rather than individual codes.
 *
 * @cntl_arg_both{int * (As a boolean)}
 */
#define LCB_CNTL_DETAILED_ERRCODES 0x2A

/**
 *
 * Sets the interval at which the retry queue will attempt to resend a failed
 * operation. When an operation fails and the retry policy (see
 * @ref LCB_CNTL_RETRYMODE) allows the operation to be retried, it shall be
 * placed into a queue, and then be retried within a given interval.
 *
 * Setting a high value will be friendlier on the network but also potentially
 * increase latency, while setting this to a low value may cause unnecessary
 * network traffic for operations which are not yet ready to be retried.
 *
 * @cntl_arg_both{lcb_U32* (microseconds)}
 */
#define LCB_CNTL_RETRY_INTERVAL 0x2C

/**
 * Whether commands are retried immediately upon receipt of not-my-vbucket
 * replies.
 *
 * Since version 2.4.8, packets by default are retried immediately on a
 * different node if it had previously failed with a not-my-vbucket
 * response, and is thus not subject to the @ref LCB_CNTL_RETRY_INTERVAL
 * setting. Disabling this setting will restore the older behavior.
 * This may be used in case there are problems with the default
 * heuristic/retry algorithm.
 *
 * @volatile
 */
#define LCB_CNTL_RETRY_NMV_IMM 0x37

/**
 * Set the maximum pool size for pooled http (view request) sockets. This should
 * be set to 1 (the default) unless you plan to execute concurrent view requests.
 * You may set this to 0 to disable pooling
 *
 * @cntl_arg_both{lcb_SIZE}
 * @volatile
 */
#define LCB_CNTL_HTTP_POOLSIZE 0x2E

/**
 * Determine whether or not a new configuration should be received when an error
 * is received over the HTTP API (i.e. via lcb_make_http_request().
 *
 * The default value is true, however you may wish to disable this if you are
 * expectedly issuing a lot of requests which may result in an error.
 *
 * @cntl_arg_both{int (as boolean)}
 * @uncommitted
 */
#define LCB_CNTL_HTTP_REFRESH_CONFIG_ON_ERROR 0x2F

/**
 * Set the behavior of the lcb_sched_leave() API call. By default the
 * lcb_sched_leave() will also set up the necessary requirements for flushing
 * to the network. If this option is off then an explicit call to
 * lcb_sched_flush() must be performed instead.
 *
 * @cntl_arg_both{int (as boolean)}
 * @volatile
 */
#define LCB_CNTL_SCHED_IMPLICIT_FLUSH 0x31

/**
 *
 * Request the server to return an additional 16 bytes of data for each
 * mutation operation. This extra information may help with more reliable
 * durability polling, but will also increase the size of the response packet.
 *
 * This should be set on the instance before issuing lcb_connect(). While this
 * may also be set after lcb_connect() is called, it will currently only take
 * effect when a server reconnects (which itself may be undefined).
 *
 * @cntl_arg_both{int (as boolean)}
 */
#define LCB_CNTL_FETCH_MUTATION_TOKENS 0x34

/**
 * This setting determines whether the lcb_durability_poll() function will
 * transparently attempt to use mutation token functionality (rather than checking
 * the CAS). This option is most useful for older code which does
 * explicitly use mutation tokens but would like to use its benefits when
 * ensuring durability constraints are satisfied.
 *
 * This option is enabled by default. Users may wish to disable this if they
 * are performing durability operations against items stored from different
 * client instances, as this will make use of a client-global state which is
 * derived on a per-vBucket basis. This means that the last mutation performed
 * on a given vBucket for the client will be used, which in some cases may be
 * older or newer than the mutations passed to the lcb_durability_poll()
 * function.
 *
 * @cntl_arg_both{int (as boolean)}
 * @volatile
 */
#define LCB_CNTL_DURABILITY_MUTATION_TOKENS 0x35

/**
 * This read-only property determines if the mutation token mechanism is supported
 * on the cluster itself. This will only be accurate once a single operation
 * has been performed on the cluster - or in other words, once a connection
 * to a data node has been established for the purposes of normal operations.
 *
 * @cntl_arg_getonly{int (as boolean)}
 * @uncommitted
 */
#define LCB_CNTL_MUTATION_TOKENS_SUPPORTED 0x38

/**
 * This setting determines if calls to lcb_wait() and lcb_wait3() will reset
 * the timeout of pending operations to the time that lcb_wait() was called,
 * rather than having the operation maintain the time of the call which
 * scheduled it. If the time between lcb_store3() and family and the lcb_wait()
 * functions is long, it is recommended to disable this setting in order to
 * avoid prematurely having operations time out.
 *
 * @cntl_arg_both{int (as boolean)}
 * @uncommitted
 *
 * Use `"readj_wait_tmo"` for the string version
 */
#define LCB_CNTL_RESET_TIMEOUT_ON_WAIT 0x3A

/**
 * Clears the internal prepared statement cache for N1QL
 *
 * This does not take any arguments, and is valid only on @ref LCB_CNTL_SET
 * @uncommitted
 */
#define LCB_CNTL_N1QL_CLEARACHE 0x3E

/**
 * Sets additional text for negotiation. This allows wrappers or applications
 * to add additional identifying information which can then be seen in the
 * server logs.
 *
 * @cntl_arg_get_and_set{`const char**`, `const char*`}
 *
 * Use `"client_string"` for the string version
 */
#define LCB_CNTL_CLIENT_STRING 0x3F

typedef const char *lcb_BUCKETCRED[2];

/**
 * Set credentials for a bucket. This is used for N1QL and CBFT APIs to allow
 * access to multiple buckets. It can also be used to set the password
 * of the current bucket when reconnecting (in case it changes).
 *
 * The format for the credentials is an array of two nul-terminated strings,
 * the first refers to the bucket and the second refers to the password.
 */
#define LCB_CNTL_BUCKET_CRED 0x40

/**
 * Set the amount of time the client should wait before retrying a
 * not-my-vbucket response packet. The default is 100ms. The value should
 * be specified in microseconds.
 *
 * Use `"retry_nmv_interval"` with lcb_cntl_string()
 *
 * @cntl_arg_both{lcb_U32*}
 */
#define LCB_CNTL_RETRY_NMV_INTERVAL 0x41

/**
 * Limit the number of bytes to be read (and thereby processed) during I/O
 * read operations. This setting may be useful when the network is faster than
 * processing resources.
 *
 * @note This setting only works for event-style I/O plugins. This means it
 * has no effect on completion style plugins such as libuv or Windows IOCP
 *
 * @cntl_arg_both{lcb_U32*}
 */
#define LCB_CNTL_READ_CHUNKSIZE 0x42

/**
 * Enable/Disable the Error Map feature. This is disabled by default.
 * Works only on servers which support error map
 *
 * Use `enable_errmap` in the connection string
 *
 * @cntl_arg_both{int* (as boolean)}
 */
#define LCB_CNTL_ENABLE_ERRMAP 0x43

/**
 * Enable/Disable sending the SELECT_BUCKET command after authentication.
 * This is useful to test auth, and should not be set by end-users.
 *
 * Note that even if this feature is enabled (the default), the client will
 * only send `SELECT_BUCKET` if the server indicates that it is supported
 * during negotiation.
 *
 * Use `select_bucket` in the connection string
 *
 * @cntl_arg_both{int* (as boolean)}
 */
#define LCB_CNTL_SELECT_BUCKET 0x44

/**
 * Enable/Disable setting the `TCP_KEEPALIVE` option on created sockets.
 * This is enabled by default for I/O backends which support it.
 *
 * The keepalive interval will be set to the operating system default.
 *
 * @cntl_arg_both{int* (as boolean)}
 */
#define LCB_CNTL_TCP_KEEPALIVE 0x45

/**
 * Set the amount of time to wait in between polling for a new configuration.
 * This will have no effect if connected to a Memcached buckets, or using
 * HTTP or File-based configurations (see the `bootstrap_on` connection
 * string option).
 *
 * This option facilitates 'fast failover' - in that the client can preemptively
 * check for any cluster topology updates before encountering an error.
 *
 * @cntl_arg_both{lcb_U32*}
 *
 * The value for this option is a time value. See the top of this header
 * in respect to how to specify this.
 *
 * Using a value of `0` disables this feature.
 *
 * You can also use `config_poll_interval` in the connection string.
 *
 * @note
 * Background polling is implemented in the library's non-blocking event loop.
 * Synchronous clients (i.e. those using `lcb_wait()`) will only be able to
 * poll as often as the library's event loop is active. If the library is
 * suspended, that is, if not inside an `lcb_wait()` call, the library will
 * be unable to do any kind of background polling.
 */
#define LCB_CNTL_CONFIG_POLL_INTERVAL 0x46

/**
 * From version 2.7.4, the C library sends a HELLO command before
 * authentication. This works on all modern server versions, but may cause
 * disconnects on more ancient variants (Couchbase 2.x for example).
 *
 * This setting will disable the sending of the HELLO command (which older
 * servers don't understand anyway). To disable the sending of hello, set this
 * value to false.
 *
 * @cntl_arg_both{int* (as boolean)}
 * @committed
 *
 * You can also use `send_hello=false` in the connection string.
 */
#define LCB_CNTL_SEND_HELLO 0x47

/**
 * Once redaction is enabled, anything at ERROR, WARN and INFO will wrap
 * sensitive information with special tags, for further processing with the goal
 * to remove or encrypt that information.  DEBUG or TRACE level logging are
 * expected to have specific info.
 *
 * Use `log_redaction` in the connection string
 *
 * @cntl_arg_both{int* (as boolean)}
 * @committed
 */
#define LCB_CNTL_LOG_REDACTION 0x4c

/**
 * Activate/deactivate end-to-end tracing.
 *
 * Use `enable_tracing` in the connection string
 *
 * @cntl_arg_both{int* (as boolean)}
 * @see lcb-tracing-api
 * @committed
 */
#define LCB_CNTL_ENABLE_TRACING 0x4e

typedef enum {
    LCBTRACE_THRESHOLD_KV = 0,
    LCBTRACE_THRESHOLD_N1QL,
    LCBTRACE_THRESHOLD_VIEW,
    LCBTRACE_THRESHOLD_FTS,
    LCBTRACE_THRESHOLD_ANALYTICS,
    LCBTRACE_THRESHOLD__MAX
} lcbtrace_THRESHOLDOPTS;

/**
 * Flush interval for orphaned spans queue in default tracer.
 *
 * This is the time the tracer will wait between repeated attempts
 * to flush most recent orphaned spans.
 *
 * Use `tracing_orphaned_queue_flush_interval` in the connection string
 *
 * @code{.c}
 * lcb_U32 tmo = 10000000; // 10 seconds in microseconds
 * lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_TRACING_ORPHANED_QUEUE_FLUSH_INTERVAL, &tmo);
 * @endcode
 *
 * @code{.c}
 * rv = lcb_cntl_string("tracing_orphaned_queue_flush_interval", "10.0");
 * @endcode
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_ORPHANED_QUEUE_FLUSH_INTERVAL 0x4f

/**
 * Size of orphaned spans queue in default tracer.
 *
 * Queues in default tracer has fixed size, and it will remove information about older spans,
 * when the limit will be reached before flushing time.
 *
 * Use `tracing_orphaned_queue_size` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_ORPHANED_QUEUE_SIZE 0x50

/**
 * Flush interval for spans with total time over threshold in default tracer.
 *
 * This is the time the tracer will wait between repeated attempts
 * to flush threshold queue.
 *
 * Use `tracing_threshold_queue_flush_interval` in the connection string
 *
 * @code{.c}
 * lcb_U32 tmo = 10000000; // 10 seconds in microseconds
 * lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_TRACING_THRESHOLD_QUEUE_FLUSH_INTERVAL, &tmo);
 * @endcode
 *
 * @code{.c}
 * rv = lcb_cntl_string("tracing_threshold_queue_flush_interval", "10.0");
 * @endcode
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_QUEUE_FLUSH_INTERVAL 0x51

/**
 * Size of threshold queue in default tracer.
 *
 * Queues in default tracer has fixed size, and it will remove information about older spans,
 * when the limit will be reached before flushing time.
 *
 * Use `tracing_threshold_queue_size` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_QUEUE_SIZE 0x52

/**
 * Minimum time for the tracing span of KV service to be considered by threshold tracer.
 *
 * Use `tracing_threshold_kv` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_KV 0x53

/**
 * Minimum time for the tracing span of N1QL service to be considered by threshold tracer.
 *
 * Use `tracing_threshold_n1ql` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_N1QL 0x54

/**
 * Minimum time for the tracing span of VIEW service to be considered by threshold tracer.
 *
 * Use `tracing_threshold_view` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_VIEW 0x55

/**
 * Minimum time for the tracing span of FTS service to be considered by threshold tracer.
 *
 * Use `tracing_threshold_fts` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_FTS 0x56

/**
 * Minimum time for the tracing span of ANALYTICS service to be considered by threshold tracer.
 *
 * Use `tracing_threshold_analytics` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_TRACING_THRESHOLD_ANALYTICS 0x57

/**
 * Options for how to handle compression
 *
 * @committed
 */
typedef enum {
    /** Do not perform compression in any direction. Data which is received
     * compressed via the server will be indicated as such by having the
     * `LCB_VALUE_F_SNAPPYCOMP` flag set in the lcb_GETRESPv0::datatype field */
    LCB_COMPRESS_NONE = 0x00,

    /**
     * Decompress incoming data, if the data has been compressed at the server.
     * If this is set, the `datatype` field in responses will always be stripped
     * of the `LCB_VALUE_F_SNAPPYCOMP` flag.
     */
    LCB_COMPRESS_IN = 1 << 0,

    /**
     * Compress outgoing data. Note that if the `datatype` field contains the
     * `LCB_VALUE_F_SNAPPYCOMP` flag, then the data will never be compressed
     * as it is assumed that it is already compressed.
     */
    LCB_COMPRESS_OUT = 1 << 1,

    LCB_COMPRESS_INOUT = (LCB_COMPRESS_IN | LCB_COMPRESS_OUT),

    /**
     * By default the library will send a HELLO command to the server to
     * determine whether compression is supported or not. Because commands may
     * be pipelined prior to the scheduing of the HELLO command it is possible
     * that the first few commands may not be compressed when schedule due to
     * the library not yet having negotiated settings with the server. Setting
     * this flag will force the client to assume that all servers support
     * compression despite a HELLO not having been intially negotiated.
     */
    LCB_COMPRESS_FORCE = 1 << 2
} lcb_COMPRESSOPTS;

/**
 * @brief Control how the library handles compression and deflation to and from
 * the server.
 *
 * Starting in Couchbase Server 3.0, compression can optionally be applied to
 * incoming and outcoming data. For incoming (i.e. `GET` requests) the data
 * may be received in compressed format and then allow the client to inflate
 * the data upon receipt. For outgoing (i.e. `SET` requests) the data may be
 * compressed on the client side and then be stored and recognized on the
 * server itself.
 *
 * The default behavior is to transparently handle compression for both incoming
 * and outgoing data.
 *
 * Note that if the lcb_STORECMDv0::datatype field is set with compression
 * flags, the data will _never_ be compressed by the library as this is an
 * indication that it is _already_ compressed.
 *
 * @cntl_arg_both{`int*` (value is one of @ref lcb_COMPRESSOPTS)}
 * @committed
 */
#define LCB_CNTL_COMPRESSION_OPTS 0x26

/**
 * Minimum size of the document payload to be compressed when compression enabled.
 *
 * Use `compression_min_size` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_COMPRESSION_MIN_SIZE 0x58

/**
 * Minimum compression ratio (compressed / original) of the compressed payload to allow sending it to cluster.
 *
 * Use `compression_min_ratio` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_COMPRESSION_MIN_RATIO 0x59

/**
 * Select type of network (alternative addresses).
 *
 * Use `network` in the connection string
 *
 * @cntl_arg_get_and_set{`const char**`, `const char*`}
 *
 * @uncommitted
 */
#define LCB_CNTL_NETWORK 0x5b

/**
 * The amount of time the pool should wait before closing idle connections.
 *
 * Use `http_pool_timeout` in the connection string
 *
 * @cntl_arg_both{lcb_U32*}
 * @committed
 */
#define LCB_CNTL_HTTP_POOL_TIMEOUT 0x5d

/**
 *
 * @cntl_arg_both{int (as boolean)}
 * @volatile
 */
#define LCB_CNTL_ENABLE_COLLECTIONS 0x4a

/**
 *
 * @cntl_arg_both{int (as boolean)}
 * @volatile
 */
#define LCB_CNTL_ENABLE_DURABLE_WRITE 0x5e

/**
 *
 * @volatile
 */
#define LCB_CNTL_PERSISTENCE_TIMEOUT_FLOOR 0x5f

/**
 *
 * @volatile
 */
#define LCB_CNTL_ALLOW_STATIC_CONFIG 0x60

/**
 * This is not a command, but rather an indicator of the last item.
 * @internal
 */
#define LCB_CNTL__MAX 0x61
/**@}*/

#ifdef __cplusplus
}
#endif

#include "cntl-private.h"

#endif /* LCB_CNTL_H */
