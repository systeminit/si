/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018-2019 Couchbase, Inc.
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

#ifndef LIBCOUCHBASE_couchbase_internalstructs_h__
#define LIBCOUCHBASE_couchbase_internalstructs_h__

#include <libcouchbase/utils.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RESET_CMD_BASE(__me)                                                                                           \
    (__me)->cmdflags = 0;                                                                                              \
    (__me)->exptime = 0;                                                                                               \
    (__me)->cas = 0;                                                                                                   \
    (__me)->cid = 0;                                                                                                   \
    (__me)->key.type = LCB_KV_COPY;                                                                                    \
    (__me)->key.contig.bytes = NULL;                                                                                   \
    (__me)->key.contig.nbytes = 0;                                                                                     \
    (__me)->timeout = 0;                                                                                               \
    (__me)->pspan = NULL

#define LCB_CMD_DURABILITY                                                                                             \
    /**                                                                                                                \
     * @uncommitted                                                                                                    \
     * The level of durability required. Supported on Couchbase Server 6.5+                                            \
     */                                                                                                                \
    lcb_DURABILITY_LEVEL dur_level

/**@brief Common ABI header for all commands. _Any_ command may be safely
 * casted to this type.*/
struct lcb_CMDBASE_ {
    LCB_CMD_BASE;
};

/**
 * Flag for lcb_CMDBASE::cmdflags which indicates that the lcb_CMDBASE::cookie
 * is a special callback object. This flag is used internally within the
 * library.
 *
 * @internal
 */
#define LCB_CMD_F_INTERNAL_CALLBACK (1 << 0)

/**
 * If this flag is set, then multiple authentication credentials will be passed
 * to the server. By default only the bucket's credentials (i.e. bucket SASL
 * password) are passed.
 */
#define LCB_CMD_F_MULTIAUTH (1 << 1)

#define LCB_CMD_F_CLONE (1 << 2)

/**@}*/

/**
 * @brief
 * Base response structure for callbacks.
 * All responses structures derive from this ABI.
 */
struct lcb_RESPBASE_ {
    LCB_RESP_BASE
};

/**
 * @committed
 *
 * @brief Set the value buffer for the command. This may be used when the new
 * value is a single contiguous buffer.
 *
 * @param scmd an lcb_CMDSTORE pointer
 * @param valbuf the buffer for the value
 * @param vallen the length of the buffer
 *
 * The buffer needs to remain valid only until the command is passed to the
 * lcb_store3() function.
 */
#define LCB_CMD_SET_VALUE(scmd, valbuf, vallen)                                                                        \
    do {                                                                                                               \
        (scmd)->value.vtype = LCB_KV_COPY;                                                                             \
        (scmd)->value.u_buf.contig.bytes = valbuf;                                                                     \
        (scmd)->value.u_buf.contig.nbytes = vallen;                                                                    \
    } while (0);

/**
 * @committed
 *
 * @brief Set value from a series of input buffers. This may be used when the
 * input buffer is not contiguous. Using this call eliminates the need for
 * creating a temporary contiguous buffer in which to store the value.
 *
 * @param scmd the command which needs a value
 * @param iovs an array of @ref lcb_IOV structures
 * @param niovs number of items in the array.
 *
 * The buffers (and the IOV array itself)
 * need to remain valid only until the scheduler function is called. Once the
 * scheduling function is called, the buffer contents are copied into the
 * library's internal buffers.
 */
#define LCB_CMD_SET_VALUEIOV(scmd, iovs, niovs)                                                                        \
    do {                                                                                                               \
        (scmd)->value.vtype = LCB_KV_IOVCOPY;                                                                          \
        (scmd)->value.u_buf.multi.iov = iovs;                                                                          \
        (scmd)->value.u_buf.multi.niov = niovs;                                                                        \
    } while (0);

/**
 * If this bit is set in lcb_CMDGET::cmdflags then the expiry time is cleared if
 * lcb_CMDGET::exptime is 0. This allows get-and-touch with an expiry of 0.
 */
#define LCB_CMDGET_F_CLEAREXP (1 << 16)

struct lcb_CMDGET_ {
    LCB_CMD_BASE;
    /**If set to true, the `exptime` field inside `options` will take to mean
     * the time the lock should be held. While the lock is held, other operations
     * trying to access the key will fail with an `LCB_ETMPFAIL` error. The
     * item may be unlocked either via `lcb_unlock3()` or via a mutation
     * operation with a supplied CAS
     */
    int lock;
    /** only for get with touch (when expiration set and lock is false) */
    LCB_CMD_DURABILITY;
};

/** @brief Response structure when retrieving a single item */
struct lcb_RESPGET_ {
    LCB_RESP_BASE
    const void *value; /**< Value buffer for the item */
    lcb_SIZE nvalue;   /**< Length of value */
    void *bufh;
    lcb_datatype_t datatype; /**< @internal */
    lcb_U32 itmflags;        /**< User-defined flags for the item */
};

struct lcb_RESPGETREPLICA_ {
    LCB_RESP_BASE
    const void *value; /**< Value buffer for the item */
    lcb_SIZE nvalue;   /**< Length of value */
    void *bufh;
    lcb_datatype_t datatype; /**< @internal */
    lcb_U32 itmflags;        /**< User-defined flags for the item */
};

/**@brief Select get-replica mode
 * @see lcb_rget3_cmd_t */
typedef enum {
    /**Query all the replicas sequentially, retrieving the first successful
     * response */
    LCB_REPLICA_FIRST = 0x00,

    /**Query all the replicas concurrently, retrieving all the responses*/
    LCB_REPLICA_ALL = 0x01,

    /**Query the specific replica specified by the
     * lcb_rget3_cmd_t#index field */
    LCB_REPLICA_SELECT = 0x02
} lcb_replica_t;

