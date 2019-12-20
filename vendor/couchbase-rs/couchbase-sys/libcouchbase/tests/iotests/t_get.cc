/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#include "config.h"
#include <libcouchbase/couchbase.h>
#include <map>
#include "iotests.h"

class GetUnitTest : public MockUnitTest
{
};

extern "C" {
static void testGetMissGetCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPGET *resp)
{
    int *counter;
    lcb_respget_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_KEY_ENOENT, lcb_respget_status(resp));
    const char *key;
    size_t nkey;
    lcb_respget_key(resp, &key, &nkey);
    std::string val(key, nkey);
    EXPECT_TRUE(val == "testGetMiss1" || val == "testGetMiss2");
    ++(*counter);
}
}

/**
 * @test
 * Get Miss
 *
 * @pre
 * Request two non-existent keys
 *
 * @post
 * Responses for both keys are received with error code
 * @c KEY_ENOENT; response structure is not NULL, and the keys match their
 * expected value
 *
 * @todo (maybe check the values too?)
 */
TEST_F(GetUnitTest, testGetMiss)
{
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)testGetMissGetCallback);
    int numcallbacks = 0;
    std::string key1("testGetMiss1"), key2("testGetMiss2");

    removeKey(instance, key1);
    removeKey(instance, key2);

    lcb_CMDGET *cmd;

    lcb_cmdget_create(&cmd);
    lcb_cmdget_key(cmd, key1.c_str(), key1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_get(instance, &numcallbacks, cmd));

    lcb_cmdget_key(cmd, key2.c_str(), key2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_get(instance, &numcallbacks, cmd));
    lcb_cmdget_destroy(cmd);

    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

extern "C" {
static void testGetHitGetCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPGET *resp)
{
    int *counter;
    lcb_respget_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_SUCCESS, lcb_respget_status(resp));
    ++(*counter);
}
}

/**
 * @test
 * Get Hit
 *
 * @pre
 * Store two keys, and retrieve them
 *
 * @post
 * Both keys exist, and their return code is successul
 */
TEST_F(GetUnitTest, testGetHit)
{
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)testGetHitGetCallback);
    int numcallbacks = 0;
    std::string key1("testGetKey1"), key2("testGetKey2");

    storeKey(instance, key1, "foo");
    storeKey(instance, key2, "foo");

    lcb_CMDGET *cmd;

    lcb_cmdget_create(&cmd);
    lcb_cmdget_key(cmd, key1.c_str(), key1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_get(instance, &numcallbacks, cmd));

    lcb_cmdget_key(cmd, key2.c_str(), key2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_get(instance, &numcallbacks, cmd));
    lcb_cmdget_destroy(cmd);

    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

extern "C" {
static void testTouchMissCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPTOUCH *resp)
{
    int *counter;
    lcb_resptouch_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_KEY_ENOENT, lcb_resptouch_status(resp));
    ++(*counter);
}
}

/**
 * @test Touch (Miss)
 * @pre Schedule a touch for a non existent key with an expiry @c 666
 * @post Touch fails with @c KEY_ENOENT
 */
TEST_F(GetUnitTest, testTouchMiss)
{
    std::string key("testTouchMissKey");
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_TOUCH, (lcb_RESPCALLBACK)testTouchMissCallback);
    removeKey(instance, key);

    int numcallbacks = 0;

    lcb_CMDTOUCH *cmd;
    lcb_cmdtouch_create(&cmd);
    lcb_cmdtouch_key(cmd, key.c_str(), key.size());
    lcb_cmdtouch_expiration(cmd, 666);
    lcb_touch(instance, &numcallbacks, cmd);
    lcb_cmdtouch_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void testTouchHitCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPTOUCH *resp)
{
    int *counter;
    lcb_resptouch_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_SUCCESS, lcb_resptouch_status(resp));
    ++(*counter);
}
}

/**
 * @test Touch (Hit)
 * @pre Store a key, and schedule a touch operation with an expiry of @c 666
 * @post Touch succeeds.
 */
TEST_F(GetUnitTest, testTouchHit)
{
    std::string key("testTouchHitKey");
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_TOUCH, (lcb_RESPCALLBACK)testTouchHitCallback);
    storeKey(instance, key, "foo");

    int numcallbacks = 0;
    lcb_CMDTOUCH *cmd;
    lcb_cmdtouch_create(&cmd);
    lcb_cmdtouch_key(cmd, key.c_str(), key.size());
    lcb_cmdtouch_expiration(cmd, 666);
    lcb_touch(instance, &numcallbacks, cmd);
    lcb_cmdtouch_destroy(cmd);

    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void flags_store_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    ASSERT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));

    const char *key;
    size_t nkey;
    lcb_respstore_key(resp, &key, &nkey);
    ASSERT_EQ(5, nkey);
    ASSERT_EQ(0, memcmp(key, "flags", 5));

    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_SET, op);
    ++(*counter);
}

