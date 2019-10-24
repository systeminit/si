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

LIBCOUCHBASE_API lcb_STATUS lcb_respget_status(const lcb_RESPGET *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_error_context(const lcb_RESPGET *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_GET, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_error_ref(const lcb_RESPGET *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_GET, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_cookie(const lcb_RESPGET *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_cas(const lcb_RESPGET *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_datatype(const lcb_RESPGET *resp, uint8_t *datatype)
{
    *datatype = resp->datatype;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_flags(const lcb_RESPGET *resp, uint32_t *flags)
{
    *flags = resp->itmflags;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_key(const lcb_RESPGET *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respget_value(const lcb_RESPGET *resp, const char **value, size_t *value_len)
{
    *value = (const char *)resp->value;
    *value_len = resp->nvalue;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_create(lcb_CMDGET **cmd)
{
    *cmd = (lcb_CMDGET *)calloc(1, sizeof(lcb_CMDGET));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_clone(const lcb_CMDGET *cmd, lcb_CMDGET **copy)
{
    LCB_CMD_CLONE(lcb_CMDGET, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_destroy(lcb_CMDGET *cmd)
{
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_timeout(lcb_CMDGET *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_parent_span(lcb_CMDGET *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_collection(lcb_CMDGET *cmd, const char *scope, size_t scope_len,
                                                  const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_key(lcb_CMDGET *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_expiration(lcb_CMDGET *cmd, uint32_t expiration)
{
    cmd->exptime = expiration;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_locktime(lcb_CMDGET *cmd, uint32_t duration)
{
    if (duration == 0) {
        return LCB_EINVAL;
    }
    cmd->exptime = duration;
    cmd->lock = 1;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdget_durability(lcb_CMDGET *cmd, lcb_DURABILITY_LEVEL level)
{
    cmd->dur_level = level;
    return LCB_SUCCESS;
}

static lcb_STATUS get_validate(lcb_INSTANCE *instance, const lcb_CMDGET *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }
    if (cmd->cas || (cmd->dur_level && !cmd->exptime && !cmd->lock)) {
        return LCB_OPTIONS_CONFLICT;
    }
    if (cmd->dur_level && !LCBT_SUPPORT_SYNCREPLICATION(instance)) {
        return LCB_NOT_SUPPORTED;
    }

    return LCB_SUCCESS;
}

static lcb_STATUS get_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDGET *cmd = (const lcb_CMDGET *)arg;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDGET *mut = const_cast< lcb_CMDGET * >(cmd);
        mut->cid = cid;
    }

    mc_PIPELINE *pl;
    mc_PACKET *pkt;
    mc_REQDATA *rdata;
    mc_CMDQUEUE *q = &instance->cmdq;
    lcb_uint8_t extlen = 0;
    lcb_uint8_t opcode = PROTOCOL_BINARY_CMD_GET;
    protocol_binary_request_gat gcmd;
    protocol_binary_request_header *hdr = &gcmd.message.header;
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);
    lcb_U8 ffextlen = 0;
    lcb_STATUS err;

    hdr->request.magic = PROTOCOL_BINARY_REQ;
    if (cmd->lock) {
        extlen = 4;
        opcode = PROTOCOL_BINARY_CMD_GET_LOCKED;
    } else if (cmd->exptime || (cmd->cmdflags & LCB_CMDGET_F_CLEAREXP)) {
        extlen = 4;
        opcode = PROTOCOL_BINARY_CMD_GAT;
        if (cmd->dur_level && new_durability_supported) {
            hdr->request.magic = PROTOCOL_BINARY_AREQ;
            ffextlen = 4;
        }
    }

    err = mcreq_basic_packet(q, (const lcb_CMDBASE *)cmd, hdr, extlen, ffextlen, &pkt, &pl,
                             MCREQ_BASICPACKET_F_FALLBACKOK);
    if (err != LCB_SUCCESS) {
        return err;
    }

    rdata = &pkt->u_rdata.reqdata;
    rdata->cookie = cookie;
    rdata->start = gethrtime();
    rdata->deadline = rdata->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    hdr->request.opcode = opcode;
    hdr->request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr->request.bodylen = htonl(extlen + ntohs(hdr->request.keylen) + ffextlen);
    hdr->request.opaque = pkt->opaque;
    hdr->request.cas = 0;

    if (extlen) {
        if (cmd->dur_level && new_durability_supported) {
            gcmd.message.body.alt.meta = (1 << 4) | 3;
            gcmd.message.body.alt.level = cmd->dur_level;
            gcmd.message.body.alt.timeout = lcb_durability_timeout(instance);
            gcmd.message.body.alt.expiration = htonl(cmd->exptime);
        } else {
            gcmd.message.body.norm.expiration = htonl(cmd->exptime);
        }
    }

    if (cmd->cmdflags & LCB_CMD_F_INTERNAL_CALLBACK) {
        pkt->flags |= MCREQ_F_PRIVCALLBACK;
    }

    memcpy(SPAN_BUFFER(&pkt->kh_span), gcmd.bytes, MCREQ_PKT_BASESIZE + extlen + ffextlen);
    LCB_SCHED_ADD(instance, pl, pkt);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_GET, pkt->opaque, rdata->span);
    TRACE_GET_BEGIN(instance, hdr, cmd);

    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_get(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGET *cmd)
{
    lcb_STATUS err;

    err = get_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, get_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdget_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdget_destroy, cmd);
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_status(const lcb_RESPUNLOCK *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_error_context(const lcb_RESPUNLOCK *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_UNLOCK, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_error_ref(const lcb_RESPUNLOCK *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_UNLOCK, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_cookie(const lcb_RESPUNLOCK *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_cas(const lcb_RESPUNLOCK *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respunlock_key(const lcb_RESPUNLOCK *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_create(lcb_CMDUNLOCK **cmd)
{
    *cmd = (lcb_CMDUNLOCK *)calloc(1, sizeof(lcb_CMDUNLOCK));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_clone(const lcb_CMDUNLOCK *cmd, lcb_CMDUNLOCK **copy)
{
    LCB_CMD_CLONE(lcb_CMDUNLOCK, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_destroy(lcb_CMDUNLOCK *cmd)
{
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_timeout(lcb_CMDUNLOCK *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_parent_span(lcb_CMDUNLOCK *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_collection(lcb_CMDUNLOCK *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_key(lcb_CMDUNLOCK *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdunlock_cas(lcb_CMDUNLOCK *cmd, uint64_t cas)
{
    cmd->cas = cas;
    return LCB_SUCCESS;
}

static lcb_STATUS unlock_validate(lcb_INSTANCE *, const lcb_CMDUNLOCK *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }

    return LCB_SUCCESS;
}

static lcb_STATUS unlock_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDUNLOCK *cmd = (const lcb_CMDUNLOCK *)arg;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDUNLOCK *mut = const_cast< lcb_CMDUNLOCK * >(cmd);
        mut->cid = cid;
    }

    mc_CMDQUEUE *cq = &instance->cmdq;
    mc_PIPELINE *pl;
    mc_PACKET *pkt;
    mc_REQDATA *rd;
    lcb_STATUS err;
    protocol_binary_request_header hdr;

    err = mcreq_basic_packet(cq, (const lcb_CMDBASE *)cmd, &hdr, 0, 0, &pkt, &pl, MCREQ_BASICPACKET_F_FALLBACKOK);
    if (err != LCB_SUCCESS) {
        return err;
    }

    rd = &pkt->u_rdata.reqdata;
    rd->cookie = cookie;
    rd->start = gethrtime();
    rd->deadline = rd->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_UNLOCK_KEY;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.bodylen = htonl((lcb_uint32_t)ntohs(hdr.request.keylen));
    hdr.request.opaque = pkt->opaque;
    hdr.request.cas = lcb_htonll(cmd->cas);

    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
    LCB_SCHED_ADD(instance, pl, pkt);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_UNLOCK, pkt->opaque, rd->span);
    TRACE_UNLOCK_BEGIN(instance, &hdr, cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_unlock(lcb_INSTANCE *instance, void *cookie, const lcb_CMDUNLOCK *cmd)
{
    lcb_STATUS err;
    err = unlock_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, unlock_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdunlock_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdunlock_destroy,
                          cmd);
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_status(const lcb_RESPGETREPLICA *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_error_context(const lcb_RESPGETREPLICA *resp, const char **ctx,
                                                             size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_GETREPLICA, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_error_ref(const lcb_RESPGETREPLICA *resp, const char **ref,
                                                         size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_GETREPLICA, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_cookie(const lcb_RESPGETREPLICA *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_cas(const lcb_RESPGETREPLICA *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_datatype(const lcb_RESPGETREPLICA *resp, uint8_t *datatype)
{
    *datatype = resp->datatype;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_flags(const lcb_RESPGETREPLICA *resp, uint32_t *flags)
{
    *flags = resp->itmflags;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_key(const lcb_RESPGETREPLICA *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetreplica_value(const lcb_RESPGETREPLICA *resp, const char **value,
                                                     size_t *value_len)
{
    *value = (const char *)resp->value;
    *value_len = resp->nvalue;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_create(lcb_CMDGETREPLICA **cmd, lcb_REPLICA_MODE mode)
{
    lcb_CMDGETREPLICA *res = (lcb_CMDGETREPLICA *)calloc(1, sizeof(lcb_CMDGETREPLICA));
    switch (mode) {
        case LCB_REPLICA_MODE_ANY:
            res->strategy = LCB_REPLICA_FIRST;
            break;
        case LCB_REPLICA_MODE_ALL:
            res->strategy = LCB_REPLICA_ALL;
            break;
        case LCB_REPLICA_MODE_IDX0:
        case LCB_REPLICA_MODE_IDX1:
        case LCB_REPLICA_MODE_IDX2:
            res->strategy = LCB_REPLICA_SELECT;
            res->index = mode - LCB_REPLICA_MODE_IDX0;
            break;
        default:
            free(res);
            return LCB_EINVAL;
    }
    *cmd = res;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_clone(const lcb_CMDGETREPLICA *cmd, lcb_CMDGETREPLICA **copy)
{
    LCB_CMD_CLONE(lcb_CMDGETREPLICA, cmd, copy);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_destroy(lcb_CMDGETREPLICA *cmd)
{
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_timeout(lcb_CMDGETREPLICA *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_parent_span(lcb_CMDGETREPLICA *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_collection(lcb_CMDGETREPLICA *cmd, const char *scope, size_t scope_len,
                                                         const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetreplica_key(lcb_CMDGETREPLICA *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

struct RGetCookie : mc_REQDATAEX {
    RGetCookie(const void *cookie, lcb_INSTANCE *instance, lcb_replica_t, int vb);
    void decref()
    {
        if (!--remaining) {
            delete this;
        }
    }

    unsigned r_cur;
    unsigned r_max;
    int remaining;
    int vbucket;
    lcb_replica_t strategy;
    lcb_INSTANCE *instance;
};

static void rget_dtor(mc_PACKET *pkt)
{
    static_cast< RGetCookie * >(pkt->u_rdata.exdata)->decref();
}

static void rget_callback(mc_PIPELINE *, mc_PACKET *pkt, lcb_STATUS err, const void *arg)
{
    RGetCookie *rck = static_cast< RGetCookie * >(pkt->u_rdata.exdata);
    lcb_RESPGETREPLICA *resp = reinterpret_cast< lcb_RESPGETREPLICA * >(const_cast< void * >(arg));
    lcb_RESPCALLBACK callback;
    lcb_INSTANCE *instance = rck->instance;

    callback = lcb_find_callback(instance, LCB_CALLBACK_GETREPLICA);

    /** Figure out what the strategy is.. */
    if (rck->strategy == LCB_REPLICA_SELECT || rck->strategy == LCB_REPLICA_ALL) {
        /** Simplest */
        if (rck->strategy == LCB_REPLICA_SELECT || rck->remaining == 1) {
            resp->rflags |= LCB_RESP_F_FINAL;
        }
        callback(instance, LCB_CALLBACK_GETREPLICA, (const lcb_RESPBASE *)resp);
    } else {
        mc_CMDQUEUE *cq = &instance->cmdq;
        mc_PIPELINE *nextpl = NULL;

        /** FIRST */
        do {
            int nextix;
            rck->r_cur++;
            nextix = lcbvb_vbreplica(cq->config, rck->vbucket, rck->r_cur);
            if (nextix > -1 && nextix < (int)cq->npipelines) {
                /* have a valid next index? */
                nextpl = cq->pipelines[nextix];
                break;
            }
        } while (rck->r_cur < rck->r_max);

        if (err == LCB_SUCCESS || rck->r_cur == rck->r_max || nextpl == NULL) {
            resp->rflags |= LCB_RESP_F_FINAL;
            callback(instance, LCB_CALLBACK_GETREPLICA, (lcb_RESPBASE *)resp);
            /* refcount=1 . Free this now */
            rck->remaining = 1;
        } else if (err != LCB_SUCCESS) {
            mc_PACKET *newpkt = mcreq_renew_packet(pkt);
            newpkt->flags &= ~MCREQ_STATE_FLAGS;
            mcreq_sched_add(nextpl, newpkt);
            /* Use this, rather than lcb_sched_leave(), because this is being
             * invoked internally by the library. */
            mcreq_sched_leave(cq, 1);
            /* wait */
            rck->remaining = 2;
        }
    }
    rck->decref();
}

static mc_REQDATAPROCS rget_procs = {rget_callback, rget_dtor};

RGetCookie::RGetCookie(const void *cookie_, lcb_INSTANCE *instance_, lcb_replica_t strategy_, int vbucket_)
    : mc_REQDATAEX(cookie_, rget_procs, gethrtime()), r_cur(0), r_max(LCBT_NREPLICAS(instance_)), remaining(0),
      vbucket(vbucket_), strategy(strategy_), instance(instance_)
{
}

static lcb_STATUS getreplica_validate(lcb_INSTANCE *instance, const lcb_CMDGETREPLICA *cmd)
{
    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }
    if (!instance->cmdq.config) {
        return LCB_CLIENT_ETMPFAIL;
    }
    if (!LCBT_NREPLICAS(instance)) {
        return LCB_NO_MATCHING_SERVER;
    }
    return LCB_SUCCESS;
}

static lcb_STATUS getreplica_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDGETREPLICA *cmd = (const lcb_CMDGETREPLICA *)arg;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDGETREPLICA *mut = const_cast< lcb_CMDGETREPLICA * >(cmd);
        mut->cid = cid;
    }

    /**
     * Because we need to direct these commands to specific servers, we can't
     * just use the 'basic_packet()' function.
     */
    mc_CMDQUEUE *cq = &instance->cmdq;
    int vbid, ixtmp;
    protocol_binary_request_header req;
    unsigned r0, r1 = 0;

    mcreq_map_key(cq, &cmd->key, MCREQ_PKT_BASESIZE, &vbid, &ixtmp);

    /* The following blocks will also validate that the entire index range is
     * valid. This is in order to ensure that we don't allocate the cookie
     * if there aren't enough replicas online to satisfy the requirements */

    if (cmd->strategy == LCB_REPLICA_SELECT) {
        r0 = r1 = cmd->index;
        if ((ixtmp = lcbvb_vbreplica(cq->config, vbid, r0)) < 0) {
            return LCB_NO_MATCHING_SERVER;
        }

    } else if (cmd->strategy == LCB_REPLICA_ALL) {
        unsigned ii;
        r0 = 0;
        r1 = LCBT_NREPLICAS(instance);
        /* Make sure they're all online */
        for (ii = 0; ii < LCBT_NREPLICAS(instance); ii++) {
            if ((ixtmp = lcbvb_vbreplica(cq->config, vbid, ii)) < 0) {
                return LCB_NO_MATCHING_SERVER;
            }
        }
    } else {
        for (r0 = 0; r0 < LCBT_NREPLICAS(instance); r0++) {
            if ((ixtmp = lcbvb_vbreplica(cq->config, vbid, r0)) > -1) {
                r1 = r0;
                break;
            }
        }
        if (r0 == LCBT_NREPLICAS(instance)) {
            return LCB_NO_MATCHING_SERVER;
        }
    }

    if (r1 < r0 || r1 >= cq->npipelines) {
        return LCB_NO_MATCHING_SERVER;
    }

    /* Initialize the cookie */
    RGetCookie *rck = new RGetCookie(cookie, instance, cmd->strategy, vbid);
    rck->deadline = rck->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    /* Initialize the packet */
    req.request.magic = PROTOCOL_BINARY_REQ;
    req.request.opcode = PROTOCOL_BINARY_CMD_GET_REPLICA;
    req.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    req.request.vbucket = htons((lcb_uint16_t)vbid);
    req.request.cas = 0;
    req.request.extlen = 0;
    req.request.keylen = htons((lcb_uint16_t)cmd->key.contig.nbytes);
    req.request.bodylen = htonl((lcb_uint32_t)cmd->key.contig.nbytes);

    rck->r_cur = r0;
    do {
        int curix;
        mc_PIPELINE *pl;
        mc_PACKET *pkt;

        curix = lcbvb_vbreplica(cq->config, vbid, r0);
        /* XXX: this is always expected to be in range. For the FIRST mode
         * it will seek to the first valid index (checked above), and for the
         * ALL mode, it will fail if not all replicas are already online
         * (also checked above) */
        pl = cq->pipelines[curix];
        pkt = mcreq_allocate_packet(pl);
        if (!pkt) {
            return LCB_CLIENT_ENOMEM;
        }

        pkt->u_rdata.exdata = rck;
        pkt->flags |= MCREQ_F_REQEXT;

        mcreq_reserve_key(pl, pkt, sizeof(req.bytes), &cmd->key, cmd->cid);

        req.request.opaque = pkt->opaque;
        rck->remaining++;
        mcreq_write_hdr(pkt, &req);
        mcreq_sched_add(pl, pkt);
    } while (++r0 < r1);

    MAYBE_SCHEDLEAVE(instance);

    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_getreplica(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETREPLICA *cmd)
{
    lcb_STATUS err;

    err = getreplica_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, getreplica_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdgetreplica_clone,
                          (lcb_COLLCACHE_ARG_DTOR)lcb_cmdgetreplica_destroy, cmd);
}
