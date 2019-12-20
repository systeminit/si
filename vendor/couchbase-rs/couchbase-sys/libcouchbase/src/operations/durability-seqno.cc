/*
 *     Copyright 2015-2019 Couchbase, Inc.
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

#include "internal.h"
#include "durability_internal.h"

using namespace lcb::durability;

namespace
{
class SeqnoDurset : public Durset
{
  public:
    SeqnoDurset(lcb_INSTANCE *instance_, const lcb_durability_opts_t *options) : Durset(instance_, options) {}

    // Override
    lcb_STATUS poll_impl();

    // Override
    lcb_STATUS after_add(Item &item, const lcb_CMDENDURE *cmd);

    void update(const lcb_RESPOBSEQNO *resp);
};
} // namespace

Durset *Durset::createSeqnoDurset(lcb_INSTANCE *instance, const lcb_durability_opts_t *options)
{
    return new SeqnoDurset(instance, options);
}

#define ENT_SEQNO(ent) (ent)->reqseqno

static void seqno_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPOBSEQNO *resp = (const lcb_RESPOBSEQNO *)rb;
    int flags = 0;
    Item *ent = static_cast< Item * >(reinterpret_cast< CallbackCookie * >(resp->cookie));

    /* Now, process the response */
    if (resp->rc != LCB_SUCCESS) {
        ent->res().rc = resp->rc;
        goto GT_TALLY;
    }

    lcb_U64 seqno_mem, seqno_disk;
    if (resp->old_uuid) {
        /* Failover! */
        seqno_mem = seqno_disk = resp->old_seqno;
        if (seqno_mem < ENT_SEQNO(ent)) {
            ent->finish(LCB_MUTATION_LOST);
            goto GT_TALLY;
        }
    } else {
        seqno_mem = resp->mem_seqno;
        seqno_disk = resp->persisted_seqno;
    }

    if (seqno_mem < ENT_SEQNO(ent)) {
        goto GT_TALLY;
    }

    flags = Item::UPDATE_REPLICATED;
    if (seqno_disk >= ENT_SEQNO(ent)) {
        flags |= Item::UPDATE_PERSISTED;
    }

    ent->update(flags, resp->server_index);

GT_TALLY:
    if (!--ent->parent->waiting) {
        /* avoid ssertion (wait==0)! */
        ent->parent->waiting = 1;
        ent->parent->on_poll_done();
    }
}

lcb_STATUS SeqnoDurset::poll_impl()
{
    lcb_STATUS ret_err = LCB_EINTERNAL; /* This should never be returned */
    bool has_ops = false;

    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < entries.size(); ii++) {
        Item &ent = entries[ii];
        lcb_U16 servers[4];
        lcb_CMDOBSEQNO cmd = {0};

        if (ent.done) {
            continue;
        }

        cmd.uuid = ent.uuid;
        cmd.vbid = ent.vbid;
        cmd.cmdflags = LCB_CMD_F_INTERNAL_CALLBACK;
        ent.callback = seqno_callback;

        size_t nservers = ent.prepare(servers);
        if (nservers == 0) {
            ret_err = LCB_DURABILITY_ETOOMANY;
            continue;
        }
        for (size_t jj = 0; jj < nservers; jj++) {
            lcb_STATUS err;
            cmd.server_index = servers[jj];
            LCB_CMD_SET_TRACESPAN(&cmd, span);
            err = lcb_observe_seqno3(instance, &ent.callback, &cmd);
            if (err == LCB_SUCCESS) {
                waiting++;
                has_ops = true;
            } else {
                ent.res().rc = ret_err = err;
            }
        }
    }
    lcb_sched_leave(instance);
    if (!has_ops) {
        return ret_err;
    } else {
        return LCB_SUCCESS;
    }
}

lcb_STATUS SeqnoDurset::after_add(Item &item, const lcb_CMDENDURE *cmd)
{
    const lcb_MUTATION_TOKEN *stok = NULL;

    if (cmd->cmdflags & LCB_CMDENDURE_F_MUTATION_TOKEN) {
        stok = cmd->mutation_token;
    }

    if (stok == NULL) {
        if (!instance->dcpinfo) {
            return LCB_DURABILITY_NO_MUTATION_TOKENS;
        }
        if (item.vbid >= LCBT_VBCONFIG(instance)->nvb) {
            return LCB_EINVAL;
        }
        stok = instance->dcpinfo + item.vbid;
        if (LCB_MUTATION_TOKEN_ID(stok) == 0) {
            return LCB_DURABILITY_NO_MUTATION_TOKENS;
        }
    }

    /* Set the fields */
    memset(item.sinfo, 0, sizeof(item.sinfo[0]) * 4);
    item.uuid = LCB_MUTATION_TOKEN_ID(stok);
    ENT_SEQNO(&item) = LCB_MUTATION_TOKEN_SEQ(stok);
    return LCB_SUCCESS;
}
