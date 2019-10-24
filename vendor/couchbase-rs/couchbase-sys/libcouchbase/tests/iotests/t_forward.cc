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

#include "iotests.h"
#include <libcouchbase/pktfwd.h>
#include <memcached/protocol_binary.h>
#include "mc/pktmaker.h"
using std::string;
using std::vector;
using namespace PacketMaker;

class ForwardTests : public MockUnitTest
{
  protected:
    virtual void createConnection(HandleWrap &hw, lcb_INSTANCE **instance)
    {
        MockEnvironment::getInstance()->createConnection(hw, instance);
        lcb_cntl_string(*instance, "enable_tracing", "off");
        ASSERT_EQ(LCB_SUCCESS, lcb_connect(*instance));
        lcb_wait(*instance);
        ASSERT_EQ(LCB_SUCCESS, lcb_get_bootstrap_status(*instance));
    }
};

struct ForwardCookie {
    vector< char > orig;
    vector< char > respbuf;
    vector< lcb_IOV > iovs;
    vector< lcb_BACKBUF > bkbuf;
    lcb_STATUS err_expected;
    lcb_STATUS err_received;
    bool called;
    bool flushed;

    ForwardCookie()
    {
        err_expected = LCB_SUCCESS;
        err_received = LCB_SUCCESS;
        called = false;
        flushed = false;
    }
};

extern "C" {
static void pktfwd_callback(lcb_INSTANCE *, const void *cookie, lcb_STATUS err, lcb_PKTFWDRESP *resp)
{
    ForwardCookie *fc = (ForwardCookie *)cookie;
    fc->called = true;
    fc->err_received = err;

    if (err != LCB_SUCCESS) {
        return;
    }

    protocol_binary_response_header *hdr = (protocol_binary_response_header *)resp->header;
    ASSERT_EQ(PROTOCOL_BINARY_RES, hdr->response.magic);
    lcb_U32 blen = ntohl(hdr->response.bodylen);

    // Gather the packets
    for (unsigned ii = 0; ii < resp->nitems; ii++) {
        lcb_backbuf_ref(resp->bufs[ii]);

        char *buf = (char *)resp->iovs[ii].iov_base;
        size_t len = resp->iovs[ii].iov_len;

        fc->iovs.push_back(resp->iovs[ii]);
        fc->bkbuf.push_back(resp->bufs[ii]);
        fc->respbuf.insert(fc->respbuf.end(), buf, buf + len);
    }

    ASSERT_EQ(blen + 24, fc->respbuf.size());
}

static void pktflush_callback(lcb_INSTANCE *, const void *cookie)
{
    ForwardCookie *fc = (ForwardCookie *)cookie;
    EXPECT_FALSE(fc->flushed);
    fc->flushed = true;
}
}

TEST_F(ForwardTests, testBasic)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_set_pktflushed_callback(instance, pktflush_callback);
    lcb_set_pktfwd_callback(instance, pktfwd_callback);

    ForwardCookie fc;
    StorageRequest req("Hello", "World");
    req.magic(PROTOCOL_BINARY_REQ);
    req.op(PROTOCOL_BINARY_CMD_SET);

    lcb_CMDPKTFWD cmd = {0};
    req.serialize(fc.orig);
    cmd.vb.vtype = LCB_KV_CONTIG;
    cmd.vb.u_buf.contig.bytes = &fc.orig[0];
    cmd.vb.u_buf.contig.nbytes = fc.orig.size();
    lcb_STATUS rc;

    lcb_sched_enter(instance);
    rc = lcb_pktfwd3(instance, &fc, &cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_sched_leave(instance);
    lcb_wait(instance);
    ASSERT_TRUE(fc.called);
    ASSERT_EQ(LCB_SUCCESS, fc.err_received);
    for (unsigned ii = 0; ii < fc.bkbuf.size(); ++ii) {
        lcb_backbuf_unref(fc.bkbuf[ii]);
    }
}

TEST_F(ForwardTests, testIncomplete)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_set_pktflushed_callback(instance, pktflush_callback);
    lcb_set_pktfwd_callback(instance, pktfwd_callback);
}
