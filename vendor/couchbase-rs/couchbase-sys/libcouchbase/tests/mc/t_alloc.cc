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

class McAlloc : public ::testing::Test
{
  protected:
    mc_CMDQUEUE cQueue;

    void setupPipeline(mc_PIPELINE *pipeline)
    {
        mcreq_queue_init(&cQueue);
        mcreq_pipeline_init(pipeline);
        pipeline->parent = &cQueue;
    }
};

TEST_F(McAlloc, testPipelineFreeAlloc)
{
    mc_PIPELINE pipeline;
    memset(&pipeline, 0, sizeof(pipeline));
    mcreq_pipeline_init(&pipeline);
    mcreq_pipeline_cleanup(&pipeline);
}

TEST_F(McAlloc, testPacketFreeAlloc)
{
    mc_PIPELINE pipeline;
    mc_PACKET *copied = NULL;
    memset(&pipeline, 0, sizeof(pipeline));
    setupPipeline(&pipeline);

    mc_PACKET *packet = mcreq_allocate_packet(&pipeline);
    ASSERT_TRUE(packet != NULL);

    mcreq_reserve_header(&pipeline, packet, 24);

    // Check to see that we can also detach a packet and use it after the
    // other resources have been released
    copied = mcreq_renew_packet(packet);

    mcreq_wipe_packet(&pipeline, packet);
    mcreq_release_packet(&pipeline, packet);
    mcreq_pipeline_cleanup(&pipeline);

    // Write to the detached packet. Ensure we don't crash
    memset(SPAN_BUFFER(&copied->kh_span), 0xff, copied->kh_span.size);
    mcreq_wipe_packet(NULL, copied);
    mcreq_release_packet(NULL, copied);
}

struct dummy_datum {
    mc_EPKTDATUM base;
    int refcount;
};
extern "C" {
static void datum_free(mc_EPKTDATUM *epd)
{
    dummy_datum *dd = (dummy_datum *)epd;
    dd->refcount--;
}
}

TEST_F(McAlloc, testExdataAlloc)
{
    mc_PIPELINE pipeline;
    mc_PACKET *copy1, *copy2;
    setupPipeline(&pipeline);
    mc_PACKET *packet = mcreq_allocate_packet(&pipeline);
    mcreq_reserve_header(&pipeline, packet, 24);

    copy1 = mcreq_renew_packet(packet);
    ASSERT_FALSE((copy1->flags & MCREQ_F_DETACHED) == 0);

    dummy_datum dd;
    dd.base.key = "Dummy";
    dd.base.dtorfn = datum_free;
    dd.refcount = 1;
    mcreq_epkt_insert((mc_EXPACKET *)copy1, &dd.base);
    // Find it back
    mc_EPKTDATUM *epd = mcreq_epkt_find((mc_EXPACKET *)copy1, "Dummy");
    ASSERT_FALSE(epd == NULL);
    ASSERT_TRUE(epd == &dd.base);

    copy2 = mcreq_renew_packet(copy1);
    epd = mcreq_epkt_find((mc_EXPACKET *)copy1, "Dummy");
    ASSERT_TRUE(epd == NULL);
    epd = mcreq_epkt_find((mc_EXPACKET *)copy2, "Dummy");
    ASSERT_FALSE(epd == NULL);

    mcreq_wipe_packet(&pipeline, packet);
    mcreq_release_packet(&pipeline, packet);
    mcreq_wipe_packet(NULL, copy1);
    mcreq_release_packet(NULL, copy1);
    mcreq_wipe_packet(NULL, copy2);
    mcreq_release_packet(NULL, copy2);
    ASSERT_EQ(0, dd.refcount);
    mcreq_pipeline_cleanup(&pipeline);
}

TEST_F(McAlloc, testKeyAlloc)
{
    CQWrap q;
    mc_PACKET *packet;
    mc_PIPELINE *pipeline;
    lcb_CMDBASE cmd;

    protocol_binary_request_header hdr;
    memset(&cmd, 0, sizeof(cmd));
    memset(&hdr, 0, sizeof(hdr));

    cmd.key.contig.bytes = const_cast< char * >("Hello");
    cmd.key.contig.nbytes = 5;

    lcb_STATUS ret;
    ret = mcreq_basic_packet(&q, &cmd, &hdr, 0, 0, &packet, &pipeline, 0);
    ASSERT_EQ(LCB_SUCCESS, ret);
    ASSERT_TRUE(packet != NULL);
    ASSERT_TRUE(pipeline != NULL);
    ASSERT_EQ(5, ntohs(hdr.request.keylen));

    int vb = lcbvb_k2vb(q.config, "Hello", 5);
    ASSERT_EQ(vb, ntohs(hdr.request.vbucket));

    // Copy the header
    memcpy(SPAN_BUFFER(&packet->kh_span), &hdr, sizeof(hdr));

    lcb_VALBUF vreq;
    memset(&vreq, 0, sizeof(vreq));

    const void *key;
    lcb_size_t nkey;
    // Get back the key we just placed inside the header
    mcreq_get_key(NULL, packet, &key, &nkey);
    ASSERT_EQ(5, nkey);
    ASSERT_EQ(0, memcmp(key, "Hello", 5));

    mcreq_wipe_packet(pipeline, packet);
    mcreq_release_packet(pipeline, packet);
}

