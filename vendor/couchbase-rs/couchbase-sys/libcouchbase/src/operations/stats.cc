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

#include "internal.h"

struct BcastCookie : mc_REQDATAEX {
    lcb_CALLBACK_TYPE type;
    int remaining;

    BcastCookie(lcb_CALLBACK_TYPE type_, const mc_REQDATAPROCS *procs_, const void *cookie_)
        : mc_REQDATAEX(cookie_, *procs_, gethrtime()), type(type_), remaining(0)
    {
    }
};

static void refcnt_dtor_common(mc_PACKET *pkt)
{
    BcastCookie *ck = static_cast< BcastCookie * >(pkt->u_rdata.exdata);
    if (!--ck->remaining) {
        delete ck;
    }
}

static const char *make_hp_string(const lcb::Server &server, std::string &out)
{
    out.assign(server.get_host().host);
    out.append(":");
    out.append(server.get_host().port);
    return out.c_str();
}

static void stats_handler(mc_PIPELINE *pl, mc_PACKET *req, lcb_STATUS err, const void *arg)
{
    BcastCookie *ck = static_cast< BcastCookie * >(req->u_rdata.exdata);
    lcb::Server *server = static_cast< lcb::Server * >(pl);
    lcb_RESPSTATS *resp = reinterpret_cast< lcb_RESPSTATS * >(const_cast< void * >(arg));

    lcb_RESPCALLBACK callback;
    lcb_INSTANCE *instance = server->get_instance();

    callback = lcb_find_callback(instance, LCB_CALLBACK_STATS);

    if (!arg) {
        lcb_RESPSTATS s_resp = {0};
        if (--ck->remaining) {
            /* still have other servers which must reply. */
            return;
        }

        s_resp.rc = err;
        s_resp.cookie = const_cast< void * >(ck->cookie);
        s_resp.rflags = LCB_RESP_F_CLIENTGEN | LCB_RESP_F_FINAL;
        callback(instance, LCB_CALLBACK_STATS, (lcb_RESPBASE *)&s_resp);
        delete ck;

    } else {
        std::string epbuf;
        resp->server = make_hp_string(*server, epbuf);
        resp->cookie = const_cast< void * >(ck->cookie);
        callback(instance, LCB_CALLBACK_STATS, (lcb_RESPBASE *)resp);
        return;
    }
}

static mc_REQDATAPROCS stats_procs = {stats_handler, refcnt_dtor_common};

LIBCOUCHBASE_API
lcb_STATUS lcb_stats3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDSTATS *cmd)
{
    unsigned ii;
    int vbid = -1;
    char ksbuf[512] = {0};
    mc_CMDQUEUE *cq = &instance->cmdq;
    lcbvb_CONFIG *vbc = cq->config;
    const lcb_CONTIGBUF *kbuf_in = &cmd->key.contig;
    lcb_KEYBUF kbuf_out;

    kbuf_out.type = LCB_KV_COPY;

    if (cmd->cmdflags & LCB_CMDSTATS_F_KV) {
        if (kbuf_in->nbytes == 0 || kbuf_in->nbytes > sizeof(ksbuf) - 30) {
            return LCB_EINVAL;
        }
        if (vbc == NULL) {
            return LCB_CLIENT_ETMPFAIL;
        }
        if (lcbvb_get_distmode(vbc) != LCBVB_DIST_VBUCKET) {
            return LCB_NOT_SUPPORTED;
        }
        vbid = lcbvb_k2vb(vbc, kbuf_in->bytes, kbuf_in->nbytes);
        if (vbid < 0) {
            return LCB_CLIENT_ETMPFAIL;
        }
        for (ii = 0; ii < kbuf_in->nbytes; ii++) {
            if (isspace(((char *)kbuf_in->bytes)[ii])) {
                return LCB_EINVAL;
            }
        }
        sprintf(ksbuf, "key %.*s %d", (int)kbuf_in->nbytes, (const char *)kbuf_in->bytes, vbid);
        kbuf_out.contig.nbytes = strlen(ksbuf);
        kbuf_out.contig.bytes = ksbuf;
    } else {
        kbuf_out.contig = *kbuf_in;
    }

    BcastCookie *ckwrap = new BcastCookie(LCB_CALLBACK_STATS, &stats_procs, cookie);
    ckwrap->deadline = ckwrap->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    for (ii = 0; ii < cq->npipelines; ii++) {
        mc_PACKET *pkt;
        mc_PIPELINE *pl = cq->pipelines[ii];
        protocol_binary_request_header hdr = {{0}};

        if (vbid > -1 && lcbvb_has_vbucket(vbc, vbid, ii) == 0) {
            continue;
        }

        pkt = mcreq_allocate_packet(pl);
        if (!pkt) {
            return LCB_CLIENT_ENOMEM;
        }

        hdr.request.opcode = PROTOCOL_BINARY_CMD_STAT;
        hdr.request.magic = PROTOCOL_BINARY_REQ;

        pkt->flags |= MCREQ_F_NOCID;
        if (cmd->key.contig.nbytes) {
            mcreq_reserve_key(pl, pkt, MCREQ_PKT_BASESIZE, &kbuf_out, 0);
            hdr.request.keylen = ntohs((lcb_U16)kbuf_out.contig.nbytes);
            hdr.request.bodylen = ntohl((lcb_U32)kbuf_out.contig.nbytes);
        } else {
            mcreq_reserve_header(pl, pkt, MCREQ_PKT_BASESIZE);
        }

        pkt->u_rdata.exdata = ckwrap;
        pkt->flags |= MCREQ_F_REQEXT;

        ckwrap->remaining++;
        hdr.request.opaque = pkt->opaque;
        memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
        mcreq_sched_add(pl, pkt);
    }

    if (!ii) {
        delete ckwrap;
        return LCB_NO_MATCHING_SERVER;
    }

    MAYBE_SCHEDLEAVE(instance);
    return LCB_SUCCESS;
}

