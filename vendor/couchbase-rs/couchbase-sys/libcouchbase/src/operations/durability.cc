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

#define LCBDUR_PRIV_SYMS 1
#define NOMINMAX

#include <cstring>

#include "internal.h"
#include "durability_internal.h"
#include <algorithm>
#include <lcbio/iotable.h>

using namespace lcb::durability;

#define LOGARGS(c, lvl) (c)->instance->settings, "endure", LCB_LOG_##lvl, __FILE__, __LINE__
#define LOGARGS_T(lvl) LOGARGS(this, lvl)

static void timer_callback(lcb_socket_t sock, short which, void *arg);

bool Item::is_all_done() const
{
    const lcb_DURABILITYOPTSv0 &opts = parent->opts;

    if (!res().exists_master) {
        /** Primary cache doesn't have correct version */
        return false;
    }
    if (opts.persist_to) {
        if (!res().persisted_master) {
            return false;
        }
        if (res().npersisted < opts.persist_to) {
            return false;
        }
    }

    if (opts.replicate_to) {
        if (res().nreplicated < opts.replicate_to) {
            return false;
        }
    }

    return true;
}

bool Item::is_server_done(const ServerInfo &info, bool is_master) const
{
    // Item not in cache. Return false
    if (!info.exists) {
        return false;
    }

    // Item is already persisted to the server
    if (info.persisted) {
        return true;
    }

    // Item not persisted, but no persistence requested
    if (parent->opts.persist_to == 0) {
        return true;
    }

    // Master persistence requested, but server is not master
    if (parent->opts.persist_to == 1 && !is_master) {
        return true;
    }

    // Require persistence from this server, but item is not persisted.
    return false;
}

size_t Item::prepare(uint16_t ixarray[4])
{
    size_t oix = 0, maxix = 0;
    lcb_INSTANCE *instance = parent->instance;

    res().persisted_master = 0;
    res().exists_master = 0;
    res().npersisted = 0;
    res().nreplicated = 0;
    res().cas = 0;
    res().rc = LCB_SUCCESS;

    if (parent->opts.persist_to == 1 && parent->opts.replicate_to == 0) {
        maxix = 1; /* Only master! */
    } else {
        maxix = LCBT_NREPLICAS(instance) + 1;
    }

    for (size_t ii = 0; ii < maxix; ii++) {
        int cur_ix;
        ServerInfo &info = sinfo[ii];

        cur_ix = lcbvb_vbserver(LCBT_VBCONFIG(instance), vbid, ii);
        if (cur_ix < 0) {
            info.clear();
            continue;
        }

        const lcb::Server *s_exp = instance->get_server(cur_ix);
        if (s_exp != info.server) {
            info.clear();

        } else if (is_server_done(info, ii == 0)) {
            /* Update counters as required */
            if (ii == 0) {
                res().exists_master = 1;
            } else {
                res().nreplicated++;
            }

            if (info.persisted) {
                res().npersisted++;
                if (ii == 0) {
                    res().persisted_master = 1;
                }
            }
            continue;
        }

        /* Otherwise, write the expected server out */
        ixarray[oix++] = s_exp->get_index();
    }

    return oix;
}

void Item::update(int flags, int srvix)
{
    if (!flags || done) {
        return;
    }

    ServerInfo *info = get_server_info(srvix);
    if (!info) {
        lcb_log(LOGARGS(parent, DEBUG), "Ignoring response from server %d. Not a master or replica for vBucket %d",
                srvix, vbid);
        return;
    }

    lcb_INSTANCE *instance = parent->instance;
    bool is_master = lcbvb_vbmaster(LCBT_VBCONFIG(instance), vbid) == srvix;
    const lcb::Server *server = instance->get_server(srvix);

    info->clear();
    info->server = server;

    if (flags & UPDATE_PERSISTED) {
        info->persisted = 1;
        res().npersisted++;
        if (is_master) {
            res().persisted_master = 1;
        }
    }

    if (flags & UPDATE_REPLICATED) {
        info->exists = 1;
        if (is_master) {
            res().exists_master = 1;
        } else {
            res().nreplicated++;
        }
    }

    if (is_all_done()) {
        finish(LCB_SUCCESS);
    }
}