/**
 * @brief Command for requesting an item from a replica
 * @note The `options.exptime` and `options.cas` fields are ignored for this
 * command.
 *
 * This structure is similar to @ref lcb_RESPGET with the addition of an
 * `index` and `strategy` field which allow you to control and select how
 * many replicas are queried.
 *
 * @see lcb_rget3(), lcb_RESPGET
 */
struct lcb_CMDGETREPLICA_ {
    LCB_CMD_BASE;
    /**
     * Strategy for selecting a replica. The default is ::LCB_REPLICA_FIRST
     * which results in the client trying each replica in sequence until a
     * successful reply is found, and returned in the callback.
     *
     * ::LCB_REPLICA_FIRST evaluates to 0.
     *
     * Other options include:
     * <ul>
     * <li>::LCB_REPLICA_ALL - queries all replicas concurrently and dispatches
     * a callback for each reply</li>
     * <li>::LCB_REPLICA_SELECT - queries a specific replica indicated in the
     * #index field</li>
     * </ul>
     *
     * @note When ::LCB_REPLICA_ALL is selected, the callback will be invoked
     * multiple times, one for each replica. The final callback will have the
     * ::LCB_RESP_F_FINAL bit set in the lcb_RESPBASE::rflags field. The final
     * response will also contain the response from the last replica to
     * respond.
     */
    lcb_replica_t strategy;

    /**
     * Valid only when #strategy is ::LCB_REPLICA_SELECT, specifies the replica
     * index number to query. This should be no more than `nreplicas-1`
     * where `nreplicas` is the number of replicas the bucket is configured with.
     */
    int index;
};

typedef enum { LCB_DURABILITY_NONE = 0, LCB_DURABILITY_POLL = 1, LCB_DURABILITY_SYNC = 2 } lcb_DURABILITY_MODE;

/**@brief
 *
 * Command for storing an item to the server. This command must contain the
 * key to mutate, the value which should be set (or appended/prepended) in the
 * lcb_CMDSTORE::value field (see LCB_CMD_SET_VALUE()) and the operation indicating
 * the mutation type (lcb_CMDSTORE::operation).
 *
 * @warning #exptime *cannot* be used with #operation set to @ref LCB_STORE_APPEND
 * or @ref LCB_STORE_PREPEND.
 */
struct lcb_CMDSTORE_ {
    LCB_CMD_BASE;

    /**
     * Value to store on the server. The value may be set using the
     * LCB_CMD_SET_VALUE() or LCB_CMD_SET_VALUEIOV() API
     */
    lcb_VALBUF value;

    /**
     * Format flags used by clients to determine the underlying encoding of
     * the value. This value is also returned during retrieval operations in the
     * lcb_RESPGET::itmflags field
     */
    lcb_U32 flags;

    /** Do not set this value for now */
    lcb_U8 datatype;

    /** Controls *how* the operation is perfomed. See the documentation for
     * @ref lcb_storage_t for the options. There is no default value for this
     * field.
     */
    lcb_STORE_OPERATION operation;

    uint8_t durability_mode;

    union {
        struct {
            /**
             * Number of nodes to persist to. If negative, will be capped at the maximum
             * allowable for the current cluster.
             * @see lcb_DURABILITYOPTSv0::persist_to
             */
            char persist_to;

            /**
             * Number of nodes to replicate to. If negative, will be capped at the maximum
             * allowable for the current cluster.
             * @see lcb_DURABILITYOPTSv0::replicate_to
             */
            char replicate_to;
        } poll;
        struct {
            LCB_CMD_DURABILITY;
        } sync;
    } durability;
};

/**
 * @brief Response structure for lcb_store3()
 */
struct lcb_RESPSTORE_ {
    LCB_RESP_BASE

    /** The type of operation which was performed */
    lcb_STORE_OPERATION op;

    /** Internal durability response structure. */
    const lcb_RESPENDURE *dur_resp;

    /**If the #rc field is not @ref LCB_SUCCESS, this field indicates
     * what failed. If this field is nonzero, then the store operation failed,
     * but the durability checking failed. If this field is zero then the
     * actual storage operation failed. */
    int store_ok;
};

/**@brief
 * Command for removing an item from the server
 * @note The lcb_CMDREMOVE::exptime field here does nothing.
 *
 * The lcb_CMDREMOVE::cas field may be
 * set to the last CAS received from a previous operation if you wish to
 * ensure the item is removed only if it has not been mutated since the last
 * retrieval
 */
struct lcb_CMDREMOVE_ {
    LCB_CMD_BASE;
    LCB_CMD_DURABILITY;
};

/**@brief
 * Response structure for removal operation. The lcb_RESPREMOVE::cas  field
 * may be used to check that it no longer exists on any node's storage
 * using the lcb_endure3_ctxnew() function. You can also use the
 * @ref lcb_MUTATION_TOKEN (via lcb_resp_get_mutation_token)
 *
 * The lcb_RESPREMOVE::rc field may be set to ::LCB_KEY_ENOENT if the item does
 * not exist, or ::LCB_KEY_EEXISTS if a CAS was specified and the item has since
 * been mutated.
 */
struct lcb_RESPREMOVE_ {
    LCB_RESP_BASE
};

/**
 * @brief Command structure for a touch request
 * @note The lcb_CMDTOUCH::cas field is ignored. The item's modification time
 * is always updated regardless if the CAS on the server differs.
 * The #exptime field is always used. If 0 then the expiry on the server is
 * cleared.
 */