static void handle_bcast(mc_PIPELINE *pipeline, mc_PACKET *req, lcb_STATUS err, const void *arg)
{
    lcb::Server *server = static_cast< lcb::Server * >(pipeline);
    BcastCookie *ck = (BcastCookie *)req->u_rdata.exdata;
    lcb_RESPCALLBACK callback;

    union {
        lcb_RESPSERVERBASE *base;
        lcb_RESPVERBOSITY *verbosity;
        lcb_RESPMCVERSION *version;
        lcb_RESPNOOP *noop;
    } u_resp;

    union {
        lcb_RESPSERVERBASE base;
        lcb_RESPVERBOSITY verbosity;
        lcb_RESPMCVERSION version;
        lcb_RESPNOOP noop;
    } u_empty;

    memset(&u_empty, 0, sizeof(u_empty));

    if (arg) {
        u_resp.base = (lcb_RESPSERVERBASE *)arg;
    } else {
        u_resp.base = &u_empty.base;
        u_resp.base->rflags = LCB_RESP_F_CLIENTGEN;
    }

    u_resp.base->rc = err;
    u_resp.base->cookie = const_cast< void * >(ck->cookie);

    std::string epbuf;
    u_resp.base->server = make_hp_string(*server, epbuf);

    callback = lcb_find_callback(server->get_instance(), ck->type);
    callback(server->get_instance(), ck->type, (lcb_RESPBASE *)u_resp.base);
    if (--ck->remaining) {
        return;
    }

    u_empty.base.server = NULL;
    u_empty.base.rc = err;
    u_empty.base.rflags = LCB_RESP_F_CLIENTGEN | LCB_RESP_F_FINAL;
    u_empty.base.cookie = const_cast< void * >(ck->cookie);
    callback(server->get_instance(), ck->type, (lcb_RESPBASE *)&u_empty.base);
    delete ck;
}

static mc_REQDATAPROCS bcast_procs = {handle_bcast, refcnt_dtor_common};

