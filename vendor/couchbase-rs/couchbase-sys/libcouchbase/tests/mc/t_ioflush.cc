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
#include <vector>
#include <map>

using std::map;
using std::vector;

/**
 * Note that this file doesn't actually do any I/O, but simulates I/O patterns
 * more realistically than t_flush would. This is basically a more advanced
 * version which handles multiple I/O models and does stricter checking on
 * items.
 */

struct Context {
    int ncalled;
    map< void *, int > children;
    Context()
    {
        ncalled = 0;
    }
};

struct IOCookie {
    Context *parent;
    mc_PACKET *pkt;
    IOCookie(Context *p)
    {
        parent = p;
        pkt = NULL;
    }
};

extern "C" {
static void failcb(mc_PIPELINE *, mc_PACKET *pkt, lcb_STATUS, void *)
{
    IOCookie *ioc = (IOCookie *)MCREQ_PKT_COOKIE(pkt);
    ioc->parent->children[ioc]++;
    ioc->parent->ncalled++;
    delete ioc;
}
}

class McIOFlush : public ::testing::Test
{
};

struct FlushInfo {
    mc_PIPELINE *pipeline;
    mc_PACKET *pkt;
    unsigned size;
};

// Test flushing using an IOCP pattern; with multiple items
// at the end and the beginning.
TEST_F(McIOFlush, testIOCPFlush)
{
    vector< FlushInfo > flushes;
    CQWrap cq;
    const int count = 20;
    Context ctx;

    for (int ii = 0; ii < count; ii++) {
        char buf[128];
        sprintf(buf, "Key_%d", ii);
        PacketWrap pw;
        pw.setCopyKey(buf);
        ASSERT_TRUE(pw.reservePacket(&cq));
        pw.setHeaderSize();
        pw.copyHeader();

        IOCookie *c = new IOCookie(&ctx);
        c->pkt = pw.pkt;

        pw.setCookie(c);
        mcreq_enqueue_packet(pw.pipeline, pw.pkt);

        nb_IOV iov[1];
        unsigned toFlush = mcreq_flush_iov_fill(pw.pipeline, iov, 1, NULL);

        FlushInfo fi;
        fi.pipeline = pw.pipeline;
        fi.size = toFlush;
        fi.pkt = pw.pkt;

        flushes.push_back(fi);
    }

    for (unsigned ii = 0; ii < cq.npipelines; ii++) {
        mcreq_pipeline_fail(cq.pipelines[ii], LCB_ERROR, failcb, NULL);
    }

    ASSERT_EQ(count, flushes.size());
    for (unsigned ii = 0; ii < count; ii++) {
        FlushInfo &fi = flushes[ii];
        ASSERT_NE(0, fi.pkt->flags & MCREQ_F_INVOKED);
        mcreq_flush_done(flushes[ii].pipeline, flushes[ii].size, flushes[ii].size);
    }

    ASSERT_EQ(count, ctx.ncalled);
    map< void *, int >::iterator iter = ctx.children.begin();
    for (; iter != ctx.children.end(); ++iter) {
        ASSERT_EQ(1, iter->second);
    }
}
