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

#ifndef LCB_VBUCKET_H
#define LCB_VBUCKET_H
#include <libcouchbase/visibility.h>
#include <libcouchbase/sysdefs.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file
 * @brief vBucket Mapping API
 */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-vbucket-api vBucket
 * @details
 * These routines contain functionality for parsing a cluster topology configuration
 * and mapping keys to cluster nodes appropriately.
 */

/**
 * @addtogroup lcb-vbucket-api
 * @{
 */

/**@brief This enum functions as a 'key' to indicate a specific service offered
 * by a node */
typedef enum {
    LCBVB_SVCTYPE_DATA = 0,  /**< memcached/Data port */
    LCBVB_SVCTYPE_VIEWS,     /**< Views/CAPI port */
    LCBVB_SVCTYPE_MGMT,      /**< Administrative/'REST' UI */
    LCBVB_SVCTYPE_IXQUERY,   /**< Index query */
    LCBVB_SVCTYPE_IXADMIN,   /**< Index administration */
    LCBVB_SVCTYPE_N1QL,      /**< N1QL Query */
    LCBVB_SVCTYPE_FTS,       /**< Fulltext */
    LCBVB_SVCTYPE_ANALYTICS, /**< Analytics Query */
/* for backward compatiblity */
#define LCBVB_SVCTYPE_CBAS LCBVB_SVCTYPE_ANALYTICS
    LCBVB_SVCTYPE__MAX
} lcbvb_SVCTYPE;

/**@brief This enum functions to indicate the 'mode' of the service. Currently
 * this is to distinguish between SSL and plain transports */
typedef enum {
    LCBVB_SVCMODE_PLAIN = 0, /**< Plain transport */
    LCBVB_SVCMODE_SSL,       /**< SSL Transport */
    LCBVB_SVCMODE__MAX
} lcbvb_SVCMODE;

/**
 * @volatile. ABI/API compatibility not guaranteed between versions
 * @brief Services which may be provided by a node
 */
typedef struct {
    lcb_U16 data;      /**< Data port for key-value operations (memcached protocol) */
    lcb_U16 mgmt;      /**< Port for adminsitrative operations (HTTP) */
    lcb_U16 views;     /**< Port for view queries (HTTP) */
    lcb_U16 ixquery;   /**< Indexing query port */
    lcb_U16 ixadmin;   /**< Indexing admin port (HTTP) */
    lcb_U16 n1ql;      /**< Query port */
    lcb_U16 fts;       /**< CBFT */
    lcb_U16 cbas;      /**< CBAS (Analytics) */
    char *views_base_; /**< Views base URL */
    char *query_base_; /**< N1QL base URL */
    char *fts_base_;
    char *cbas_base_;
    char *hoststrs[LCBVB_SVCTYPE__MAX];
} lcbvb_SERVICES;

/**
 * @volatile. ABI/API compatibility not guaranteed between versions.
 *
 * @brief Node in the cluster
 * This structure represents a node in the cluster. The node has a hostname
 * (@ref #hostname), and various services.
 */
typedef struct {
    lcbvb_SERVICES svc;         /**< Plain services */
    lcbvb_SERVICES svc_ssl;     /**< SSL Services */
    char *authority;            /**< host:dataport for comparison */
    char *hostname;             /**< Hostname for the node */
    char *viewpath;             /**< Path prefix for view queries */
    char *querypath;            /**< Path prefix for n1ql queries */
    char *ftspath;              /**< Path prefix for fulltext queries */
    char *cbaspath;             /**< Path prefix for analytics queries */
    unsigned nvbs;              /**< Total number of vbuckets the server has assigned */
    char *alt_hostname;         /**< selected alternative hostname for the node */
    lcbvb_SERVICES alt_svc;     /**< selected alternative plain services */
    lcbvb_SERVICES alt_svc_ssl; /**< selected alternative SSL Services */
} lcbvb_SERVER;

/**@volatile. ABI/API compatibility not guaranteed between versions */
typedef struct {
    int servers[4];
} lcbvb_VBUCKET;

/**@volatile*/
typedef struct {
    lcb_U32 index;
    lcb_U32 point;
} lcbvb_CONTINUUM;

/** @brief Type of algorithm used to distribute keys.
 * This also indicates the type of bucket */
