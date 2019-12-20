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

#include <libcouchbase/couchbase.h>
#include <libcouchbase/pktfwd.h>
#include "mc/mcreq.h"
#include "mc/forward.h"
#include "internal.h"
#include "rdb/rope.h"

LIBCOUCHBASE_API
lcb_STATUS lcb_pktfwd3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDPKTFWD *cmd)
{
    int fwdopts = 0;
    mc_PIPELINE *pl;
    mc_PACKET *packet;
    nb_IOV *iov, iov_s;
    unsigned niov;
    mc_IOVINFO ioi = {{0}};
    lcb_STATUS err;

    if (cmd->nomap) {
        fwdopts |= MC_FWD_OPT_NOMAP;
        if (cmd->server_index >= LCBT_NSERVERS(instance)) {
            return LCB_NO_MATCHING_SERVER;
        } else {
            pl = (mc_PIPELINE *)LCBT_GET_SERVER(instance, cmd->server_index);
        }
    }

    if (cmd->vb.vtype != LCB_KV_IOV) {
        iov_s.iov_base = (void *)cmd->vb.u_buf.contig.bytes;
        iov_s.iov_len = cmd->vb.u_buf.contig.nbytes;
        iov = &iov_s;
        niov = 1;

        if (cmd->vb.vtype == LCB_KV_COPY) {
            fwdopts |= MC_FWD_OPT_COPY;
        }
    } else {
        iov = (nb_IOV *)cmd->vb.u_buf.multi.iov;
        niov = cmd->vb.u_buf.multi.niov;
        ioi.total = cmd->vb.u_buf.multi.total_length;
    }
    mc_iovinfo_init(&ioi, iov, niov);

    err = mc_forward_packet(&instance->cmdq, &ioi, &packet, &pl, fwdopts);
    if (err != LCB_SUCCESS) {
        return err;
    }

    /* set the cookie */
    packet->u_rdata.reqdata.cookie = cookie;
    packet->u_rdata.reqdata.start = gethrtime();
    packet->u_rdata.reqdata.deadline = packet->u_rdata.reqdata.start + LCB_US2NS(LCBT_SETTING(instance, operation_timeout));
    return err;
}

LIBCOUCHBASE_API
void lcb_backbuf_ref(lcb_BACKBUF buf)
{
    rdb_seg_ref(buf);
}

LIBCOUCHBASE_API
void lcb_backbuf_unref(lcb_BACKBUF buf)
{
    rdb_seg_unref(buf);
}
