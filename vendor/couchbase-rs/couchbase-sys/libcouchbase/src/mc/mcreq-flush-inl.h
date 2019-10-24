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
#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    mc_PIPELINE *pl;
    hrtime_t now;
} mc__FLUSHINFO;

/**
 * Inline operations for flush. To use this, include this file into your
 * own source code.
 */

/**
 * Fill a series of IOVs with data to flush
 * @param pipeline the pipeline to flush
 * @param iov the iov array to fill
 * @param niov the number of input items
 * @param nused set to the number of IOVs actually used
 * @return the number of data inside all the IOVs
 */
static unsigned int mcreq_flush_iov_fill(mc_PIPELINE *pipeline, nb_IOV *iov, int niov, int *nused)
{
    return netbuf_start_flush(&pipeline->nbmgr, iov, niov, nused);
}

static nb_SIZE mcreq__pktflush_callback(void *p, nb_SIZE hint, void *arg)
{
    nb_SIZE pktsize;
    mc_PACKET *pkt = (mc_PACKET *)p;
    mc__FLUSHINFO *info = (mc__FLUSHINFO *)arg;

    pktsize = mcreq_get_size(pkt);

    if (info->now && hint) {
        MCREQ_PKT_RDATA(pkt)->start = info->now;
    }

    if (hint < pktsize) {
        return pktsize;
    }

    /** Packet is flushed */
    pkt->flags |= MCREQ_F_FLUSHED;

    if (pkt->flags & MCREQ_F_INVOKED) {
        mcreq_packet_done(info->pl, pkt);
    }
    if (info->pl->metrics) {
        info->pl->metrics->packets_sent++;
        info->pl->metrics->packets_queued--;
        info->pl->metrics->bytes_queued -= pktsize;
    }
    return pktsize;
}

/**
 * Called when a chunk of data has been flushed from the network.
 * @param pl the pipeline which was to be flushed
 * @param nflushed how much data was actually flushed
 * @param expected how much data was expected to be flushed (i.e. the return
 *        value from the corresponding iov_fill).
 *
 * @param now if present, will reset the start time of each traversed packet
 *        to the value passed.
 *
 * This is a thin wrapper around netbuf_end_flush (and optionally
 * nebtuf_reset_flush())
 */
static void mcreq_flush_done_ex(mc_PIPELINE *pl, unsigned nflushed, unsigned expected, lcb_U64 now)
{
    if (nflushed) {
        mc__FLUSHINFO info = {pl, now};
        netbuf_end_flush2(&pl->nbmgr, nflushed, mcreq__pktflush_callback, offsetof(mc_PACKET, sl_flushq), &info);
    }
    if (nflushed < expected) {
        netbuf_reset_flush(&pl->nbmgr);
    }
}

/* Mainly for tests */
static void mcreq_flush_done(mc_PIPELINE *pl, unsigned nflushed, unsigned expected)
{
    mcreq_flush_done_ex(pl, nflushed, expected, 0);
}

#ifdef __cplusplus
}
#endif