typedef enum {
    LCBVB_DIST_VBUCKET = 0, /**< vBucket hashing ("couchbase") bucket */
    LCBVB_DIST_KETAMA = 1,  /**< Ketama hashing ("memcached") bucket */
    LCBVB_DIST_UNKNOWN = 2  /**< Unknown distribution (cluster config) */
} lcbvb_DISTMODE;

typedef enum {
    LCBVB_CAP_XATTR = 1 << 0,
    LCBVB_CAP_CBHELLO = 1 << 1,
    LCBVB_CAP_CCCP = 1 << 2,
    LCBVB_CAP_COUCHAPI = 1 << 3,
    LCBVB_CAP_DCP = 1 << 4,
    LCBVB_CAP_NODES_EXT = 1 << 5,
    LCBVB_CAP_TOUCH = 1 << 6,
    LCBVB_CAP_XDCR_CHECKPOINTING = 1 << 7,
    LCBVB_CAP_COLLECTIONS = 1 << 8,
    LCBVB_CAP_DURABLE_WRITE = 1 << 9
} lcbvb_BUCKET_CAPABILITIES;

typedef enum {
    LCBVB_CCAP_N1QL_ENHANCED_PREPARED_STATEMENTS = 1 << 0,
} lcbvb_CLUSTER_CAPABILITIES;

/**@volatile. ABI/API compatibility not guaranteed between versions.
 * @brief Structure containing the configuration.*/
typedef struct lcbvb_CONFIG_st {
    lcbvb_DISTMODE dtype;       /**< Type of bucket/distribution */
    unsigned nvb;               /**< Number of vbuckets */
    unsigned ndatasrv;          /**< Number of data (memcached) servers */
    unsigned nsrv;              /** Number of servers */
    unsigned nrepl;             /**< Number of replicas */
    unsigned ncontinuum;        /* number of continuum points */
    unsigned is3x;              /* whether server 3.0 config semantics are in place */
    int revid;                  /* revision ID from the config (-1 if not present) */
    char *buuid;                /* bucket UUID */
    char *bname;                /* bucket name */
    const char *errstr;         /* last error */
    lcbvb_SERVER *servers;      /* nodes */
    lcbvb_VBUCKET *vbuckets;    /* vbucket map */
    lcbvb_VBUCKET *ffvbuckets;  /* fast-forward map */
    lcbvb_CONTINUUM *continuum; /* ketama continuums */
    int *randbuf;               /* Used for random server selection */
    uint64_t caps;              /**< Bucket capabilities */
    uint64_t ccaps;             /**< Cluster capabilities */
} lcbvb_CONFIG;

#define LCBVB_BUCKET_NAME(cfg) (cfg)->bname
#define LCBVB_NSERVERS(cfg) (cfg)->nsrv
#define LCBVB_NDATASERVERS(cfg) (cfg)->ndatasrv
#define LCBVB_NREPLICAS(cfg) (cfg)->nrepl
#define LCBVB_DISTTYPE(cfg) (cfg)->dtype
#define LCBVB_CAPS(cfg) (cfg)->caps
#define LCBVB_CCAPS(cfg) (cfg)->ccaps
#define LCBVB_GET_SERVER(conf, ix) ((conf)->servers + ix)

/**
 * @uncommitted
 * @brief Allocate a new config
 * This can be used to create new config object and load it with a JSON config,
 * optionally retrieving the error code
 * @code{.c}
 * lcbvb_CONFIG *cfg = lcbvb_create();
 * if (0 != lcbvb_load_json(cfg, json)) {
 *   printf("Got error!", lcbvb_get_error(cfg));
 *   lcbvb_destroy(cfg);
 * }
 * @endcode
 */
LIBCOUCHBASE_API
lcbvb_CONFIG *lcbvb_create(void);

/**
 * @uncommitted
 * Parse the configuration string in `data` and return a new config object
 * @param data
 * @return A new config object, or NULL on error.
 */
LIBCOUCHBASE_API
lcbvb_CONFIG *lcbvb_parse_json(const char *data);

/**
 * @committed
 * Load a JSON-based configuration string into a configuration object
 * @param vbc Object to populate
 * @param data NUL-terminated string to parse
 * @param source hostname of the node, which emitted configuration. Pointer
 *   to NULL will disable heuristic when network argument is pointer to NULL
 *   string.
 * @param network pointer to string, which specified key in alternative
 *   addresses dict. Use pointer NULL string to trigger heuristic, in this
 *   case, the function will try to match configuration source address to
 *   the list of addresses to determine best network.
 * @return 0 on success, nonzero on failure
 * @note it is recommended to use this function rather than lcbvb_parse_json()
 *  as this will contain the error string in the configuration in case of parse
 *  failures.
 */