ServerInfo *Item::get_server_info(int srvix)
{
    size_t ii;
    lcb_INSTANCE *instance = parent->instance;

    for (ii = 0; ii < LCBT_NREPLICAS(instance) + 1; ii++) {
        int ix = lcbvb_vbserver(LCBT_VBCONFIG(instance), vbid, ii);
        if (ix > -1 && ix == srvix) {
            return &sinfo[ii];
        }
    }
    return NULL;
}

void Item::finish()
{
    lcb_RESPCALLBACK cb;
    lcb_INSTANCE *instance;

    if (done) {
        return;
    }

    done = 1;
    parent->nremaining--;

    /** Invoke the callback now :) */
    result.cookie = (void *)parent->cookie;
    instance = parent->instance;

    if (parent->is_durstore) {
        lcb_RESPSTORE resp = {0};
        resp.key = result.key;
        resp.nkey = result.nkey;
        resp.rc = result.rc;
        resp.cas = reqcas;
        resp.cookie = result.cookie;
        resp.store_ok = 1;
        resp.dur_resp = &result;

        cb = lcb_find_callback(instance, LCB_CALLBACK_STORE);
        cb(instance, LCB_CALLBACK_STORE, (lcb_RESPBASE *)&resp);
    } else {
        cb = lcb_find_callback(instance, LCB_CALLBACK_ENDURE);
        cb(instance, LCB_CALLBACK_ENDURE, (lcb_RESPBASE *)&result);
    }

    if (parent->nremaining == 0) {
        parent->decref();
    }
}

/**
 * Called when the last (primitive) OBSERVE response is received for the entry.
 */
void Durset::on_poll_done()
{
    lcb_assert(waiting || ("Got NULL callback twice!" && 0));

    waiting = 0;

    if (nremaining > 0) {
        switch_state(STATE_OBSPOLL);
    } else {
        if (span) {
            lcbtrace_span_finish(span, LCBTRACE_NOW);
            span = NULL;
        }
    }
    decref();
}

/**
 * Schedules a single sweep of observe requests.
 * The `initial` parameter determines if this is a retry or if this is the
 * initial scheduling.
 */
void Durset::poll()
{
    lcb_STATUS err;

    /* We should never be called while an 'iter' operation is still in progress */
    lcb_assert(waiting == 0);
    incref();

    err = poll_impl();
    if (err == LCB_SUCCESS) {
        incref();
        switch_state(STATE_TIMEOUT);
    } else {
        lasterr = err;
        switch_state(STATE_OBSPOLL);
    }

    decref();
}

LIBCOUCHBASE_API
lcb_STATUS lcb_durability_validate(lcb_INSTANCE *instance, lcb_U16 *persist_to, lcb_U16 *replicate_to, int options)
{
    if (!LCBT_VBCONFIG(instance)) {
        return LCB_CLIENT_ENOCONF;
    }
    int replica_max = std::min(LCBT_NREPLICAS(instance), LCBT_NDATASERVERS(instance) - 1);
    int persist_max = replica_max + 1;

    if (*persist_to == 0 && *replicate_to == 0) {
        /* Empty values! */
        return LCB_EINVAL;
    }

    /* persist_max is always one more than replica_max */
    if (static_cast< int >(*persist_to) > persist_max) {
        if (options & LCB_DURABILITY_VALIDATE_CAPMAX) {
            *persist_to = persist_max;
        } else {
            return LCB_DURABILITY_ETOOMANY;
        }
    }

    if (*replicate_to == 0) {
        return LCB_SUCCESS;
    }

    if (replica_max < 0) {
        replica_max = 0;
    }

    /* now, we need at least as many nodes as we have replicas */
    if (static_cast< int >(*replicate_to) > replica_max) {
        if (options & LCB_DURABILITY_VALIDATE_CAPMAX) {
            *replicate_to = replica_max;
        } else {
            return LCB_DURABILITY_ETOOMANY;
        }
    }
    return LCB_SUCCESS;
}

void Durset::MCTX_setspan(lcbtrace_SPAN *span_)
{
    span = span_;
}

