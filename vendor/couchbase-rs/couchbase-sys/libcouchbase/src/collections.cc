/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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
#include "settings.h"
#include "internal.h"
#include "mcserver/negotiate.h"

#include <string>
#include <sstream>

#define LOGARGS(instance, lvl) ()->m_instance->settings, "c9smgmt", LCB_LOG_##lvl, __FILE__, __LINE__

namespace lcb {
    CollectionCache::CollectionCache(): cache_n2i(), cache_i2n()
    {
    }

    CollectionCache::~CollectionCache()
    {
    }

    std::string CollectionCache::id_to_name(uint32_t cid)
    {
        std::map<uint32_t, std::string>::const_iterator pos = cache_i2n.find(cid);
        if (pos != cache_i2n.end()) {
            return pos->second;
        }
        return "";
    }

    bool CollectionCache::get(std::string path, uint32_t *cid)
    {
        std::map<std::string, uint32_t>::const_iterator pos = cache_n2i.find(path);
        if (pos != cache_n2i.end()) {
            *cid = pos->second;
            return true;
        }
        return false;
    }

    void CollectionCache::put(std::string path, uint32_t cid)
    {
        cache_n2i[path] = cid;
        cache_i2n[cid] = path;
    }

    void CollectionCache::erase(uint32_t cid)
    {
        std::map<uint32_t, std::string>::iterator pos = cache_i2n.find(cid);
        if (pos != cache_i2n.end()) {
            cache_n2i.erase(pos->second);
            cache_i2n.erase(pos);
        }
    }
}

struct GetCidCtx : mc_REQDATAEX {
    lcb_INSTANCE *instance;
    std::string path;
    lcb_COLLCACHE_CALLBACK cb;
    lcb_COLLCACHE_ARG_DTOR dtor;
    void *arg;

    static mc_REQDATAPROCS proctable;

    GetCidCtx(lcb_INSTANCE *instance_, void *cookie_, std::string path_, lcb_COLLCACHE_CALLBACK cb_,
            lcb_COLLCACHE_ARG_CLONE clone_, lcb_COLLCACHE_ARG_DTOR dtor_, const void *arg_)
        : mc_REQDATAEX(cookie_, proctable, gethrtime()), instance(instance_), path(path_), cb(cb_), dtor(dtor_), arg(NULL)
    {
        clone_(arg_, &arg);
    }

    ~GetCidCtx() {
        if (arg) {
            dtor(arg);
            arg = NULL;
        }
    }
};

static void handle_collcache_proc(mc_PIPELINE *, mc_PACKET *pkt, lcb_STATUS err, const void *rb)
{
    GetCidCtx *ctx = static_cast< GetCidCtx * >(pkt->u_rdata.exdata);
    const lcb_RESPGETCID *resp = (const lcb_RESPGETCID *)rb;
    uint32_t cid = resp->collection_id;
    ctx->instance->collcache->put(ctx->path, cid);
    lcb_STATUS rc = ctx->cb(cid, ctx->instance, (void *)ctx->cookie, ctx->arg);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "failed to schedule command\n");
    }
    delete ctx;
}

static void handle_collcache_schedfail(mc_PACKET *pkt)
{
    delete static_cast< GetCidCtx * >(pkt->u_rdata.exdata);
}

mc_REQDATAPROCS GetCidCtx::proctable = {handle_collcache_proc, handle_collcache_schedfail};

lcb_STATUS collcache_exec_str(std::string collection, lcb_INSTANCE *instance, void *cookie, lcb_COLLCACHE_CALLBACK cb,
        lcb_COLLCACHE_ARG_CLONE clone, lcb_COLLCACHE_ARG_DTOR dtor, const void *arg)
{
    if (!LCBT_SETTING(instance, use_collections)) {
        if (!collection.empty()) {
            return LCB_NOT_SUPPORTED;
        }
        return cb(0, instance, cookie, arg);
    }

    uint32_t cid = 0;
    if (instance->collcache->get(collection, &cid)) {
        return cb(cid, instance, cookie, arg);
    }

    mc_CMDQUEUE *cq = &instance->cmdq;
    if (cq->config == NULL) {
        return LCB_CLIENT_ETMPFAIL;
    }

    /* TODO: rotate pipelines */
    if (cq->npipelines < 1) {
        return LCB_NO_MATCHING_SERVER;
    }
    mc_PIPELINE *pl = cq->pipelines[0];
    mc_PACKET *pkt = mcreq_allocate_packet(pl);
    if (!pkt) {
        return LCB_CLIENT_ENOMEM;
    }
    mcreq_reserve_header(pl, pkt, MCREQ_PKT_BASESIZE);
    lcb_KEYBUF key = {};
    LCB_KREQ_SIMPLE(&key, collection.c_str(), collection.size());
    pkt->flags |= MCREQ_F_NOCID;
    mcreq_reserve_key(pl, pkt, MCREQ_PKT_BASESIZE, &key, 0);
    protocol_binary_request_header hdr = {0};
    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_COLLECTIONS_GET_CID;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.opaque = pkt->opaque;
    hdr.request.keylen = ntohs(collection.size());
    hdr.request.bodylen = htonl(collection.size());
    mcreq_write_hdr(pkt, &hdr);

    GetCidCtx *ctx = new GetCidCtx(instance, cookie, collection, cb, clone, dtor, arg);
    pkt->u_rdata.exdata = ctx;
    pkt->flags |= MCREQ_F_REQEXT;

    LCB_SCHED_ADD(instance, pl, pkt);
    return LCB_SUCCESS;
}

