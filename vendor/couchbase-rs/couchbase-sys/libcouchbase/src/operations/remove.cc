/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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
#include "trace.h"

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_status(const lcb_RESPREMOVE *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_error_context(const lcb_RESPREMOVE *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_REMOVE, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_error_ref(const lcb_RESPREMOVE *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_REMOVE, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_cookie(const lcb_RESPREMOVE *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_cas(const lcb_RESPREMOVE *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_key(const lcb_RESPREMOVE *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respremove_mutation_token(const lcb_RESPREMOVE *resp, lcb_MUTATION_TOKEN *token)
{
    const lcb_MUTATION_TOKEN *mt = lcb_resp_get_mutation_token(LCB_CALLBACK_REMOVE, (const lcb_RESPBASE *)resp);
    if (token && mt) {
        *token = *mt;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_create(lcb_CMDREMOVE **cmd)
{
    *cmd = (lcb_CMDREMOVE *)calloc(1, sizeof(lcb_CMDREMOVE));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_clone(const lcb_CMDREMOVE *cmd, lcb_CMDREMOVE **copy)
{
    LCB_CMD_CLONE(lcb_CMDREMOVE, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_destroy(lcb_CMDREMOVE *cmd)
{
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_timeout(lcb_CMDREMOVE *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_parent_span(lcb_CMDREMOVE *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_collection(lcb_CMDREMOVE *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_key(lcb_CMDREMOVE *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_cas(lcb_CMDREMOVE *cmd, uint64_t cas)
{
    cmd->cas = cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdremove_durability(lcb_CMDREMOVE *cmd, lcb_DURABILITY_LEVEL level)
{
    cmd->dur_level = level;
    return LCB_SUCCESS;
}

static lcb_STATUS remove_validate(lcb_INSTANCE *instance, const lcb_CMDREMOVE *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }
    if (cmd->dur_level && !LCBT_SUPPORT_SYNCREPLICATION(instance)) {
        return LCB_NOT_SUPPORTED;
    }
    return LCB_SUCCESS;
}

static lcb_STATUS remove_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDREMOVE *cmd = (const lcb_CMDREMOVE *)arg;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDREMOVE *mut = const_cast< lcb_CMDREMOVE * >(cmd);
        mut->cid = cid;
    }

    mc_CMDQUEUE *cq = &instance->cmdq;
    mc_PIPELINE *pl;
    mc_PACKET *pkt;
    protocol_binary_request_delete req = {0};
    protocol_binary_request_header *hdr = &req.message.header;
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);
    lcb_U8 ffextlen = 0;
    size_t hsize;
    lcb_STATUS err;

    if (cmd->dur_level && new_durability_supported) {
        hdr->request.magic = PROTOCOL_BINARY_AREQ;
        ffextlen = 4;
    }

    err = mcreq_basic_packet(cq, (const lcb_CMDBASE *)cmd, hdr, 0, ffextlen, &pkt, &pl, MCREQ_BASICPACKET_F_FALLBACKOK);
    if (err != LCB_SUCCESS) {
        return err;
    }
    hsize = hdr->request.extlen + sizeof(*hdr) + ffextlen;

    hdr->request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr->request.magic = PROTOCOL_BINARY_REQ;
    hdr->request.opcode = PROTOCOL_BINARY_CMD_DELETE;
    hdr->request.cas = lcb_htonll(cmd->cas);
    hdr->request.opaque = pkt->opaque;
    hdr->request.bodylen = htonl(ffextlen + hdr->request.extlen + (lcb_uint32_t)ntohs(hdr->request.keylen));
    if (cmd->dur_level && new_durability_supported) {
        req.message.body.alt.meta = (1 << 4) | 3;
        req.message.body.alt.level = cmd->dur_level;
        req.message.body.alt.timeout = lcb_durability_timeout(instance);
    }

    pkt->u_rdata.reqdata.cookie = cookie;
    pkt->u_rdata.reqdata.start = gethrtime();
    pkt->u_rdata.reqdata.deadline = pkt->u_rdata.reqdata.start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr->bytes, hsize);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_REMOVE, pkt->opaque, pkt->u_rdata.reqdata.span);
    TRACE_REMOVE_BEGIN(instance, hdr, cmd);
    LCB_SCHED_ADD(instance, pl, pkt);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_remove(lcb_INSTANCE *instance, void *cookie, const lcb_CMDREMOVE *cmd)
{
    lcb_STATUS err;

    err = remove_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, remove_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdremove_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdremove_destroy,
                          cmd);
}
