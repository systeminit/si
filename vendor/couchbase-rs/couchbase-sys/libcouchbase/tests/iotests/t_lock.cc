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
#include <map>
#include "iotests.h"

class LockUnitTest : public MockUnitTest
{
};

extern "C" {
static void getLockedCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPGET *resp)
{
    Item *itm;
    lcb_respget_cookie(resp, (void **)&itm);
    itm->assign(resp);
}

static void unlockCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPUNLOCK *resp)
{
    lcb_STATUS *rc;
    lcb_respunlock_cookie(resp, (void **)&rc);
    *rc = lcb_respunlock_status(resp);
}
}

/**
 * @test
 * Lock (lock and unlock)
 *
 * @pre
 * Set a key, and get the value specifying the lock option with a timeout
 * of @c 10.
 *
 * @post
 * Lock operation succeeds.
 *
 * @pre Unlock the key using the CAS from the previous get result.
 * @post Unlock succeeds
 */
TEST_F(LockUnitTest, testSimpleLockAndUnlock)
{
    LCB_TEST_REQUIRE_FEATURE("lock")

    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    std::string key = "lockKey";
    std::string value = "lockValue";

    removeKey(instance, key);
    storeKey(instance, key, value);

    lcb_CMDGET *cmd;
    lcb_cmdget_create(&cmd);
    lcb_cmdget_key(cmd, key.c_str(), key.size());
    lcb_cmdget_locktime(cmd, 10);
    Item itm;

    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getLockedCallback);

    ASSERT_EQ(LCB_SUCCESS, lcb_get(instance, &itm, cmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, itm.err);
    lcb_cmdget_destroy(cmd);

    lcb_CMDUNLOCK *ucmd;
    lcb_cmdunlock_create(&ucmd);
    lcb_cmdunlock_key(ucmd, key.c_str(), key.size());
    lcb_cmdunlock_cas(ucmd, itm.cas);

    lcb_STATUS reserr = LCB_ERROR;
    lcb_install_callback3(instance, LCB_CALLBACK_UNLOCK, (lcb_RESPCALLBACK)unlockCallback);
    ASSERT_EQ(LCB_SUCCESS, lcb_unlock(instance, &reserr, ucmd));
    lcb_cmdunlock_destroy(ucmd);
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, reserr);
}

/**
 * @test Lock (Missing CAS)
 *
 * @pre
 * Store a key and attempt to unlock it with an invalid CAS
 *
 * @post
 * Error result of @c ETMPFAIL
 */
TEST_F(LockUnitTest, testUnlockMissingCas)
{
    LCB_TEST_REQUIRE_FEATURE("lock")

    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);

    lcb_STATUS reserr = LCB_ERROR;
    std::string key = "lockKey2";
    std::string value = "lockValue";

    storeKey(instance, key, value);

    lcb_CMDUNLOCK *cmd;
    lcb_cmdunlock_create(&cmd);
    lcb_cmdunlock_key(cmd, key.c_str(), key.size());
    lcb_cmdunlock_cas(cmd, 0);

    lcb_install_callback3(instance, LCB_CALLBACK_UNLOCK, (lcb_RESPCALLBACK)unlockCallback);

    ASSERT_EQ(LCB_SUCCESS, lcb_unlock(instance, &reserr, cmd));
    lcb_cmdunlock_destroy(cmd);
    lcb_wait(instance);
    if (CLUSTER_VERSION_IS_HIGHER_THAN(MockEnvironment::VERSION_50)) {
        ASSERT_EQ(LCB_EINVAL_MCD, reserr);
    } else {
        ASSERT_EQ(LCB_ETMPFAIL, reserr);
    }
}

extern "C" {
static void lockedStorageCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    Item *itm;
    lcb_respstore_cookie(resp, (void **)&itm);
    itm->assign(resp);
}
}
/**
 * @test Lock (Storage Contention)
 *
 * @pre
 * Store a key, perform a GET operation with the lock option, specifying a
 * timeout of @c 10.
 *
 * Then attempt to store the key (without specifying any CAS).
 *
 * @post Store operation fails with @c KEY_EEXISTS. Getting the key retains
 * the old value.
 *
 * @pre store the key using the CAS specified from the first GET
 * @post Storage succeeds. Get returns new value.
 */