static void flags_get_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPGET *resp)
{
    int *counter;
    lcb_respget_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_SUCCESS, lcb_respget_status(resp));

    const char *key;
    size_t nkey;
    lcb_respget_key(resp, &key, &nkey);
    ASSERT_EQ(5, nkey);
    ASSERT_EQ(0, memcmp(key, "flags", 5));

    const char *value;
    size_t nvalue;
    lcb_respget_value(resp, &value, &nvalue);
    ASSERT_EQ(1, nvalue);
    ASSERT_EQ(0, memcmp(value, "x", 1));

    uint32_t flags;
    lcb_respget_flags(resp, &flags);
    ASSERT_EQ(0xdeadbeef, flags);
    ++(*counter);
}
}

TEST_F(GetUnitTest, testFlags)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;

    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)flags_get_callback);
    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)flags_store_callback);

    int numcallbacks = 0;

    lcb_CMDSTORE *scmd;
    lcb_cmdstore_create(&scmd, LCB_STORE_SET);
    lcb_cmdstore_key(scmd, "flags", 5);
    lcb_cmdstore_value(scmd, "x", 1);
    lcb_cmdstore_flags(scmd, 0xdeadbeef);

    ASSERT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, scmd));
    lcb_cmdstore_destroy(scmd);

    // Wait for it to be persisted
    lcb_wait(instance);

    lcb_CMDGET *gcmd;
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, "flags", 5);
    ASSERT_EQ(LCB_SUCCESS, lcb_get(instance, &numcallbacks, gcmd));
    lcb_cmdget_destroy(gcmd);

    /* Wait for it to be received */
    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

struct RGetCookie {
    unsigned remaining;
    lcb_STATUS expectrc;
    std::string value;
    lcb_U64 cas;
};

extern "C" {
static void rget_callback(lcb_INSTANCE *instance, int, const lcb_RESPGETREPLICA *resp)
{
    RGetCookie *rck;
    lcb_respgetreplica_cookie(resp, (void **)&rck);

    lcb_STATUS rc = lcb_respgetreplica_status(resp);
    ASSERT_EQ(rck->expectrc, rc);
    ASSERT_NE(0, rck->remaining);
    rck->remaining--;

    if (rc == LCB_SUCCESS) {
        const char *v;
        size_t n;
        lcb_respgetreplica_value(resp, &v, &n);
        std::string value(v, n);
        ASSERT_STREQ(rck->value.c_str(), value.c_str());

        uint64_t cas;
        lcb_respgetreplica_cas(resp, &cas);
        ASSERT_EQ(rck->cas, cas);
    }
}
static void rget_noop_callback(lcb_INSTANCE *, int, const lcb_RESPGETREPLICA *) {}
}

