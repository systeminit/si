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
#include "iotests.h"

class MutateUnitTest : public MockUnitTest
{
};

extern "C" {
static void testSimpleSetStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    using namespace std;
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_SET, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    const char *key;
    size_t nkey;
    lcb_respstore_key(resp, &key, &nkey);
    std::string val(key, nkey);
    EXPECT_TRUE(val == "testSimpleStoreKey1" || val == "testSimpleStoreKey2");
    ++(*counter);
    uint64_t cas;
    lcb_respstore_cas(resp, &cas);
    EXPECT_NE(0, cas);
}
}

/**
 * @test
 * Simple Set
 *
 * @pre
 * Set two keys
 *
 * @post
 *
 * @c SUCCESS, both keys are received
 */
TEST_F(MutateUnitTest, testSimpleSet)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testSimpleSetStoreCallback);

    std::string key1("testSimpleStoreKey1"), val1("key1"), key2("testSimpleStoreKey2"), val2("key2");

    int numcallbacks = 0;
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, key1.c_str(), key1.size());
    lcb_cmdstore_value(cmd, val1.c_str(), val1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));

    lcb_cmdstore_key(cmd, key2.c_str(), key2.size());
    lcb_cmdstore_value(cmd, val2.c_str(), val2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);

    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

/**
 * @test Zero length key
 * @pre set a zero length for a key foo
 * @post should not be able to schedule operation
 */
TEST_F(MutateUnitTest, testStoreZeroLengthKey)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    lcb_sched_enter(instance);
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, NULL, 0);
    lcb_cmdstore_value(cmd, "bar", 3);
    EXPECT_EQ(LCB_EMPTY_KEY, lcb_store(instance, NULL, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_sched_leave(instance);
}

extern "C" {
static void testStoreZeroLengthValueCallback(lcb_INSTANCE *, int, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_SET, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    ++(*counter);
}
}
/**
 * @test Zero length value
 * @pre set a zero length value for a key foo
 * @post should be able to retreive back empty value
 */
TEST_F(MutateUnitTest, testStoreZeroLengthValue)
{
    std::string key("foo");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    lcb_sched_enter(instance);
    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testStoreZeroLengthValueCallback);
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, NULL, 0);
    int numcallbacks = 0;
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait3(instance, LCB_WAIT_NOCHECK);
    EXPECT_EQ(1, numcallbacks);

    Item itm;
    getKey(instance, key, itm);
    EXPECT_EQ(0, itm.val.length());
}

extern "C" {
static void testRemoveCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPREMOVE *resp)
{
    int *counter;
    lcb_respremove_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_SUCCESS, lcb_respremove_status(resp));
    ++(*counter);
}
}

/**
 * @test Remove
 *
 * @pre Set two keys and remove them
 * @post Remove succeeds for both keys
 */
TEST_F(MutateUnitTest, testRemove)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    std::string key1("testRemoveKey1"), key2("testRemoveKey2");

    (void)lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, (lcb_RESPCALLBACK)testRemoveCallback);
    int numcallbacks = 0;
    storeKey(instance, key1, "foo");
    storeKey(instance, key2, "foo");

    lcb_CMDREMOVE *cmd;
    lcb_cmdremove_create(&cmd);

    lcb_cmdremove_key(cmd, key1.c_str(), key1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_remove(instance, &numcallbacks, cmd));

    lcb_cmdremove_key(cmd, key2.c_str(), key2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_remove(instance, &numcallbacks, cmd));

    lcb_cmdremove_destroy(cmd);

    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

extern "C" {
static void testRemoveMissCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPREMOVE *resp)
{
    int *counter;
    lcb_respremove_cookie(resp, (void **)&counter);
    EXPECT_EQ(LCB_KEY_ENOENT, lcb_respremove_status(resp));
    ++(*counter);
}
}

/**
 * @test Remove (Miss)
 * @pre Remove two non-existent keys
 * @post Remove fails for both keys with @c KEY_ENOENT
 */
TEST_F(MutateUnitTest, testRemoveMiss)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, (lcb_RESPCALLBACK)testRemoveMissCallback);
    int numcallbacks = 0;
    std::string key1("testRemoveMissKey1"), key2("testRemoveMissKey2");
    removeKey(instance, key1);
    removeKey(instance, key2);

    lcb_CMDREMOVE *cmd;
    lcb_cmdremove_create(&cmd);

    lcb_cmdremove_key(cmd, key1.c_str(), key1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_remove(instance, &numcallbacks, cmd));

    lcb_cmdremove_key(cmd, key2.c_str(), key2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_remove(instance, &numcallbacks, cmd));

    lcb_cmdremove_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
}