TEST_F(McAlloc, testValueAlloc)
{
    CQWrap q;
    mc_PACKET *packet;
    mc_PIPELINE *pipeline;
    lcb_CMDBASE cmd;
    protocol_binary_request_header hdr;
    lcb_VALBUF vreq;

    memset(&cmd, 0, sizeof(cmd));
    memset(&hdr, 0, sizeof(hdr));
    memset(&vreq, 0, sizeof(vreq));

    const char *key = "Hello";
    const char *value = "World";

    lcb_STATUS ret;
    cmd.key.contig.bytes = const_cast< char * >(key);
    cmd.key.contig.nbytes = 5;
    vreq.u_buf.contig.bytes = const_cast< char * >(value);
    vreq.u_buf.contig.nbytes = 5;

    ret = mcreq_basic_packet(&q, &cmd, &hdr, 0, 0, &packet, &pipeline, 0);
    ASSERT_EQ(LCB_SUCCESS, ret);
    ret = mcreq_reserve_value(pipeline, packet, &vreq);
    ASSERT_EQ(ret, LCB_SUCCESS);
    ASSERT_EQ(packet->flags, MCREQ_F_HASVALUE);

    ASSERT_EQ(0, memcmp(SPAN_BUFFER(&packet->u_value.single), value, 5));
    ASSERT_NE(SPAN_BUFFER(&packet->u_value.single), value);
    mcreq_wipe_packet(pipeline, packet);
    mcreq_release_packet(pipeline, packet);

    // Allocate another packet, but this time, use our own reserved value
    ret = mcreq_basic_packet(&q, &cmd, &hdr, 0, 0, &packet, &pipeline, 0);
    ASSERT_EQ(ret, LCB_SUCCESS);
    vreq.vtype = LCB_KV_CONTIG;
    ret = mcreq_reserve_value(pipeline, packet, &vreq);
    ASSERT_EQ(SPAN_BUFFER(&packet->u_value.single), value);
    ASSERT_EQ(MCREQ_F_HASVALUE | MCREQ_F_VALUE_NOCOPY, packet->flags);
    mcreq_wipe_packet(pipeline, packet);
    mcreq_release_packet(pipeline, packet);

    nb_IOV iov[2];
    iov[0].iov_base = (void *)value;
    iov[0].iov_len = 3;
    iov[1].iov_base = (void *)(value + 3);
    iov[1].iov_len = 2;

    vreq.u_buf.multi.iov = (lcb_IOV *)iov;
    vreq.u_buf.multi.niov = 2;
    vreq.vtype = LCB_KV_IOV;
    ret = mcreq_basic_packet(&q, &cmd, &hdr, 0, 0, &packet, &pipeline, 0);
    ASSERT_EQ(LCB_SUCCESS, ret);
    ret = mcreq_reserve_value(pipeline, packet, &vreq);
    ASSERT_EQ(LCB_SUCCESS, ret);
    ASSERT_EQ(MCREQ_F_HASVALUE | MCREQ_F_VALUE_IOV | MCREQ_F_VALUE_NOCOPY, packet->flags);
    ASSERT_NE(&iov[0], (nb_IOV *)packet->u_value.multi.iov);
    ASSERT_EQ(2, packet->u_value.multi.niov);
    ASSERT_EQ(5, packet->u_value.multi.total_length);
    mcreq_wipe_packet(pipeline, packet);
    mcreq_release_packet(pipeline, packet);

    iov[0].iov_base = (void *)value;
    iov[0].iov_len = 3;
    iov[1].iov_base = (void *)(value + 3);
    iov[1].iov_len = 2;
    vreq.u_buf.multi.iov = (lcb_IOV *)iov;
    vreq.u_buf.multi.niov = 2;
    vreq.u_buf.multi.total_length = 0;

    vreq.vtype = LCB_KV_IOVCOPY;
    ret = mcreq_basic_packet(&q, &cmd, &hdr, 0, 0, &packet, &pipeline, 0);
    ASSERT_EQ(LCB_SUCCESS, ret);

    ret = mcreq_reserve_value(pipeline, packet, &vreq);
    ASSERT_EQ(LCB_SUCCESS, ret);

    ASSERT_EQ(MCREQ_F_HASVALUE, packet->flags);
    ASSERT_EQ(0, memcmp(SPAN_BUFFER(&packet->u_value.single), value, 5));
    mcreq_wipe_packet(pipeline, packet);
    mcreq_release_packet(pipeline, packet);
}

struct ExtraCookie : mc_REQDATAEX {
    int remaining;
    ExtraCookie(const mc_REQDATAPROCS &procs_) : mc_REQDATAEX(NULL, procs_, 0), remaining(0) {}
};

extern "C" {
static void pkt_dtor(mc_PACKET *pkt)
{
    ExtraCookie *ec = static_cast< ExtraCookie * >(pkt->u_rdata.exdata);
    ec->remaining--;
}
}

TEST_F(McAlloc, testRdataExDtor)
{
    CQWrap q;
    lcb_CMDBASE basecmd;
    const static mc_REQDATAPROCS procs = {NULL, pkt_dtor};
    protocol_binary_request_header hdr;

    memset(&hdr, 0, sizeof hdr);
    memset(&basecmd, 0, sizeof basecmd);

    basecmd.key.contig.bytes = "foo";
    basecmd.key.contig.nbytes = 3;

    ExtraCookie ec(procs);

    mcreq_sched_enter(&q);
    for (unsigned ii = 0; ii < 5; ii++) {
        lcb_STATUS err;
        mc_PIPELINE *pl;
        mc_PACKET *pkt;
        err = mcreq_basic_packet(&q, &basecmd, &hdr, 0, 0, &pkt, &pl, 0);
        ASSERT_EQ(LCB_SUCCESS, err);
        pkt->flags |= MCREQ_F_REQEXT;
        pkt->u_rdata.exdata = &ec;
        mcreq_sched_add(pl, pkt);
        ec.remaining++;
    }
    mcreq_sched_fail(&q);
    ASSERT_EQ(0, ec.remaining);
}
