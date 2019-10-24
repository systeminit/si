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

#include <cstdio>

class ConfigCacheUnitTest : public MockUnitTest
{
};

extern "C" {
static void bootstrap_callback(lcb_INSTANCE *instance, lcb_STATUS err)
{
    EXPECT_EQ(LCB_SUCCESS, err);
    int *pp = (int *)lcb_get_cookie(instance);
    *pp += 1;
}
}

TEST_F(ConfigCacheUnitTest, testConfigCache)
{
    lcb_INSTANCE *instance;
    lcb_STATUS err;
    lcb_create_st cropts;

    // Get the filename:
    char filename[L_tmpnam + 0];
    ASSERT_TRUE(NULL != tmpnam(filename));
    memset(&cropts, 0, sizeof(cropts));

    MockEnvironment::getInstance()->makeConnectParams(cropts, NULL);
    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CONFIGCACHE, (void *)filename);
    ASSERT_EQ(LCB_SUCCESS, err);

    int is_loaded;
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_CONFIG_CACHE_LOADED, &is_loaded);

    ASSERT_EQ(err, LCB_SUCCESS);
    ASSERT_EQ(is_loaded, 0);

    err = lcb_connect(instance);
    ASSERT_EQ(err, LCB_SUCCESS);

    err = lcb_wait(instance);
    ASSERT_EQ(err, LCB_SUCCESS);

    // now try another one
    lcb_destroy(instance);
    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CONFIGCACHE, (void *)filename);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_set_bootstrap_callback(instance, bootstrap_callback);
    int bsCalled = 0;
    lcb_set_cookie(instance, &bsCalled);

    err = lcb_connect(instance);
    ASSERT_EQ(LCB_SUCCESS, err);

    err = lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, err);

    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_CONFIG_CACHE_LOADED, &is_loaded);
    ASSERT_NE(0, is_loaded);
    ASSERT_EQ(1, bsCalled);

    /* Just make sure we can schedule a command */
    storeKey(instance, "a_key", "a_value");

    lcb_destroy(instance);

    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl_string(instance, "config_cache", filename);
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_connect(instance);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_wait(instance);
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_CONFIG_CACHE_LOADED, &is_loaded);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_NE(0, is_loaded);
    lcb_destroy(instance);

    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CONFIGCACHE_RO, (void *)filename);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_destroy(instance);

    // Try one more time, with a directory (name with trailing slash)
    std::string dirname(filename);
    dirname += '/';
    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CONFIGCACHE, (void *)dirname.c_str());
    ASSERT_EQ(LCB_SUCCESS, err);
    char *bucketname = NULL;
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_BUCKETNAME, &bucketname);
    ASSERT_EQ(LCB_SUCCESS, err);
    char *cachefile = NULL;
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_CONFIGCACHE, &cachefile);
    ASSERT_EQ(LCB_SUCCESS, err);
    std::string expected(dirname + bucketname);
    ASSERT_STREQ(expected.c_str(), cachefile);
    lcb_destroy(instance);

    remove(filename);

    // Try one more time, with a file that does not exist..
    doLcbCreate(&instance, &cropts, MockEnvironment::getInstance());
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CONFIGCACHE_RO, (void *)filename);
    ASSERT_NE(LCB_SUCCESS, err);
    lcb_destroy(instance);
}
