/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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
#include "packetutils.h"
#include <bucketconfig/clconfig.h>

static void ext_callback_proxy(mc_PIPELINE *pl, mc_PACKET *req, lcb_STATUS rc, const void *resdata)
{
    lcb::Server *server = static_cast< lcb::Server * >(pl);
    const lcb::MemcachedResponse *res = reinterpret_cast< const lcb::MemcachedResponse * >(resdata);

    mc_REQDATAEX *rd = req->u_rdata.exdata;
    switch (res->opcode()) {
        case PROTOCOL_BINARY_CMD_SELECT_BUCKET:
            lcb::clconfig::select_status(rd->cookie, rc);
            break;
        case PROTOCOL_BINARY_CMD_GET_CLUSTER_CONFIG:
            lcb::clconfig::cccp_update(rd->cookie, rc, res->value(), res->vallen(), &server->get_host());
            break;
    }
    free(rd);
}

static mc_REQDATAPROCS procs = {ext_callback_proxy};

lcb_STATUS lcb_st::request_config(const void *cookie_, lcb::Server *server)
{
    lcb_STATUS err;
    mc_PACKET *packet;
    mc_REQDATAEX *rd;

    packet = mcreq_allocate_packet(server);
    if (!packet) {
        return LCB_CLIENT_ENOMEM;
    }

    err = mcreq_reserve_header(server, packet, 24);
    if (err != LCB_SUCCESS) {
        mcreq_release_packet(server, packet);
        return err;
    }

    rd = reinterpret_cast< mc_REQDATAEX * >(calloc(1, sizeof(*rd)));
    rd->procs = &procs;
    rd->cookie = cookie_;
    rd->start = gethrtime();
    rd->deadline = rd->start + LCB_US2NS(LCBT_SETTING(reinterpret_cast<lcb_INSTANCE *>(cmdq.cqdata), config_node_timeout));
    packet->u_rdata.exdata = rd;
    packet->flags |= MCREQ_F_REQEXT;

    lcb::MemcachedRequest hdr(PROTOCOL_BINARY_CMD_GET_CLUSTER_CONFIG, packet->opaque);
    hdr.opaque(packet->opaque);
    memcpy(SPAN_BUFFER(&packet->kh_span), hdr.data(), hdr.size());

    mcreq_sched_enter(&cmdq);
    mcreq_sched_add(server, packet);
    mcreq_sched_leave(&cmdq, 1);
    return LCB_SUCCESS;
}

lcb_STATUS lcb_st::select_bucket(const void *cookie_, lcb::Server *server)
{
    lcb_STATUS err;
    mc_PACKET *packet;
    mc_REQDATAEX *rd;

    packet = mcreq_allocate_packet(server);
    if (!packet) {
        return LCB_CLIENT_ENOMEM;
    }

    err = mcreq_reserve_header(server, packet, 24);
    if (err != LCB_SUCCESS) {
        mcreq_release_packet(server, packet);
        return err;
    }

    rd = reinterpret_cast< mc_REQDATAEX * >(calloc(1, sizeof(*rd)));
    rd->procs = &procs;
    rd->cookie = cookie_;
    rd->start = gethrtime();
    rd->deadline = rd->start + LCB_US2NS(LCBT_SETTING(reinterpret_cast<lcb_INSTANCE *>(cmdq.cqdata), config_node_timeout));
    packet->u_rdata.exdata = rd;
    packet->flags |= MCREQ_F_REQEXT;

    lcb_KEYBUF key = {};
    LCB_KREQ_SIMPLE(&key, settings->bucket, strlen(settings->bucket));
    packet->flags |= MCREQ_F_NOCID;
    mcreq_reserve_key(server, packet, MCREQ_PKT_BASESIZE, &key, 0);

    lcb::MemcachedRequest hdr(PROTOCOL_BINARY_CMD_SELECT_BUCKET, packet->opaque);
    hdr.opaque(packet->opaque);
    hdr.sizes(0, strlen(settings->bucket), 0);
    memcpy(SPAN_BUFFER(&packet->kh_span), hdr.data(), hdr.size());

    mcreq_sched_enter(&cmdq);
    mcreq_sched_add(server, packet);
    mcreq_sched_leave(&cmdq, 0);
    return LCB_SUCCESS;
}
