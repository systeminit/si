/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#include "mctest.h"
#include "mc/mcreq-flush-inl.h"

class McContext : public ::testing::Test
{
};

struct CtxCookie {
    int ncalled;
    size_t plLength;
    CtxCookie() : ncalled(0), plLength(0) {}
};

extern "C" {
static void failcb(mc_PIPELINE *, mc_PACKET *pkt, lcb_STATUS, void *)
{
    CtxCookie *cookie = (CtxCookie *)MCREQ_PKT_COOKIE(pkt);
    cookie->ncalled++;
    cookie->plLength += mcreq_get_size(pkt);
}
}

TEST_F(McContext, testBasicContext)
{
    CQWrap cq;
    CtxCookie cookie;

    mcreq_sched_enter(&cq);

    for (int ii = 0; ii < 20; ii++) {
        PacketWrap pw;
        char kbuf[128];
        sprintf(kbuf, "key_%d", ii);
        pw.setCopyKey(kbuf);

        ASSERT_TRUE(pw.reservePacket(&cq));

        pw.setHeaderSize();
        pw.copyHeader();
        pw.setCookie(&cookie);

        mcreq_sched_add(pw.pipeline, pw.pkt);
        ASSERT_FALSE(SLLIST_IS_EMPTY(&pw.pipeline->requests) == 0);
        ASSERT_TRUE(SLLIST_IS_EMPTY(&pw.pipeline->ctxqueued) == 0);
    }

    mcreq_sched_fail(&cq);

    for (unsigned ii = 0; ii < cq.npipelines; ii++) {
        unsigned nFail = 0;
        mc_PIPELINE *pl = cq.pipelines[ii];
        cookie.plLength = 0;

        nFail = mcreq_pipeline_fail(pl, LCB_ERROR, failcb, NULL);
        if (!nFail) {
            continue;
        }

        nb_IOV iov[50];
        unsigned toFlush;
        toFlush = mcreq_flush_iov_fill(pl, iov, 50, NULL);
        ASSERT_EQ(cookie.plLength, toFlush);
        mcreq_flush_done(pl, toFlush, toFlush);
    }
}

TEST_F(McContext, testFailedContext)
{
    CQWrap cq;
    CtxCookie cookie;

    mcreq_sched_enter(&cq);

    for (int ii = 0; ii < 20; ii++) {
        PacketWrap pw;
        char kbuf[128];
        sprintf(kbuf, "Key_%d", ii);
        pw.setCopyKey(kbuf);

        ASSERT_TRUE(pw.reservePacket(&cq));

        pw.setHeaderSize();
        pw.copyHeader();
        mcreq_sched_add(pw.pipeline, pw.pkt);
    }

    mcreq_sched_fail(&cq);

    for (unsigned ii = 0; ii < cq.npipelines; ii++) {
        mc_PIPELINE *pl = cq.pipelines[ii];
        if (!cq.scheds[pl->index]) {
            continue;
        }

        ASSERT_TRUE(SLLIST_IS_EMPTY(&pl->requests));
        ASSERT_TRUE(SLLIST_IS_EMPTY(&pl->ctxqueued));

        nb_IOV iov[1];
        ASSERT_EQ(0, mcreq_flush_iov_fill(pl, iov, 1, NULL));
    }
}