LIBCOUCHBASE_API
int lcbvb_load_json(lcbvb_CONFIG *vbc, const char *data);

/**
 * @uncommitted
 */
LIBCOUCHBASE_API
int lcbvb_load_json_ex(lcbvb_CONFIG *vbc, const char *data, const char *source, char **network);

/**@brief Serialize the current config as a JSON string.
 * @volatile
 * Serialize the current configuration as a JSON string. The string returned is
 * NUL-terminated and should be freed using the free() function.
 */
LIBCOUCHBASE_API
char *lcbvb_save_json(lcbvb_CONFIG *vbc);

/**
 * @committed
 * @brief Return a string indicating why parsing the configuration failed
 * @return An error string. Do not free this string
 */
LIBCOUCHBASE_API
const char *lcbvb_get_error(const lcbvb_CONFIG *vbc);

/**
 * @volatile
 * @brief Replace hostname placeholders with specific host string
 * This function shall replace hostname placeholists with the actual host string
 * specified in `hoststr`.
 * @param cfg the configuration
 * @param hostname the actual hostname to use.
 *
 * Use this immediately after a successful parsing of the configuration file.
 */
LIBCOUCHBASE_API
void lcbvb_replace_host(lcbvb_CONFIG *cfg, const char *hostname);

/**
 * @committed
 * Destroy the configuration object
 * @param conf
 */
LIBCOUCHBASE_API
void lcbvb_destroy(lcbvb_CONFIG *conf);

/**
 * @committed
 *
 * Gets the master node index for the given vbucket
 * @param cfg The configuration
 * @param vbid The vbucket to query
 * @return The master index. -1 if offline
 * @warning This function does no bounds checking for `vbid`. Ensure it is
 * within range of `0 < vbid < cfg->nvbs`
 */
LIBCOUCHBASE_API
int lcbvb_vbmaster(lcbvb_CONFIG *cfg, int vbid);

/**
 * @committed
 *
 * Return the 0-based replica index for the given vbucket.
 * @param cfg The configuration object
 * @param vbid The vbucket to query
 * @param ix the replica index to retrieve. This is a number ranging from
 *        0 to vbc->nrepl exclusive.
 * @return The replica index, or -1 if offline
 *
 * @warning This function does no bounds checking for `vbix` or `ix`. Ensure
 * that `0 < vbid < cfg->nvbs` and `-1 < ix < cfg->nrepl`
 */
LIBCOUCHBASE_API
int lcbvb_vbreplica(lcbvb_CONFIG *cfg, int vbid, unsigned ix);

/**
 * @volatile
 * This allows to get the given index for a vbucket server. If the index is
 * 0 then this returns the master index, if the index is greater then it
 * returns the replica index
 */
#define lcbvb_vbserver(cfg, vbid, ix) ((ix == 0) ? lcbvb_vbmaster(cfg, vbid) : lcbvb_vbreplica(cfg, vbid, ix - 1))

/**
 * uncommitted
 * Equivalent to
 * @code{.c}
 * lcbvb_nmv_remap_ex(cfg, vbid, bad, 0);
 * @endcode
 */
#define lcbvb_nmv_remap(cfg, vbid, bad) lcbvb_nmv_remap_ex(cfg, vbid, bad, 0)

/**
 * @uncommitted
 *
 * Using various guesswork and heuristics, attempt to locate an alternate node
 * for the master of a given vbucket. This should be used if the master index
 * is -1 or if the master index is deemed incorrect by some other means.
 *
 * @param cfg the configuration object
 * @param vbid the vbucket index to loop up
 * @param bad the index known to be bad. Passing this parameter allows the
 *  handler to safely call this function and be sure that a previous call's
 *  applied heuristics will not affect the modified map.
 * @param use_heuristics whether additional heuristics should be used. If
 *  heuristics is off, only the fast-forward map is employed.
 */
int lcbvb_nmv_remap_ex(lcbvb_CONFIG *cfg, int vbid, int bad, int use_heuristics);

/**
 * @committed
 *
 * Map a given string to a vbucket and server
 * @param cfg The configuration object
 * @param key Key to map
 * @param n Length of key
 * @param[out] vbid Will contain the vBucket
 * @param[out] srvix Will contain the server index
 * @return 0 for now
 */