extern "C" {
static void testSimpleAddStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    using namespace std;
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_ADD, op);

    const char *key;
    size_t nkey;
    lcb_respstore_key(resp, &key, &nkey);
    std::string val(key, nkey);
    EXPECT_STREQ("testSimpleAddKey", val.c_str());

    lcb_STATUS rc = lcb_respstore_status(resp);
    if (*counter == 0) {
        uint64_t cas;
        EXPECT_EQ(LCB_SUCCESS, rc);
        lcb_respstore_cas(resp, &cas);
        EXPECT_NE(0, cas);
    } else {
        EXPECT_EQ(LCB_KEY_EEXISTS, rc);
    }
    ++(*counter);
}
}

/**
 * @test Add (Simple)
 * @pre Schedule to Add operations on the same key
 * @post First operation is a success. Second fails with @c KEY_EEXISTS
 */
TEST_F(MutateUnitTest, testSimpleAdd)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testSimpleAddStoreCallback);
    removeKey(instance, "testSimpleAddKey");
    int numcallbacks = 0;
    std::string key("testSimpleAddKey"), val1("key1"), val2("key2");
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_ADD);
    lcb_cmdstore_key(cmd, key.c_str(), key.size());

    lcb_cmdstore_value(cmd, val1.c_str(), val1.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));

    lcb_cmdstore_value(cmd, val2.c_str(), val2.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));

    lcb_wait(instance);
    EXPECT_EQ(2, numcallbacks);
    lcb_cmdstore_destroy(cmd);
}

extern "C" {
static void testSimpleAppendStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    using namespace std;
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_APPEND, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    uint64_t cas;
    lcb_respstore_cas(resp, &cas);
    EXPECT_NE(0, cas);
    ++(*counter);
}
}

/**
 * @test Append
 * @pre Set a key to @c foo, append it with @c bar. Retrieve the key
 * @post Key is now @c foobar
 */
TEST_F(MutateUnitTest, testSimpleAppend)
{
    std::string key("testSimpleAppendKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testSimpleAppendStoreCallback);
    storeKey(instance, key, "foo");
    int numcallbacks = 0;

    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_APPEND);

    std::string val("bar");
    lcb_cmdstore_key(cmd, key.c_str(), key.size());
    lcb_cmdstore_value(cmd, val.c_str(), val.size());
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);

    Item itm;
    getKey(instance, key, itm);
    EXPECT_STREQ("foobar", itm.val.c_str());
}

extern "C" {
static void testAppendNonExistingKeyCallback(lcb_INSTANCE *, int, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_APPEND, op);
    EXPECT_EQ(LCB_NOT_STORED, lcb_respstore_status(resp));
    ++(*counter);
}
}

/**
 * @test Append
 * @pre Append a non existing key
 * @post Returns key not stored
 */
TEST_F(MutateUnitTest, testAppendNonExistingKey)
{
    std::string key("testAppendNonExistingKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    lcb_sched_enter(instance);
    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testAppendNonExistingKeyCallback);
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_APPEND);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    int numcallbacks = 0;
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait3(instance, LCB_WAIT_NOCHECK);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void testSimplePrependStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    using namespace std;
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_PREPEND, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    uint64_t cas;
    lcb_respstore_cas(resp, &cas);
    EXPECT_NE(0, cas);
    ++(*counter);
}
}

/**
 * @test Prepend
 * @pre Set a key with the value @c foo, prepend it with the value @c bar.
 * Get the key
 *
 * @post Key is now @c barfoo
 */
TEST_F(MutateUnitTest, testSimplePrepend)
{
    std::string key("testSimplePrependKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testSimplePrependStoreCallback);
    storeKey(instance, key, "foo");
    int numcallbacks = 0;

    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_PREPEND);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);

    Item itm;
    getKey(instance, key, itm);
    EXPECT_STREQ("barfoo", itm.val.c_str());
}

extern "C" {
static void testPrependNonExistingKeyCallback(lcb_INSTANCE *, int, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_PREPEND, op);
    EXPECT_EQ(LCB_NOT_STORED, lcb_respstore_status(resp));
    ++(*counter);
}
}

/**
 * @test Prepend
 * @pre prepend a non existing key
 * @post Returns key not stored
 */
