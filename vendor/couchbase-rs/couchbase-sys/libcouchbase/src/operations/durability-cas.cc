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

#define LCBDUR_PRIV_SYMS

#include <libcouchbase/couchbase.h>
#include "internal.h"
#include "durability_internal.h"

using namespace lcb::durability;

namespace
{
struct CasDurset : public Durset {
    CasDurset(lcb_INSTANCE *instance_, const lcb_durability_opts_t *options) : Durset(instance_, options), ht(NULL) {}

    virtual ~CasDurset();

    void update(lcb_STATUS err, const lcb_RESPOBSERVE *resp);
    Item &find(const char *s, size_t n)
    {
        if (entries.size() == 1) {
            return entries.back();
        } else {
            return *reinterpret_cast< Item * >(genhash_find(ht, s, n));
        }
    }

    // Override
    lcb_STATUS prepare_schedule();
    lcb_STATUS poll_impl();

    genhash_t *ht;
};
} // namespace

Durset *Durset::createCasDurset(lcb_INSTANCE *instance, const lcb_durability_opts_t *options)
{
    return new CasDurset(instance, options);
}

/* Called when the criteria is to ensure the key exists somewhow */
static int check_positive_durability(Item &ent, const lcb_RESPOBSERVE *res)
{
    switch (res->status) {
        case LCB_OBSERVE_NOT_FOUND:
        case LCB_OBSERVE_LOGICALLY_DELETED:
            /* If we get NOT_FOUND from the master, this means the key
             * simply does not exists (and we don't have to continue polling) */
            if (res->ismaster) {
                ent.finish(LCB_KEY_ENOENT);
            }
            return Item::NO_CHANGES;

        case LCB_OBSERVE_PERSISTED:
            return Item::UPDATE_PERSISTED | Item::UPDATE_REPLICATED;

        case LCB_OBSERVE_FOUND:
            return Item::UPDATE_REPLICATED;

        default:
            ent.finish(LCB_EINTERNAL);
            return Item::NO_CHANGES;
    }
}

/* Called when the criteria is to ensure that the key is deleted somehow */
static int check_negative_durability(Item &ent, const lcb_RESPOBSERVE *res)
{
    switch (res->status) {
        case LCB_OBSERVE_PERSISTED:
        case LCB_OBSERVE_FOUND:
            /* Still there! */
            return Item::NO_CHANGES;

        case LCB_OBSERVE_LOGICALLY_DELETED:
            /* removed from cache, but not actually deleted from disk */
            return Item::UPDATE_REPLICATED;

        case LCB_OBSERVE_NOT_FOUND:
            /* No knowledge of key. */
            return Item::UPDATE_PERSISTED | Item::UPDATE_REPLICATED;

        default:
            ent.finish(LCB_EINTERNAL);
            return Item::NO_CHANGES;
    }
}

void lcbdur_cas_update(lcb_INSTANCE *, void *dset, lcb_STATUS err, const lcb_RESPOBSERVE *resp)
{
    reinterpret_cast< CasDurset * >(dset)->update(err, resp);
}

/* Observe callback. Called internally by observe.c */
void CasDurset::update(lcb_STATUS err, const lcb_RESPOBSERVE *resp)
{
    if (resp->key == NULL) {
        /* Last observe response for requests. Start polling after interval */
        on_poll_done();
        return;
    }

    Item &ent = find(reinterpret_cast< const char * >(resp->key), resp->nkey);

    if (ent.done) {
        /* ignore subsequent errors */
        return;
    }

    if (err != LCB_SUCCESS) {
        ent.res().rc = err;
        return;
    }

    ent.res().nresponses++;
    if (resp->cas && resp->ismaster) {
        ent.res().cas = resp->cas;

        if (ent.reqcas && ent.reqcas != resp->cas) {
            ent.finish(LCB_KEY_EEXISTS);
            return;
        }
    }

    int flags;
    if (opts.check_delete) {
        flags = check_negative_durability(ent, resp);
    } else {
        flags = check_positive_durability(ent, resp);
    }

    ent.update(flags, resp->ttp);
}

lcb_STATUS CasDurset::poll_impl()
{
    lcb_MULTICMD_CTX *mctx;
    lcb_STATUS err;

    mctx = lcb_observe_ctx_dur_new(instance);
    if (!mctx) {
        return LCB_CLIENT_ENOMEM;
    }

    for (size_t ii = 0; ii < entries.size(); ii++) {
        lcb_CMDOBSERVE cmd = {0};
        uint16_t servers[4];

        Item &ent = entries[ii];
        if (ent.done) {
            continue;
        }

        size_t nservers = ent.prepare(servers);
        if (nservers == 0) {
            ent.res().rc = LCB_NO_MATCHING_SERVER;
            continue;
        }

        LCB_KREQ_SIMPLE(&cmd.key, ent.res().key, ent.res().nkey);
        cmd.key.vbid = ent.vbid;
        cmd.key.type = LCB_KV_VBID;
        cmd.servers_ = servers;
        cmd.nservers_ = nservers;

        if (instance->settings->tracer) {
            lcbtrace_REF ref;
            ref.type = LCBTRACE_REF_CHILD_OF;
            ref.span = span;
            lcbtrace_SPAN *child =
                lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_OBSERVE_CAS_ROUND, LCBTRACE_NOW, &ref);
            lcbtrace_span_add_system_tags(child, instance->settings, LCBTRACE_TAG_SERVICE_KV);
            mctx->setspan(mctx, child);
        }

        err = mctx->addcmd(mctx, (lcb_CMDBASE *)&cmd);
        if (err != LCB_SUCCESS) {
            mctx->fail(mctx);
            return err;
        }
    }

    lcb_sched_enter(instance);
    err = mctx->done(mctx, this);
    mctx = NULL;

    if (err == LCB_SUCCESS) {
        lcb_sched_leave(instance);
        waiting = 1;
    } else {
        lcb_sched_fail(instance);
    }
    return err;
}

lcb_STATUS CasDurset::prepare_schedule()
{
    Durset::prepare_schedule();
    if (entries.size() < 2) {
        return LCB_SUCCESS;
    }

    ht = lcb_hashtable_nc_new(entries.size());
    if (!ht) {
        return LCB_CLIENT_ENOMEM;
    }

    for (size_t ii = 0; ii < entries.size(); ++ii) {
        int mt;
        Item &ent = entries[ii];

        mt = genhash_update(ht, ent.res().key, ent.res().nkey, &ent, 0);
        if (mt != NEW) {
            return LCB_DUPLICATE_COMMANDS;
        }
    }
    return LCB_SUCCESS;
}

CasDurset::~CasDurset()
{
    if (ht) {
        genhash_free(ht);
        ht = NULL;
    }
}
