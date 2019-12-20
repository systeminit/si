/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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

#ifndef LCB_UTILS_H
#define LCB_UTILS_H

/**
 * @file
 * Various utility functions
 *
 * @uncommitted
 */

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Set the key for the command.
 * @param cmd A command derived from lcb_CMDBASE
 * @param keybuf the buffer for the key
 * @param keylen the length of the key.
 *
 * @code{.c}
 * lcb_CMDGET cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "key", strlen("key"));
 * @endcode
 *
 * The storage for `keybuf` may be released or modified after the command has
 * been spooled.
 */
#define LCB_CMD_SET_KEY(cmd, keybuf, keylen) LCB_KREQ_SIMPLE(&(cmd)->key, keybuf, keylen)

/**
 * @name Creating Commands
 * @details
 *
 * Issuing a command to the Cluster involves selecting the correct command
 * structure, populating it with the data relevant for the command, optionally
 * associating the command with your own application data, issuing the command
 * to a spooling function, and finally receiving the response.
 *
 * Command structures all derive from the common @ref lcb_CMDBASE structure. This
 * structure defines the common fields for all commands.
 *
 * Almost all commands need to contain a key, which should be assigned using
 * the LCB_CMD_SET_KEY() macro.
 *
 * @{*/

#define LCB_CMD_BASE                                                                                                   \
    /**Common flags for the command. These modify the command itself. Currently                                        \
     the lower 16 bits of this field are reserved, and the higher 16 bits are                                          \
     used for individual commands.*/                                                                                   \
    lcb_U32 cmdflags;                                                                                                  \
                                                                                                                       \
    /**Specify the expiration time. This is either an absolute Unix time stamp                                         \
     or a relative offset from now, in seconds. If the value of this number                                            \
     is greater than the value of thirty days in seconds, then it is a Unix                                            \
     timestamp.                                                                                                        \
                                                                                                                       \
     This field is used in mutation operations (lcb_store3()) to indicate                                              \
     the lifetime of the item. It is used in lcb_get3() with the lcb_CMDGET::lock                                      \
     option to indicate the lock expiration itself. */                                                                 \
    lcb_U32 exptime;                                                                                                   \
                                                                                                                       \
    /**The known CAS of the item. This is passed to mutation to commands to                                            \
     ensure the item is only changed if the server-side CAS value matches the                                          \
     one specified here. For other operations (such as lcb_CMDENDURE) this                                             \
     is used to ensure that the item has been persisted/replicated to a number                                         \
     of servers with the value specified here. */                                                                      \
    lcb_U64 cas;                                                                                                       \
                                                                                                                       \
    /**< Collection ID */                                                                                              \
    lcb_U32 cid;                                                                                                       \
    const char *scope;                                                                                                 \
    size_t nscope;                                                                                                     \
    const char *collection;                                                                                            \
    size_t ncollection;                                                                                                \
    /**The key for the document itself. This should be set via LCB_CMD_SET_KEY() */                                    \
    lcb_KEYBUF key;                                                                                                    \
                                                                                                                       \
    /** Operation timeout (in microseconds). When zero, the library will use default value. */                         \
    lcb_U32 timeout;                                                                                                   \
    /** Parent tracing span */                                                                                         \
    lcbtrace_SPAN *pspan

/**
 * @name Receiving Responses
 * @details
 *
 * This section describes the APIs used in receiving responses.
 *
 * Each command will have a callback invoked (typically once, for some commands
 * this may be more than once) with a response structure. The response structure
 * will be of a type that extends lcb_RESPBASE. The response structure should
 * not be modified and any of its fields should be considered to point to memory
 * which will be released after the callback exits.
 *
 * The common response header contains the lcb_RESPBASE::cookie field which
 * is the pointer to your application context (passed as the second argument
 * to the spooling function) and allows you to associate a specific command
 * with a specific response.
 *
 * The header will also contain the key (lcb_RESPBASE::key) field which can
 * also help identify the specific command. This is useful if you maintain a
 * single _cookie_ for multiple commands, and have per-item specific data
 * you wish to associate within the _cookie_ itself.
 *
 * Success or failure of the operation is signalled through the lcb_RESPBASE::rc
 * field. Note that even in the case of failure, the lcb_RESPBASE::cookie and
 * lcb_RESPBASE::key fields will _always_ be populated.
 *
 * Most commands also return the CAS of the item (as it exists on the server)
 * and this is placed inside the lcb_RESPBASE::cas field, however it is
 * only valid in the case where lcb_RESPBASE::rc is LCB_SUCCESS.
 *
 * @{
 */

#define LCB_RESP_BASE                                                                                                  \
    /**                                                                                                                \
     Application-defined pointer passed as the `cookie` parameter when                                                 \
     scheduling the command.                                                                                           \
     */                                                                                                                \
    void *cookie;                                                                                                      \
    const void *key; /**< Key for request */                                                                           \
    lcb_SIZE nkey;   /**< Size of key */                                                                               \
    lcb_CAS cas;     /**< CAS for response (if applicable) */                                                          \
    lcb_STATUS rc;   /**< Status code */                                                                               \
    lcb_U16 version; /**< ABI version for response */                                                                  \
    /** Response specific flags. see ::lcb_RESPFLAGS */                                                                \
    lcb_U16 rflags;

#define LCB_RESP_SERVER_FIELDS                                                                                         \
    /** String containing the `host:port` of the server which sent this response */                                    \
    const char *server;

/**
 * @brief Base structure for informational commands from servers
 * This contains an additional lcb_RESPSERVERBASE::server field containing the
 * server which emitted this response.
 */