TEST_F(MutateUnitTest, testPrependNonExistingKey)
{
    std::string key("testPrependNonExistingKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    lcb_sched_enter(instance);
    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testPrependNonExistingKeyCallback);
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_PREPEND);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "foo", 3);
    int numcallbacks = 0;
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait3(instance, LCB_WAIT_NOCHECK);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void testSimpleReplaceNonexistingStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_REPLACE, op);
    EXPECT_EQ(LCB_KEY_ENOENT, lcb_respstore_status(resp));
    ++(*counter);
}
}

/**
 * @test Replace (Non-Existing)
 *
 * @pre Replace a non-existing key
 * @post Fails with @c KEY_ENOENT
 */
TEST_F(MutateUnitTest, testSimpleReplaceNonexisting)
{
    std::string key("testSimpleReplaceNonexistingKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE,
                                (lcb_RESPCALLBACK)testSimpleReplaceNonexistingStoreCallback);
    removeKey(instance, key);
    int numcallbacks = 0;
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_REPLACE);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void testSimpleReplaceStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_REPLACE, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    uint64_t cas;
    lcb_respstore_cas(resp, &cas);
    EXPECT_NE(0, cas);
    ++(*counter);
}
}

/**
 * @test Replace (Hit)
 * @pre
 * Set a key to the value @c foo, replace it with the value @c bar, get the key
 *
 * @post
 * Replace is a success, and the value is now @c bar
 */
TEST_F(MutateUnitTest, testSimpleReplace)
{
    std::string key("testSimpleReplaceKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testSimpleReplaceStoreCallback);
    storeKey(instance, key, "foo");
    int numcallbacks = 0;
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_REPLACE);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
    Item itm;
    getKey(instance, key, itm);
    EXPECT_STREQ("bar", itm.val.c_str());
}

extern "C" {
static void testIncorrectCasReplaceStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_REPLACE, op);
    EXPECT_EQ(LCB_KEY_EEXISTS, lcb_respstore_status(resp));
    ++(*counter);
}
}

/**
 * @test Replace (Invalid CAS)
 *
 * @pre Set a key to the value @c foo. Replace the key specifying a garbage
 * CAS value.
 *
 * @post Replace fails with @c KEY_EEXISTS
 */
TEST_F(MutateUnitTest, testIncorrectCasReplace)
{
    std::string key("testIncorrectCasReplaceKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testIncorrectCasReplaceStoreCallback);
    storeKey(instance, key, "foo");
    Item itm;
    getKey(instance, key, itm);

    int numcallbacks = 0;
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_REPLACE);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    lcb_cmdstore_cas(cmd, itm.cas + 1);

    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
}

extern "C" {
static void testCasReplaceStoreCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    int *counter;
    lcb_respstore_cookie(resp, (void **)&counter);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_REPLACE, op);
    EXPECT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    ++(*counter);
}
}

/**
 * @test Replace (CAS)
 *
 * @pre Store a key with the value @c foo, retrieve its CAS, and use retrieved
 * cas to replace the value with @c bar
 *
 * @post Replace succeeds, get on the key yields the new value @c bar.
 */
TEST_F(MutateUnitTest, testCasReplace)
{
    std::string key("testCasReplaceKey");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    (void)lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)testCasReplaceStoreCallback);
    storeKey(instance, key, "foo");
    Item itm;
    getKey(instance, key, itm);

    int numcallbacks = 0;
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_REPLACE);
    lcb_cmdstore_key(cmd, key.data(), key.length());
    lcb_cmdstore_value(cmd, "bar", 3);
    lcb_cmdstore_cas(cmd, itm.cas);
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, &numcallbacks, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
    EXPECT_EQ(1, numcallbacks);
    getKey(instance, key, itm);
    EXPECT_STREQ("bar", itm.val.c_str());
}

extern "C" {
static void storeCb(lcb_INSTANCE *, int, const lcb_RESPSTORE *resp)
{
    bool *rv;
    ASSERT_EQ(LCB_SUCCESS, lcb_respstore_status(resp));
    lcb_respstore_cookie(resp, (void **)&rv);
    *rv = true;
}
}

TEST_F(MutateUnitTest, testSetDefault)
{
    std::string key("testDefaultMode");
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)storeCb);

    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, key.c_str(), key.size());
    lcb_cmdstore_value(cmd, "foo", 3);
    bool cookie = false;
    ASSERT_EQ(LCB_SUCCESS, lcb_store(instance, &cookie, cmd));
    lcb_cmdstore_destroy(cmd);
    lcb_wait(instance);
}