struct lcb_CMDTOUCH_ {
    LCB_CMD_BASE;
    LCB_CMD_DURABILITY;
};

/**@brief Response structure for a touch request
 * @note the lcb_RESPTOUCH::cas field contains the current CAS of the item*/
struct lcb_RESPTOUCH_ {
    LCB_RESP_BASE
};

/**@brief Command for lcb_unlock3()
 * @attention lcb_CMDBASE::cas must be specified, or the operation will fail on
 * the server*/
struct lcb_CMDUNLOCK_ {
    LCB_CMD_BASE;
};

/**@brief Response structure for an unlock command.
 * @note the lcb_RESPBASE::cas field does not contain the CAS of the item*/
struct lcb_RESPUNLOCK_ {
    LCB_RESP_BASE
};

struct lcb_CMDEXISTS_ {
    LCB_CMD_BASE;
};

struct lcb_RESPEXISTS_ {
    LCB_RESP_BASE
    lcb_U8 state;   /**<Bit set of flags */
};


/**
 * @brief Command for counter operations.
 * @see lcb_counter3(), lcb_RESPCOUNTER.
 *
 * @warning You may only set the #exptime member if the #create member is set
 * to a true value. Setting `exptime` otherwise will cause the operation to
 * fail with @ref LCB_OPTIONS_CONFLICT
 *
 * @warning The #cas member should be set to 0 for this operation. As this
 * operation itself is atomic, specifying a CAS is not necessary.
 */
struct lcb_CMDCOUNTER_ {
    LCB_CMD_BASE;
    /**Delta value. If this number is negative the item on the server is
     * decremented. If this number is positive then the item on the server
     * is incremented */
    lcb_int64_t delta;
    /**If the item does not exist on the server (and `create` is true) then
     * this will be the initial value for the item. */
    lcb_U64 initial;
    /**Boolean value. Create the item and set it to `initial` if it does not
     * already exist */
    int create;

    LCB_CMD_DURABILITY;
};

/**
 * @brief Response structure for counter operations
 * @see lcb_counter3()
 */
struct lcb_RESPCOUNTER_ {
    LCB_RESP_BASE
    /** Contains the _current_ value after the operation was performed */
    lcb_U64 value;
};

/**
 * Command flag for HTTP to indicate that the callback is to be invoked
 * multiple times for each new chunk of incoming data. Once the entire body
 * have been received, the callback will be invoked once more with the
 * LCB_RESP_F_FINAL flag (in lcb_RESPHTTP::rflags) and an empty content.
 *
 * To use streaming requests, this flag should be set in the
 * lcb_CMDHTTP::cmdflags field
 */
#define LCB_CMDHTTP_F_STREAM (1 << 16)

/**
 * @internal
 * If specified, the lcb_CMDHTTP::cas field becomes the timeout for this
 * specific request.
 */
#define LCB_CMDHTTP_F_CASTMO (1 << 17)

/**
 * @internal
 * Do not inject authentication header into the request.
 */
#define LCB_CMDHTTP_F_NOUPASS (1 << 18)

/**
 * Structure for performing an HTTP request.
 * Note that the key and nkey fields indicate the _path_ for the API
 */
struct lcb_CMDHTTP_ {
    LCB_CMD_BASE;
    /**Type of request to issue. LCB_HTTP_TYPE_VIEW will issue a request
     * against a random node's view API. LCB_HTTP_TYPE_MANAGEMENT will issue
     * a request against a random node's administrative API, and
     * LCB_HTTP_TYPE_RAW will issue a request against an arbitrary host. */
    lcb_HTTP_TYPE type;
    lcb_HTTP_METHOD method; /**< HTTP Method to use */

    /** If the request requires a body (e.g. `PUT` or `POST`) then it will
     * go here. Be sure to indicate the length of the body too. */
    const char *body;

    /** Length of the body for the request */
    lcb_SIZE nbody;

    /** If non-NULL, will be assigned a handle which may be used to
     * subsequently cancel the request */
    lcb_HTTP_HANDLE **reqhandle;

    /** For views, set this to `application/json` */
    const char *content_type;

    /** Username to authenticate with, if left empty, will use the credentials
     * passed to lcb_create() */
    const char *username;

    /** Password to authenticate with, if left empty, will use the credentials
     * passed to lcb_create() */
    const char *password;

    /** If set, this must be a string in the form of `http://host:port`. Should
     * only be used for raw requests. */
    const char *host;
};

/**
 * Structure for HTTP responses.
 * Note that #rc being `LCB_SUCCESS` does not always indicate that the HTTP
 * request itself was successful. It only indicates that the outgoing request
 * was submitted to the server and the client received a well-formed HTTP
 * response. Check the #hstatus field to see the actual HTTP-level status
 * code received.
 */
struct lcb_RESPHTTP_ {
    LCB_RESP_BASE
    /**HTTP status code. The value is only valid if #rc is ::LCB_SUCCESS
     * (if #rc is not LCB_SUCCESS then this field may be 0 as the response may
     * have not been read/sent) */
    short htstatus;

    /**List of key-value headers. This field itself may be `NULL`. The list
     * is terminated by a `NULL` pointer to indicate no more headers. */
    const char *const *headers;

    /**If @ref LCB_CMDHTTP_F_STREAM is true, contains the current chunk
     * of response content. Otherwise, contains the entire response body.*/
    const void *body;
    /** Length of buffer in #body */
    lcb_SIZE nbody;
    /**@internal*/
    lcb_HTTP_HANDLE *_htreq;
};

/**
 * Response structure for full-text searches.
 */