typedef struct {
    LCB_RESP_BASE
    LCB_RESP_SERVER_FIELDS
} lcb_RESPSERVERBASE;

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-mctx MULTICMD API
 * @addtogroup lcb-mctx
 * @{
 */
/**
 * Multi Command Context API
 * Some commands (notably, OBSERVE and its higher level equivalent, endue)
 * are handled more efficiently at the cluster side by stuffing multiple
 * items into a single packet.
 *
 * This structure defines three function pointers to invoke. The #addcmd()
 * function will add a new command to the current packet, the #done()
 * function will schedule the packet(s) into the current scheduling context
 * and the #fail() function will destroy the context without progressing
 * further.
 *
 * Some commands will return an lcb_MULTICMD_CTX object to be used for this
 * purpose:
 *
 * @code{.c}
 * lcb_MUTLICMD_CTX *ctx = lcb_observe3_ctxnew(instance);
 *
 * lcb_CMDOBSERVE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "key1", strlen("key1"));
 * ctx->addcmd(ctx, &cmd);
 * LCB_CMD_SET_KEY(&cmd.key, "key2", strlen("key2"));
 * ctx->addcmd(ctx, &cmd);
 * LCB_CMD_SET_KEY(&cmd.key, "key3", strlen("key3"));
 * ctx->addcmd(ctx, &cmd);
 *
 * ctx->done(ctx);
 * lcb_wait(instance);
 * @endcode
 */
typedef struct lcb_MULTICMD_CTX_st {
    /**
     * Add a command to the current context
     * @param ctx the context
     * @param cmd the command to add. Note that `cmd` may be a subclass of lcb_CMDBASE
     * @return LCB_SUCCESS, or failure if a command could not be added.
     */
    lcb_STATUS (*addcmd)(struct lcb_MULTICMD_CTX_st *ctx, const lcb_CMDBASE *cmd);

    /**
     * Indicate that no more commands are added to this context, and that the
     * context should assemble the packets and place them in the current
     * scheduling context
     * @param ctx The multi context
     * @param cookie The cookie for all commands
     * @return LCB_SUCCESS if scheduled successfully, or an error code if there
     * was a problem constructing the packet(s).
     */
    lcb_STATUS (*done)(struct lcb_MULTICMD_CTX_st *ctx, const void *cookie);

    /**
     * Indicate that no more commands should be added to this context, and that
     * the context should not add its contents to the packet queues, but rather
     * release its resources. Called if you don't want to actually perform
     * the operations.
     * @param ctx
     */
    void (*fail)(struct lcb_MULTICMD_CTX_st *ctx);

    /**
     * Associate parent tracing span with the group operation
     *
     * @param ctx The multi context
     * @param span Parent span
     */
    void (*setspan)(struct lcb_MULTICMD_CTX_st *ctx, lcbtrace_SPAN *span);
} lcb_MULTICMD_CTX;
/**@}*/

/**
 * @ingroup lcb-kv-api
 * @defgroup lcb-durability Durability
 * @brief Ensure persistence and mutation of documents
 * @addtogroup lcb-durability
 * @{
 */

/**
 * @name Wait for a mutation to be persisted/replicated
 * @{
 */

/**
 * Type of durability polling to use.
 */
typedef enum {
    /**
     * Use the preferred durability. If ::LCB_CNTL_FETCH_MUTATION_TOKENS is
     * enabled and the server version is 4.0 or greater then ::LCB_DURABILITY_MODE_SEQNO
     * is used. Otherwise ::LCB_DURABILITY_MODE_CAS is used.
     */
    LCB_DURABILITY_MODE_DEFAULT = 0,

    /**
     * Explicitly request CAS-based durability. This is done by checking the
     * CAS of the item on each server with the item specified in the input.
     * The durability operation is considered complete when all items' CAS
     * values match. If the CAS value on the master node changes then the
     * durability operation will fail with ::LCB_KEY_EEXISTS.
     *
     * @note
     * CAS may change either because of a failover or because of another
     * subsequent mutation. These scenarios are possible (though unlikely).
     * The ::LCB_DURABILITY_MODE_SEQNO mode is not subject to these constraints.
     */
    LCB_DURABILITY_MODE_CAS,

    /**
     * Use sequence-number based polling. This is done by checking the current
     * "mutation sequence number" for the given mutation. To use this mode
     * either an explicit @ref lcb_MUTATION_TOKEN needs to be passed, or
     * the ::LCB_CNTL_DURABILITY_MUTATION_TOKENS should be set, allowing
     * the caching of the latest mutation token for each vBucket.
     */
    LCB_DURABILITY_MODE_SEQNO
} lcb_DURMODE;

/** @brief Options for lcb_endure3_ctxnew() */
typedef struct {
    /**
     * Upper limit in microseconds from the scheduling of the command. When
     * this timeout occurs, all remaining non-verified keys will have their
     * callbacks invoked with @ref LCB_ETIMEDOUT.
     *
     * If this field is not set, the value of @ref LCB_CNTL_DURABILITY_TIMEOUT
     * will be used.
     */
    lcb_U32 timeout;

    /**
     * The durability check may involve more than a single call to observe - or
     * more than a single packet sent to a server to check the key status. This
     * value determines the time to wait (in microseconds)
     * between multiple probes for the same server.
     * If not set, the @ref LCB_CNTL_DURABILITY_INTERVAL will be used
     * instead.
     */
    lcb_U32 interval;

    /**
     * how many nodes the key should be persisted to (including master).
     * If set to 0 then persistence will not be checked. If set to a large
     * number (i.e. UINT16_MAX) and #cap_max is also set, will be set to the
     * maximum number of nodes to which persistence is possible (which will
     * always contain at least the master node).
     *
     * The maximum valid value for this field is
     * 1 + the total number of configured replicas for the bucket which are part
     * of the cluster. If this number is higher then it will either be
     * automatically capped to the maximum available if (#cap_max is set) or
     * will result in an ::LCB_DURABILITY_ETOOMANY error.
     */
    lcb_U16 persist_to;

    /**
     * how many nodes the key should be persisted to (excluding master).
     * If set to 0 then replication will not be checked. If set to a large
     * number (i.e. UINT16_MAX) and #cap_max is also set, will be set to the
     * maximum number of nodes to which replication is possible (which may
     * be 0 if the bucket is not configured for replicas).
     *
     * The maximum valid value for this field is the total number of configured
     * replicas which are part of the cluster. If this number is higher then
     * it will either be automatically capped to the maximum available
     * if (#cap_max is set) or will result in an ::LCB_DURABILITY_ETOOMANY
     * error.
     * */
    lcb_U16 replicate_to;

    /**
     * this flag inverts the sense of the durability check and ensures that
     * the key does *not* exist. This should be used if checking durability
     * after an lcb_remove3() operation.
     */
    lcb_U8 check_delete;

    /**
     * If replication/persistence requirements are excessive, cap to
     * the maximum available
     */
    lcb_U8 cap_max;

    /**
     * Set the polling method to use.
     * The value for this field should be one of the @ref lcb_DURMODE constants.
     */
    lcb_U8 pollopts;
} lcb_DURABILITYOPTSv0;

/**@brief Options for lcb_endure3_ctxnew() (wrapper)
 * @see lcb_DURABILITYOPTSv0 */
typedef struct lcb_durability_opts_st {
    int version;
    union {
        lcb_DURABILITYOPTSv0 v0;
    } v;
} lcb_durability_opts_t;

/**Must specify this flag if using the 'mutation_token' field, as it was added in
 * a later version */
#define LCB_CMDENDURE_F_MUTATION_TOKEN (1 << 16)

/**@brief Command structure for endure.
 * If the lcb_CMDENDURE::cas field is specified, the operation will check and
 * verify that the CAS found on each of the nodes matches the CAS specified
 * and only then consider the item to be replicated and/or persisted to the
 * nodes. If the item exists on the master's cache with a different CAS then
 * the operation will fail
 */
typedef struct {
    LCB_CMD_BASE;
    const lcb_MUTATION_TOKEN *mutation_token;
} lcb_CMDENDURE;

/**@brief Response structure for endure */
typedef struct {
    LCB_RESP_BASE
    /**
     * Total number of polls (i.e. how many packets per server) did this
     * operation require
     */
    lcb_U16 nresponses;

    /**
     * Whether this item exists in the master in its current form. This can be
     * true even if #rc is not successful
     */
    lcb_U8 exists_master;

    /**
     * True if item was persisted on the master node. This may be true even if
     * #rc is not successful.
     */
    lcb_U8 persisted_master;

    /**
     * Total number of nodes (including master) on which this mutation has
     * been persisted. Valid even if #rc is not successful.
     */
    lcb_U8 npersisted;

    /**
     * Total number of replica nodes to which this mutation has been replicated.
     * Valid even if #rc is not successful.
     */
    lcb_U8 nreplicated;
} lcb_RESPENDURE;

/**
 * @committed
 *
 * @details
 * Ensure a key is replicated to a set of nodes
 *
 * The lcb_endure3_ctxnew() API is used to wait asynchronously until the item
 * have been persisted and/or replicated to at least the number of nodes
 * specified in the durability options.
 *
 * The command is implemented by sending a series of `OBSERVE` broadcasts
 * (see lcb_observe3_ctxnew()) to all the nodes in the cluster which are either
 * master or replica for a specific key. It polls repeatedly
 * (see lcb_DURABILITYOPTSv0::interval) until all the items have been persisted and/or
 * replicated to the number of nodes specified in the criteria, or the timeout
 * (aee lcb_DURABILITYOPTsv0::timeout) has been reached.
 *
 * The lcb_DURABILITYOPTSv0::persist_to and lcb_DURABILITYOPTS::replicate_to
 * control the number of nodes the item must be persisted/replicated to
 * in order for the durability polling to succeed.
 *
 * @brief Return a new command context for scheduling endure operations
 * @param instance the instance
 * @param options a structure containing the various criteria needed for
 * durability requirements
 * @param[out] err Error code if a new context could not be created
 * @return the new context, or NULL on error. Note that in addition to memory
 * allocation failure, this function might also return NULL because the `options`
 * structure contained bad values. Always check the `err` result.
 *
 * @par Scheduling Errors
 * The following errors may be encountered when scheduling:
 *
 * @cb_err ::LCB_DURABILITY_ETOOMANY if either lcb_DURABILITYOPTS::persist_to or
 * lcb_DURABILITYOPTS::replicate_to is too big. `err` may indicate this.
 * @cb_err ::LCB_DURABILITY_NO_MUTATION_TOKENS if no relevant mutation token
 * could be found for a given command (this is returned from the relevant
 * lcb_MULTICMD_CTX::addcmd call).
 * @cb_err ::LCB_DUPLICATE_COMMANDS if using CAS-based durability and the
 * same key was submitted twice to lcb_MULTICMD_CTX::addcmd(). This error is
 * returned from lcb_MULTICMD_CTX::done()
 *
 * @par Callback Errors
 * The following errors may be returned in the callback:
 * @cb_err ::LCB_ETIMEDOUT if the criteria could not be verified within the
 * accepted timeframe (see lcb_DURABILITYOPTSv0::timeout)
 * @cb_err ::LCB_KEY_EEXISTS if using CAS-based durability and the item's
 * CAS has been changed on the master node
 * @cb_err ::LCB_MUTATION_LOST if using sequence-based durability and the
 * server has detected that data loss has occurred due to a failover.
 *
 * @par Creating request context
 * @code{.c}
 * lcb_durability_opts_t dopts;
 * lcb_STATUS rc;
 * memset(&dopts, 0, sizeof dopts);
 * dopts.v.v0.persist_to = -1;
 * dopts.v.v0.replicate_to = -1;
 * dopts.v.v0.cap_max = 1;
 * mctx = lcb_endure3_ctxnew(instance, &dopts, &rc);
 * // Check mctx != NULL and rc == LCB_SUCCESS
 * @endcode
 *
 * @par Adding keys - CAS
 * This can be used to add keys using CAS-based durability. This shows an
 * example within a store callback.
 * @code{.c}
 * lcb_RESPSTORE *resp = get_store_response();
 * lcb_CMDENDURE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, resp->key, resp->nkey);
 * cmd.cas = resp->cas;
 * rc = mctx->addcmd(mctx, (const lcb_CMDBASE*)&cmd);
 * rc = mctx->done(mctx, cookie);
 * @endcode
 *
 * @par Adding keys - explicit sequence number
 * Shows how to use an explicit sequence number as a basis for polling
 * @code{.c}
 * // during instance creation:
 * lcb_cntl_string(instance, "fetch_mutation_tokens", "true");
 * lcb_connect(instance);
 * // ...
 * lcb_RESPSTORE *resp = get_store_response();
 * lcb_CMDENDURE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, resp->key, resp->nkey);
 * cmd.mutation_token = &resp->mutation_token;
 * cmd.cmdflags |= LCB_CMDENDURE_F_MUTATION_TOKEN;
 * rc = mctx->addcmd(mctx, (const lcb_CMDBASE*)&cmd);
 * rc = mctx->done(mctx, cookie);
 * @endcode
 *
 * @par Adding keys - implicit sequence number
 * Shows how to use an implicit mutation token (internally tracked by the
 * library) for durability:
 * @code{.c}
 * // during instance creation
 * lcb_cntl_string(instance, "fetch_mutation_tokens", "true");
 * lcb_cntl_string(instance, "dur_mutation_tokens", "true");
 * lcb_connect(instance);
 * // ...
 * lcb_CMDENDURE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "key", strlen("key"));
 * mctx->addcmd(mctx, (const lcb_CMDBASE*)&cmd);
 * mctx->done(mctx, cookie);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_ENDURE, dur_callback);
 * void dur_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPENDURE *resp = (const lcb_RESPENDURE*)rb;
 *     printf("Durability polling result for %.*s.. ", (int)resp->nkey, resp->key);
 *     if (resp->rc != LCB_SUCCESS) {
 *         printf("Failed: %s\n", lcb_strerror(NULL, resp->rc);
 *         return;
 *     }
 * }
 * @endcode
 */
LIBCOUCHBASE_API
lcb_MULTICMD_CTX *lcb_endure3_ctxnew(lcb_INSTANCE *instance, const lcb_durability_opts_t *options, lcb_STATUS *err);

#define LCB_DURABILITY_VALIDATE_CAPMAX (1 << 1)

/**
 * @committed
 *
 * Validate durability settings.
 *
 * This function will validate (and optionally modify) the settings. This is
 * helpful to ensure the durability options are valid _before_ issuing a command
 *
 * @param instance the instance
 *
 * @param[in,out] persist_to The desired number of servers to persist to.
 *  If the `CAPMAX` flag is set, on output this will contain the effective
 *  number of servers the item can be persisted to
 *
 * @param[in,out] replicate_to The desired number of servers to replicate to.
 *  If the `CAPMAX` flag is set, on output this will contain the effective
 *  number of servers the item can be replicated to
 *
 * @param options Options to use for validating. The only recognized option
 *  is currently `LCB_DURABILITY_VALIDATE_CAPMAX` which has the same semantics
 *  as lcb_DURABILITYOPTSv0::cap_max.
 *
 * @return LCB_SUCCESS on success
 * @return LCB_DURABILITY_ETOOMANY if the requirements exceed the number of
 *  servers currently available, and `CAPMAX` was not specifie
 * @return LCB_EINVAL if both `persist_to` and `replicate_to` are 0.
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_durability_validate(lcb_INSTANCE *instance, lcb_U16 *persist_to, lcb_U16 *replicate_to, int options);

/**@} (NAME) */

/**
 * @name Retrieve current persistence/replication status
 * @{
 */

/**Set this bit in the cmdflags field to indicate that only the master node
 * should be contacted*/
#define LCB_CMDOBSERVE_F_MASTER_ONLY (1 << 16)

/**@brief Structure for an observe request.
 * To request the status from _only_ the master node of the key, set the
 * LCB_CMDOBSERVE_F_MASTERONLY bit inside the lcb_CMDOBSERVE::cmdflags field
 */
typedef struct {
    LCB_CMD_BASE;
    /**For internal use: This determines the servers the command should be
     * routed to. Each entry is an index within the server. */
    const lcb_U16 *servers_;
    size_t nservers_;
} lcb_CMDOBSERVE;

/**
 * @brief Possible statuses for keys in OBSERVE response
 */
typedef enum {
    /** The item found in the memory, but not yet on the disk */
    LCB_OBSERVE_FOUND = 0x00,
    /** The item hit the disk */
    LCB_OBSERVE_PERSISTED = 0x01,
    /** The item missing on the disk and the memory */
    LCB_OBSERVE_NOT_FOUND = 0x80,
    /** No knowledge of the key :) */
    LCB_OBSERVE_LOGICALLY_DELETED = 0x81,

    LCB_OBSERVE_MAX = 0x82
} lcb_observe_t;

/**@brief Response structure for an observe command.
 * Note that the lcb_RESPOBSERVE::cas contains the CAS of the item as it is
 * stored within that specific server. The CAS may be incorrect or stale
 * unless lcb_RESPOBSERVE::ismaster is true.
 */
typedef struct {
    LCB_RESP_BASE
    lcb_U8 status;   /**<Bit set of flags */
    lcb_U8 ismaster; /**< Set to true if this response came from the master node */
    lcb_U32 ttp;     /**<Unused. For internal requests, contains the server index */
    lcb_U32 ttr;     /**<Unused */
} lcb_RESPOBSERVE;

/**
 * @brief Create a new multi context for an observe operation
 * @param instance the instance
 * @return a new multi command context, or NULL on allocation failure.
 * @committed
 *
 * Note that the callback for this command will be invoked multiple times,
 * one for each node. To determine when no more callbacks will be invoked,
 * check for the LCB_RESP_F_FINAL flag inside the lcb_RESPOBSERVE::rflags
 * field.
 *
 * @code{.c}
 * void callback(lcb_INSTANCE, lcb_CALLBACK_TYPE, const lcb_RESPOBSERVE *resp)
 * {
 *   if (resp->rflags & LCB_RESP_F_FINAL) {
 *     return;
 *   }
 *
 *   printf("Got status for key %.*s\n", (int)resp->nkey, resp->key);
 *   printf("  Node Type: %s\n", resp->ismaster ? "MASTER" : "REPLICA");
 *   printf("  Status: 0x%x\n", resp->status);
 *   printf("  Current CAS: 0x%"PRIx64"\n", resp->cas);
 * }
 *
 * lcb_MULTICMD_CTX *mctx = lcb_observe3_ctxnew(instance);
 * lcb_CMDOBSERVE cmd = { 0 };
 * LCB_CMD_SET_KEY(&cmd, "key", 3);
 * mctx->addcmd(mctx, (lcb_CMDBASE *)&cmd);
 * mctx->done(mctx, cookie);
 * lcb_install_callback3(instance, LCB_CALLBACK_OBSERVE, (lcb_RESP_cb)callback);
 * @endcode
 *
 * @warning
 * Operations created by observe cannot be undone using lcb_sched_fail().
 */
LIBCOUCHBASE_API
lcb_MULTICMD_CTX *lcb_observe3_ctxnew(lcb_INSTANCE *instance);

/**
 * @brief Command structure for lcb_observe_seqno3().
 * Note #key, #nkey, and #cas are not used in this command.
 */
typedef struct {
    LCB_CMD_BASE;
    /**
     * Server index to target. The server index must be valid and must also
     * be either a master or a replica for the vBucket indicated in #vbid
     */
    lcb_U16 server_index;
    lcb_U16 vbid; /**< vBucket ID to query */
    lcb_U64 uuid; /**< UUID known to client which should be queried */
} lcb_CMDOBSEQNO;

/**
 * @brief Response structure for lcb_observe_seqno3()
 *
 * Note that #key, #nkey and #cas are empty because the operand is the relevant
 * mutation token fields in @ref lcb_CMDOBSEQNO
 */
typedef struct {
    LCB_RESP_BASE
    lcb_U16 vbid;            /**< vBucket ID (for potential mapping) */
    lcb_U16 server_index;    /**< Input server index */
    lcb_U64 cur_uuid;        /**< UUID for this vBucket as known to the server */
    lcb_U64 persisted_seqno; /**< Highest persisted sequence */
    lcb_U64 mem_seqno;       /**< Highest known sequence */

    /**
     * In the case where the command's uuid is not the most current, this
     * contains the last known UUID
     */
    lcb_U64 old_uuid;

    /**
     * If #old_uuid is nonzero, contains the highest sequence number persisted
     * in the #old_uuid snapshot.
     */
    lcb_U64 old_seqno;
} lcb_RESPOBSEQNO;

/**
 * @volatile
 * @brief Get the persistence/replication status for a given mutation token
 * @param instance the handle
 * @param cookie callback cookie
 * @param cmd the command
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_observe_seqno3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDOBSEQNO *cmd);

/**@} (Name: OBSERVE) */

/**
 * Retrieves the mutation token from the response structure
 * @param cbtype the type of callback invoked
 * @param rb the pointer to the response
 * @return The embedded mutation token, or NULL if the response does not have a
 *         mutation token. This may be either because the command does not support
 *         mutation tokens, or because they have been disabled at the connection
 *         level.
 */
LIBCOUCHBASE_API
const lcb_MUTATION_TOKEN *lcb_resp_get_mutation_token(int cbtype, const lcb_RESPBASE *rb);

/**
 * @volatile
 *
 * Retrieves the last mutation token for a given key.
 * This relies on the @ref LCB_CNTL_DURABILITY_MUTATION_TOKENS option, and will
 * check the instance-level log to determine the latest MUTATION_TOKEN for
 * the given vBucket ID which the key maps to.
 *
 * @param instance the instance
 * @param kb The buffer representing the key. The type of the buffer (see
 * lcb_KEYBUF::type) may either be ::LCB_KV_COPY or ::LCB_KV_VBID
 * @param[out] errp Set to an error if this function returns NULL
 * @return The mutation token if successful, otherwise NULL.
 *
 * Getting the latest mutation token for a key:
 *
 * @code{.c}
 * lcb_KEYBUF kb;
 * kb.type = LCB_KV_COPY;
 * kb.contig.bytes = "Hello";
 * kv.config.nbytes = 5;
 * mt = lcb_get_mutation_token(instance, &kb, &rc);
 * @endcode
 *
 * Getting the latest mutation token for a vbucket:
 * @code{.c}
 * lcb_KEYBUF kb;
 * kv.type = LCB_KV_VBID;
 * kv.contig.nbytes = 543;
 * kv.config.bytes = NULL;
 * mt = lcb_get_mutation_token(instance, &kb, &rc);
 * @endcode
 *
 * Getting the mutation token for each vbucket
 * @code{.c}
 * size_t ii, nvb;
 * lcbvb_CONFIG *vbc;
 * lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
 * nvb = vbucket_get_num_vbuckets(vbc);
 * for (ii = 0; ii < nvb; ii++) {
 *   lcb_KEYBUF kb;
 *   const lcb_MUTATION_TOKEN *mt;
 *   kb.type = LCB_KV_VBID;
 *   kb.contig.nbytes = ii;
 *   kb.config.bytes = NULL;
 *   mt = lcb_get_mutation_token(instance, &kb, &rc);
 * }
 * @endcode
 */
LIBCOUCHBASE_API
const lcb_MUTATION_TOKEN *lcb_get_mutation_token(lcb_INSTANCE *instance, const lcb_KEYBUF *kb, lcb_STATUS *errp);

/**@} (Group: Durability) */

/**@ingroup lcb-public-api
 * @defgroup lcb-misc-cmds Miscellaneous Commands
 * @brief Additional miscellaneous commands which can be executed on the server.
 *
 * @addtogroup lcb-misc-cmds
 * @{
 */

/**
 * @name Server Statistics
 * @{
 */

/**
 * @brief Command structure for stats request
 * The lcb_CMDSTATS::key field should contain the statistics key, or be empty
 * if the default statistics are desired.
 * The #cmdflags field may contain the @ref LCB_CMDSTATS_F_KV flag.
 */
typedef struct {
    LCB_CMD_BASE;
} lcb_CMDSTATS;

/**
 * The key is a stored item for which statistics should be retrieved. This
 * invokes the 'keystats' semantics. Note that when using _keystats_, a key
 * must be present, and must not have any spaces in it.
 */
#define LCB_CMDSTATS_F_KV (1 << 16)

/**@brief Response structure for cluster statistics.
 * The lcb_RESPSTATS::key field contains the statistic name (_not_ the same
 * as was passed in lcb_CMDSTATS::key which is the name of the statistical
 * _group_).*/
typedef struct {
    LCB_RESP_BASE
    LCB_RESP_SERVER_FIELDS
    const char *value; /**< The value, if any, for the given statistic */
    lcb_SIZE nvalue;   /**< Length of value */
} lcb_RESPSTATS;

/**@committed
 * @brief Schedule a request for statistics from the cluster.
 * @param instance the instance
 * @param cookie pointer to associate with the request
 * @param cmd the command
 * @return LCB_SUCCESS on success, other error code on failure.
 *
 * Note that the callback for this command is invoked an indeterminate amount
 * of times. The callback is invoked once for each statistic for each server.
 * When all the servers have responded with their statistics, a final callback
 * is delivered to the application with the LCB_RESP_F_FINAL flag set in the
 * lcb_RESPSTATS::rflags field. When this response is received no more callbacks
 * for this command shall be invoked.
 *
 * @par Request
 * @code{.c}
 * lcb_CMDSTATS cmd = { 0 };
 * // Using default stats, no further initialization
 * lcb_stats3(instance, fp, &cmd);
 * lcb_wait(instance);
 * @endcode
 *
 * @par Response
 * @code{.c}
 * lcb_install_callback3(instance, LCB_CALLBACK_STATS, stats_callback);
 * void stats_callback(lcb_INSTANCE, int, const lcb_RESPBASE *rb)
 * {
 *     const lcb_RESPSTATS *resp = (const lcb_RESPSTATS*)rb;
 *     if (resp->key) {
 *         printf("Server %s: %.*s = %.*s\n", resp->server,
 *            (int)resp->nkey, resp->key,
 *            (int)resp->nvalue, resp->value);
 *     }
 *     if (resp->rflags & LCB_RESP_F_FINAL) {
 *       printf("No more replies remaining!\n");
 *     }
 * }
 * @endcode
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_stats3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDSTATS *cmd);

LIBCOUCHBASE_API lcb_CMDSTATS *lcb_cmdstats_alloc(void);
LIBCOUCHBASE_API void lcb_cmdstats_dispose(lcb_CMDSTATS *cmd);
/**@} (Name: Stats) */

/**@name Server Versions
 * @warning This does not return the actual _Couchbase_ version but rather
 * the internal version of the memcached server.
 * @{
 */

typedef struct {
    LCB_CMD_BASE;
} lcb_CMDVERSIONS;

/**@brief Response structure for the version command */
typedef struct {
    LCB_RESP_BASE
    LCB_RESP_SERVER_FIELDS
    const char *mcversion; /**< The version string */
    lcb_SIZE nversion;     /**< Length of the version string */
} lcb_RESPMCVERSION;

/**
 * @volatile
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_server_versions3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDVERSIONS *cmd);

LIBCOUCHBASE_API lcb_CMDVERSIONS *lcb_cmdversions_alloc(void);
LIBCOUCHBASE_API void lcb_cmdversions_dispose(lcb_CMDVERSIONS *cmd);
/**@} (Name: MCversion) */

/**
 * @name Server Log Verbosity
 * @{
 */

/** @brief `level` field for lcb_server_verbosity3 () */
typedef enum {
    LCB_VERBOSITY_DETAIL = 0x00,
    LCB_VERBOSITY_DEBUG = 0x01,
    LCB_VERBOSITY_INFO = 0x02,
    LCB_VERBOSITY_WARNING = 0x03
} lcb_verbosity_level_t;

typedef struct {
    /* unused */
    LCB_CMD_BASE;
    const char *server;
    lcb_verbosity_level_t level;
} lcb_CMDVERBOSITY;
typedef lcb_RESPSERVERBASE lcb_RESPVERBOSITY;
/**@volatile*/
LIBCOUCHBASE_API
lcb_STATUS lcb_server_verbosity3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDVERBOSITY *cmd);

