/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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
#include "internal.h"

lcb_STATUS lcb_observe_seqno3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDOBSEQNO *cmd)
{
    mc_PACKET *pkt;
    protocol_binary_request_header hdr;
    lcb_U64 uuid;

    if (cmd->server_index > LCBT_NSERVERS(instance)) {
        return LCB_EINVAL;
    }

    lcb::Server *server = instance->get_server(cmd->server_index);
    pkt = mcreq_allocate_packet(server);
    mcreq_reserve_header(server, pkt, MCREQ_PKT_BASESIZE);
    mcreq_reserve_value2(server, pkt, 8);

    /* Set the static fields */
    MCREQ_PKT_RDATA(pkt)->cookie = cookie;
    MCREQ_PKT_RDATA(pkt)->start = gethrtime();
    MCREQ_PKT_RDATA(pkt)->deadline = MCREQ_PKT_RDATA(pkt)->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
    if (cmd->cmdflags & LCB_CMD_F_INTERNAL_CALLBACK) {
        pkt->flags |= MCREQ_F_PRIVCALLBACK;
    }

    memset(&hdr, 0, sizeof hdr);
    hdr.request.opaque = pkt->opaque;
    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.opcode = PROTOCOL_BINARY_CMD_OBSERVE_SEQNO;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.bodylen = htonl((lcb_U32)8);
    hdr.request.vbucket = htons(cmd->vbid);
    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof hdr.bytes);

    uuid = lcb_htonll(cmd->uuid);
    memcpy(SPAN_BUFFER(&pkt->u_value.single), &uuid, sizeof uuid);
    LCB_SCHED_ADD(instance, server, pkt);
    LCBTRACE_KV_START(instance->settings, cmd, LCBTRACE_OP_OBSERVE_SEQNO, pkt->opaque, MCREQ_PKT_RDATA(pkt)->span);
    return LCB_SUCCESS;
}

const lcb_MUTATION_TOKEN *lcb_get_mutation_token(lcb_INSTANCE *instance, const lcb_KEYBUF *kb, lcb_STATUS *errp)
{
    int vbix, srvix;
    lcb_STATUS err_s;
    const lcb_MUTATION_TOKEN *existing;

    if (!errp) {
        errp = &err_s;
    }

    if (!LCBT_VBCONFIG(instance)) {
        *errp = LCB_CLIENT_ETMPFAIL;
        return NULL;
    }
    if (LCBT_VBCONFIG(instance)->dtype != LCBVB_DIST_VBUCKET) {
        *errp = LCB_NOT_SUPPORTED;
        return NULL;
    }
    if (!LCBT_SETTING(instance, fetch_mutation_tokens)) {
        *errp = LCB_NOT_SUPPORTED;
        return NULL;
    }

    if (!instance->dcpinfo) {
        *errp = LCB_DURABILITY_NO_MUTATION_TOKENS;
        return NULL;
    }

    mcreq_map_key(&instance->cmdq, kb, 0, &vbix, &srvix);
    existing = instance->dcpinfo + vbix;
    if (existing->uuid_ == 0 && existing->seqno_ == 0) {
        *errp = LCB_DURABILITY_NO_MUTATION_TOKENS;
        return NULL;
    }
    *errp = LCB_SUCCESS;
    return existing;
}