TEST_F(GetUnitTest, testGetReplica)
{
    SKIP_UNLESS_MOCK();
    MockEnvironment *mock = MockEnvironment::getInstance();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);
    std::string key("a_key_GETREPLICA");
    std::string val("a_value");

    lcb_CMDGETREPLICA *rcmd;

    lcb_install_callback3(instance, LCB_CALLBACK_GETREPLICA, (lcb_RESPCALLBACK)rget_callback);
    RGetCookie rck;
    rck.remaining = 1;
    rck.expectrc = LCB_SUCCESS;
    unsigned nreplicas = lcb_get_num_replicas(instance);

    for (unsigned ii = 0; ii < nreplicas; ii++) {
        MockMutationCommand mcCmd(MockCommand::CACHE, key);
        mcCmd.cas = ii + 100;
        rck.cas = mcCmd.cas;
        mcCmd.replicaList.clear();
        mcCmd.replicaList.push_back(ii);

        mock->sendCommand(mcCmd);
        mock->getResponse();

        lcb_STATUS err;
        lcb_REPLICA_MODE mode;
        switch (ii) {
            case 0:
                mode = LCB_REPLICA_MODE_IDX0;
                break;
            case 1:
                mode = LCB_REPLICA_MODE_IDX1;
                break;
            case 2:
                mode = LCB_REPLICA_MODE_IDX2;
                break;
            default:
                ASSERT_FALSE("Unexpected replica index");
                break;
        }
        lcb_cmdgetreplica_create(&rcmd, mode);
        lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());

        rck.remaining = 1;
        lcb_sched_enter(instance);
        err = lcb_getreplica(instance, &rck, rcmd);
        ASSERT_EQ(LCB_SUCCESS, err);
        lcb_cmdgetreplica_destroy(rcmd);

        lcb_sched_leave(instance);
        lcb_wait(instance);
        ASSERT_EQ(0, rck.remaining);
    }

    // Test with the "All" mode
    MockMutationCommand mcCmd(MockCommand::CACHE, key);
    mcCmd.cas = 999;
    mcCmd.onMaster = false;
    mcCmd.replicaCount = nreplicas;
    mock->sendCommand(mcCmd);
    mock->getResponse();

    rck.remaining = nreplicas;
    rck.cas = mcCmd.cas;
    rck.expectrc = LCB_SUCCESS;

    lcb_cmdgetreplica_create(&rcmd, LCB_REPLICA_MODE_ALL);
    lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());
    lcb_sched_enter(instance);
    lcb_STATUS err = lcb_getreplica(instance, &rck, rcmd);
    lcb_cmdgetreplica_destroy(rcmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_sched_leave(instance);

    lcb_wait(instance);
    ASSERT_EQ(0, rck.remaining);

    MockMutationCommand purgeCmd(MockCommand::PURGE, key);
    purgeCmd.onMaster = true;
    purgeCmd.replicaCount = nreplicas;
    mock->sendCommand(purgeCmd);
    mock->getResponse();

    // Test with the "First" mode. Ensure that only the _last_ replica
    // contains the item
    mcCmd.onMaster = false;
    mcCmd.replicaCount = 0;
    mcCmd.replicaList.clear();
    mcCmd.replicaList.push_back(nreplicas - 1);
    mcCmd.cas = 42;
    rck.cas = mcCmd.cas;

    // Set the timeout to something higher, since we have more than one packet
    // to send.
    lcb_cntl_setu32(instance, LCB_CNTL_OP_TIMEOUT, 10000000);

    // The first replica should respond with ENOENT, the second should succeed
    // though
    mock->sendCommand(mcCmd);
    mock->getResponse();
    lcb_cmdgetreplica_create(&rcmd, LCB_REPLICA_MODE_ANY);
    lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());
    rck.remaining = 1;
    lcb_sched_enter(instance);
    err = lcb_getreplica(instance, &rck, rcmd);
    lcb_cmdgetreplica_destroy(rcmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_sched_leave(instance);
    lcb_wait(instance);
    ASSERT_EQ(0, rck.remaining);

    // Test with an invalid index
    rcmd = NULL;
    ASSERT_EQ(LCB_EINVAL, lcb_cmdgetreplica_create(&rcmd, (lcb_REPLICA_MODE)42));
    ASSERT_EQ((lcb_CMDGETREPLICA *)NULL, rcmd);

    // If no crash, it's good.
    if (lcb_get_num_replicas(instance) > 1) {
        // Use the 'first' mode, but make the second replica index be -1, so
        // that in the retry we need to skip over an index.

        lcbvb_CONFIG *vbc;
        err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, (void *)&vbc);
        int vbid = lcbvb_k2vb(vbc, key.c_str(), key.size());
        int oldix;

        lcbvb_VBUCKET *vb = &vbc->vbuckets[vbid];
        oldix = vb->servers[2];
        vb->servers[2] = -1;

        rck.expectrc = LCB_KEY_ENOENT;
        rck.remaining = 1;
        lcb_sched_enter(instance);
        lcb_cmdgetreplica_create(&rcmd, LCB_REPLICA_MODE_ANY);
        lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());
        err = lcb_getreplica(instance, &rck, rcmd);
        lcb_cmdgetreplica_destroy(rcmd);
        ASSERT_EQ(LCB_SUCCESS, err);
        lcb_sched_leave(instance);
        lcb_wait(instance);
        ASSERT_EQ(0, rck.remaining);

        // Try with ALL again (should give an error)
        lcb_cmdgetreplica_create(&rcmd, LCB_REPLICA_MODE_ALL);
        lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());
        lcb_sched_enter(instance);
        err = lcb_getreplica(instance, NULL, rcmd);
        lcb_cmdgetreplica_destroy(rcmd);
        ASSERT_EQ(LCB_NO_MATCHING_SERVER, err);
        lcb_sched_leave(instance);

        vb->servers[2] = oldix;
    } else {
        printf("Not enough replicas for get-with-replica test\n");
    }

    // Test rget with a missing key. Fixes a potential bug
    lcb_install_callback3(instance, LCB_CALLBACK_GETREPLICA, (lcb_RESPCALLBACK)rget_noop_callback);
    removeKey(instance, key);
    lcb_cmdgetreplica_create(&rcmd, LCB_REPLICA_MODE_ANY);
    lcb_cmdgetreplica_key(rcmd, key.c_str(), key.size());
    lcb_sched_enter(instance);
    err = lcb_getreplica(instance, NULL, rcmd);
    lcb_cmdgetreplica_destroy(rcmd);
    lcb_sched_leave(instance);
    lcb_wait(instance);
}
