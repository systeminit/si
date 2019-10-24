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

#ifndef LCB_DURABILITY_INTERNAL_H
#define LCB_DURABILITY_INTERNAL_H

#ifdef __cplusplus
#include "mctx-helper.h"

extern "C" {
#endif

/**
 * @name internal durability functions
 * These functions are used internally beyond the durability module.
 * @{
 */

/**
 * Called from the OBSERVE codebase to update an item's status for CAS-based
 * observe
 */
void lcbdur_cas_update(lcb_INSTANCE *, void *dset, lcb_STATUS, const lcb_RESPOBSERVE *);

/**
 * Called from the OBSERVE codebase to update an item's status for seqno-based
 * observe
 */
void lcbdur_update_seqno(lcb_INSTANCE *, void *dset, const lcb_RESPOBSEQNO *);

/** Indicate that this durability command context is for an original storage op */
void lcbdurctx_set_durstore(lcb_MULTICMD_CTX *ctx, int enabled);

void lcbdur_destroy(void *dset);

/** Called from durability-cas to request an OBSERVE with a special callback */
lcb_MULTICMD_CTX *lcb_observe_ctx_dur_new(lcb_INSTANCE *instance);

/**@}
 *
 * The rest of this file is internal to the various durability operations and
 * is not accessed by the rest of the codebase
 */

#ifdef __cplusplus
}
#endif

#ifdef LCBDUR_PRIV_SYMS
namespace lcb
{
namespace durability
{

/**
 * Here is the internal API for the durability functions.
 *
 * Durability works on polling multiple observe responses and waiting until a
 * key (or set of keys) have either been persisted, or the wait period has
 * expired.
 *
 * The operation maintains an internal counter which counts how many keys
 * do not have a conclusive observe response yet (i.e. how many do not have
 * their criteria satisfied yet). The operation is considered complete when
 * the counter reaches 0.
 */

/**
 * Information about a particular server's state -- whether it has been
 * persisted to or replicated to. This is tied to a given mc_SERVER
 * instance.
 */
struct ServerInfo {
    const lcb::Server *server; /**< Server pointer (for comparison only) */
    lcb_U16 persisted;         /**< Exists on server */
    lcb_U16 exists;            /**< Persisted to server */

    ServerInfo() : server(NULL), persisted(0), exists(0) {}

    void clear()
    {
        server = NULL;
        persisted = 0;
        exists = 0;
    }
};

struct Durset;

// For use in conjunction with MCREQ_F_PRIVCALLBACK
struct CallbackCookie {
    lcb_RESPCALLBACK callback;

    CallbackCookie() : callback(NULL) {}
};

/**Information a single entry in a durability set. Each entry contains a single
 * key */
struct Item : public CallbackCookie {
    Item() : reqcas(0), reqseqno(0), uuid(0), result(), parent(NULL), vbid(0), done(0) {}

    /**
     * Returns true if the entry is complete, false otherwise. This only assumes
     * successful entries.
     */
    bool is_all_done() const;

    /**
     * Determine if this item has been satisfied on a specific server. This
     * function is used to determine if a probe should be sent to the server.
     *
     * If there are no items "tied" to the server (because they have all been
     * completed) then we employ a bandwidth saving optimization by not sending
     * additional probes to it.
     *
     * @param info Server to check against
     * @param is_master If this server is the master for the item's vbucket
     *
     * @return true if the item is both persisted and replicated on the server,
     *         OR if the item has been replicated, but replication is not
     *         required for the item.
     */
    bool is_server_done(const ServerInfo &info, bool is_master) const;

    /**
     * Updates the state of the given entry and synchronizes it with the
     * current server list.
     *
     * Specifically this function will return a list of
     * servers which still need to be contacted, and will increment internal
     * counters on behalf of those (still active) servers which the item has
     * already been replicated to (and persisted to, if requested).
     *
     * This will invalidate any cached information of the cluster configuration
     * in respect to this item has changed -- this includes things like servers
     * moving indices or being recreated entirely.
     *
     * This function should be called during poll().
     * @param[out] ixarray An array of server indices which should be queried
     * @return the number of effective entries in the array.
     */
    size_t prepare(uint16_t ixarray[4]);

    enum UpdateFlags { NO_CHANGES = 0x00, UPDATE_PERSISTED = 0x01, UPDATE_REPLICATED = 0x02 };