LIBCOUCHBASE_API
int lcbvb_map_key(lcbvb_CONFIG *cfg, const void *key, lcb_SIZE n, int *vbid, int *srvix);

/**
 * @committed
 *
 * Maps a key to a vBucket ID
 * @param cfg The configuration
 * @param key The key to retrieve
 * @param n The size of the key
 * @return the vBucket for the key
 */
LIBCOUCHBASE_API
int lcbvb_k2vb(lcbvb_CONFIG *cfg, const void *key, lcb_SIZE n);

/**
 * @uncommitted
 * Determines if a given server index is either a master or a replica for a
 * vbucket
 * @param vbc the configuration
 * @param vbid the vbucket number
 * @param ix the server index to check against
 * @returns nonzero if the server `ix` is either a master or a replica for the
 * vbucket `vbid`. Returns 0 otherwise.
 */
LIBCOUCHBASE_API
int lcbvb_has_vbucket(lcbvb_CONFIG *vbc, int vbid, int ix);

/**@committed
 * @brief Get the number of servers in the bucket. Note that not all servers
 * may actually be available.
 * @param cfg The configuration
 * @return The number of servers
 **/
LIBCOUCHBASE_API
unsigned lcbvb_get_nservers(const lcbvb_CONFIG *cfg);

/**@committed
 * @brief Get the number of replicas the bucket is configured with
 * Note that not all replicas may necessarily be online or available.
 * @param cfg the configuration
 * @return the number of configured replicas
 */
LIBCOUCHBASE_API
unsigned lcbvb_get_nreplicas(const lcbvb_CONFIG *cfg);

/**@committed
 * @brief Get the number of vbuckets the bucket is configured with.
 * @return the number of vbuckets, or zero if not applicable
 */
LIBCOUCHBASE_API
unsigned lcbvb_get_nvbuckets(const lcbvb_CONFIG *cfg);

/**@committed
 * @brief Get the distribution mode (AKA bucket type) of the bucket
 * @param cfg the configuration
 * @return the distribution mode
 */
LIBCOUCHBASE_API
lcbvb_DISTMODE lcbvb_get_distmode(const lcbvb_CONFIG *cfg);

/**
 * @committed
 *
 * @brief Get the revision for this configuration.
 *
 * The revision is an
 * integer which is increased each time the cluster generates a new
 * configuration. This feature is available only on configurations generated
 * by nodes of Couchbase Server v2.5 or later.
 *
 * @param cfg the configuration
 * @return The revision ID, or `-1` if the config does not have a revision
 */
LIBCOUCHBASE_API
int lcbvb_get_revision(const lcbvb_CONFIG *cfg);

/**
 * @committed
 * @brief Gets the port associated with a given service of a given mode on a given
 * server
 * @param cfg the config object
 * @param ix the index of the server to query
 * @param type the type of service being provided
 * @param mode the mode of transport being used (e.g. plain, ssl)
 * @return a number greater than zero if the port exists, 0 otherwise
 */
LIBCOUCHBASE_API
unsigned lcbvb_get_port(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode);

/**
 * @committed
 *
 * @brief Return a string for the given service
 * This is like lcbvb_get_port but returns a string in the form of `host:port`
 * rather than the numeric port
 *
 * @param cfg
 * @param ix
 * @param type
 * @param mode
 * @return A string if the service is found, NULL otherwise. The storage
 * duration of the string is valid until the configuration object is
 * destroyed.
 */
LIBCOUCHBASE_API
const char *lcbvb_get_hostport(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode);

/**
 * @committed
 *
 * @brief Get the hostname of a given server index. This may be used if all
 * nodes reside on different hostnames, and can be used to answer the question
 * of "which node does this index belong to" without having to perform
 * additional string processing on the port of the string.
 *
 * @param cfg the configuration
 * @param ix the index of the server to look up
 * @return a hostname without a port, or NULL if the index is out of bounds
 */
LIBCOUCHBASE_API
const char *lcbvb_get_hostname(const lcbvb_CONFIG *cfg, unsigned ix);

