/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#include "internal.h"
#include "mc/compress.h"
#include "trace.h"
#include "durability_internal.h"

LIBCOUCHBASE_API int lcb_mutation_token_is_valid(const lcb_MUTATION_TOKEN *token)
{
    return token && !(token->uuid_ == 0 && token->seqno_ == 0 && token->vbid_ == 0);
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_status(const lcb_RESPSTORE *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_error_context(const lcb_RESPSTORE *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_STORE, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_error_ref(const lcb_RESPSTORE *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_STORE, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_cookie(const lcb_RESPSTORE *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_cas(const lcb_RESPSTORE *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_key(const lcb_RESPSTORE *resp, const char **key, size_t *key_len)
{
    *key = (char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_operation(const lcb_RESPSTORE *resp, lcb_STORE_OPERATION *operation)
{
    *operation = resp->op;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_stored(const lcb_RESPSTORE *resp, int *store_ok)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *store_ok = resp->store_ok;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_respstore_observe_attached(const lcb_RESPSTORE *resp)
{
    return resp->dur_resp != NULL;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_master_exists(const lcb_RESPSTORE *resp, int *master_exists)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *master_exists = resp->dur_resp->exists_master;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_master_persisted(const lcb_RESPSTORE *resp, int *master_persisted)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *master_persisted = resp->dur_resp->persisted_master;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_responses(const lcb_RESPSTORE *resp, uint16_t *num_responses)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *num_responses = resp->dur_resp->nresponses;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_persisted(const lcb_RESPSTORE *resp, uint16_t *num_persisted)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *num_persisted = resp->dur_resp->npersisted;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_observe_num_replicated(const lcb_RESPSTORE *resp, uint16_t *num_replicated)
{
    if (resp->dur_resp == NULL) {
        return LCB_NOT_SUPPORTED;
    }
    *num_replicated = resp->dur_resp->nreplicated;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respstore_mutation_token(const lcb_RESPSTORE *resp, lcb_MUTATION_TOKEN *token)
{
    const lcb_MUTATION_TOKEN *mt = lcb_resp_get_mutation_token(LCB_CALLBACK_STORE, (const lcb_RESPBASE *)resp);
    if (token && mt) {
        *token = *mt;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_create(lcb_CMDSTORE **cmd, lcb_STORE_OPERATION operation)
{
    *cmd = (lcb_CMDSTORE *)calloc(1, sizeof(lcb_CMDSTORE));
    (*cmd)->operation = operation;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_clone(const lcb_CMDSTORE *cmd, lcb_CMDSTORE **copy)
{
    LCB_CMD_CLONE_WITH_VALUE(lcb_CMDSTORE, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_destroy(lcb_CMDSTORE *cmd)
{
    LCB_CMD_DESTROY_CLONE_WITH_VALUE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_timeout(lcb_CMDSTORE *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_parent_span(lcb_CMDSTORE *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_collection(lcb_CMDSTORE *cmd, const char *scope, size_t scope_len,
                                                    const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_key(lcb_CMDSTORE *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_value(lcb_CMDSTORE *cmd, const char *value, size_t value_len)
{
    LCB_CMD_SET_VALUE(cmd, value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_value_iov(lcb_CMDSTORE *cmd, const lcb_IOV *value, size_t value_len)
{
    LCB_CMD_SET_VALUEIOV(cmd, (lcb_IOV *)value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_expiration(lcb_CMDSTORE *cmd, uint32_t expiration)
{
    cmd->exptime = expiration;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_cas(lcb_CMDSTORE *cmd, uint64_t cas)
{
    cmd->cas = cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_flags(lcb_CMDSTORE *cmd, uint32_t flags)
{
    cmd->flags = flags;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_datatype(lcb_CMDSTORE *cmd, uint8_t datatype)
{
    cmd->datatype = datatype;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_durability(lcb_CMDSTORE *cmd, lcb_DURABILITY_LEVEL level)
{
    cmd->durability_mode = LCB_DURABILITY_SYNC;
    cmd->durability.sync.dur_level = level;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdstore_durability_observe(lcb_CMDSTORE *cmd, int persist_to, int replicate_to)
{
    cmd->durability_mode = LCB_DURABILITY_POLL;
    cmd->durability.poll.persist_to = persist_to;
    cmd->durability.poll.replicate_to = replicate_to;
    return LCB_SUCCESS;
}

struct DurStoreCtx : mc_REQDATAEX {
    lcb_INSTANCE *instance;
    lcb_U16 persist_to;
    lcb_U16 replicate_to;

    static mc_REQDATAPROCS proctable;

    DurStoreCtx(lcb_INSTANCE *instance_, lcb_U16 persist_, lcb_U16 replicate_, const void *cookie_)
        : mc_REQDATAEX(cookie_, proctable, 0), instance(instance_), persist_to(persist_),
          replicate_to(replicate_)
    {
    }
};

/** Observe stuff */
static void handle_dur_storecb(mc_PIPELINE *, mc_PACKET *pkt, lcb_STATUS err, const void *arg)
{
    lcb_RESPCALLBACK cb;
    lcb_RESPSTORE resp = {0};
    lcb_CMDENDURE dcmd = {0};
    const lcb_MUTATION_TOKEN *mt;
    DurStoreCtx *dctx = static_cast< DurStoreCtx * >(pkt->u_rdata.exdata);
    lcb_MULTICMD_CTX *mctx;
    lcb_durability_opts_t opts = {0};
    const lcb_RESPSTORE *sresp = (const lcb_RESPSTORE *)arg;
    lcbtrace_SPAN *span = NULL;

    if (err != LCB_SUCCESS) {
        goto GT_BAIL;
    }
    if (sresp->rc != LCB_SUCCESS) {
        err = sresp->rc;
        goto GT_BAIL;
    }

    resp.store_ok = 1;
    LCB_CMD_SET_KEY(&dcmd, sresp->key, sresp->nkey);
    dcmd.cas = sresp->cas;

    mt = lcb_resp_get_mutation_token(LCB_CALLBACK_STORE, (const lcb_RESPBASE *)sresp);
    if (LCB_MUTATION_TOKEN_ISVALID(mt)) {
        dcmd.mutation_token = mt;
    }

    /* Set the options.. */
    opts.v.v0.persist_to = dctx->persist_to;
    opts.v.v0.replicate_to = dctx->replicate_to;

    mctx = lcb_endure3_ctxnew(dctx->instance, &opts, &err);
    if (mctx == NULL) {
        goto GT_BAIL;
    }

    span = MCREQ_PKT_RDATA(pkt)->span;
    if (span) {
        mctx->setspan(mctx, span);
    }

    lcbdurctx_set_durstore(mctx, 1);
    err = mctx->addcmd(mctx, (lcb_CMDBASE *)&dcmd);
    if (err != LCB_SUCCESS) {
        mctx->fail(mctx);
        goto GT_BAIL;
    }
    lcb_sched_enter(dctx->instance);
    err = mctx->done(mctx, sresp->cookie);
    lcb_sched_leave(dctx->instance);

    if (err == LCB_SUCCESS) {
        /* Everything OK? */
        delete dctx;
        return;
    }

GT_BAIL : {
    lcb_RESPENDURE dresp = {0};
    resp.key = sresp->key;
    resp.nkey = sresp->nkey;
    resp.cookie = sresp->cookie;
    resp.rc = err;
    resp.dur_resp = &dresp;
    cb = lcb_find_callback(dctx->instance, LCB_CALLBACK_STORE);
    cb(dctx->instance, LCB_CALLBACK_STORE, (const lcb_RESPBASE *)&resp);
    delete dctx;
}
}

static void handle_dur_schedfail(mc_PACKET *pkt)
{
    delete static_cast< DurStoreCtx * >(pkt->u_rdata.exdata);
}

mc_REQDATAPROCS DurStoreCtx::proctable = {handle_dur_storecb, handle_dur_schedfail};

static lcb_size_t get_key_size(protocol_binary_request_header *hdr)
{
    if (hdr->request.magic == PROTOCOL_BINARY_AREQ) {
        return (hdr->request.keylen >> 8) & 0xff;
    } else {
        return ntohs(hdr->request.keylen);
    }
}

static lcb_size_t get_value_size(mc_PACKET *packet)
{
    if (packet->flags & MCREQ_F_VALUE_IOV) {
        return packet->u_value.multi.total_length;
    } else {
        return packet->u_value.single.size;
    }
}

static lcb_STATUS get_esize_and_opcode(lcb_STORE_OPERATION ucmd, lcb_uint8_t *opcode, lcb_uint8_t *esize)
{
    if (ucmd == LCB_STORE_SET || ucmd == LCB_STORE_UPSERT) {
        *opcode = PROTOCOL_BINARY_CMD_SET;
        *esize = 8;
    } else if (ucmd == LCB_STORE_ADD) {
        *opcode = PROTOCOL_BINARY_CMD_ADD;
        *esize = 8;
    } else if (ucmd == LCB_STORE_REPLACE) {
        *opcode = PROTOCOL_BINARY_CMD_REPLACE;
        *esize = 8;
    } else if (ucmd == LCB_STORE_APPEND) {
        *opcode = PROTOCOL_BINARY_CMD_APPEND;
        *esize = 0;
    } else if (ucmd == LCB_STORE_PREPEND) {
        *opcode = PROTOCOL_BINARY_CMD_PREPEND;
        *esize = 0;
    } else {
        return LCB_EINVAL;
    }
    return LCB_SUCCESS;
}

static int can_compress(lcb_INSTANCE *instance, const mc_PIPELINE *pipeline, lcb_datatype_t datatype)
{
    const lcb::Server *server = static_cast< const lcb::Server * >(pipeline);
    int compressopts = LCBT_SETTING(instance, compressopts);

    if ((compressopts & LCB_COMPRESS_OUT) == 0) {
        return 0;
    }
    if (server->supports_compression() == false && (compressopts & LCB_COMPRESS_FORCE) == 0) {
        return 0;
    }
    if (datatype & LCB_VALUE_F_SNAPPYCOMP) {
        return 0;
    }
    return 1;
}

static lcb_STATUS store_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDSTORE *cmd = (const lcb_CMDSTORE *)arg;
    lcb_STATUS err;

    mc_PIPELINE *pipeline;
    mc_PACKET *packet;
    mc_CMDQUEUE *cq = &instance->cmdq;
    protocol_binary_request_set scmd = {0};
    protocol_binary_request_header *hdr = &scmd.message.header;
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);

    int hsize;
    int should_compress = 0;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDSTORE *mut = const_cast< lcb_CMDSTORE * >(cmd);
        mut->cid = cid;
    }
    hdr->request.magic = PROTOCOL_BINARY_REQ;

    lcb_U8 ffextlen = 0;
    if (cmd->durability_mode == LCB_DURABILITY_SYNC && cmd->durability.sync.dur_level && new_durability_supported) {
        hdr->request.magic = PROTOCOL_BINARY_AREQ;
        /* 1 byte for id and size
         * 1 byte for level
         * 2 bytes for timeout
         */
        ffextlen = 4;
    }

    err = get_esize_and_opcode(cmd->operation, &hdr->request.opcode, &hdr->request.extlen);
    if (err != LCB_SUCCESS) {
        return err;
    }
    hsize = hdr->request.extlen + sizeof(*hdr) + ffextlen;
    err = mcreq_basic_packet(cq, (const lcb_CMDBASE *)cmd, hdr, hdr->request.extlen, ffextlen, &packet, &pipeline,
                             MCREQ_BASICPACKET_F_FALLBACKOK);
    if (err != LCB_SUCCESS) {
        return err;
    }

    should_compress = can_compress(instance, pipeline, cmd->datatype);
    if (should_compress) {
        int rv = mcreq_compress_value(pipeline, packet, &cmd->value, instance->settings, &should_compress);
        if (rv != 0) {
            mcreq_release_packet(pipeline, packet);
            return LCB_CLIENT_ENOMEM;
        }
    } else {
        mcreq_reserve_value(pipeline, packet, &cmd->value);
    }

    if (cmd->durability_mode == LCB_DURABILITY_POLL) {
        int duropts = 0;
        lcb_U16 persist_u, replicate_u;
        persist_u = cmd->durability.poll.persist_to;
        replicate_u = cmd->durability.poll.replicate_to;
        if (cmd->durability.poll.replicate_to == (char)-1 || cmd->durability.poll.persist_to == (char)-1) {
            duropts = LCB_DURABILITY_VALIDATE_CAPMAX;
        }

        err = lcb_durability_validate(instance, &persist_u, &replicate_u, duropts);
        if (err != LCB_SUCCESS) {
            mcreq_wipe_packet(pipeline, packet);
            mcreq_release_packet(pipeline, packet);
            return err;
        }

        DurStoreCtx *dctx = new DurStoreCtx(instance, persist_u, replicate_u, cookie);
        dctx->start = gethrtime();
        dctx->deadline = dctx->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
        packet->u_rdata.exdata = dctx;
        packet->flags |= MCREQ_F_REQEXT;
    } else {
        mc_REQDATA *rdata = MCREQ_PKT_RDATA(packet);
        rdata->cookie = cookie;
        rdata->start = gethrtime();
        rdata->deadline = rdata->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
        if (cmd->durability.sync.dur_level && new_durability_supported) {
            scmd.message.body.alt.expiration = htonl(cmd->exptime);
            scmd.message.body.alt.flags = htonl(cmd->flags);
            scmd.message.body.alt.meta = (1 << 4) | 3;
            scmd.message.body.alt.level = cmd->durability.sync.dur_level;
            scmd.message.body.alt.timeout = lcb_durability_timeout(instance);
        } else {
            scmd.message.body.norm.expiration = htonl(cmd->exptime);
            scmd.message.body.norm.flags = htonl(cmd->flags);
        }
    }

    hdr->request.cas = lcb_htonll(cmd->cas);
    hdr->request.datatype = PROTOCOL_BINARY_RAW_BYTES;

    if (should_compress || (cmd->datatype & LCB_VALUE_F_SNAPPYCOMP)) {
        hdr->request.datatype |= PROTOCOL_BINARY_DATATYPE_COMPRESSED;
    }

    if ((cmd->datatype & LCB_VALUE_F_JSON) && static_cast< const lcb::Server * >(pipeline)->supports_json()) {
        hdr->request.datatype |= PROTOCOL_BINARY_DATATYPE_JSON;
    }

    hdr->request.opaque = packet->opaque;
    hdr->request.bodylen = htonl(hdr->request.extlen + ffextlen + get_key_size(hdr) + get_value_size(packet));

    if (cmd->cmdflags & LCB_CMD_F_INTERNAL_CALLBACK) {
        packet->flags |= MCREQ_F_PRIVCALLBACK;
    }
    memcpy(SPAN_BUFFER(&packet->kh_span), scmd.bytes, hsize);
    LCB_SCHED_ADD(instance, pipeline, packet);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_STORE2NAME(cmd->operation), packet->opaque,
                      MCREQ_PKT_RDATA(packet)->span);
    TRACE_STORE_BEGIN(instance, hdr, (lcb_CMDSTORE *)cmd);

    return LCB_SUCCESS;
}

static lcb_STATUS store_validate(lcb_INSTANCE *instance, const lcb_CMDSTORE *cmd)
{
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);

    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }

    if (cmd->durability_mode == LCB_DURABILITY_SYNC) {
        if (cmd->durability.sync.dur_level && !new_durability_supported) {
            return LCB_NOT_SUPPORTED;
        }
    }
    switch (cmd->operation) {
        case LCB_STORE_APPEND:
        case LCB_STORE_PREPEND:
            if (cmd->exptime || cmd->flags) {
                return LCB_OPTIONS_CONFLICT;
            }
            break;
        case LCB_STORE_ADD:
            if (cmd->cas) {
                return LCB_OPTIONS_CONFLICT;
            }
            break;
        default:
            break;
    }

    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_store(lcb_INSTANCE *instance, void *cookie, const lcb_CMDSTORE *cmd)
{
    lcb_STATUS err;

    err = store_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, store_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdstore_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdstore_destroy,
                          cmd);
}