    /**
     * Update an item's status.
     * @param flags OR'd set of UPDATE_PERSISTED and UPDATE_REPLICATED
     * @param ix The server index
     */
    void update(int flags, int srvix);

    /**
     * Set the logical state of the entry to done, and invoke the callback.
     * It is safe to call this multiple times
     */
    void finish();

    void finish(lcb_STATUS err)
    {
        result.rc = err;
        finish();
    }

    lcb_RESPENDURE &res()
    {
        return result;
    }
    const lcb_RESPENDURE &res() const
    {
        return result;
    }
    ServerInfo *get_server_info(int index);

    lcb_U64 reqcas;   /**< Last known CAS for the user */
    lcb_U64 reqseqno; /**< Last known seqno for the user */
    lcb_U64 uuid;
    lcb_RESPENDURE result; /**< Result to be passed to user */
    Durset *parent;
    lcb_U16 vbid; /**< vBucket ID (computed via hashkey) */
    lcb_U8 done;  /**< Whether we have a conclusive result for this entry */

    /** Array of servers which have satisfied constraints */
    ServerInfo sinfo[4];
};

/**
 * A collection encompassing one or more entries which are to be checked for
 * persistence
 */
struct Durset : public MultiCmdContext {
    /**
     * Call this when the polling method (poll_impl()) has completed. This will
     * trigger a new poll after the interval.
     */
    void on_poll_done();

    void incref()
    {
        refcnt++;
    }

    /**
     * Decrement the refcount for the 'dset'. When it hits zero then the dset is
     * freed
     */
    void decref()
    {
        if (!--refcnt) {
            delete this;
        }
    }

    enum State { STATE_OBSPOLL, STATE_INIT, STATE_TIMEOUT, STATE_IGNORE };

    /**
     * Schedules us to be notified with the given state within a particular amount
     * of time. This is used both for the timeout and for the interval
     */
    void switch_state(State state);

    /**
     * Called from mctx_schedule(). This allows implementations to do any
     * additional bookeeping, having guaranteed that all items are now
     * added.
     */
    virtual lcb_STATUS prepare_schedule()
    {
        return LCB_SUCCESS;
    }

    /**
     * Called from mctx_add. Called to register any item-specific data (i.e.
     * to associate item data with internal structures)
     * @param itm the newly added item
     * @param the original command, for more context
     */
    virtual lcb_STATUS after_add(Item &, const lcb_CMDENDURE *)
    {
        return LCB_SUCCESS;
    }

    /**
     * Called to actually check for persistence/replication. This must be
     * implemented.
     */
    virtual lcb_STATUS poll_impl() = 0;

    virtual ~Durset();
    Durset(lcb_INSTANCE *instance, const lcb_durability_opts_t *options);

    // Implementation for MULTICMD_CTX
    lcb_STATUS MCTX_done(const void *cookie);
    lcb_STATUS MCTX_addcmd(const lcb_CMDBASE *cmd);
    void MCTX_fail();
    void MCTX_setspan(lcbtrace_SPAN *span);

    /**
     * This function calls poll_impl(). The implementation should then call
     * on_poll_done() once the polling is finished
     */
    inline void poll();

    /** Called after timeouts and intervals. */
    inline void tick();

    static Durset *createCasDurset(lcb_INSTANCE *, const lcb_durability_opts_t *);
    static Durset *createSeqnoDurset(lcb_INSTANCE *, const lcb_durability_opts_t *);

    lcb_DURABILITYOPTSv0 opts; /**< Sanitized user options */
    std::vector< Item > entries;
    unsigned nremaining; /**< Number of entries remaining to poll for */
    int waiting;         /**< Set if currently awaiting an observe callback */
    unsigned refcnt;     /**< Reference count */
    State next_state;    /**< Internal state */
    lcb_STATUS lasterr;
    bool is_durstore;    /** Whether the callback should be DURSTORE */
    std::string kvbufs;  /**< Backing storage for key buffers */
    const void *cookie;  /**< User cookie */
    hrtime_t ns_timeout; /**< Timestamp of next timeout */
    void *timer;
    lcb_INSTANCE *instance;
    lcbtrace_SPAN *span;
};

} // namespace durability
} // namespace lcb
#endif // __cplusplus
#endif // LCB_DURABILITY_INTERNAL_H