lcb_STATUS Durset::MCTX_addcmd(const lcb_CMDBASE *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }

    entries.resize(entries.size() + 1);
    Item &ent = entries.back();

    int vbid, srvix;
    mcreq_map_key(&instance->cmdq, &cmd->key, MCREQ_PKT_BASESIZE, &vbid, &srvix);

    /* ok. now let's initialize the entry..*/
    ent.res().nkey = cmd->key.contig.nbytes;
    ent.reqcas = cmd->cas;
    ent.parent = this;
    ent.vbid = vbid;

    kvbufs.append(reinterpret_cast< const char * >(cmd->key.contig.bytes), cmd->key.contig.nbytes);

    return after_add(ent, reinterpret_cast< const lcb_CMDENDURE * >(cmd));
}

lcb_STATUS Durset::MCTX_done(const void *cookie_)
{
    lcb_STATUS err;
    const char *kptr = kvbufs.c_str();

    if (entries.empty()) {
        delete this;
        return LCB_EINVAL;
    }

    for (size_t ii = 0; ii < entries.size(); ii++) {
        Item *ent = &entries[ii];
        ent->res().key = kptr;
        kptr += ent->res().nkey;
    }

    if ((err = prepare_schedule()) != LCB_SUCCESS) {
        delete this;
        return err;
    }

    incref();

    cookie = cookie_;
    nremaining = entries.size();
    ns_timeout = gethrtime() + LCB_US2NS(opts.timeout);

    lcb_aspend_add(&instance->pendops, LCB_PENDTYPE_DURABILITY, this);
    switch_state(STATE_INIT);
    return LCB_SUCCESS;
}

void Durset::MCTX_fail()
{
    if (span) {
        lcbtrace_span_finish(span, LCBTRACE_NOW);
        span = NULL;
    }
    delete this;
}

void lcbdurctx_set_durstore(lcb_MULTICMD_CTX *mctx, int enabled)
{
    static_cast< Durset * >(mctx)->is_durstore = enabled;
}

static lcb_U8 get_poll_meth(lcb_INSTANCE *instance, const lcb_durability_opts_t *options)
{
    /* Need to call this first, so we can actually allocate the appropriate
     * data for this.. */
    uint8_t meth;
    if (options->version > 0) {
        meth = options->v.v0.pollopts;
    } else {
        meth = LCB_DURABILITY_MODE_DEFAULT;
    }

    if (meth == LCB_DURABILITY_MODE_DEFAULT) {
        meth = LCB_DURABILITY_MODE_CAS;

        if (LCBT_SETTING(instance, fetch_mutation_tokens) && LCBT_SETTING(instance, dur_mutation_tokens)) {
            for (size_t ii = 0; ii < LCBT_NSERVERS(instance); ii++) {
                if (instance->get_server(ii)->supports_mutation_tokens()) {
                    meth = LCB_DURABILITY_MODE_SEQNO;
                    break;
                }
            }
        }
    }

    return meth;
}

Durset::Durset(lcb_INSTANCE *instance_, const lcb_durability_opts_t *options)
    : MultiCmdContext(), nremaining(0), waiting(0), refcnt(0), next_state(STATE_OBSPOLL), lasterr(LCB_SUCCESS),
      is_durstore(false), cookie(NULL), ns_timeout(0), timer(NULL), instance(instance_), span(NULL)
{
    const lcb_DURABILITYOPTSv0 *opts_in = &options->v.v0;

    std::memset(&opts, 0, sizeof opts);

    /* Ensure we don't clobber options from older versions */
    opts.cap_max = opts_in->cap_max;
    opts.check_delete = opts_in->check_delete;
    opts.interval = opts_in->interval;
    opts.persist_to = opts_in->persist_to;
    opts.replicate_to = opts_in->replicate_to;
    opts.timeout = opts_in->timeout;

    if (!opts.timeout) {
        opts.timeout = LCBT_SETTING(instance, durability_timeout);
    }

    if (!opts.interval) {
        opts.interval = LCBT_SETTING(instance, durability_interval);
    }

    lcbio_pTABLE io = instance->iotable;
    timer = io->timer.create(io->p);

    lasterr = lcb_durability_validate(instance, &opts.persist_to, &opts.replicate_to,
                                      opts.cap_max ? LCB_DURABILITY_VALIDATE_CAPMAX : 0);
}