static lcb_STATUS pkt_bcast_simple(lcb_INSTANCE *instance, const void *cookie, lcb_CALLBACK_TYPE type, const lcb_CMDBASE *cmd)
{
    mc_CMDQUEUE *cq = &instance->cmdq;
    unsigned ii;

    if (!cq->config) {
        return LCB_CLIENT_ETMPFAIL;
    }

    BcastCookie *ckwrap = new BcastCookie(type, &bcast_procs, cookie);
    ckwrap->deadline = ckwrap->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    for (ii = 0; ii < cq->npipelines; ii++) {
        mc_PIPELINE *pl = cq->pipelines[ii];
        mc_PACKET *pkt = mcreq_allocate_packet(pl);
        protocol_binary_request_header hdr;
        memset(&hdr, 0, sizeof(hdr));

        if (!pkt) {
            return LCB_CLIENT_ENOMEM;
        }

        pkt->u_rdata.exdata = ckwrap;
        pkt->flags |= MCREQ_F_REQEXT;

        hdr.request.magic = PROTOCOL_BINARY_REQ;
        hdr.request.opaque = pkt->opaque;
        if (type == LCB_CALLBACK_VERSIONS) {
            hdr.request.opcode = PROTOCOL_BINARY_CMD_VERSION;
        } else if (type == LCB_CALLBACK_NOOP) {
            hdr.request.opcode = PROTOCOL_BINARY_CMD_NOOP;
        } else {
            fprintf(stderr, "pkt_bcast_simple passed unknown type %u\n", type);
            lcb_assert(0);
        }

        mcreq_reserve_header(pl, pkt, MCREQ_PKT_BASESIZE);
        memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
        mcreq_sched_add(pl, pkt);
        ckwrap->remaining++;
    }

    if (ii == 0) {
        delete ckwrap;
        return LCB_NO_MATCHING_SERVER;
    }
    MAYBE_SCHEDLEAVE(instance);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_server_versions3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDVERSIONS *cmd)
{
    return pkt_bcast_simple(instance, cookie, LCB_CALLBACK_VERSIONS, (const lcb_CMDBASE *)cmd);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_noop3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDNOOP *cmd)
{
    return pkt_bcast_simple(instance, cookie, LCB_CALLBACK_NOOP, (const lcb_CMDBASE *)cmd);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_server_verbosity3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDVERBOSITY *cmd)
{
    mc_CMDQUEUE *cq = &instance->cmdq;
    unsigned ii;

    if (!cq->config) {
        return LCB_CLIENT_ETMPFAIL;
    }

    BcastCookie *ckwrap = new BcastCookie(LCB_CALLBACK_VERBOSITY, &bcast_procs, cookie);
    ckwrap->deadline = ckwrap->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    for (ii = 0; ii < cq->npipelines; ii++) {
        mc_PACKET *pkt;
        lcb::Server *server = static_cast< lcb::Server * >(cq->pipelines[ii]);
        protocol_binary_request_verbosity vcmd;
        protocol_binary_request_header *hdr = &vcmd.message.header;
        uint32_t level;

        std::string cmpbuf;
        make_hp_string(*server, cmpbuf);
        if (cmd->server && cmpbuf != cmd->server) {
            continue;
        }

        if (cmd->level == LCB_VERBOSITY_DETAIL) {
            level = 3;
        } else if (cmd->level == LCB_VERBOSITY_DEBUG) {
            level = 2;
        } else if (cmd->level == LCB_VERBOSITY_INFO) {
            level = 1;
        } else {
            level = 0;
        }

        pkt = mcreq_allocate_packet(server);
        if (!pkt) {
            return LCB_CLIENT_ENOMEM;
        }

        pkt->u_rdata.exdata = ckwrap;
        pkt->flags |= MCREQ_F_REQEXT;

        mcreq_reserve_header(server, pkt, MCREQ_PKT_BASESIZE + 4);
        hdr->request.magic = PROTOCOL_BINARY_REQ;
        hdr->request.opcode = PROTOCOL_BINARY_CMD_VERBOSITY;
        hdr->request.datatype = PROTOCOL_BINARY_RAW_BYTES;
        hdr->request.cas = 0;
        hdr->request.vbucket = 0;
        hdr->request.opaque = pkt->opaque;
        hdr->request.extlen = 4;
        hdr->request.keylen = 0;
        hdr->request.bodylen = htonl((uint32_t)hdr->request.extlen);
        vcmd.message.body.level = htonl((uint32_t)level);

        memcpy(SPAN_BUFFER(&pkt->kh_span), vcmd.bytes, sizeof(vcmd.bytes));
        mcreq_sched_add(server, pkt);
        ckwrap->remaining++;
    }

    if (!ckwrap->remaining) {
        delete ckwrap;
        return LCB_NO_MATCHING_SERVER;
    }
    MAYBE_SCHEDLEAVE(instance);
    return LCB_SUCCESS;
}