struct lcb_RESPFTS_ {
    LCB_RESP_BASE
    /**
     * A query hit, or response metadta
     * (if #rflags contains @ref LCB_RESP_F_FINAL). The format of the row will
     * be JSON, and should be decoded by a JSON decoded in your application.
     */
    const char *row;
    /** Length of #row */
    size_t nrow;
    /** Original HTTP response obejct */
    const lcb_RESPHTTP *htresp;
    lcb_FTS_HANDLE *handle;
};

/**
 * @brief Search Command
 */
struct lcb_CMDFTS_ {
    LCB_CMD_BASE;
    /** Encoded JSON query */
    const char *query;
    /** Length of JSON query */
    size_t nquery;
    /** Callback to be invoked. This must be supplied */
    lcb_FTS_CALLBACK callback;
    /**
     * Optional pointer to store the handle. The handle may then be
     * used for query cancellation via lcb_fts_cancel()
     */
    lcb_FTS_HANDLE **handle;
};

/**
 * Prepare and cache the query if required. This may be used on frequently
 * issued queries, so they perform better.
 */
#define LCB_CMDN1QL_F_PREPCACHE (1 << 16)

/** The lcb_CMDN1QL::query member is an internal JSON structure. @internal */
#define LCB_CMDN1QL_F_JSONQUERY (1 << 17)

/**
 * This is an Analytics query.
 *
 * @committed
 */
#define LCB_CMDN1QL_F_ANALYTICSQUERY (1 << 18)
/* @private an alias for compatibility */
#define LCB_CMDN1QL_F_CBASQUERY LCB_CMDN1QL_F_ANALYTICSQUERY

/**
 * Response for a N1QL query. This is delivered in the @ref lcb_N1QLCALLBACK
 * callback function for each result row received. The callback is also called
 * one last time when all
 */
struct lcb_RESPN1QL_ {
    LCB_RESP_BASE

    /**Current result row. If #rflags has the ::LCB_RESP_F_FINAL bit set, then
     * this field does not contain the actual row, but the remainder of the
     * data not included with the resultset; e.g. the JSON surrounding
     * the "results" field with any errors or metadata for the response.
     */
    const char *row;
    /** Length of the row */
    size_t nrow;
    /** Raw HTTP response, if applicable */
    const lcb_RESPHTTP *htresp;
    lcb_N1QL_HANDLE *handle;
};

/** Set this flag to execute an actual get with each response */
#define LCB_CMDVIEWQUERY_F_INCLUDE_DOCS (1 << 16)

/**Set this flag to only parse the top level row, and not its constituent
 * parts. Note this is incompatible with `F_INCLUDE_DOCS`*/
#define LCB_CMDVIEWQUERY_F_NOROWPARSE (1 << 17)

/**This view is spatial. Modifies how the final view path will be constructed */
#define LCB_CMDVIEWQUERY_F_SPATIAL (1 << 18)

/** Command structure for querying a view */
struct lcb_CMDVIEW_ {
    LCB_CMD_BASE;

    /** The design document as a string; e.g. `"beer"` */
    const char *ddoc;
    /** Length of design document name */
    size_t nddoc;

    /** The name of the view as a string; e.g. `"brewery_beers"` */
    const char *view;
    /** Length of the view name */
    size_t nview;

    /**Any URL parameters to be passed to the view should be specified here.
     * The library will internally insert a `?` character before the options
     * (if specified), so do not place one yourself.
     *
     * The format of the options follows the standard for passing parameters
     * via HTTP requests; thus e.g. `key1=value1&key2=value2`. This string
     * is itself not parsed by the library but simply appended to the URL. */
    const char *optstr;

    /** Length of the option string */
    size_t noptstr;

    /**Some query parameters (in particular; 'keys') may be send via a POST
     * request within the request body, since it might be too long for the
     * URI itself. If you have such data, place it here. */
    const char *postdata;
    size_t npostdata;

    /**
     * The maximum number of internal get requests to issue concurrently for
     * @c F_INCLUDE_DOCS. This is useful for large view responses where
     * there is a potential for a large number of responses resulting in a large
     * number of get requests; increasing memory usage.
     *
     * Setting this value will attempt to throttle the number of get requests,
     * so that no more than this number of requests will be in progress at any
     * given time.
     */
    unsigned docs_concurrent_max;

    /**Callback to invoke for each row. If not provided, @ref LCB_EINVAL will
     * be returned from lcb_view_query() */
    lcb_VIEW_CALLBACK callback;

    /**If not NULL, this will be set to a handle which may be passed to
     * lcb_view_cancel(). See that function for more details */
    lcb_VIEW_HANDLE **handle;
};

/**@brief Response structure representing a row.
 *
 * This is provided for each invocation of the
 * lcb_CMDVIEWQUERY::callback invocation. The `key` and `nkey` fields here
 * refer to the first argument passed to the `emit` function by the
 * `map` function.
 *
 * This response structure may be used as-is, in case the values are simple,
 * or may be relayed over to a more advanced JSON parser to decode the
 * individual key and value properties.
 *
 * @note
 * The #key and #value fields are JSON encoded. This means that if they are
 * bare strings, they will be surrounded by quotes. On the other hand, the
 * #docid is _not_ JSON encoded and is provided with any surrounding quotes
 * stripped out (this is because the document ID is always a string). Please
 * take note of this if doing any form of string comparison/processing.
 *
 * @note
 * If the @ref LCB_CMDVIEWQUERY_F_NOROWPARSE flag has been set, the #value
 * field will contain the raw row contents, rather than the constituent
 * elements.
 *
 */
struct lcb_RESPVIEW_ {
    LCB_RESP_BASE

