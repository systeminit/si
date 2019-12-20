/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2019 Couchbase, Inc.
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
#include "trace.h"

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_status(const lcb_RESPEXISTS *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API int lcb_respexists_is_persisted(const lcb_RESPEXISTS *resp)
{
    return resp->state == 0x01;
}

LIBCOUCHBASE_API int lcb_respexists_is_found(const lcb_RESPEXISTS *resp)
{
    return resp->state == 0x01 || resp->state == 0x00;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_error_context(const lcb_RESPEXISTS *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_EXISTS, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(*ctx);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_error_ref(const lcb_RESPEXISTS *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_EXISTS, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_cookie(const lcb_RESPEXISTS *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_cas(const lcb_RESPEXISTS *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_key(const lcb_RESPEXISTS *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respexists_mutation_token(const lcb_RESPEXISTS *resp, lcb_MUTATION_TOKEN *token)
{
    const lcb_MUTATION_TOKEN *mt = lcb_resp_get_mutation_token(LCB_CALLBACK_EXISTS, (const lcb_RESPBASE *)resp);
    if (token && mt) {
        *token = *mt;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_create(lcb_CMDEXISTS **cmd)
{
    *cmd = (lcb_CMDEXISTS *)calloc(1, sizeof(lcb_CMDEXISTS));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_clone(const lcb_CMDEXISTS *cmd, lcb_CMDEXISTS **copy)
{
    LCB_CMD_CLONE(lcb_CMDEXISTS, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_destroy(lcb_CMDEXISTS *cmd)
{
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_timeout(lcb_CMDEXISTS *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_parent_span(lcb_CMDEXISTS *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_collection(lcb_CMDEXISTS *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdexists_key(lcb_CMDEXISTS *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

static lcb_STATUS exists_validate(lcb_INSTANCE *instance, const lcb_CMDEXISTS *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }
    if (!instance->cmdq.config) {
        return LCB_CLIENT_ETMPFAIL;
    }
    if (LCBVB_DISTTYPE(LCBT_VBCONFIG(instance)) != LCBVB_DIST_VBUCKET) {
        return LCB_NOT_SUPPORTED;
    }
    return LCB_SUCCESS;
}

static lcb_STATUS exists_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDEXISTS *cmd = (const lcb_CMDEXISTS *)arg;
    uint8_t ncid = 0;
    uint8_t ecid[5] = {0}; /* encoded */

    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDEXISTS *mut = const_cast< lcb_CMDEXISTS * >(cmd);
        mut->cid = cid;
        ncid = leb128_encode(cid, ecid);
    }

    mc_CMDQUEUE *cq = &instance->cmdq;

    int vbid, srvix;
    mcreq_map_key(cq, &cmd->key, 24, &vbid, &srvix);

    protocol_binary_request_header hdr;
    mc_PIPELINE *pipeline = cq->pipelines[srvix];
    mc_PACKET *pkt = mcreq_allocate_packet(pipeline);

    uint32_t bodylen = (sizeof(uint16_t) * 2) + ncid + cmd->key.contig.nbytes;
    mcreq_reserve_header(pipeline, pkt, MCREQ_PKT_BASESIZE);
    mcreq_reserve_value2(pipeline, pkt, bodylen);

    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_OBSERVE;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.keylen = 0;
    hdr.request.cas = 0;
    hdr.request.vbucket = 0;
    hdr.request.extlen = 0;
    hdr.request.opaque = pkt->opaque;
    hdr.request.bodylen = htonl(bodylen);

    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
    char *ptr = SPAN_BUFFER(&pkt->u_value.single);
    uint16_t tmp = htons(vbid);
    memcpy(ptr, &tmp, sizeof(tmp));
    ptr += sizeof(tmp);
    tmp = htons(ncid + cmd->key.contig.nbytes);
    memcpy(ptr, &tmp, sizeof(tmp));
    ptr += sizeof(tmp);
    if (ncid) {
        memcpy(ptr, ecid, ncid);
        ptr += ncid;
    }
    memcpy(ptr, cmd->key.contig.bytes, cmd->key.contig.nbytes);

    pkt->u_rdata.reqdata.cookie = cookie;
    pkt->u_rdata.reqdata.start = gethrtime();
    pkt->u_rdata.reqdata.deadline = pkt->u_rdata.reqdata.start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
    LCB_SCHED_ADD(instance, pipeline, pkt);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_EXISTS, pkt->opaque, pkt->u_rdata.reqdata.span);
    TRACE_EXISTS_BEGIN(instance, &hdr, cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_exists(lcb_INSTANCE *instance, void *cookie, const lcb_CMDEXISTS *cmd)
{
    lcb_STATUS err;

    err = exists_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, exists_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdexists_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdexists_destroy,
                          cmd);
}