lcb_STATUS collcache_exec(const char *scope, size_t nscope, const char *collection, size_t ncollection,
        lcb_INSTANCE *instance, void *cookie, lcb_COLLCACHE_CALLBACK cb,
        lcb_COLLCACHE_ARG_CLONE clone, lcb_COLLCACHE_ARG_DTOR dtor, const void *arg)
{
    if (LCBT_SETTING(instance, conntype) != LCB_TYPE_BUCKET) {
        return LCB_NOT_SUPPORTED;
    }
    if (!LCBT_SETTING(instance, use_collections)) {
        if (scope != NULL || collection != NULL) {
            return LCB_NOT_SUPPORTED;
        }
        return cb(0, instance, cookie, arg);
    }

    std::string s(scope, nscope);
    std::string c(collection, ncollection);

    return collcache_exec_str(s + "." + c, instance, cookie, cb, clone, dtor, arg);
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_status(const lcb_RESPGETMANIFEST *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_cookie(const lcb_RESPGETMANIFEST *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetmanifest_value(const lcb_RESPGETMANIFEST *resp, const char **json, size_t *json_len)
{
    *json = resp->value;
    *json_len = resp->nvalue;
    return LCB_SUCCESS;
}


LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_create(lcb_CMDGETMANIFEST **cmd)
{
    *cmd = (lcb_CMDGETMANIFEST *)calloc(1, sizeof(lcb_CMDGETMANIFEST));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_destroy(lcb_CMDGETMANIFEST *cmd)
{
    free(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetmanifest_timeout(lcb_CMDGETMANIFEST *cmd, uint32_t timeout)
{
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_getmanifest(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETMANIFEST *cmd)
{
    mc_CMDQUEUE *cq = &instance->cmdq;
    if (cq->config == NULL) {
        return LCB_CLIENT_ETMPFAIL;
    }
    if (!LCBT_SETTING(instance, use_collections)) {
        return LCB_NOT_SUPPORTED;
    }
    if (cq->npipelines < 1) {
        return LCB_NO_MATCHING_SERVER;
    }
    mc_PIPELINE *pl = cq->pipelines[0];

    mc_PACKET *pkt = mcreq_allocate_packet(pl);
    if (!pkt) {
        return LCB_CLIENT_ENOMEM;
    }
    mcreq_reserve_header(pl, pkt, MCREQ_PKT_BASESIZE);

    protocol_binary_request_header hdr = {0};
    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_COLLECTIONS_GET_MANIFEST;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.opaque = pkt->opaque;
    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));

    pkt->u_rdata.reqdata.cookie = cookie;
    pkt->u_rdata.reqdata.start = gethrtime();
    pkt->u_rdata.reqdata.deadline = pkt->u_rdata.reqdata.start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    LCB_SCHED_ADD(instance, pl, pkt);
    (void)cmd;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_status(const lcb_RESPGETCID *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_scoped_collection(const lcb_RESPGETCID *resp, const char **name, size_t *name_len)
{
    *name = (const char *)resp->key;
    *name_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_collection_id(const lcb_RESPGETCID *resp, uint32_t *id)
{
    *id = resp->collection_id;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_manifest_id(const lcb_RESPGETCID *resp, uint64_t *id)
{
    *id = resp->manifest_id;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respgetcid_cookie(const lcb_RESPGETCID *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}


LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_create(lcb_CMDGETCID **cmd)
{
    *cmd = (lcb_CMDGETCID *)calloc(1, sizeof(lcb_CMDGETCID));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_destroy(lcb_CMDGETCID *cmd)
{
    free(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_timeout(lcb_CMDGETCID *cmd, uint32_t timeout)
{
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_scope(lcb_CMDGETCID *cmd, const char *scope, size_t scope_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdgetcid_collection(lcb_CMDGETCID *cmd, const char *collection, size_t collection_len)
{
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}


LIBCOUCHBASE_API
lcb_STATUS lcb_getcid(lcb_INSTANCE *instance, void *cookie, const lcb_CMDGETCID *cmd)
{
    mc_CMDQUEUE *cq = &instance->cmdq;
    if (cq->config == NULL) {
        return LCB_CLIENT_ETMPFAIL;
    }
    if (!LCBT_SETTING(instance, use_collections)) {
        return LCB_NOT_SUPPORTED;
    }
    if (cmd->nscope == 0 || cmd->scope == NULL || cmd->ncollection == 0 || cmd->collection == NULL) {
        return LCB_EINVAL;
    }
    if (cq->npipelines < 1) {
        return LCB_NO_MATCHING_SERVER;
    }
    mc_PIPELINE *pl = cq->pipelines[0];

    mc_PACKET *pkt = mcreq_allocate_packet(pl);
    if (!pkt) {
        return LCB_CLIENT_ENOMEM;
    }
    mcreq_reserve_header(pl, pkt, MCREQ_PKT_BASESIZE);

    std::string path("");
    path.append(cmd->scope, cmd->nscope);
    path.append(".");
    path.append(cmd->collection, cmd->ncollection);

    lcb_KEYBUF key = {};
    LCB_KREQ_SIMPLE(&key, path.c_str(), path.size());
    pkt->flags |= MCREQ_F_NOCID;
    mcreq_reserve_key(pl, pkt, MCREQ_PKT_BASESIZE, &key, 0);

    protocol_binary_request_header hdr = {0};
    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_COLLECTIONS_GET_CID;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.opaque = pkt->opaque;
    hdr.request.keylen = ntohs(path.size());
    hdr.request.bodylen = htonl(path.size());
    mcreq_write_hdr(pkt, &hdr);


    pkt->u_rdata.reqdata.cookie = cookie;
    pkt->u_rdata.reqdata.start = gethrtime();
    pkt->u_rdata.reqdata.deadline = pkt->u_rdata.reqdata.start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    LCB_SCHED_ADD(instance, pl, pkt);
    return LCB_SUCCESS;
}