    const char *docid; /**< Document ID (i.e. memcached key) associated with this row */
    size_t ndocid;     /**< Length of document ID */

    /**Emitted value. If `rflags & LCB_RESP_F_FINAL` is true then this will
     * contain the _metadata_ of the view response itself. This includes the
     * `total_rows` field among other things, and should be parsed as JSON */
    const char *value;

    size_t nvalue; /**< Length of emitted value */

    /**If this is a spatial view, the GeoJSON geometry fields will be here */
    const char *geometry;
    size_t ngeometry;

    /**If the request failed, this will contain the raw underlying request.
     * You may inspect this request and perform some other processing on
     * the underlying HTTP data. Note that this may not necessarily contain
     * the entire response body; just the chunk at which processing failed.*/
    const lcb_RESPHTTP *htresp;

    /**If @ref LCB_CMDVIEWQUERY_F_INCLUDE_DOCS was specified in the request,
     * this will contain the response for the _GET_ command. This is the same
     * response as would be received in the `LCB_CALLBACK_GET` for
     * lcb_get3().
     *
     * Note that this field should be checked for various errors as well, as it
     * is remotely possible the get request did not succeed.
     *
     * If the @ref LCB_CMDVIEWQUERY_F_INCLUDE_DOCS flag was not specified, this
     * field will be `NULL`.
     */
    const lcb_RESPGET *docresp;

    lcb_VIEW_HANDLE *handle;
};

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

/**
 * @brief Sub-Document command codes
 *
 * These command codes should be applied as values to lcb_SDSPEC::sdcmd and
 * indicate which type of subdoc command the server should perform.
 */
typedef enum {
    /**
     * Retrieve the value for a path
     */
    LCB_SDCMD_GET = 1,

    /**
     * Check if the value for a path exists. If the path exists then the error
     * code will be @ref LCB_SUCCESS
     */
    LCB_SDCMD_EXISTS,

    /**
     * Replace the value at the specified path. This operation can work
     * on any existing and valid path.
     */
    LCB_SDCMD_REPLACE,

    /**
     * Add the value at the given path, if the given path does not exist.
     * The penultimate path component must point to an array. The operation
     * may be used in conjunction with @ref LCB_SDSPEC_F_MKINTERMEDIATES to
     * create the parent dictionary (and its parents as well) if it does not
     * yet exist.
     */
    LCB_SDCMD_DICT_ADD,

    /**
     * Unconditionally set the value at the path. This logically
     * attempts to perform a @ref LCB_SDCMD_REPLACE, and if it fails, performs
     * an @ref LCB_SDCMD_DICT_ADD.
     */
    LCB_SDCMD_DICT_UPSERT,

    /**
     * Prepend the value(s) to the array indicated by the path. The path should
     * reference an array. When the @ref LCB_SDSPEC_F_MKINTERMEDIATES flag
     * is specified then the array may be created if it does not exist.
     *
     * Note that it is possible to add more than a single value to an array
     * in an operation (this is valid for this commnand as well as
     * @ref LCB_SDCMD_ARRAY_ADD_LAST and @ref LCB_SDCMD_ARRAY_INSERT). Multiple
     * items can be specified by placing a comma between then (the values should
     * otherwise be valid JSON).
     */
    LCB_SDCMD_ARRAY_ADD_FIRST,

    /**
     * Identical to @ref LCB_SDCMD_ARRAY_ADD_FIRST but places the item(s)
     * at the end of the array rather than at the beginning.
     */
    LCB_SDCMD_ARRAY_ADD_LAST,

    /**
     * Add the value to the array indicated by the path, if the value is not
     * already in the array. The @ref LCB_SDSPEC_F_MKINTERMEDIATES flag can
     * be specified to create the array if it does not already exist.
     *
     * Currently the value for this operation must be a JSON primitive (i.e.
     * no arrays or dictionaries) and the existing array itself must also
     * contain only primitives (otherwise a @ref LCB_SUBDOC_PATH_MISMATCH
     * error will be received).
     */
    LCB_SDCMD_ARRAY_ADD_UNIQUE,

    /**
     * Add the value at the given array index. Unlike other array operations,
     * the path specified should include the actual index at which the item(s)
     * should be placed, for example `array[2]` will cause the value(s) to be
     * the 3rd item(s) in the array.
     *
     * The array must already exist and the @ref LCB_SDSPEC_F_MKINTERMEDIATES
     * flag is not honored.
     */
    LCB_SDCMD_ARRAY_INSERT,

    /**
     * Increment or decrement an existing numeric path. If the number does
     * not exist, it will be created (though its parents will not, unless
     * @ref LCB_SDSPEC_F_MKINTERMEDIATES is specified).
     *
     * The value for this operation should be a valid JSON-encoded integer and
     * must be between `INT64_MIN` and `INT64_MAX`, inclusive.
     */
    LCB_SDCMD_COUNTER,

    /**
     * Remove an existing path in the document.
     */
    LCB_SDCMD_REMOVE,

    /**
     * Count the number of elements in an array or dictionary
     */
    LCB_SDCMD_GET_COUNT,

    /**
     * Retrieve the entire document
     */
    LCB_SDCMD_GET_FULLDOC,

    /**
     * Replace the entire document
     */
    LCB_SDCMD_SET_FULLDOC,

    /**
     * Remove the entire document
     */
    LCB_SDCMD_REMOVE_FULLDOC,

    LCB_SDCMD_MAX
} lcb_SUBDOCOP;

/**
 * @brief Subdoc command specification.
 * This structure describes an operation and its path, and possibly its value.
 * This structure is provided in an array to the lcb_CMDSUBDOC::specs field.
 */