LIBCOUCHBASE_API lcb_CMDVERBOSITY *lcb_cmdverbosity_alloc(void);
LIBCOUCHBASE_API void lcb_cmdverbosity_dispose(lcb_CMDVERBOSITY *cmd);
/**@} (Name: Verbosity) */
/**@} (Group: Misc) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-flush Flush
 * @brief Clear the contents of a bucket
 *
 * Flush is useful for development environments (for example clearing a bucket
 * before running tests).
 *
 * @addtogroup lcb-flush
 * @{
 */
typedef struct {
    LCB_CMD_BASE;
} lcb_CMDCBFLUSH;

typedef struct {
    LCB_RESP_BASE
} lcb_RESPCBFLUSH;

/**
 * @uncommitted
 *
 * Flush a bucket
 * This function will properly flush any type of bucket using the REST API
 * via HTTP.
 *
 * The callback invoked under ::LCB_CALLBACK_CBFLUSH will be invoked with either
 * a success or failure status depending on the outcome of the operation. Note
 * that in order for lcb_cbflush3() to succeed, flush must already be enabled
 * on the bucket via the administrative interface.
 *
 * @param instance the library handle
 * @param cookie the cookie passed in the callback
 * @param cmd empty command structure. Currently there are no options for this
 *  command.
 * @return status code for scheduling.
 *
 * @attention
 * Because this command is built using HTTP, this is not subject to operation
 * pipeline calls such as lcb_sched_enter()/lcb_sched_leave()
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_cbflush3(lcb_INSTANCE *instance, void *cookie, const lcb_CMDCBFLUSH *cmd);

LIBCOUCHBASE_API lcb_CMDCBFLUSH *lcb_cmdcbflush_alloc(void);
LIBCOUCHBASE_API void lcb_cmdcbflush_dispose(lcb_CMDCBFLUSH *cmd);

typedef struct {
    LCB_CMD_BASE;
} lcb_CMDFLUSH;
typedef lcb_RESPSERVERBASE lcb_RESPFLUSH;
/**@} (Group: Flush) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-noop NOOP
 * @brief Send NOOP command to server
 *
 * @addtogroup lcb-noop
 * @{
 */
