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

#include "config.h"
#include "iotests.h"
#include "internal.h"

class SnappyUnitTest : public MockUnitTest
{
  protected:
    void setCompression(std::string mode)
    {
        MockEnvironment::getInstance()->setCompression(mode);
    }

    bool isCompressed(std::string &key)
    {
        const Json::Value info = MockEnvironment::getInstance()->getKeyInfo(key);
        for (Json::Value::const_iterator ii = info.begin(); ii != info.end(); ii++) {
            const Json::Value &node = *ii;
            if (node.isNull()) {
                continue;
            }
            if (node["Conf"]["Type"] == "master") {
                return node["Cache"]["Snappy"].asBool();
            }
        }
        return false;
    }
};

struct SnappyCookie {
    lcb_STATUS rc;
    bool called;
    std::string value;

    void reset()
    {
        rc = LCB_SUCCESS;
        called = false;
    }
    SnappyCookie() : rc(LCB_SUCCESS), called(false) {}

    ~SnappyCookie() {}
};

extern "C" {
static void storecb(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    SnappyCookie *cookie = reinterpret_cast< SnappyCookie * >(rb->cookie);
    cookie->called = true;
    cookie->rc = rb->rc;
}
static void getcb(lcb_INSTANCE *, int, const lcb_RESPGET *resp)
{
    SnappyCookie *cookie;
    lcb_respget_cookie(resp, (void **)&cookie);
    cookie->called = true;
    cookie->rc = lcb_respget_status(resp);
    const char *value;
    size_t nvalue;
    lcb_respget_value(resp, &value, &nvalue);
    cookie->value.assign(value, nvalue);
}
}

TEST_F(SnappyUnitTest, testSpec)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;

    setCompression("passive");
    createConnection(hw, &instance);
    lcb_cntl_setu32(instance, LCB_CNTL_COMPRESSION_OPTS, LCB_COMPRESS_INOUT);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getcb);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, storecb);

    std::string key("hello");
    std::string value("A big black bug bit a big black bear, made the big black bear bleed blood");
    std::string compressed("IPA big black bug bit a.\x14");

    SnappyCookie cookie;
    lcb_CMDSTORE *scmd;
    lcb_CMDGET *gcmd;

    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, value.c_str(), value.size());
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    /* now we have negotiated snappy feature */
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(value.c_str(), cookie.value.c_str());
    ASSERT_TRUE(isCompressed(key));
    lcb_cmdget_destroy(gcmd);

    lcb_cntl_setu32(instance, LCB_CNTL_COMPRESSION_OPTS, LCB_COMPRESS_OUT);
    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(compressed.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);

    setCompression("off");
    createConnection(hw, &instance);
    lcb_cntl_setu32(instance, LCB_CNTL_COMPRESSION_OPTS, LCB_COMPRESS_INOUT);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getcb);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, storecb);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(value.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);

    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_FALSE(isCompressed(key));
    lcb_cmdstore_destroy(scmd);
}

TEST_F(SnappyUnitTest, testIOV)
{

    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;

    setCompression("passive");
    createConnection(hw, &instance);
    lcb_cntl_setu32(instance, LCB_CNTL_COMPRESSION_OPTS, LCB_COMPRESS_INOUT);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getcb);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, storecb);

    std::string key("hello");
    std::string value1("A big black bug bit ");
    std::string value2("a big black bear, ");
    std::string value3("made the big black ");
    std::string value4("bear bleed blood");
    std::string value = value1 + value2 + value3 + value4;
    std::string compressed("IPA big black bug bit a.\x14");

    SnappyCookie cookie;
    lcb_CMDSTORE *scmd;
    lcb_CMDGET *gcmd;

    lcb_IOV iov[4];
    unsigned int niov = 4;
    iov[0].iov_base = (void *)value1.c_str();
    iov[0].iov_len = value1.size();
    iov[1].iov_base = (void *)value2.c_str();
    iov[1].iov_len = value2.size();
    iov[2].iov_base = (void *)value3.c_str();
    iov[2].iov_len = value3.size();
    iov[3].iov_base = (void *)value4.c_str();
    iov[3].iov_len = value4.size();

    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value_iov(scmd, iov, niov);
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    /* now we have negotiated snappy feature */
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    lcb_cmdstore_destroy(scmd);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(value.c_str(), cookie.value.c_str());
    ASSERT_TRUE(isCompressed(key));
    lcb_cmdget_destroy(gcmd);

    lcb_cntl_setu32(instance, LCB_CNTL_COMPRESSION_OPTS, LCB_COMPRESS_OUT);
    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(compressed.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);
}

TEST_F(SnappyUnitTest, testSettings)
{

    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;

    setCompression("passive");
    createConnection(hw, &instance);
    lcb_cntl_string(instance, "compression", "deflate_only");
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)getcb);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, storecb);

    std::string key("hello");
    std::string value("A big black bug bit a big black bear, made the big black bear bleed blood");
    std::string compressed("IPA big black bug bit a.\x14");

    SnappyCookie cookie;
    lcb_CMDSTORE *scmd;
    lcb_CMDGET *gcmd;

    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, value.c_str(), value.size());
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    /* now we have negotiated snappy feature */
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    lcb_cmdstore_destroy(scmd);

    value = "A big black bug";
    compressed = "A big black bug";
    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, value.c_str(), value.size());
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    lcb_cmdstore_destroy(scmd);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(compressed.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);

    lcb_cntl_string(instance, "compression_min_size", "1024"); /* greater than size of the value */
    value = "A big black bug bit a big black bear, made the big black bear bleed blood";
    compressed = "A big black bug bit a big black bear, made the big black bear bleed blood";
    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, value.c_str(), value.size());
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    lcb_cmdstore_destroy(scmd);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(compressed.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);

    lcb_cntl_string(instance, "compression_min_size", "40");   /* less than size of the value */
    lcb_cntl_string(instance, "compression_min_ratio", "0.1"); /* expect to reduce size in 10 times */
    value = "A big black bug bit a big black bear, made the big black bear bleed blood";
    compressed = "A big black bug bit a big black bear, made the big black bear bleed blood";
    lcb_cmdstore_create(&scmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(scmd, key.c_str(), key.size());
    lcb_cmdstore_value(scmd, value.c_str(), value.size());
    cookie = SnappyCookie();
    lcb_store(instance, &cookie, scmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    lcb_cmdstore_destroy(scmd);

    cookie = SnappyCookie();
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key.c_str(), key.size());
    lcb_get(instance, &cookie, gcmd);
    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_SUCCESS, cookie.rc);
    ASSERT_STREQ(compressed.c_str(), cookie.value.c_str());
    lcb_cmdget_destroy(gcmd);
}