typedef struct {
    /**
     * The command code, @ref lcb_SUBDOCOP. There is no default for this
     * value, and it therefore must be set.
     */
    lcb_U32 sdcmd;

    /**
     * Set of option flags for the command. Currently the only option known
     * is @ref LCB_SDSPEC_F_MKINTERMEDIATES
     */
    lcb_U32 options;

    /**
     * Path for the operation. This should be assigned using
     * @ref LCB_SDSPEC_SET_PATH. The contents of the path should be valid
     * until the operation is scheduled (lcb_subdoc3())
     */
    lcb_KEYBUF path;

    /**
     * Value for the operation. This should be assigned using
     * @ref LCB_SDSPEC_SET_VALUE. The contents of the value should be valid
     * until the operation is scheduled (i.e. lcb_subdoc3())
     */
    lcb_VALBUF value;
} lcb_SDSPEC;

/**
 * Set the path for an @ref lcb_SDSPEC structure
 * @param s pointer to spec
 * @param p the path buffer
 * @param n the length of the path buffer
 */
#define LCB_SDSPEC_SET_PATH(s, p, n)                                                                                   \
    do {                                                                                                               \
        (s)->path.contig.bytes = p;                                                                                    \
        (s)->path.contig.nbytes = n;                                                                                   \
        (s)->path.type = LCB_KV_COPY;                                                                                  \
    } while (0);

/**
 * Set the value for the @ref lcb_SDSPEC structure
 * @param s pointer to spec
 * @param v the value buffer
 * @param n the length of the value buffer
 */
#define LCB_SDSPEC_SET_VALUE(s, v, n) LCB_CMD_SET_VALUE(s, v, n)

#define LCB_SDSPEC_INIT(spec, cmd_, path_, npath_, val_, nval_)                                                        \
    do {                                                                                                               \
        (spec)->sdcmd = cmd_;                                                                                          \
        LCB_SDSPEC_SET_PATH(spec, path_, npath_);                                                                      \
        LCB_CMD_SET_VALUE(spec, val_, nval_);                                                                          \
    } while (0);

#define LCB_SDMULTI_MODE_INVALID 0
#define LCB_SDMULTI_MODE_LOOKUP 1
#define LCB_SDMULTI_MODE_MUTATE 2
/**
 * This command flag should be used if the document is to be created
 * if it does not exist.
 */
#define LCB_CMDSUBDOC_F_UPSERT_DOC (1 << 16)

/**
 * This command flag should be used if the document must be created anew.
 * In this case, it will fail if it already exists
 */
#define LCB_CMDSUBDOC_F_INSERT_DOC (1 << 17)

/**
 * Access a potentially deleted document. For internal Couchbase use
 */
#define LCB_CMDSUBDOC_F_ACCESS_DELETED (1 << 18)

struct lcb_SUBDOCOPS_ {
    uint32_t options;

    lcb_SDSPEC *specs;
    /**
     * Number of entries in #specs
     */
    size_t nspecs;
};

struct lcb_CMDSUBDOC_ {
    LCB_CMD_BASE;

    /**
     * An array of one or more command specifications. The storage
     * for the array need only persist for the duration of the
     * lcb_subdoc3() call.
     *
     * The specs array must be valid only through the invocation
     * of lcb_subdoc3(). As such, they can reside on the stack and
     * be re-used for scheduling multiple commands. See subdoc-simple.cc
     */
    const lcb_SDSPEC *specs;
    /**
     * Number of entries in #specs
     */
    size_t nspecs;
    /**
     * If the scheduling of the command failed, the index of the entry which
     * caused the failure will be written to this pointer.
     *
     * If the value is -1 then the failure took place at the command level
     * and not at the spec level.
     */
    int *error_index;
    /**
     * Operation mode to use. This can either be @ref LCB_SDMULTI_MODE_LOOKUP
     * or @ref LCB_SDMULTI_MODE_MUTATE.
     *
     * This field may be left empty, in which case the mode is implicitly
     * derived from the _first_ command issued.
     */
    lcb_U32 multimode;

    LCB_CMD_DURABILITY;
};

/**
 * Structure for a single sub-document mutation or lookup result.
 * Note that #value and #nvalue are only valid if #status is ::LCB_SUCCESS
 */
typedef struct {
    /** Value for the mutation (only applicable for ::LCB_SDCMD_COUNTER, currently) */
    const void *value;
    /** Length of the value */
    size_t nvalue;
    /** Status code */
    lcb_STATUS status;

    /**
     * Request index which this result pertains to. This field only
     * makes sense for multi mutations where not all request specs are returned
     * in the result
     */
    lcb_U8 index;
} lcb_SDENTRY;

/**
 * Response structure for multi lookups. If the top level response is successful
 * then the individual results may be retrieved using lcb_sdmlookup_next()
 */