/**
 * Function to return the URL prefix for a REST service.
 *
 * Returns a string suitable for being passed as a URL. This is only valid
 * for ::LCBVB_SVCTYPE_VIEWS and ::LCBVB_SVCTYPE_N1QL.
 *
 * This function is different from lcbvb_get_hostport() -- it is mainly a
 * convenience, but does cache the string. Also, theoretically the cluster
 * is free to choose a _different_ URL prefix for a given service. Using this
 * function will guarantee the URL prefix is correct.
 */
LIBCOUCHBASE_API
const char *lcbvb_get_resturl(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode);

/**
 * Convenience function to select a random node for a service.
 * @return 0 or greater if a node was found; a negative number if no node
 * contains a service with the given criteria.
 */
LIBCOUCHBASE_API
int lcbvb_get_randhost(const lcbvb_CONFIG *cfg, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode);

/**
 * Get random node, excluding nodes already tried
 * @param cfg the config
 * @param type type of service
 * @param mode transport mode
 * @param used an array of integers representing server indexes (should be of
 * size LCBVB_NSERVERS). Servers whose indexes in the `used` array are nonzero
 * will be *skipped*.
 *
 * @return a server index, or -1 if no server remains (either because no
 * server has the service, or because all available servers are in the
 * exclude list)
 */
LIBCOUCHBASE_API
int lcbvb_get_randhost_ex(const lcbvb_CONFIG *cfg, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode, int *used);

/** @brief Structure representing changes between two configurations */
typedef struct {
    /** List of strings of servers added (via `host:data_port`) */
    char **servers_added;
    /** List of strings of servers removed (via `host:data_port`) */
    char **servers_removed;
    /** How many vBuckets have had an ownership change */
    int n_vb_changes;
    /** Whether the ordering of the nodes has changed as well */
    int sequence_changed;
} lcbvb_CONFIGDIFF, VBUCKET_CONFIG_DIFF;

/** @brief Convenience enum to determine the mode of change */
typedef enum {
    LCBVB_NO_CHANGES = 0,            /**< No changes between configs */
    LCBVB_SERVERS_MODIFIED = 1 << 0, /**< Servers have been added or removed */
    LCBVB_MAP_MODIFIED = 1 << 1      /**< vBuckets have been transferred */
} lcbvb_CHANGETYPE,
    VBUCKET_CHANGE_STATUS;

/**
 * @volatile
 *
 * @brief Compare two configurations and return information on the changes
 * @param from the original configuration to use as the base
 * @param to the new configuration
 * @return an object which may be inspected, or NULL on allocation failure. The
 *         returned object should be freed with lcbvb_free_diff()
 * @see lcbvb_get_changetype()
 */
LIBCOUCHBASE_API
lcbvb_CONFIGDIFF *lcbvb_compare(lcbvb_CONFIG *from, lcbvb_CONFIG *to);

/** @brief Free the structure returned by lcbvb_compare() */
LIBCOUCHBASE_API
void lcbvb_free_diff(lcbvb_CONFIGDIFF *diff);

/**@brief Get a quick summary of the changes in the passed object
 * @param diff the diff returned from lcbvb_compare()
 */
LIBCOUCHBASE_API
lcbvb_CHANGETYPE lcbvb_get_changetype(lcbvb_CONFIGDIFF *diff);

/**
 * @volatile
 *
 * @brief Generate a sample configuration.
 * @param vb a new configuration object returned via lcbvb_create()
 * @param name the name of the bucket
 * @param uuid UUID for the bucket
 * @param servers an array of server objects which will serve as the basis
 * for the server list within the configuration. The memory pointed to by
 * servers may be released after this function has completed
 * @param nservers number of servers in the array
 * @param nreplica how many replicas for the bucket
 * @param nvbuckets how many vbuckets for the bucket
 */
LIBCOUCHBASE_API
int lcbvb_genconfig_ex(lcbvb_CONFIG *vb, const char *name, const char *uuid, const lcbvb_SERVER *servers,
                       unsigned nservers, unsigned nreplica, unsigned nvbuckets);

/**
 * @volatile
 *
 * @brief Generate a sample configuration used for testing.
 * @param vb a new configuration object returned via lcbvb_create()
 * @param nservers how many nodes to place into the configuration
 * @param nreplica how many replicas should be assigned to the bucket
 * @param nvbuckets how many vbuckets to create
 * @return 0 on success, nonzero on error
 *
 * @note The base port for the lcbvb_SERVICES::data starts at 1000; the base
 * port for lcbvb_SERVICES::views starts at 2000 and the base port for
 * lcbvb_SERVICES::mgmt starts at 3000. The port number is incremented for
 * each additional node.
 */