LIBCOUCHBASE_API
lcb_MULTICMD_CTX *lcb_endure3_ctxnew(lcb_INSTANCE *instance, const lcb_durability_opts_t *options, lcb_STATUS *errp)
{
    lcb_STATUS err_s;
    if (!errp) {
        errp = &err_s;
    }

    *errp = LCB_SUCCESS;

    if (!LCBT_VBCONFIG(instance)) {
        *errp = LCB_CLIENT_ETMPFAIL;
        return NULL;
    }

    Durset *dset = NULL;
    uint8_t pollmeth = get_poll_meth(instance, options);
    if (pollmeth == LCB_DURABILITY_MODE_CAS) {
        dset = Durset::createCasDurset(instance, options);
    } else if (pollmeth == LCB_DURABILITY_MODE_SEQNO) {
        dset = Durset::createSeqnoDurset(instance, options);
    } else {
        *errp = LCB_EINVAL;
        return NULL;
    }

    if ((*errp = dset->lasterr) != LCB_SUCCESS) {
        delete dset;
        dset = NULL;
    }

    return dset;
}

/**
 * Actually free the resources allocated by the dset (and all its entries).
 * Called by some other functions in libcouchbase
 */
void lcbdur_destroy(void *dset)
{
    delete reinterpret_cast< Durset * >(dset);
}

Durset::~Durset()
{
    if (timer) {
        lcbio_TABLE *io = instance->iotable;
        io->timer.cancel(io->p, timer);
        io->timer.destroy(io->p, timer);
        timer = NULL;
    }

    lcb_aspend_del(&instance->pendops, LCB_PENDTYPE_DURABILITY, this);
    lcb_maybe_breakout(instance);
}

/**
 * All-purpose callback dispatcher.
 */
static void timer_callback(lcb_socket_t, short, void *arg)
{
    reinterpret_cast< Durset * >(arg)->tick();
}

void Durset::tick()
{
    hrtime_t now = gethrtime();

    if (ns_timeout && now > ns_timeout) {
        next_state = STATE_TIMEOUT;
    }

    switch (next_state) {
        case STATE_OBSPOLL:
        case STATE_INIT:
            poll();
            break;

        case STATE_TIMEOUT: {
            lcb_STATUS err = lasterr ? lasterr : LCB_ETIMEDOUT;
            ns_timeout = 0;
            next_state = STATE_IGNORE;

            lcb_log(LOGARGS_T(WARN), "Polling durability timed out!");

            incref();

            for (size_t ii = 0; ii < entries.size(); ii++) {
                Item *ent = &entries[ii];
                if (ent->done) {
                    continue;
                }
                if (ent->res().rc == LCB_SUCCESS) {
                    ent->res().rc = err;
                }
                ent->finish();
            }

            decref();
            break;
        }

        case STATE_IGNORE:
            break;

        default:
            lcb_assert("unexpected state" && 0);
            break;
    }
}

/**
 * Schedules us to be notified with the given state within a particular amount
 * of time. This is used both for the timeout and for the interval
 */
void Durset::switch_state(State state)
{
    uint32_t delay = 0;
    lcbio_TABLE *io = instance->iotable;
    hrtime_t now = gethrtime();

    if (state == STATE_TIMEOUT) {
        if (ns_timeout && now < ns_timeout) {
            delay = LCB_NS2US(ns_timeout - now);
        } else {
            delay = 0;
        }
    } else if (state == STATE_OBSPOLL) {
        if (now + LCB_US2NS(opts.interval) < ns_timeout) {
            delay = opts.interval;
        } else {
            delay = 0;
            state = STATE_TIMEOUT;
        }
    } else if (state == STATE_INIT) {
        delay = 0;
    }

    next_state = state;
    io->timer.cancel(io->p, timer);
    io->timer.schedule(io->p, timer, delay, this, timer_callback);
}
