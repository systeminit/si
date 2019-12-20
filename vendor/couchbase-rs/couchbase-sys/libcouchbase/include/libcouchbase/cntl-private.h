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

/**@file*/

/**
 * @internal
 * @addtogroup lcb-cntl-settings
 * @{
 */

/**@internal*/
#define LCB_CNTL_SERVER_COMMON_FIELDS                                                                                  \
    /** Server index to query */                                                                                       \
    int index;                                                                                                         \
                                                                                                                       \
    /** NUL-terminated string containing the address */                                                                \
    const char *host;                                                                                                  \
    /** NUL-terminated string containing the port */                                                                   \
    const char *port;                                                                                                  \
    /** Whether the node is connected */                                                                               \
    int connected;                                                                                                     \
                                                                                                                       \
    /**                                                                                                                \
     * Socket information. If a v0 IO plugin is being used, the sockfd                                                 \
     * is set to the socket descriptor. If a v1 plugin is being used, the                                              \
     * sockptr is set to point to the appropriate structure.                                                           \
     *                                                                                                                 \
     * Note that you *MAY* perform various 'setsockopt' calls on the                                                   \
     * sockfd (though it is your responsibility to ensure those options                                                \
     * are valid); however the actual socket descriptor may change                                                     \
     * in the case of a cluster configuration update.                                                                  \
     */                                                                                                                \
    union {                                                                                                            \
        lcb_socket_t sockfd;                                                                                           \
        lcb_sockdata_t *sockptr;                                                                                       \
    } sock;

/**
 * @internal
 * @brief Information describing the server
 */
typedef struct lcb_cntl_server_st {
    /** Structure version */
    int version;

    union {
        struct {
            LCB_CNTL_SERVER_COMMON_FIELDS
        } v0;

        /** Current information here */
        struct {
            LCB_CNTL_SERVER_COMMON_FIELDS
            /** Chosen SASL mechanism */
            const char *sasl_mech;
        } v1;
    } v;
} lcb_cntl_server_t;
#undef LCB_CNTL_SERVER_COMMON_FIELDS

/**
 * @internal
 *
 * @brief Get information about a memcached node.
 *
 * This function will populate a structure containing various information
 * about the specific host
 *
 * Note that all fields in the structure are only valid until the following
 * happens (whichever is first)
 *
 * 1. Another libcouchbase API function is called
 * 2. The event loop regains control
 *
 * @cntl_arg_getonly{lcb_cntl_server_t*}
 * @volatile
 */
#define LCB_CNTL_MEMDNODE_INFO 0x08

/**
 * @internal
 *
 * @brief Get information about the configuration node.
 *
 * Note that this may not be available if the configuration mode is not HTTP
 *
 * @cntl_arg_getonly{lcb_cntl_server_t*}
 * @volatile
 */
#define LCB_CNTL_CONFIGNODE_INFO 0x09

/**@internal
 * @brief Information about the I/O plugin
 */
struct lcb_cntl_iops_info_st {
    int version;
    union {
        struct {
            /**
             * Pass here options, used to create IO structure with
             * lcb_create_io_ops(3), to find out whether the library
             * will override them in the current environment
             */
            const struct lcb_create_io_ops_st *options;

            /**
             * The default IO ops type. This is hard-coded into the library
             * and is used if nothing else was specified in creation options
             * or the environment
             */
            lcb_io_ops_type_t os_default;

            /**
             * The effective plugin type after reading environment variables.
             * If this is set to 0, then a manual (non-builtin) plugin has been
             * specified.
             */
            lcb_io_ops_type_t effective;
        } v0;
    } v;
};

/**
 * @internal
 * @brief Get the default IOPS types for this build.
 *
 * This provides a convenient
 * way to determine what libcouchbase will use for IO when not explicitly
 * specifying an iops structure to lcb_create()
 *
 * @cntl_arg_getonly{lcb_cntl_io_ops_info_st*}
 *
 * @note You may pass NULL to lcb_cntl for the 'instance' parameter,
 * as this does not read anything specific on the handle
 *
 * @uncommitted
 */
#define LCB_CNTL_IOPS_DEFAULT_TYPES 0x10

/**
 * @internal
 *
 * @brief Set the nodes for the HTTP provider.
 *
 * @uncommitted
 *
 * This sets the initial list
 * for the nodes to be used for bootstrapping the cluster. This may also
 * be used subsequently in runtime to provide an updated list of nodes
 * if the current list malfunctions.
 *
 * The argument for this cntl accepts a NUL-terminated string containing
 * one or more nodes. The format for this string is the same as the
 * `host` parameter in lcb_create_st
 *
 * Ports should specify the REST API port.
 * @cntl_arg_setonly{char** (Array of strings)}
 */