TEST_F(LockUnitTest, testStorageLockContention)
{
    LCB_TEST_REQUIRE_FEATURE("lock")

    lcb_INSTANCE *instance;
    HandleWrap hw;
    lcb_STATUS err;

    createConnection(hw, &instance);
    Item itm;
    std::string key = "lockedKey", value = "lockedValue", newvalue = "newUnlockedValue";

    /* undo any funny business on our key */
    removeKey(instance, key);
    storeKey(instance, key, value);

    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getLockedCallback);
    lcb_install_callback3(instance, LCB_CALLBACK_UNLOCK, (lcb_RESPCALLBACK)unlockCallback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)lockedStorageCallback);

    /* get the key and lock it */
    lcb_CMDGET *gcmd;
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_cmdget_locktime(gcmd, 10);
    ASSERT_EQ(LCB_SUCCESS, lcb_get(instance, &itm, gcmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, itm.err);
    ASSERT_GT(itm.cas, 0);
    lcb_cmdget_destroy(gcmd);

    /* now try to set the key, while the lock is still in place */
    lcb_CMDSTORE *scmd;
    lcb_cmdstore_create(&scmd, LCB_STORE_SET);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, newvalue.c_str(), newvalue.size());
    Item s_itm;
    ASSERT_EQ(LCB_SUCCESS, lcb_store(instance, &s_itm, scmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_KEY_EEXISTS, s_itm.err);

    /* verify the value is still the old value */
    Item ritem;
    getKey(instance, key, ritem);
    ASSERT_EQ(ritem.val, value);

    /* now try to set it with the correct cas, implicitly unlocking the key */
    lcb_cmdstore_cas(scmd, itm.cas);
    ASSERT_EQ(LCB_SUCCESS, lcb_store(instance, &s_itm, scmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, itm.err);

    /* verify the value is now the new value */
    getKey(instance, key, ritem);
    ASSERT_EQ(ritem.val, newvalue);
    lcb_cmdstore_destroy(scmd);
}

/**
 * @test
 * Lock (Unlocking)
 *
 * @pre
 * Store a key, get it with the lock option, specifying an expiry of @c 10.
 * Try to unlock the key (using the @c lcb_unlock function) without a valid
 * CAS.
 *
 * @post Unlock fails with @c ETMPFAIL
 *
 * @pre
 * Unlock the key using the valid cas retrieved from the first lock operation.
 * Then try to store the key with a new value.
 *
 * @post Unlock succeeds and retrieval of key yields new value.
 */
TEST_F(LockUnitTest, testUnlLockContention)
{
    LCB_TEST_REQUIRE_FEATURE("lock")

    lcb_INSTANCE *instance;
    HandleWrap hw;
    lcb_STATUS err, reserr = LCB_ERROR;
    createConnection(hw, &instance);

    std::string key = "lockedKey2", value = "lockedValue2";
    storeKey(instance, key, value);
    Item gitm;

    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getLockedCallback);
    lcb_install_callback3(instance, LCB_CALLBACK_UNLOCK, (lcb_RESPCALLBACK)unlockCallback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)lockedStorageCallback);

    lcb_CMDGET *gcmd;
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_cmdget_locktime(gcmd, 10);

    ASSERT_EQ(LCB_SUCCESS, lcb_get(instance, &gitm, gcmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, gitm.err);

    lcb_cas_t validCas = gitm.cas;
    ASSERT_EQ(LCB_SUCCESS, lcb_get(instance, &gitm, gcmd));
    lcb_wait(instance);
    ASSERT_EQ(LCB_ETMPFAIL, gitm.err);
    lcb_cmdget_destroy(gcmd);

    lcb_CMDUNLOCK *ucmd;
    lcb_cmdunlock_create(&ucmd);
    lcb_cmdunlock_key(ucmd, key.c_str(), key.size());
    lcb_cmdunlock_cas(ucmd, validCas);

    ASSERT_EQ(LCB_SUCCESS, lcb_unlock(instance, &reserr, ucmd));
    lcb_cmdunlock_destroy(ucmd);
    lcb_wait(instance);
    ASSERT_EQ(reserr, LCB_SUCCESS);

    std::string newval = "lockedValueNew2";
    storeKey(instance, key, newval);
    getKey(instance, key, gitm);
    ASSERT_EQ(gitm.val, newval);
}