LIBCOUCHBASE_API
int lcbvb_genconfig(lcbvb_CONFIG *vb, unsigned nservers, unsigned nreplica, unsigned nvbuckets);

/**
 * @volatile
 * Generate a fast-forward vBucket map for the configuration. This simply
 * provides alternate indices.
 */
LIBCOUCHBASE_API
void lcbvb_genffmap(lcbvb_CONFIG *vb);

/**
 * @volatile
 * Convert the configuration to a ketama one.
 * @param vb The configuration object.
 */
LIBCOUCHBASE_API
void lcbvb_make_ketama(lcbvb_CONFIG *vb);

/**
 * @committed
 *
 * Get the views URL base.
 * @param cfg The configuration
 * @param ix The index of the server to fetch
 * @param mode The mode, either plain or ssl
 * @return A string reprenting the URL, or NULL if not available.
 */
LIBCOUCHBASE_API
const char *lcbvb_get_capibase(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCMODE mode);
/**@}*/

/*the rest of these symbols are deprecated and should not be touched by
 * doxygen */

typedef enum { LIBVBUCKET_SOURCE_FILE, LIBVBUCKET_SOURCE_MEMORY } vbucket_source_t;

typedef lcbvb_CONFIG *VBUCKET_CONFIG_HANDLE;
typedef lcbvb_DISTMODE VBUCKET_DISTRIBUTION_TYPE;
#define VBUCKET_DISTRIBUTION_VBUCKET LCBVB_DIST_VBUCKET
#define VBUCKET_DISTRIBUTION_KETAMA LCBVB_DIST_KETAMA
#define VBUCKET_NO_CHANGES LCBVB_NO_CHANGES
#define VBUCKET_SERVERS_MODIFIED LCBVB_SERVERS_MODIFIED
#define VBUCKET_MAP_MODIFIED LCVBVB_MAP_MODIFIED

LIBCOUCHBASE_API int vbucket_config_parse(lcbvb_CONFIG *, vbucket_source_t, const char *);
LIBCOUCHBASE_API const char *vbucket_get_error_message(lcbvb_CONFIG *);
LIBCOUCHBASE_API lcbvb_CONFIG *vbucket_config_create(void);
LIBCOUCHBASE_API void vbucket_config_destroy(lcbvb_CONFIG *);
LIBCOUCHBASE_API int vbucket_config_get_num_replicas(lcbvb_CONFIG *);
LIBCOUCHBASE_API int vbucket_config_get_num_vbuckets(lcbvb_CONFIG *);
LIBCOUCHBASE_API int vbucket_config_get_num_servers(lcbvb_CONFIG *);
LIBCOUCHBASE_API const char *vbucket_config_get_server(lcbvb_CONFIG *, int);
LIBCOUCHBASE_API const char *vbucket_config_get_couch_api_base(lcbvb_CONFIG *, int);
LIBCOUCHBASE_API const char *vbucket_config_get_rest_api_server(lcbvb_CONFIG *, int);
LIBCOUCHBASE_API lcbvb_DISTMODE vbucket_config_get_distribution_type(lcbvb_CONFIG *);
LIBCOUCHBASE_API int vbucket_map(lcbvb_CONFIG *, const void *, lcb_SIZE, int *, int *);
LIBCOUCHBASE_API int vbucket_get_vbucket_by_key(lcbvb_CONFIG *, const void *, lcb_SIZE);
LIBCOUCHBASE_API int vbucket_get_master(lcbvb_CONFIG *, int);
LIBCOUCHBASE_API int vbucket_get_replica(lcbvb_CONFIG *, int, int);
LIBCOUCHBASE_API lcbvb_CONFIGDIFF *vbucket_compare(lcbvb_CONFIG *, lcbvb_CONFIG *);
LIBCOUCHBASE_API void vbucket_free_diff(lcbvb_CONFIGDIFF *);
LIBCOUCHBASE_API int vbucket_config_get_revision(lcbvb_CONFIG *);
LIBCOUCHBASE_API lcbvb_CHANGETYPE vbucket_what_changed(lcbvb_CONFIGDIFF *diff);
LIBCOUCHBASE_API int vbucket_config_generate(lcbvb_CONFIG *vb, unsigned, unsigned, unsigned);

#ifdef __cplusplus
}
#endif
#endif