typedef struct {
    LCB_CMD_BASE;
} lcb_CMDNOOP;
typedef lcb_RESPSERVERBASE lcb_RESPNOOP;

/**
 * @committed
 *
 * Send NOOP to the node
 *
 * @param instance the library handle
 * @param cookie the cookie passed in the callback
 * @param cmd empty command structure.
 * @return status code for scheduling.
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_noop3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDNOOP *cmd);

LIBCOUCHBASE_API lcb_CMDNOOP *lcb_cmdnoop_alloc(void);
LIBCOUCHBASE_API void lcb_cmdnoop_dispose(lcb_CMDNOOP *cmd);
/**@} (Group: NOOP) */

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-timings Timings
 * @brief Determine how long operations are taking to be completed
 *
 * libcouchbase provides a simple form of per-command timings you may use
 * to figure out the current lantency for the request-response cycle as
 * generated by your application. Please note that these numbers are not
 * necessarily accurate as you may affect the timing recorded by doing
 * work in the event loop.
 *
 * The time recorded with this library is the time elapsed from the
 * command being called, and the response packet being received from the
 * server.  Everything the application does before driving the event loop
 * will affect the timers.
 *
 * The function lcb_enable_timings() is used to enable the timings for
 * the given instance, and lcb_disable_timings is used to disable the
 * timings. The overhead of using the timers should be negligible.
 *
 * The function lcb_get_timings is used to retrieve the current timing.
 * values from the given instance. The cookie is passed transparently to
 * the callback function.
 *
 * Here is an example of the usage of this module:
 *
 * @code{.c}
 * #include <libcouchbase/couchbase.h>
 *
 * static void callback(
 *  lcb_INSTANCE *instance, const void *cookie, lcb_timeunit_t timeunit, lcb_U32 min,
 *  lcb_U32 max, lcb_U32 total, lcb_U32 maxtotal)
 * {
 *   FILE* out = (void*)cookie;
 *   int num = (float)10.0 * (float)total / ((float)maxtotal);
 *   fprintf(out, "[%3u - %3u]", min, max);
 *   switch (timeunit) {
 *   case LCB_TIMEUNIT_NSEC:
 *      fprintf(out, "ns");
 *      break;
 *   case LCB_TIMEUNIT_USEC:
 *      fprintf(out, "us");
 *      break;
 *   case LCB_TIMEUNIT_MSEC:
 *      fsprintf(out, "ms");
 *      break;
 *   case LCB_TIMEUNIT_SEC:
 *      fprintf(out, "s ");
 *      break;
 *   default:
 *      ;
 *   }
 *
 *   fprintf(out, " |");
 *   for (int ii = 0; ii < num; ++ii) {
 *      fprintf(out, "#");
 *   }
 *   fprintf(out, " - %u\n", total);
 * }
 *
 *
 * lcb_enable_timings(instance);
 * ... do a lot of operations ...
 * fprintf(stderr, "              +---------+\n"
 * lcb_get_timings(instance, stderr, callback);
 * fprintf(stderr, "              +---------+\n"
 * lcb_disable_timings(instance);
 * @endcode
 *
 * @addtogroup lcb-timings
 * @{
 */

/**
 * @brief Time units reported by lcb_get_timings()
 */
enum lcb_timeunit_t {
    LCB_TIMEUNIT_NSEC = 0, /**< @brief Time is in nanoseconds */
    LCB_TIMEUNIT_USEC = 1, /**< @brief Time is in microseconds */
    LCB_TIMEUNIT_MSEC = 2, /**< @brief Time is in milliseconds */
    LCB_TIMEUNIT_SEC = 3   /**< @brief Time is in seconds */
};
typedef enum lcb_timeunit_t lcb_timeunit_t;

/**
 * Start recording timing metrics for the different operations.
 * The timer is started when the command is called (and the data
 * spooled to the server), and the execution time is the time until
 * we parse the response packets. This means that you can affect
 * the timers by doing a lot of other stuff before checking if
 * there is any results available..
 *
 * @param instance the handle to lcb
 * @return Status of the operation.
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_enable_timings(lcb_INSTANCE *instance);

/**
 * Stop recording (and release all resources from previous measurements)
 * timing metrics.
 *
 * @param instance the handle to lcb
 * @return Status of the operation.
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_disable_timings(lcb_INSTANCE *instance);

/**
 * The following function is called for each bucket in the timings
 * histogram when you call lcb_get_timings.
 * You are guaranteed that the callback will be called with the
 * lowest [min,max] range first.
 *
 * @param instance the handle to lcb
 * @param cookie the cookie you provided that allows you to pass
 *               arbitrary user data to the callback
 * @param timeunit the "scale" for the values
 * @param min The lower bound for this histogram bucket
 * @param max The upper bound for this histogram bucket
 * @param total The number of hits in this histogram bucket
 * @param maxtotal The highest value in all of the buckets
 */
typedef void (*lcb_timings_callback)(lcb_INSTANCE *instance, const void *cookie, lcb_timeunit_t timeunit, lcb_U32 min,
                                     lcb_U32 max, lcb_U32 total, lcb_U32 maxtotal);

/**
 * Get the timings histogram
 *
 * @param instance the handle to lcb
 * @param cookie a cookie that will be present in all of the callbacks
 * @param callback Callback to invoke which will handle the timings
 * @return Status of the operation.
 * @committed
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_get_timings(lcb_INSTANCE *instance, const void *cookie, lcb_timings_callback callback);
/**@} (Group: Timings) */

typedef enum {
    /** Dump the raw vbucket configuration */
    LCB_DUMP_VBCONFIG = 0x01,
    /** Dump information about each packet */
    LCB_DUMP_PKTINFO = 0x02,
    /** Dump memory usage/reservation information about buffers */
    LCB_DUMP_BUFINFO = 0x04,
    /** Dump various metrics information */
    LCB_DUMP_METRICS = 0x08,
    /** Dump everything */
    LCB_DUMP_ALL = 0xff
} lcb_DUMPFLAGS;

/**
 * @volatile
 * @brief Write a textual dump to a file.
 *
 * This function will inspect the various internal structures of the current
 * client handle (indicated by `instance`) and write the state information
 * to the file indicated by `fp`.
 * @param instance the handle to dump
 * @param fp the file to which the dump should be written
 * @param flags a set of modifiers (of @ref lcb_DUMPFLAGS) indicating what
 * information to dump. Note that a standard set of information is always
 * dumped, but by default more verbose information is hidden, and may be
 * enabled with these flags.
 */
LIBCOUCHBASE_API
void lcb_dump(lcb_INSTANCE *instance, FILE *fp, lcb_U32 flags);

/** Volatile histogram APIs, used by pillowfight and others */
struct lcb_histogram_st;
typedef struct lcb_histogram_st lcb_HISTOGRAM;

/**
 * @volatile
 * Create a histogram structure
 * @return a new histogram structure
 */
LIBCOUCHBASE_API
lcb_HISTOGRAM *lcb_histogram_create(void);

/**
 * @volatile free a histogram structure
 * @param hg the histogram
 */
LIBCOUCHBASE_API
void lcb_histogram_destroy(lcb_HISTOGRAM *hg);

/**
 * @volatile
 * Add an entry to a histogram structure
 * @param hg the histogram
 * @param duration the duration in nanoseconds
 */
LIBCOUCHBASE_API
void lcb_histogram_record(lcb_HISTOGRAM *hg, lcb_U64 duration);

typedef void (*lcb_HISTOGRAM_CALLBACK)(const void *cookie, lcb_timeunit_t timeunit, lcb_U32 min, lcb_U32 max,
                                       lcb_U32 total, lcb_U32 maxtotal);

/**
 * @volatile
 * Repeatedly invoke a callback for all entries in the histogram
 * @param hg the histogram
 * @param cookie pointer passed to callback
 * @param cb callback to invoke
 */
LIBCOUCHBASE_API
void lcb_histogram_read(const lcb_HISTOGRAM *hg, const void *cookie, lcb_HISTOGRAM_CALLBACK cb);

/**
 * Print the histogram to the specified FILE.
 *
 * This essentially outputs the same raw information as lcb_histogram_read(),
 * except it prints in implementation-defined format. It's simpler to use
 * than lcb_histogram_read, but less flexible.
 *
 * @param hg the histogram
 * @param stream File to print the histogram to.
 */
LIBCOUCHBASE_API
void lcb_histogram_print(lcb_HISTOGRAM *hg, FILE *stream);

/**
 * @volatile
 *
 * Retrieves the extra error context from the response structure.
 *
 * This context does not duplicate information described by status
 * code rendered by lcb_strerror() function, and should be logged
 * if available.
 *
 * @return the pointer to string or NULL if context wasn't specified.
 */
LIBCOUCHBASE_API
const char *lcb_resp_get_error_context(int cbtype, const lcb_RESPBASE *rb);

/**
 * @uncommitted
 *
 * Retrieves the error reference id from the response structure.
 *
 * Error reference id (or event id) should be logged to allow
 * administrators match client-side events with cluster logs.
 *
 * @return the pointer to string or NULL if ref wasn't specified.
 */
LIBCOUCHBASE_API
const char *lcb_resp_get_error_ref(int cbtype, const lcb_RESPBASE *rb);

/**
 * @defgroup lcb-collections-api Collections Management
 * @brief Managing collections in the bucket
 */

/*
 * @addtogroup lcb-collection-api
 * @{
 */

typedef struct lcb_RESPGETMANIFEST_ lcb_RESPGETMANIFEST;

LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_status(const lcb_RESPGETMANIFEST *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_cookie(const lcb_RESPGETMANIFEST *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_value(const lcb_RESPGETMANIFEST *resp, const char **json,
                                                      size_t *json_len);

typedef struct lcb_CMDGETMANIFEST_ lcb_CMDGETMANIFEST;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_create(lcb_CMDGETMANIFEST **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_destroy(lcb_CMDGETMANIFEST *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_timeout(lcb_CMDGETMANIFEST *cmd, uint32_t timeout);
LIBCOUCHBASE_API lcb_STATUS lcb_getmanifest(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETMANIFEST *cmd);

typedef struct lcb_RESPGETCID_ lcb_RESPGETCID;

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_status(const lcb_RESPGETCID *resp);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_cookie(const lcb_RESPGETCID *resp, void **cookie);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_manifest_id(const lcb_RESPGETCID *resp, uint64_t *id);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_collection_id(const lcb_RESPGETCID *resp, uint32_t *id);
LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_scoped_collection(const lcb_RESPGETCID *resp, const char **name,
                                                             size_t *name_len);

typedef struct lcb_CMDGETCID_ lcb_CMDGETCID;

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_create(lcb_CMDGETCID **cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_destroy(lcb_CMDGETCID *cmd);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_scope(lcb_CMDGETCID *cmd, const char *scope, size_t scope_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_collection(lcb_CMDGETCID *cmd, const char *collection, size_t collection_len);
LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_timeout(lcb_CMDGETCID *cmd, uint32_t timeout);

LIBCOUCHBASE_API lcb_STATUS lcb_getcid(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETCID *cmd);
/** @} */

#ifdef __cplusplus
}
#endif
#endif /* LCB_UTILS_H */