#define LCB_CNTL_CONFIG_HTTP_NODES 0x1D

/**
 * @internal
 *
 * @brief Set the nodes for the CCCP provider.
 *
 * Similar to @ref LCB_CNTL_CONFIG_HTTP_NODES, but affects the CCCP provider
 * instead. Ports should specify the _memcached_ port
 * @cntl_arg_setonly{char** (Array of strings)}
 * @uncommitted
 */
#define LCB_CNTL_CONFIG_CCCP_NODES 0x1E

/**
 * @internal
 *
 * @brief Set the config nodes for the relevant providers.
 *
 * This is passed an lcb_create_st2 structure which is used to initialize
 * the providers. Useful if you wish to reinitialize or modify the
 * provider settings _after_ the instance itself has already been
 * constructed.
 *
 * Note that the username, password, bucket, and io fields are
 * ignored.
 *
 * @cntl_arg_setonly{lcb_create_st2*}
 * @uncommitted
 */
#define LCB_CNTL_CONFIG_ALL_NODES 0x20

/**
 * Reinitialize the instance using a connection string. Only options and
 * the hostlists are used from this string. The bucket in the string (if specified)
 * and any SSL options (i.e. `couchbases://` or `ssl=no_verify`) are ignored.
 *
 *
 * This is the newer variant of @ref LCB_CNTL_CONFIG_ALL_NODES
 * @cntl_arg_setonly{const char *}
 * @internal
 */
#define LCB_CNTL_REINIT_CONNSTR 0x2B

struct rdb_ALLOCATOR;
typedef struct rdb_ALLOCATOR *(*lcb_RDBALLOCFACTORY)(void);

/**Structure being used because function pointers can't technically be cast
 * to void*
 */
struct lcb_cntl_rdballocfactory {
    lcb_RDBALLOCFACTORY factory;
};
/**
 * Set the allocator factory used by libcouchbase. The allocator factory is
 * a function invoked with no arguments which yields a new rdb_ALLOCATOR
 * object. Currently the use and API of this object is considered internal
 * and its API and header files are in `src/rdb`.
 *
 * Mode|Arg
 * ----|---
 * Set, Get | `lcb_cntl_rdballocfactory*`
 * @volatile
 */
#define LCB_CNTL_RDBALLOCFACTORY 0x27

/**
 * @brief Persist heuristic vbucket information across updates.
 *
 * As of version 2.4.8 this option no longer has any effect, and vBucket
 * heuristics are always retained for a maximum of 20 seconds.
 * @cntl_arg_both{int*}
 * @volatile
 */
#define LCB_CNTL_VBGUESS_PERSIST 0x32

/**
 * This is a collection of various options which sacrifice data safety for
 * speed.
 * @volatile
 */
#define LCB_CNTL_UNSAFE_OPTIMIZE 0x33

/**
 * Disable or enable Nagle's algorithm. The default is to disable it, as it
 * will typically reduce latency. In general it is recommended not to touch
 * this setting. It is here mainly for debugging.
 *
 * Conventionally, the option to disable Nagle's algorithm is called "TCP_NODELAY",
 * thus if this value is one, Nagle is off, and vice versa.
 * @volatile
 */
#define LCB_CNTL_TCP_NODELAY 0x39

/**
 * Get the lcb_HISTOGRAM object for key-value timings
 * @cntl_arg_getonly{lcb_HISTOGRAM**}
 * @volatile
 */
#define LCB_CNTL_KVTIMINGS 0x3C

/**
 * @volatile
 * Activate/Get library metrics per-server
 *
 * If using @ref LCB_CNTL_SET, then this will activate the metrics, and should
 * be called immediately after lcb_create. The `arg` parameter should be a pointer
 * to an integer with the activation value (any non-zero value to activate).
 *
 * If using @ref LCB_CNTL_GET, the `arg` parameter should be a @ref `lcb_METRICS**`
 * variable, which will contain the pointer to the metrics upon completion.
 */
#define LCB_CNTL_METRICS 0x49

/**
 * Do not use fast-forward map from cluster configuration.
 *
 * Use `vb_noremap` in the connection string
 *
 * @cntl_arg_both{int* (as boolean)}
 * @uncommitted
 */
#define LCB_CNTL_VB_NOREMAP 0x5a

/**
 * Do not wait for GET_CLUSTER_CONFIG request to finish in lcb_wait(),
 * when it is the only request in retry queue. Consider such retry queue
 * as empty, and breakout to the place where lcb_wait() was invoked.
 *
 * @cntl_arg_both{int* (as boolean)}
 *
 * @uncommitted
 */
#define LCB_CNTL_WAIT_FOR_CONFIG 0x5c

/**@}*/