struct lcb_RESPSUBDOC_ {
    LCB_RESP_BASE
    const void *responses;
    /** Use with lcb_backbuf_ref/unref */
    void *bufh;
    size_t nres;
    lcb_SDENTRY *res;
};

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-mutation-tokens Mutation Tokens
 *
 * @details Mutation Tokens are returned with mutations if
 * ::LCB_CNTL_FETCH_MUTATION_TOKENS is enabled (off by default). Mutation tokens
 * are largely of internal use, but can be used by N1QL queries and durability
 * requirement polling
 *
 * @addtogroup lcb-mutation-tokens
 * @{
 */

/**
 * @brief
 * Structure representing a synchronization token. This token may be used
 * for durability operations and N1QL queries. The contents of this structure
 * should be considered opaque, and accessed via the various macros
 * @struct lcb_MUTATION_TOKEN
 */

/** Get the vBucket UUID */
#define LCB_MUTATION_TOKEN_ID(p) ((p)->uuid_)
/** Get the sequence number */
#define LCB_MUTATION_TOKEN_SEQ(p) ((p)->seqno_)
/** Get the vBucket number itself */
#define LCB_MUTATION_TOKEN_VB(p) ((p)->vbid_)
/** Whether this mutation token has valid contents */
#define LCB_MUTATION_TOKEN_ISVALID(p) (p && !((p)->uuid_ == 0 && (p)->seqno_ == 0 && (p)->vbid_ == 0))

/**@} (Group: Mutation Tokens) */

/**
 * Ping data (Key/Value) service. Used in lcb_CMDPING#services
 */
#define LCB_PINGSVC_F_KV 0x01

/**
 * Ping query (N1QL) service. Used in lcb_CMDPING#services
 */
#define LCB_PINGSVC_F_N1QL 0x02

/**
 * Ping views (Map/Reduce) service. Used in lcb_CMDPING#services
 */
#define LCB_PINGSVC_F_VIEWS 0x04

/**
 * Ping full text search (FTS) service. Used in lcb_CMDPING#services
 */
#define LCB_PINGSVC_F_FTS 0x08

/**
 * Ping Analytics for N1QL service. Used in lcb_CMDPING#services
 */
#define LCB_PINGSVC_F_ANALYTICS 0x10

/**
 * Do not record any metrics or status codes from ping responses.
 * This might be useful to reduce overhead, when user-space
 * keep-alive mechanism is not interested in actual latencies,
 * but rather need keep sockets active. Used in lcb_CMDPING#options
 */
#define LCB_PINGOPT_F_NOMETRICS 0x01

/**
 * Automatically encode PING result as JSON. See njson/json fields
 * of #lcb_RESPPING structure. Used in lcb_CMDPING#options
 */
#define LCB_PINGOPT_F_JSON 0x02

/**
 * Add extra details about service status into generated JSON.
 * Requires LCB_PINGOPT_F_JSON to be set. Used in lcb_CMDPING#options
 */
#define LCB_PINGOPT_F_JSONDETAILS 0x04

/**
 * Generate indented JSON, which is better for reading. Used in lcb_CMDPING#options
 */
#define LCB_PINGOPT_F_JSONPRETTY 0x08

/**
 * Structure for PING requests.
 *
 * @committed
 */
struct lcb_CMDPING_ {
    LCB_CMD_BASE;
    uint32_t services; /**< bitmap for services to ping */
    uint32_t options;  /**< extra options, e.g. for result representation */
    const char *id;    /**< optional, zero-terminated string to identify the report */
    size_t nid;
};

/**
 * Entry describing the status of the service in the cluster.
 * It is part of lcb_RESPING structure.
 *
 * @committed
 */
typedef struct {
    lcb_PING_SERVICE type; /**< type of the service */
    /* TODO: rename to "remote" */
    const char *server;     /**< server host:port */
    lcb_U64 latency;        /**< latency in nanoseconds */
    lcb_STATUS rc;          /**< raw return code of the operation */
    const char *local;      /**< server host:port */
    const char *id;         /**< service identifier (unique in scope of lcb_INSTANCE *connection instance) */
    const char *scope;      /**< optional scope name (typically equals to the bucket name) */
    lcb_PING_STATUS status; /**< status of the operation */
} lcb_PINGSVC;

/**
 * Structure for PING responses.
 *
 * @committed
 */
struct lcb_RESPPING_ {
    LCB_RESP_BASE
    LCB_RESP_SERVER_FIELDS
    lcb_SIZE nservices;    /**< number of the nodes, replied to ping */
    lcb_PINGSVC *services; /**< the nodes, replied to ping, if any */
    lcb_SIZE njson;        /**< length of JSON string (when #LCB_PINGOPT_F_JSON was specified) */
    const char *json;      /**< pointer to JSON string */
};

struct lcb_CMDDIAG_ {
    LCB_CMD_BASE;
    int options;    /**< extra options, e.g. for result representation */
    const char *id; /**< optional, zero-terminated string to identify the report */
    size_t nid;
};

struct lcb_RESPDIAG_ {
    LCB_RESP_BASE
    lcb_SIZE njson;   /**< length of JSON string (when #LCB_PINGOPT_F_JSON was specified) */
    const char *json; /**< pointer to JSON string */
};

/**
 * Command to fetch collections manifest
 * @uncommitted
 */
struct lcb_CMDGETMANIFEST_ {
    LCB_CMD_BASE;
};

/**
 * Response structure for collection manifest
 * @uncommitted
 */
struct lcb_RESPGETMANIFEST_ {
    LCB_RESP_BASE
    size_t nvalue;
    const char *value;
};

struct lcb_CMDGETCID_ {
    LCB_CMD_BASE;
};

struct lcb_RESPGETCID_ {
    LCB_RESP_BASE
    lcb_U64 manifest_id;
    lcb_U32 collection_id;
};

#define LCB_CMD_CLONE(TYPE, SRC, DST)                                                                                  \
    do {                                                                                                               \
        TYPE *ret = (TYPE *)calloc(1, sizeof(TYPE));                                                                   \
        memcpy(ret, SRC, sizeof(TYPE));                                                                                \
        if (SRC->key.contig.bytes) {                                                                                   \
            ret->key.type = LCB_KV_COPY;                                                                               \
            ret->key.contig.bytes = calloc(SRC->key.contig.nbytes, sizeof(uint8_t));                                   \
            ret->key.contig.nbytes = SRC->key.contig.nbytes;                                                           \
            memcpy((void *)ret->key.contig.bytes, SRC->key.contig.bytes, ret->key.contig.nbytes);                      \
        }                                                                                                              \
        ret->cmdflags |= LCB_CMD_F_CLONE;                                                                              \
        *DST = ret;                                                                                                    \
    } while (0)

#define LCB_CMD_DESTROY_CLONE(cmd)                                                                                     \
    do {                                                                                                               \
        if (cmd->cmdflags & LCB_CMD_F_CLONE) {                                                                         \
            if (cmd->key.contig.bytes && cmd->key.type == LCB_KV_CONTIG) {                                             \
                free((void *)cmd->key.contig.bytes);                                                                   \
            }                                                                                                          \
        }                                                                                                              \
        free(cmd);                                                                                                     \
    } while (0)

#define LCB_CMD_CLONE_WITH_VALUE(TYPE, SRC, DST)                                                                       \
    do {                                                                                                               \
        TYPE *ret = (TYPE *)calloc(1, sizeof(TYPE));                                                                   \
        memcpy(ret, SRC, sizeof(TYPE));                                                                                \
        if (SRC->key.contig.bytes) {                                                                                   \
            ret->key.type = LCB_KV_COPY;                                                                               \
            ret->key.contig.bytes = calloc(SRC->key.contig.nbytes, sizeof(uint8_t));                                   \
            ret->key.contig.nbytes = SRC->key.contig.nbytes;                                                           \
            memcpy((void *)ret->key.contig.bytes, SRC->key.contig.bytes, ret->key.contig.nbytes);                      \
        }                                                                                                              \
        switch (SRC->value.vtype) {                                                                                    \
            case LCB_KV_COPY:                                                                                          \
            case LCB_KV_CONTIG:                                                                                        \
                ret->value.vtype = LCB_KV_COPY;                                                                        \
                ret->value.u_buf.contig.bytes = calloc(SRC->value.u_buf.contig.nbytes, sizeof(uint8_t));               \
                ret->value.u_buf.contig.nbytes = SRC->value.u_buf.contig.nbytes;                                       \
                memcpy((void *)ret->value.u_buf.contig.bytes, SRC->value.u_buf.contig.bytes,                           \
                       ret->value.u_buf.contig.nbytes);                                                                \
                break;                                                                                                 \
            case LCB_KV_IOV:                                                                                           \
            case LCB_KV_IOVCOPY:                                                                                       \
                if (SRC->value.u_buf.multi.iov) {                                                                      \
                    ret->value.vtype = LCB_KV_IOVCOPY;                                                                 \
                    const lcb_FRAGBUF *msrc = &SRC->value.u_buf.multi;                                                 \
                    lcb_FRAGBUF *mdst = &ret->value.u_buf.multi;                                                       \
                    mdst->total_length = 0;                                                                            \
                    mdst->iov = (lcb_IOV *)calloc(msrc->niov, sizeof(lcb_IOV));                                        \
                    for (size_t ii = 0; ii < msrc->niov; ii++) {                                                       \
                        if (msrc->iov[ii].iov_len) {                                                                   \
                            mdst->iov[ii].iov_base = calloc(msrc->iov[ii].iov_len, sizeof(uint8_t));                   \
                            mdst->iov[ii].iov_len = msrc->iov[ii].iov_len;                                             \
                            mdst->total_length += msrc->iov[ii].iov_len;                                               \
                            memcpy(mdst->iov[ii].iov_base, msrc->iov[ii].iov_base, mdst->iov[ii].iov_len);             \
                        }                                                                                              \
                    }                                                                                                  \
                }                                                                                                      \
                break;                                                                                                 \
            default:                                                                                                   \
                free(ret);                                                                                             \
                return LCB_EINVAL;                                                                                     \
                break;                                                                                                 \
        }                                                                                                              \
        ret->cmdflags |= LCB_CMD_F_CLONE;                                                                              \
        *DST = ret;                                                                                                    \
    } while (0)

#define LCB_CMD_DESTROY_CLONE_WITH_VALUE(cmd)                                                                          \
    do {                                                                                                               \
        if (cmd->cmdflags & LCB_CMD_F_CLONE) {                                                                         \
            if (cmd->key.contig.bytes && cmd->key.type == LCB_KV_COPY) {                                               \
                free((void *)cmd->key.contig.bytes);                                                                   \
            }                                                                                                          \
            switch (cmd->value.vtype) {                                                                                \
                case LCB_KV_COPY:                                                                                      \
                case LCB_KV_CONTIG:                                                                                    \
                    free((void *)cmd->value.u_buf.contig.bytes);                                                       \
                    break;                                                                                             \
                case LCB_KV_IOV:                                                                                       \
                case LCB_KV_IOVCOPY:                                                                                   \
                    if (cmd->value.u_buf.multi.iov) {                                                                  \
                        lcb_FRAGBUF *buf = &cmd->value.u_buf.multi;                                                    \
                        for (size_t ii = 0; ii < buf->niov; ii++) {                                                    \
                            if (buf->iov[ii].iov_len) {                                                                \
                                free(buf->iov[ii].iov_base);                                                           \
                            }                                                                                          \
                        }                                                                                              \
                        free(cmd->value.u_buf.multi.iov);                                                              \
                    }                                                                                                  \
                    break;                                                                                             \
                default:                                                                                               \
                    break;                                                                                             \
            }                                                                                                          \
        }                                                                                                              \
        free(cmd);                                                                                                     \
    } while (0)

#ifdef __cplusplus
}
#endif

#endif
