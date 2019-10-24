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
#include <libcouchbase/utils.h>

using namespace std;
class ObseqnoTest : public MockUnitTest
{
};

extern "C" {
static void storeCb_getstok(lcb_INSTANCE *, int cbtype, const lcb_RESPSTORE *resp)
{
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    lcb_MUTATION_TOKEN *res = NULL;
    lcb_respstore_cookie(resp, (void **)&res);
    lcb_respstore_mutation_token(resp, res);
}
}

static void storeGetStok(lcb_INSTANCE *instance, const string &k, const string &v, lcb_MUTATION_TOKEN *res)
{
    lcb_RESPCALLBACK oldcb = lcb_get_callback3(instance, LCB_CALLBACK_STORE);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)storeCb_getstok);
    lcb_sched_enter(instance);

    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, k.c_str(), k.size());
    lcb_cmdstore_value(cmd, v.c_str(), v.size());

    lcb_STATUS rc = lcb_store(instance, res, cmd);
    EXPECT_EQ(LCB_SUCCESS, rc);
    lcb_cmdstore_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait(instance);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, oldcb);
}

TEST_F(ObseqnoTest, testFetchImplicit)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    lcb_STATUS rc;
    createConnection(hw, &instance);
    const char *key = "obseqBasic";
    const char *value = "value";

    rc = lcb_cntl_string(instance, "dur_mutation_tokens", "true");
    ASSERT_EQ(LCB_SUCCESS, rc);

    lcb_MUTATION_TOKEN st_fetched = {0};
    storeGetStok(instance, key, value, &st_fetched);
    ASSERT_TRUE(st_fetched.uuid_ != 0);

    lcb_KEYBUF kb;
    LCB_KREQ_SIMPLE(&kb, key, strlen(key));

    const lcb_MUTATION_TOKEN *ss = lcb_get_mutation_token(instance, &kb, &rc);
    ASSERT_EQ(LCB_SUCCESS, rc);
    ASSERT_TRUE(ss != NULL);
    st_fetched = *ss;
}

extern "C" {
static void obseqCallback(lcb_INSTANCE *, int, const lcb_RESPOBSEQNO *rb)
{
    lcb_RESPOBSEQNO *pp = (lcb_RESPOBSEQNO *)rb->cookie;
    *pp = *rb;
}
}

static void doObserveSeqno(lcb_INSTANCE *instance, lcb_MUTATION_TOKEN *ss, int server, lcb_RESPOBSEQNO &resp)
{
    lcb_CMDOBSEQNO cmd = {0};
    cmd.vbid = ss->vbid_;
    cmd.uuid = ss->uuid_;
    cmd.server_index = server;
    lcb_STATUS rc;

    lcb_sched_enter(instance);
    rc = lcb_observe_seqno3(instance, &resp, &cmd);
    if (rc != LCB_SUCCESS) {
        resp.rc = rc;
        resp.rflags |= LCB_RESP_F_CLIENTGEN;
        return;
    }

    lcb_RESPCALLBACK oldcb = lcb_get_callback3(instance, LCB_CALLBACK_OBSEQNO);
    lcb_install_callback3(instance, LCB_CALLBACK_OBSEQNO, (lcb_RESPCALLBACK)obseqCallback);
    lcb_sched_leave(instance);
    lcb_wait(instance);
    lcb_install_callback3(instance, LCB_CALLBACK_OBSEQNO, oldcb);
}

TEST_F(ObseqnoTest, testObserve)
{
    SKIP_UNLESS_MOCK();

    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);
    lcbvb_CONFIG *vbc;

    lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);

    lcb_MUTATION_TOKEN st_fetched = {0};
    const char *key = "testObserve";
    const char *value = "value";

    // Get the synctoken
    storeGetStok(instance, key, value, &st_fetched);
    ASSERT_NE(0, st_fetched.vbid_);
    ASSERT_NE(0, st_fetched.uuid_);
    ASSERT_NE(0, st_fetched.seqno_);

    for (size_t ii = 0; ii < lcbvb_get_nreplicas(vbc) + 1; ii++) {
        int ix = lcbvb_vbserver(vbc, st_fetched.vbid_, ii);
        lcb_RESPOBSEQNO resp = {0};
        doObserveSeqno(instance, &st_fetched, ix, resp);
        ASSERT_EQ(LCB_SUCCESS, resp.rc);
        ASSERT_EQ(st_fetched.uuid_, resp.cur_uuid);
        ASSERT_EQ(0, resp.old_uuid);
        //        printf("SEQ_MEM: %lu. SEQ_DISK: %lu\n", resp.mem_seqno, resp.persisted_seqno);
        ASSERT_GT(resp.mem_seqno, 0);
        ASSERT_EQ(resp.mem_seqno, resp.persisted_seqno);
    }
}

TEST_F(ObseqnoTest, testFailoverFormat)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);
    lcbvb_CONFIG *vbc;

    lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
    const char *key = "testObserve";
    const char *value = "value";

    lcb_MUTATION_TOKEN st_fetched = {0};
    storeGetStok(instance, key, value, &st_fetched);

    MockEnvironment *env = MockEnvironment::getInstance();
    env->regenVbCoords();

    // Now we should get a different sequence number
    lcb_RESPOBSEQNO rr;
    doObserveSeqno(instance, &st_fetched, lcbvb_vbmaster(vbc, st_fetched.vbid_), rr);
    ASSERT_EQ(LCB_SUCCESS, rr.rc);
    //    printf("Old UUID: %llu\n", rr.old_uuid);
    //    printf("Cur UUID: %llu\n", rr.cur_uuid);
    ASSERT_GT(rr.old_uuid, 0);
    ASSERT_EQ(rr.old_uuid, st_fetched.uuid_);
    ASSERT_NE(rr.old_uuid, rr.cur_uuid);
    ASSERT_EQ(rr.old_seqno, st_fetched.seqno_);
}

// TODO: We should add more tests here, but in order to do this, we must
// first validate the mock.
