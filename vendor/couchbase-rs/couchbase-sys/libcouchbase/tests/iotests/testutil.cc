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

#include "mock-unit-test.h"
#include "testutil.h"
#include <map>

/*
 * Helper functions
 */
extern "C" {
static void storeKvoCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    KVOperation *kvo;
    lcb_respstore_cookie(resp, (void **)&kvo);
    kvo->cbCommon(lcb_respstore_status(resp));
    kvo->result.assign(resp);
    lcb_STORE_OPERATION op;
    lcb_respstore_operation(resp, &op);
    ASSERT_EQ(LCB_STORE_SET, op);
}

static void getKvoCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPGET *resp)
{
    KVOperation *kvo;
    lcb_respget_cookie(resp, (void **)&kvo);
    kvo->cbCommon(lcb_respget_status(resp));
    kvo->result.assign(resp);
}

static void removeKvoCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPREMOVE *resp)
{
    KVOperation *kvo;
    lcb_respremove_cookie(resp, (void **)&kvo);
    kvo->cbCommon(lcb_respremove_status(resp));
    kvo->result.assign(resp);
}
}

void KVOperation::handleInstanceError(lcb_INSTANCE *instance, lcb_STATUS err, const char *)
{
    KVOperation *kvo = reinterpret_cast< KVOperation * >(const_cast< void * >(lcb_get_cookie(instance)));
    kvo->assertOk(err);
    kvo->globalErrors.insert(err);
}

void KVOperation::enter(lcb_INSTANCE *instance)
{
    callbacks.get = lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getKvoCallback);
    callbacks.rm = lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, (lcb_RESPCALLBACK)removeKvoCallback);
    callbacks.store = lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)storeKvoCallback);
    oldCookie = lcb_get_cookie(instance);
    lcb_set_cookie(instance, this);
}

void KVOperation::leave(lcb_INSTANCE *instance)
{
    lcb_install_callback3(instance, LCB_CALLBACK_GET, callbacks.get);
    lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, callbacks.rm);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, callbacks.store);
    lcb_set_cookie(instance, oldCookie);
}

void KVOperation::assertOk(lcb_STATUS err)
{
    if (ignoreErrors) {
        return;
    }

    if (allowableErrors.empty()) {
        ASSERT_EQ(LCB_SUCCESS, err) << "Unexpected error: " << lcb_strerror_short(err);
        return;
    }
    ASSERT_TRUE(allowableErrors.find(err) != allowableErrors.end())
        << "Unable to find " << lcb_strerror_short(err) << " in allowable errors";
}

void KVOperation::store(lcb_INSTANCE *instance)
{
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, request->key.data(), request->key.length());
    lcb_cmdstore_value(cmd, request->val.data(), request->val.length());
    lcb_cmdstore_flags(cmd, request->flags);
    lcb_cmdstore_expiration(cmd, request->exp);
    lcb_cmdstore_cas(cmd, request->cas);
    lcb_cmdstore_datatype(cmd, request->datatype);

    enter(instance);
    EXPECT_EQ(LCB_SUCCESS, lcb_store(instance, this, cmd));
    lcb_cmdstore_destroy(cmd);
    EXPECT_EQ(LCB_SUCCESS, lcb_wait(instance));
    leave(instance);

    ASSERT_EQ(1, callCount);
}

void KVOperation::remove(lcb_INSTANCE *instance)
{
    lcb_CMDREMOVE *cmd;
    lcb_cmdremove_create(&cmd);
    lcb_cmdremove_key(cmd, request->key.data(), request->key.length());

    enter(instance);
    EXPECT_EQ(LCB_SUCCESS, lcb_remove(instance, this, cmd));
    lcb_cmdremove_destroy(cmd);
    EXPECT_EQ(LCB_SUCCESS, lcb_wait(instance));
    leave(instance);

    ASSERT_EQ(1, callCount);
}

void KVOperation::get(lcb_INSTANCE *instance)
{
    lcb_CMDGET *cmd;
    lcb_cmdget_create(&cmd);
    lcb_cmdget_key(cmd, request->key.data(), request->key.length());
    lcb_cmdget_expiration(cmd, request->exp);

    enter(instance);
    EXPECT_EQ(LCB_SUCCESS, lcb_get(instance, this, cmd));
    EXPECT_EQ(LCB_SUCCESS, lcb_wait(instance));
    leave(instance);

    lcb_cmdget_destroy(cmd);

    ASSERT_EQ(1, callCount);
}

void storeKey(lcb_INSTANCE *instance, const std::string &key, const std::string &value)
{
    Item req = Item(key, value);
    KVOperation kvo = KVOperation(&req);
    kvo.store(instance);
}

void removeKey(lcb_INSTANCE *instance, const std::string &key)
{
    Item req = Item();
    req.key = key;
    KVOperation kvo = KVOperation(&req);
    kvo.allowableErrors.insert(LCB_SUCCESS);
    kvo.allowableErrors.insert(LCB_KEY_ENOENT);
    kvo.remove(instance);
}

void getKey(lcb_INSTANCE *instance, const std::string &key, Item &item)
{
    Item req = Item();
    req.key = key;
    KVOperation kvo = KVOperation(&req);
    kvo.result.cas = 0xdeadbeef;

    kvo.get(instance);
    ASSERT_NE(0xdeadbeef, kvo.result.cas);
    item = kvo.result;
}

void genDistKeys(lcbvb_CONFIG *vbc, std::vector< std::string > &out)
{
    char buf[1024] = {'\0'};
    int servers_max = lcbvb_get_nservers(vbc);
    std::map< int, bool > found_servers;
    EXPECT_TRUE(servers_max > 0);

    for (int cur_num = 0; found_servers.size() != servers_max; cur_num++) {
        int ksize = sprintf(buf, "VBKEY_%d", cur_num);
        int vbid;
        int srvix;
        lcbvb_map_key(vbc, buf, ksize, &vbid, &srvix);

        if (!found_servers[srvix]) {
            out.push_back(std::string(buf));
            found_servers[srvix] = true;
        }
    }

    EXPECT_EQ(servers_max, out.size());
}

void genStoreCommands(const std::vector< std::string > &keys, std::vector< lcb_CMDSTORE * > &cmds)
{
    for (unsigned int ii = 0; ii < keys.size(); ii++) {
        lcb_CMDSTORE *cmd;
        lcb_cmdstore_create(&cmd, LCB_STORE_SET);
        lcb_cmdstore_key(cmd, keys[ii].c_str(), keys[ii].size());
        lcb_cmdstore_value(cmd, keys[ii].c_str(), keys[ii].size());
        cmds.push_back(cmd);
    }
}

/**
 * This doesn't _actually_ attempt to make sense of an operation. It simply
 * will try to keep the event loop alive.
 */
void doDummyOp(lcb_INSTANCE *instance)
{
    Item itm("foo", "bar");
    KVOperation kvo(&itm);
    kvo.ignoreErrors = true;
    kvo.store(instance);
}

/**
 * Dump the item object to a stream
 * @param out where to dump the object to
 * @param item the item to print
 * @return the stream
 */
std::ostream &operator<<(std::ostream &out, const Item &item)
{
    using namespace std;
    out << "Key: " << item.key << endl;
    if (item.val.length()) {
        out << "Value: " << item.val << endl;
    }

    out << ios::hex << "CAS: 0x" << item.cas << endl << "Flags: 0x" << item.flags << endl;

    if (item.err != LCB_SUCCESS) {
        out << "Error: " << item.err << endl;
    }

    return out;
}
