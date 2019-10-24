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

#include "mcreq.h"
#include "forward.h"
#include "iovcursor-inl.h"
#include <stdio.h>

#define MINIMUM(a, b) (a) < (b) ? a : b

static void span_from_first(mc_IOVCURSOR *cursor, unsigned size, nb_SPAN *span)
{
    nb_IOV dummy;
    iovcursor_adv_first(cursor, size, &dummy);
    CREATE_STANDALONE_SPAN(span, dummy.iov_base, dummy.iov_len);
}

void mc_iovinfo_init(mc_IOVINFO *info, const nb_IOV *iov, unsigned niov)
{
    unsigned ii;
    info->c.iov = (void *)iov;
    info->c.niov = niov;
    info->c.offset = 0;
    info->total = 0;

    for (ii = 0; ii < niov; ii++) {
        info->total += iov[ii].iov_len;
    }
}

#define REQLEN_HDR(req) sizeof(req.bytes)
#define REQLEN_

lcb_STATUS mc_forward_packet(mc_CMDQUEUE *cq, mc_IOVINFO *info, mc_PACKET **pkt_p, mc_PIPELINE **pl_p, int options)
{
    /* stack based header with our modifications. this is copied into the
     * packet's actual header */
    protocol_binary_request_header hdr;
    int vbid, srvix;
    mc_IOVCURSOR *mincur = &info->c;

    unsigned n_packet;     /* total packet size */
    unsigned n_header;     /* extras + key + memcached header */
    unsigned n_body_total; /* size of everything following the memcached header */
    unsigned n_body_key;   /* length of the key */
    unsigned n_body_value; /* packetsize - hdrsize */

    unsigned offset;

    /* stack buffer and key pointer. stack buffer is used if the key is not
     * contiguous */
    char kbuf_s[256];
    const char *kptr;

    /* pipeline and packet for command */
    mc_PIPELINE *pl;
    mc_PACKET *pkt;
    info->wanted = 0;

    /* not enough bytes */
    if (info->total < 24) {
        info->wanted = 24;
        return LCB_INCOMPLETE_PACKET;
    }

    iovcursor_peek(mincur, (char *)hdr.bytes, sizeof hdr.bytes, 0);

    /* Initialize our size variables */
    n_body_total = ntohl(hdr.request.bodylen);
    n_body_key = ntohs(hdr.request.keylen);
    n_header = sizeof hdr.bytes + n_body_key + hdr.request.extlen;
    n_packet = n_body_total + sizeof hdr.bytes;
    n_body_value = n_packet - n_header;

    if (n_packet > info->total) {
        info->wanted = n_packet;
        return LCB_INCOMPLETE_PACKET;
    }

    info->total -= n_packet;

    /* seek ahead to read the item's key into the header */
    offset = sizeof hdr.bytes + hdr.request.extlen;

    iovcursor_peek_ex(mincur, kbuf_s, &kptr, n_body_key, offset);

    if (kptr == NULL) {
        /* key is not contiguous? that's ok. use the static buffer */
        kptr = kbuf_s;
    }

    if ((options & MC_FWD_OPT_NOMAP) == 0) {
        lcbvb_map_key(cq->config, kptr, n_body_key, &vbid, &srvix);
        if (srvix < 0 || (unsigned)srvix >= cq->npipelines) {
            return LCB_NO_MATCHING_SERVER;
        }
        pl = cq->pipelines[srvix];
        hdr.request.vbucket = htons(vbid);

    } else {
        pl = *pl_p;
        if (!pl) {
            return LCB_EINVAL;
        }
        srvix = pl->index;
    }

    pkt = mcreq_allocate_packet(pl);

    if (pkt == NULL) {
        return LCB_CLIENT_ENOMEM;
    }

    hdr.request.opaque = pkt->opaque;
    pkt->extlen = hdr.request.extlen;
    info->consumed = n_packet;

    if (options & MC_FWD_OPT_COPY) {
        /* reserve bytes for the entire packet */
        mcreq_reserve_header(pl, pkt, n_header);
        iovcursor_adv_copy(mincur, SPAN_BUFFER(&pkt->kh_span), n_header);
        if (n_body_value) {
            mcreq_reserve_value2(pl, pkt, n_body_value);
            iovcursor_adv_copy(mincur, SPAN_BUFFER(&pkt->u_value.single), n_body_value);
            pkt->flags |= MCREQ_F_HASVALUE;
        }

    } else {
        if (IOVCURSOR_HAS_CONTIG(mincur, n_header)) {
            span_from_first(mincur, n_header, &pkt->kh_span);
            pkt->flags |= MCREQ_F_KEY_NOCOPY;

        } else {
            /* header is fragmented into multiple IOVs */
            mcreq_reserve_header(pl, pkt, n_header);
            iovcursor_adv_copy(mincur, SPAN_BUFFER(&pkt->kh_span), n_header);
        }

        /* do we have a value payload still? */
        if (n_body_value) {
            pkt->flags |= MCREQ_F_HASVALUE | MCREQ_F_VALUE_NOCOPY;
            if (IOVCURSOR_HAS_CONTIG(mincur, n_body_value)) {
                span_from_first(mincur, n_body_value, &pkt->u_value.single);

            } else {
                /* body is fragmented */
                iovcursor_adv_iovalloc(mincur, n_body_value, (nb_IOV **)&pkt->u_value.multi.iov,
                                       &pkt->u_value.multi.niov);
                pkt->u_value.multi.total_length = n_body_value;
                pkt->flags |= MCREQ_F_VALUE_IOV;
            }
        }
    }

    /* Copy the first 24 bytes into the header span */
    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof hdr.bytes);

    *pkt_p = pkt;
    *pl_p = pl;

    /* Set the UFWD flag. This causes the rest of the system to invoke the
     * handler for the raw response, rather than the "Contiguous" structures*/
    pkt->flags |= MCREQ_F_UFWD;
    mcreq_sched_add(pl, pkt);
    return LCB_SUCCESS;
}
