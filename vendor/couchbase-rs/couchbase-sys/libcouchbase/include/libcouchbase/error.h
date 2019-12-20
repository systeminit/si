/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
 * @brief
 * Definition of all of the error codes used by libcouchbase
 */
#ifndef LIBCOUCHBASE_ERROR_H
#define LIBCOUCHBASE_ERROR_H 1

#ifndef LIBCOUCHBASE_COUCHBASE_H
#error "Include libcouchbase/couchbase.h instead"
#endif

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-error-codes Error Codes
 * @brief Status codes returned by the library
 *
 * @addtogroup lcb-error-codes
 * @{
 */
#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Error Categories
 *
 * These error categories are assigned as a series of OR'd bits to each
 * of the error codes in lcb_STATUS.
 *
 * @see lcb_get_errtype
 */
typedef enum {
    /** Error type indicating a likely issue in user input */
    LCB_ERRTYPE_INPUT = 1 << 0,

    /** Error type indicating a likely network failure */
    LCB_ERRTYPE_NETWORK = 1 << 1,

    /** Error type indicating a fatal condition within the server or library */
    LCB_ERRTYPE_FATAL = 1 << 2,

    /** Error type indicating a transient condition within the server */
    LCB_ERRTYPE_TRANSIENT = 1 << 3,

    /** Error type indicating a negative server reply for the data */
    LCB_ERRTYPE_DATAOP = 1 << 4,

    /** Error codes which should never be visible to the user */
    LCB_ERRTYPE_INTERNAL = 1 << 5,

    /** Error code indicating a plugin failure */
    LCB_ERRTYPE_PLUGIN = 1 << 6,

    /** Error code indicating the server is under load */
    LCB_ERRTYPE_SRVLOAD = 1 << 7,

    /** Error code indicating the server generated this message */
    LCB_ERRTYPE_SRVGEN = 1 << 8,

    /**
     * Error code indicates document (fulldoc) access ok, but
     * error in performing subdocument operation. Note that this only
     * covers errors which relate to a specific operation, rather than
     * operations which prevent _any_ subdoc operation from executing.
     */
    LCB_ERRTYPE_SUBDOC = 1 << 9,
    LCB_ERRTYPE_DURABILITY = 1 << 10
} lcb_errflags_t;

/* PRIVATE. This is just here to instruct/inform users to use the more detailed codes */
#define LCB__ERR_USEDETAILS                                                                                            \
    "Enable detailed error codes (via LCB_CNTL_DETAILED_ERRCODES, or via "                                             \
    "`detailed_errcodes` in the connection string) and/or enable logging to "                                          \
    "get more information"

/**
 * @brief XMacro for all error types
 * @param X macro to be invoked for each function. This will accept the following
 * arguments:
 *  - Raw unquoted literal error identifier (e.g. `LCB_EINVAL`)
 *  - Code for the error (e.g. `0x23`)
 *  - Set of categories for the specific error (e.g. `LCB_ERRTYPE_FOO|LCB_ERRTYPE_BAR`)
 *  - Quoted string literal describing the error (e.g. `"This is sad"`)
 */
#define LCB_XERR(X)                                                                                                    \
    /** Success */                                                                                                     \
    X(LCB_SUCCESS, 0x00, 0, "Success (Not an error)")                                                                  \
                                                                                                                       \
    X(LCB_AUTH_CONTINUE, 0x01, LCB_ERRTYPE_INTERNAL | LCB_ERRTYPE_FATAL | LCB_ERRTYPE_SRVGEN,                          \
      "Error code used internally within libcouchbase for SASL auth. Should "                                          \
      "not be visible from the API")                                                                                   \
                                                                                                                       \
    /**This error code is received in callbacks when connecting or reconnecting                                        \
     to the cluster. If received during initial bootstrap                                                              \
     (i.e. lcb_get_bootstrap_status()) then it should be considered a fatal                                            \
     errror. This error should not be visible after initial bootstrap.                                                 \
                                                                                                                       \
     This error may also be received if CCCP bootstrap is used and the bucket does                                     \
     not exist.*/                                                                                                      \
    X(LCB_AUTH_ERROR, 0x02, LCB_ERRTYPE_FATAL | LCB_ERRTYPE_INPUT,                                                     \
      "Authentication failed. You may have provided an invalid "                                                       \
      "username/password combination")                                                                                 \
                                                                                                                       \
    /**This error is received in callbacks. It is a result of trying to perform                                        \
     an lcb_arithmetic() operation on an item which has an existing value that                                         \
     cannot be parsed as a number. */                                                                                  \
    X(LCB_DELTA_BADVAL, 0x03, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN,                             \
      "The value requested to be incremented is not stored as a number")                                               \
                                                                                                                       \
    /**This error is received in callbacks. It indicates that the key and value                                        \
     exceeded the constraints within the server. The current constraints are                                           \
     150 bytes for a key and 20MB for a value */                                                                       \
    X(LCB_E2BIG, 0x04, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN,                                    \
      "The object requested is too big to store in the server")                                                        \
                                                                                                                       \
    X(LCB_EBUSY, 0x05, LCB_ERRTYPE_TRANSIENT, "The server is busy. Try again later")                                   \
    /**Internal error within the library. This may be a result of a bug */                                             \
    X(LCB_EINTERNAL, 0x06, LCB_ERRTYPE_INTERNAL, "Internal libcouchbase error")                                        \
                                                                                                                       \
    /**If returned from an API call, it indicates invalid values were passed                                           \
     to the function. If received within a callback, it indicates that a                                               \
     malformed packet was sent to the server. */                                                                       \
    X(LCB_EINVAL, 0x07, LCB_ERRTYPE_INPUT, "Invalid input/arguments")                                                  \
                                                                                                                       \
    /**This code is received in callbacks. It means the server has no more memory                                      \
     left to store or modify the item. */                                                                              \
    X(LCB_ENOMEM, 0x08, LCB_ERRTYPE_TRANSIENT, "The server is out of memory. Try again later")                         \
                                                                                                                       \
    X(LCB_ERANGE, 0x09, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN, "Invalid range")                  \
                                                                                                                       \
    /**Generic error */                                                                                                \
    X(LCB_ERROR, 0x0A, 0, "Generic error")                                                                             \
                                                                                                                       \
    /**This error is received in callbacks from the server itself to indicate                                          \
    that it could not perform the requested operation. This is usually due to memory and/or                            \
    resource constraints on the server. This error may also be returned if a                                           \
    key has been locked (see lcb_get()) and an operation has been performed on it                                      \
    without unlocking the item (see lcb_unlock(), or pass the correct CAS value                                        \
    to a mutation function). */                                                                                        \
    X(LCB_ETMPFAIL, 0x0B, LCB_ERRTYPE_TRANSIENT | LCB_ERRTYPE_SRVLOAD | LCB_ERRTYPE_SRVGEN,                            \
      "Temporary failure received from server. Try again later")                                                       \
                                                                                                                       \
    /**The key already exists in the cluster. This error code is received within                                       \
    callbacks as a result of an _add_ operation in which the key already exists.                                       \
    It is also received for other operations in which a CAS was specified but has                                      \
    changed on the server. */                                                                                          \
    X(LCB_KEY_EEXISTS, 0x0C, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN,                                                  \
      "The key already exists in the server. If you have supplied a CAS then "                                         \
      "the key exists with a CAS value different than specified")                                                      \
                                                                                                                       \
    /**Received in callbacks to indicate that the server does not contain the item */                                  \
    X(LCB_KEY_ENOENT, 0x0D, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN, "The key does not exist on the server")           \
                                                                                                                       \
    /**Error code thrown if an I/O plugin could not be located */                                                      \
    X(LCB_DLOPEN_FAILED, 0x0E, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL | LCB_ERRTYPE_PLUGIN,                             \
      "Could not locate plugin library")                                                                               \
                                                                                                                       \
    /**Error code thrown of an I/O plugin did not contain a proper initialization routine */                           \
    X(LCB_DLSYM_FAILED, 0x0F, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL | LCB_ERRTYPE_PLUGIN,                              \
      "Required plugin initializer not found")                                                                         \
                                                                                                                       \
    /** This is a generic error code returned for various forms of socket                                              \
     operation failures. Newer applications are recommended to enable the                                              \
     @ref LCB_CNTL_DETAILED_ERRCODES setting via lcb_cntl() and receive more                                           \
     detailed information about a socket error.                                                                        \
                                                                                                                       \
     @see lcb_cntl(), @ref LCB_CNTL_DETAILED_ERRCODES */                                                               \
    X(LCB_NETWORK_ERROR, 0x10, LCB_ERRTYPE_NETWORK, "Generic network failure. " LCB__ERR_USEDETAILS)                   \
                                                                                                                       \
    /**Error code received in callbacks if the command was forwarded to the wrong                                      \
    server (for example, during a rebalance) and the library settings are configured                                   \
    that the command should not be remapped to a new server */                                                         \
    X(LCB_NOT_MY_VBUCKET, 0x11, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT | LCB_ERRTYPE_SRVGEN,                      \
      "The server which received this command claims it is not hosting this key")                                      \
                                                                                                                       \
    /**Received in callbacks as a response to an LCB_APPEND or LCB_PREPEND on an                                       \
    item that did not exist in the cluster. Equivalent to LCB_KEY_ENOENT */                                            \
    X(LCB_NOT_STORED, 0x12, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN,                                                   \
      "Item not stored (did you try to append/prepend to a missing key?)")                                             \
                                                                                                                       \
    /**Returned from API calls if a specific operation is valid but is unsupported                                     \
     in the current version or state of the library. May also be received in a                                         \
     callback if the cluster does not support the operation.                                                           \
                                                                                                                       \
     This will be returned for unknown settings passed to lcb_cntl() unless                                            \
     @ref LCB_CNTL_DETAILED_ERRCODES is set                                                                            \
     */                                                                                                                \
    X(LCB_NOT_SUPPORTED, 0x13, 0, "Operation not supported")                                                           \
                                                                                                                       \
    /**Received in callbacks if the cluster does not know about the command.                                           \
     Similar to LCB_NOT_SUPPORTED */                                                                                   \
    X(LCB_UNKNOWN_COMMAND, 0x14, LCB_ERRTYPE_SRVGEN, "Unknown command")                                                \
                                                                                                                       \
    /**Error code received if the hostname specified could not be found. It may                                        \
     also be received if a socket could not be created to the host supplied.                                           \
                                                                                                                       \
     A more detailed error code may be returned instead if                                                             \
     @ref LCB_CNTL_DETAILED_ERRCODES is set.*/                                                                         \
    X(LCB_UNKNOWN_HOST, 0x15, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_INPUT, "DNS/Hostname lookup failed")                   \
                                                                                                                       \
    /**Error code received if the server replied with an unexpected response */                                        \
    X(LCB_PROTOCOL_ERROR, 0x16, LCB_ERRTYPE_NETWORK, "Data received on socket was not in the expected format")         \
                                                                                                                       \
    /**Error code received in callbacks for operations which did not receive a                                         \
     reply from the server within the timeout limit.                                                                   \
     @see LCB_CNTL_OP_TIMEOUT */                                                                                       \
    X(LCB_ETIMEDOUT, 0x17, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT,                                                \
      "Client-Side timeout exceeded for operation. Inspect network conditions "                                        \
      "or increase the timeout")                                                                                       \
                                                                                                                       \
    /** @see LCB_NETWORK_ERROR, LCB_UNKNOWN_HOST, @ref LCB_CNTL_DETAILED_ERRCODES */                                   \
    X(LCB_CONNECT_ERROR, 0x18, LCB_ERRTYPE_NETWORK, "Error while establishing TCP connection. " LCB__ERR_USEDETAILS)   \
                                                                                                                       \
    /** Received on initial bootstrap if the bucket does not exist. Note that                                          \
     for CCCP bootstrap, @ref LCB_AUTH_ERROR will be received instead */                                               \
    X(LCB_BUCKET_ENOENT, 0x19, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL, "The bucket requested does not exist")           \
                                                                                                                       \
    /** Client could not allocate memory for internal structures */                                                    \
    X(LCB_CLIENT_ENOMEM, 0x1A, LCB_ERRTYPE_FATAL, "Memory allocation for libcouchbase failed. Severe problems ahead")  \
                                                                                                                       \
    /** Client could not schedule the request. This is typically received when                                         \
     an operation is requested before the initial bootstrap has completed */                                           \
    X(LCB_CLIENT_ENOCONF, 0x1B, LCB_ERRTYPE_TRANSIENT,                                                                 \
      "Client not bootstrapped. Ensure bootstrap/connect was attempted and was successful")                            \
                                                                                                                       \
    X(LCB_EBADHANDLE, 0x1C, LCB_ERRTYPE_INPUT,                                                                         \
      "Bad handle type for operation. "                                                                                \
      "You cannot perform administrative operations on a data handle, or data "                                        \
      "operations on a cluster handle")                                                                                \
                                                                                                                       \
    X(LCB_SERVER_BUG, 0x1D, 0, "Encountered a server bug")                                                             \
                                                                                                                       \
    X(LCB_PLUGIN_VERSION_MISMATCH, 0x1E, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL | LCB_ERRTYPE_PLUGIN,                   \
      "This version of libcouchbase cannot load the specified plugin")                                                 \
                                                                                                                       \
    X(LCB_INVALID_HOST_FORMAT, 0x1F, LCB_ERRTYPE_INPUT, "Hostname specified for URI is in an invalid format")          \
                                                                                                                       \
    X(LCB_INVALID_CHAR, 0x20, LCB_ERRTYPE_INPUT, "Illegal characted")                                                  \
                                                                                                                       \
    /** Received in response to the durability API call, if the amount of nodes                                        \
     or replicas to persist/replicate to exceed the total number of replicas the                                       \
     bucket was configured with. */                                                                                    \
    X(LCB_DURABILITY_ETOOMANY, 0x21, LCB_ERRTYPE_INPUT,                                                                \
      "Durability constraints requires more nodes/replicas than the cluster "                                          \
      "configuration allows. Durability constraints will never be satisfied")                                          \
                                                                                                                       \
    /** Received in scheduling if a command with the same key was specified more                                       \
     than once. Some commands will accept this, but others (notably `observe`)                                         \
     will not */                                                                                                       \
    X(LCB_DUPLICATE_COMMANDS, 0x22, LCB_ERRTYPE_INPUT,                                                                 \
      "The same key was specified more than once in the command list")                                                 \
                                                                                                                       \
    /** This error is received from API calls if the master node for the vBucket                                       \
     the key has been hashed to is not present. This will happen in the result                                         \
     of a node failover where no replica exists to replace it. */                                                      \
    X(LCB_NO_MATCHING_SERVER, 0x23, LCB_ERRTYPE_TRANSIENT,                                                             \
      "The node the request was mapped to does not exist in the current cluster "                                      \
      "map. This may be the result of a failover.")                                                                    \
                                                                                                                       \
    /** Received during initial creation (lcb_create()) if an environment variable                                     \
     was specified with an incorrect or invalid value.                                                                 \
                                                                                                                       \
     @see @ref lcb-env-vars-page */                                                                                    \
    X(LCB_BAD_ENVIRONMENT, 0x24, LCB_ERRTYPE_FATAL | LCB_ERRTYPE_INPUT,                                                \
      "The value for an environment variable recognized by libcouchbase was "                                          \
      "specified in an incorrect format. Check your environment for entries "                                          \
      "starting with 'LCB_' or 'LIBCOUCHBASE_'")                                                                       \
                                                                                                                       \
    X(LCB_BUSY, 0x25, LCB_ERRTYPE_INTERNAL, "Busy. This is an internal error")                                         \
                                                                                                                       \
    /** Received from lcb_create() if the username does not match the bucket */                                        \
    X(LCB_INVALID_USERNAME, 0x26, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL,                                               \
      "The username must match the bucket name (or be NULL) for data access")                                          \
                                                                                                                       \
    X(LCB_CONFIG_CACHE_INVALID, 0x27, LCB_ERRTYPE_INPUT,                                                               \
      "The contents of the configuration cache file were invalid. Configuration "                                      \
      "will be fetched from the network")                                                                              \
                                                                                                                       \
    /** Received during initial bootstrap if the library was configured to force                                       \
     the usage of a specific SASL mechanism and the server did not support this                                        \
     mechanism. @see LCB_CNTL_FORCE_SASL_MECH */                                                                       \
    X(LCB_SASLMECH_UNAVAILABLE, 0x28, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_FATAL,                                           \
      "The requested SASL mechanism was not supported by the server. Either "                                          \
      "upgrade the server or change the mechanism requirements")                                                       \
                                                                                                                       \
    /** Received in the HTTP callback if the response was redirected too many                                          \
     times. @see LCB_CNTL_MAX_REDIRECTS */                                                                             \
    X(LCB_TOO_MANY_REDIRECTS, 0x29, LCB_ERRTYPE_NETWORK,                                                               \
      "Maximum allowed number of redirects reached. See lcb_cntl and the "                                             \
      "LCB_CNTL_MAX_REDIRECTS option to modify this limit")                                                            \
                                                                                                                       \
    /** May be received in operation callbacks if the cluster toplogy changed                                          \
     and the library could not remap the command to a new node. This may be                                            \
     because the internal structure lacked sufficient information to recreate                                          \
     the packet, or because the configuration settings indicated that the command                                      \
     should not be retried. @see LCB_CNTL_RETRYMODE */                                                                 \
    X(LCB_MAP_CHANGED, 0x2A, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT,                                              \
      "The cluster map has changed and this operation could not be completed "                                         \
      "or retried internally. Try this operation again")                                                               \
                                                                                                                       \
    /** Returned from the lcb_pktfwd3() function if an incomplete packet was                                           \
     passed */                                                                                                         \
    X(LCB_INCOMPLETE_PACKET, 0x2B, LCB_ERRTYPE_TRANSIENT | LCB_ERRTYPE_INPUT,                                          \
      "Incomplete packet was passed to forward function")                                                              \
                                                                                                                       \
    /** Mapped directly to the system `ECONNREFUSED` errno. This is received                                           \
     in callbacks if an initial connection to the node could not be established.                                       \
     Check your firewall settings and ensure the specified service is online. */                                       \
    X(LCB_ECONNREFUSED, 0x2C, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT,                                             \
      "The remote host refused the connection. Is the service up?")                                                    \
                                                                                                                       \
    /** Returned in a callback if the socket connection was gracefully closed,                                         \
     but the library wasn't expecting it. This may happen if the system is                                             \
     being shut down.                                                                                                  \
     @lcb_see_detailed_neterr */                                                                                       \
    X(LCB_ESOCKSHUTDOWN, 0x2D, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT, "The remote host closed the connection")   \
                                                                                                                       \
    /** Returned in a callback if the socket connection was forcefully reset,                                          \
     Equivalent to the system `ECONNRESET`.                                                                            \
     @lcb_see_detailed_neterr                                                                                          \
     */                                                                                                                \
    X(LCB_ECONNRESET, 0x2E, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT,                                               \
      "The connection was forcibly reset by the remote host")                                                          \
                                                                                                                       \
    /** Returned in a callback if the library could not allocated a local socket                                       \
     due to TCP local port exhaustion. This means you have either found a bug                                          \
     in the library or are creating too many TCP connections. Keep in mind that                                        \
     a TCP connection will still occupy a slot in your system socket table even                                        \
     after it has been closed (and will thus appear in a `TIME_WAIT` state).                                           \
                                                                                                                       \
     @lcb_see_detailed_neterr                                                                                          \
     */                                                                                                                \
    X(LCB_ECANTGETPORT, 0x2F, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_FATAL,                                                 \
      "Could not assign a local port for this socket. For client sockets this means "                                  \
      "there are too many TCP sockets open")                                                                           \
                                                                                                                       \
    /** Returned if the library could not allocate a new file descriptor for a                                         \
     socket or other resource. This may be more common on systems (such as                                             \
     Mac OS X) which have relatively low limits for file descriptors. To raise                                         \
     the file descriptor limit, refer to the `ulimit -n` command                                                       \
                                                                                                                       \
     @lcb_see_detailed_neterr                                                                                          \
     */                                                                                                                \
    X(LCB_EFDLIMITREACHED, 0x30, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_FATAL,                                              \
      "The system or process has reached its maximum number of file descriptors")                                      \
                                                                                                                       \
    /** Returned in callback if the host or subnet containing a node could                                             \
     not be contacted. This may be a result of a bad routing table or being                                            \
     physically disconnected from the network.                                                                         \
     @lcb_see_detailed_neterr. */                                                                                      \
    X(LCB_ENETUNREACH, 0x31, LCB_ERRTYPE_NETWORK | LCB_ERRTYPE_TRANSIENT,                                              \
      "The remote host was unreachable - is your network OK?")                                                         \
                                                                                                                       \
    /** An unrecognized setting was passed to the lcb_cntl() function                                                  \
     @lcb_see_detailed_neterr */                                                                                       \
    X(LCB_ECTL_UNKNOWN, 0x32, LCB_ERRTYPE_INPUT, "Control code passed was unrecognized")                               \
                                                                                                                       \
    /** An invalid operation was supplied for a setting to lcb_cntl(). This will                                       \
     happen if you try to write to a read-only setting, or retrieve a value                                            \
     which may only be set. Refer to the documentation for an individual setting                                       \
     to see what modes it supports.                                                                                    \
     @lcb_see_detailed_neterr                                                                                          \
     */                                                                                                                \
    X(LCB_ECTL_UNSUPPMODE, 0x33, LCB_ERRTYPE_INPUT,                                                                    \
      "Invalid modifier for cntl operation (e.g. tried to read a write-only value")                                    \
                                                                                                                       \
    /** A malformed argument was passed to lcb_cntl() for the given setting. See                                       \
     the documentation for the setting to see what arguments it supports and                                           \
     how they are to be supplied.                                                                                      \
                                                                                                                       \
     @lcb_see_detailed_neterr  */                                                                                      \
    X(LCB_ECTL_BADARG, 0x34, LCB_ERRTYPE_INPUT, "Argument passed to cntl was badly formatted")                         \
                                                                                                                       \
    /**An empty key was passed to an operation. Most commands do not accept                                            \
      empty keys. */                                                                                                   \
    X(LCB_EMPTY_KEY, 0x35, LCB_ERRTYPE_INPUT, "An empty key was passed to an operation")                               \
                                                                                                                       \
    /** A problem with the SSL system was encountered. Use logging to discover                                         \
     what happened. This error will only be thrown if something internal to the                                        \
     SSL library failed (for example, a bad certificate or bad user input);                                            \
     otherwise a network error will be thrown if an SSL connection was terminated */                                   \
    X(LCB_SSL_ERROR, 0x36, LCB_ERRTYPE_FATAL,                                                                          \
      "A generic error related to the SSL subsystem was encountered. Enable logging "                                  \
      "to see more details")                                                                                           \
                                                                                                                       \
    /** The certificate the server sent cannot be verified. This is a possible                                         \
     case of a man-in-the-middle attack, but also of forgetting to supply                                              \
     the path to the CA authority to the library. */                                                                   \
    X(LCB_SSL_CANTVERIFY, 0x37, LCB_ERRTYPE_FATAL, "Client could not verify server's certificate")                     \
                                                                                                                       \
    X(LCB_SCHEDFAIL_INTERNAL, 0x38, 0, "Internal error used for destroying unscheduled command data")                  \
                                                                                                                       \
    /** An optional client feature was requested, but the current configuration                                        \
     does not allow it to be used. This might be because it is not available                                           \
     on a particular platform/architecture/operating system/configuration, or                                          \
     it has been disabled at the time the library was built.                                                           \
     */                                                                                                                \
    X(LCB_CLIENT_FEATURE_UNAVAILABLE, 0x39, LCB_ERRTYPE_INPUT,                                                         \
      "The requested feature is not supported by the client, either because of "                                       \
      "settings in the configured instance, or because of options disabled at "                                        \
      "the time the library was compiled")                                                                             \
                                                                                                                       \
    /**An option was passed to a command which is incompatible with other                                              \
     options. This may happen if two fields are mutually exclusive */                                                  \
    X(LCB_OPTIONS_CONFLICT, 0x3A, LCB_ERRTYPE_INPUT, "The operation structure contains conflicting options")           \
                                                                                                                       \
    /**Received in callbacks if an operation failed because of a negative HTTP                                         \
     status code */                                                                                                    \
    X(LCB_HTTP_ERROR, 0x3B, 0, "HTTP Operation failed. Inspect status code for details")                               \
                                                                                                                       \
    /**Scheduling error received if @ref LCB_CNTL_DURABILITY_MUTATION_TOKENS was                                       \
     enabled, but there is no available mutation token for the key. */                                                 \
    X(LCB_DURABILITY_NO_MUTATION_TOKENS, 0x3C, LCB_ERRTYPE_INPUT,                                                      \
      "The given item does not have a mutation token associated with it. "                                             \
      "this is either because fetching mutation tokens was not enabled, or "                                           \
      "you are trying to check on something not stored by this instance")                                              \
                                                                                                                       \
    /** The server replied with an unrecognized status code */                                                         \
    X(LCB_UNKNOWN_MEMCACHED_ERROR, 0x3D, LCB_ERRTYPE_SRVGEN,                                                           \
      "The server replied with an unrecognized status code. A newer version "                                          \
      "of this library may be able to decode it")                                                                      \
                                                                                                                       \
    /** The server replied that the given mutation has been lost */                                                    \
    X(LCB_MUTATION_LOST, 0x3E, LCB_ERRTYPE_SRVGEN,                                                                     \
      "The given mutation has been permanently lost due to the node failing "                                          \
      "before replication")                                                                                            \
    /** Sub-document path does not exist */                                                                            \
    X(LCB_SUBDOC_PATH_ENOENT, 0x3F, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                      \
      "Sub-document path does not exist")                                                                              \
    /** Type of element in sub-document path conflicts with type in document */                                        \
    X(LCB_SUBDOC_PATH_MISMATCH, 0x40, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                    \
      "Type of element in sub-document path conflicts with type in document")                                          \
    /** Malformed sub-document path */                                                                                 \
    X(LCB_SUBDOC_PATH_EINVAL, 0x41, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                       \
      "Malformed sub-document path")                                                                                   \
    /** Sub-document contains too many components */                                                                   \
    X(LCB_SUBDOC_PATH_E2BIG, 0x42, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                        \
      "Sub-document contains too many components")                                                                     \
    /** Existing document contains too many levels of nesting */                                                       \
    X(LCB_SUBDOC_DOC_E2DEEP, 0x43, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                       \
      "Existing document contains too many levels of nesting")                                                         \
    /** Subdocument operation would invalidate the JSON */                                                             \
    X(LCB_SUBDOC_VALUE_CANTINSERT, 0x44, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                  \
      "Subdocument operation would invalidate the JSON")                                                               \
    /** Existing document is not valid JSON */                                                                         \
    X(LCB_SUBDOC_DOC_NOTJSON, 0x45, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                      \
      "Existing document is not valid JSON")                                                                           \
    /** The existing numeric value is too large */                                                                     \
    X(LCB_SUBDOC_NUM_ERANGE, 0x46, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                       \
      "The existing numeric value is too large")                                                                       \
    /** Delta must be numeric, within the 64 bit signed range, and non-zero */                                         \
    X(LCB_SUBDOC_BAD_DELTA, 0x47, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                        \
      "Delta must be numeric, within the 64 bit signed range, and non-zero")                                           \
    /** The given path already exists in the document */                                                               \
    X(LCB_SUBDOC_PATH_EEXISTS, 0x48, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                     \
      "The given path already exists in the document")                                                                 \
    /** Could not execute one or more multi lookups or mutations */                                                    \
    X(LCB_SUBDOC_MULTI_FAILURE, 0x49, LCB_ERRTYPE_DATAOP | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                    \
      "Could not execute one or more multi lookups or mutations")                                                      \
    /** Value is too deep to insert */                                                                                 \
    X(LCB_SUBDOC_VALUE_E2DEEP, 0x4A, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                      \
      "Value is too deep to insert")                                                                                   \
    /** A badly formatted packet was sent to the server. Please report this in a bug */                                \
    X(LCB_EINVAL_MCD, 0x4B, LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_INTERNAL,                                                 \
      "A badly formatted packet was sent to the server. Please report this in a bug")                                  \
    /** Missing subdocument path */                                                                                    \
    X(LCB_EMPTY_PATH, 0x4C, LCB_ERRTYPE_INPUT, "Missing subdocument path")                                             \
    /** Unknown subdocument command */                                                                                 \
    X(LCB_UNKNOWN_SDCMD, 0x4D, LCB_ERRTYPE_INPUT, "Unknown subdocument command")                                       \
    /** No commands specified */                                                                                       \
    X(LCB_ENO_COMMANDS, 0x4E, LCB_ERRTYPE_INPUT, "No commands specified")                                              \
    /** Query execution failed. Inspect raw response object for information */                                         \
    X(LCB_QUERY_ERROR, 0x4F, LCB_ERRTYPE_SRVGEN,                                                                       \
      "Query execution failed. Inspect raw response object for information")                                           \
    /** Generic temporary error received from server*/                                                                 \
    X(LCB_GENERIC_TMPERR, 0x50, LCB_ERRTYPE_TRANSIENT | LCB_ERRTYPE_SRVGEN,                                            \
      "Generic temporary error received from server")                                                                  \
    /** Generic subdocument error received from server*/                                                               \
    X(LCB_GENERIC_SUBDOCERR, 0x51, LCB_ERRTYPE_SUBDOC | LCB_ERRTYPE_SRVGEN,                                            \
      "Generic subdocument error received from server")                                                                \
    /** Generic constraint error received from server*/                                                                \
    X(LCB_GENERIC_CONSTRAINT_ERR, 0x52, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN,                                        \
      "Generic constraint error received from server")                                                                 \
    /** Invalid reply received from nameserver*/                                                                       \
    X(LCB_NAMESERVER_ERROR, 0x53, LCB_ERRTYPE_NETWORK, "Invalid reply received from nameserver")                       \
    /** Not authorized for operation*/                                                                                 \
    X(LCB_NOT_AUTHORIZED, 0x54, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN, "Not authorized for operation")                \
    /** An invalid combination of commands was specified*/                                                             \
    X(LCB_SUBDOC_INVALID_COMBO, 0x55, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                     \
      "An invalid combination of commands was specified")                                                              \
    /** Specified key was successfully found, but one or more path operations failed*/                                 \
    X(LCB_SUBDOC_MULTI_PATH_FAILURE, 0x56, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                \
      "Specified key was successfully found, but one or more path operations failed")                                  \
    /** The operation completed successfully, but operated on a deleted document*/                                     \
    X(LCB_SUBDOC_SUCCESS_DELETED, 0x57, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,                   \
      "The operation completed successfully, but operated on a deleted document")                                      \
    /** The combination of the subdoc flags for the xattrs is not valid*/                                              \
    X(LCB_SUBDOC_XATTR_INVALID_FLAG_COMBO, 0x58, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,          \
      "The combination of the subdoc flags for the xattrs is not valid")                                               \
    /** Only a single xattr key may be accessed at the same time*/                                                     \
    X(LCB_SUBDOC_XATTR_INVALID_KEY_COMBO, 0x59, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,           \
      "Only a single xattr key may be accessed at the same time")                                                      \
    /** The server has no knowledge of the requested macro*/                                                           \
    X(LCB_SUBDOC_XATTR_UNKNOWN_MACRO, 0x5a, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,               \
      "The server has no knowledge of the requested macro")                                                            \
    /** The server has no knowledge of the requested virtual xattr */                                                  \
    X(LCB_SUBDOC_XATTR_UNKNOWN_VATTR, 0x5b, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,               \
      "The server has no knowledge of the requested virtual xattr")                                                    \
    /** Virtual xattrs cannot be modified */                                                                           \
    X(LCB_SUBDOC_XATTR_CANT_MODIFY_VATTR, 0x5c, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,           \
      "Virtual xattrs cannot be modified")                                                                             \
    /** Specified key was found as a deleted document, but one or more path operations failed*/                        \
    X(LCB_SUBDOC_MULTI_PATH_FAILURE_DELETED, 0x5d, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,        \
      "Specified key was found as a deleted document, but one or more path operations failed")                         \
    /** According to the spec all xattr commands should come first, followed by the commands for the document body */  \
    X(LCB_SUBDOC_INVALID_XATTR_ORDER, 0x5e, LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_SUBDOC,               \
      "According to the spec all xattr commands should come first, followed by the commands for the document body")    \
    X(LCB_COLLECTION_UNKNOWN, 0x5f, LCB_ERRTYPE_INPUT, "Collection does not exists")                                   \
    /** Operation attempted and requires that the collections manifest is set.  */                                     \
    X(LCB_COLLECTION_NO_MANIFEST, 0x60, LCB_ERRTYPE_INPUT, "No Collections Manifest")                                  \
    /** Bucket Manifest update could not be applied to vbucket(s) */                                                   \
    X(LCB_COLLECTION_CANNOT_APPLY_MANIFEST, 0x61, LCB_ERRTYPE_INPUT, "Cannot apply collections manifest")              \
    /** Client has a collection's manifest which is from the future. This means                                        \
     * they have a uid that is greater than ours.  */                                                                  \
    X(LCB_COLLECTION_MANIFEST_IS_AHEAD, 0x62, LCB_ERRTYPE_INPUT, "Collections manifest of SDK is ahead of Server's")   \
    X(LCB_DURABILITY_INVALID_LEVEL, 0x63, LCB_ERRTYPE_DURABILITY | LCB_ERRTYPE_INPUT | LCB_ERRTYPE_SRVGEN,             \
      "Invalid durability level was specified")                                                                        \
    /** Valid request, but given durability requirements are impossible to                                             \
     * achieve - because insufficient configured replicas are connected.                                               \
     * Assuming level=majority and C=number of configured nodes, durability                                            \
     * becomes impossible if floor((C + 1) / 2) nodes or greater are offline. */                                       \
    X(LCB_DURABILITY_IMPOSSIBLE, 0x64, LCB_ERRTYPE_DURABILITY | LCB_ERRTYPE_SRVGEN,                                    \
      "Given durability requirements are impossible to achieve")                                                       \
    /** Returned if an attempt is made to mutate a key which already has a                                             \
     * SyncWrite pending. Client would typically retry (possibly with backoff).                                        \
     * Similar to ELOCKED */                                                                                           \
    X(LCB_DURABILITY_SYNC_WRITE_IN_PROGRESS, 0x65,                                                                     \
      LCB_ERRTYPE_DURABILITY | LCB_ERRTYPE_SRVGEN | LCB_ERRTYPE_TRANSIENT,                                             \
      "There is a synchronous mutation pending for given key")                                                         \
    /** The SyncWrite request has not completed in the specified time and has ambiguous result - it may Succeed or     \
     * Fail; but the final value is not yet known */                                                                   \
    X(LCB_DURABILITY_SYNC_WRITE_AMBIGUOUS, 0x66, LCB_ERRTYPE_DURABILITY | LCB_ERRTYPE_SRVGEN,                          \
      "Synchronous mutation has not completed in the specified time and has ambiguous result")

/** Error codes returned by the library. */
typedef enum {
#define X(n, v, cls, s) n = v,
    LCB_XERR(X)
#undef X

#ifdef LIBCOUCHBASE_INTERNAL
    /**
     * This is a private value used by the tests in libcouchbase
     */
    LCB_MAX_ERROR_VAL,
#endif

    /** The errors below this value reserved for libcouchbase usage. */
    LCB_MAX_ERROR = 0x1000
} lcb_STATUS;

/** @deprecated Use new, less ambiguous identifier (@ref LCB_CLIENT_ENOCONF) */
#define LCB_CLIENT_ETMPFAIL LCB_CLIENT_ENOCONF

/** @brief If the error is a result of bad input */
#define LCB_EIFINPUT(e) (lcb_get_errtype(e) & LCB_ERRTYPE_INPUT)

/** @brief if the error is a result of a network condition */
#define LCB_EIFNET(e) (lcb_get_errtype(e) & LCB_ERRTYPE_NETWORK)

/** @brief if the error is fatal */
#define LCB_EIFFATAL(e) (lcb_get_errtype(e) & LCB_ERRTYPE_FATAL)

/** @brief if the error is transient */
#define LCB_EIFTMP(e) (lcb_get_errtype(e) & LCB_ERRTYPE_TRANSIENT)

/** @brief if the error is a routine negative server reply */
#define LCB_EIFDATA(e) (lcb_get_errtype(e) & LCB_ERRTYPE_DATAOP)

/** @brief if the error is a result of a plugin implementation */
#define LCB_EIFPLUGIN(e) (lcb_get_errtype(e) & LCB_ERRTYPE_PLUGIN)
#define LCB_EIFSRVLOAD(e) (lcb_get_errtype(e) & LCB_ERRTYPE_SRVLOAD)
#define LCB_EIFSRVGEN(e) (lcb_get_errtype(e) & LCB_ERRTYPE_SRVGEN)
#define LCB_EIFSUBDOC(e) (lcb_get_errtype(e) & LCB_ERRTYPE_SUBDOC)

/**
 * @brief Get error categories for a specific code
 * @param err the error received
 * @return a set of flags containing the categories for the given error
 * @committed
 */
LIBCOUCHBASE_API
int lcb_get_errtype(lcb_STATUS err);

/**
 * Get a textual descrtiption for the given error code
 * @param instance the instance the error code belongs to (you might
 *                 want different localizations for the different instances)
 * @param error the error code
 * @return A textual description of the error message. The caller should
 *         <b>not</b> release the memory returned from this function.
 * @committed
 */
LIBCOUCHBASE_API
const char *lcb_strerror(lcb_INSTANCE *instance, lcb_STATUS error);

/**
 * Get a shorter textual description of an error message. This is the
 * constant name
 */
LCB_INTERNAL_API
const char *lcb_strerror_short(lcb_STATUS error);

/**
 * Get a longer textual description of an error message.
 */
LCB_INTERNAL_API
const char *lcb_strerror_long(lcb_STATUS error);

/**
 * This may be used in conjunction with the errmap callback if it wishes
 * to fallback for default behavior for the given code.
 * @uncommitted
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_errmap_default(lcb_INSTANCE *instance, lcb_U16 code);

/**
 * Callback for error mappings. This will be invoked when requesting whether
 * the user has a possible mapping for this error code.
 *
 * This will be called for response codes which may be ambiguous in most
 * use cases, or in cases where detailed response codes may be mapped to
 * more generic ones.
 */
typedef lcb_STATUS (*lcb_errmap_callback)(lcb_INSTANCE *instance, lcb_U16 bincode);

/**@uncommitted*/
LIBCOUCHBASE_API
lcb_errmap_callback lcb_set_errmap_callback(lcb_INSTANCE *instance, lcb_errmap_callback);

#ifdef __cplusplus
}
#endif
/**@}*/
#endif
