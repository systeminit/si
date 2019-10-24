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
#include "internal.h"
#include "auth-priv.h"
#include <gtest/gtest.h>
#define LIBCOUCHBASE_INTERNAL 1
#include <libcouchbase/couchbase.h>

class CredsTest : public ::testing::Test
{
};

static lcb_INSTANCE *create(const char *connstr = NULL)
{
    lcb_create_st crst;
    memset(&crst, 0, sizeof crst);
    crst.version = 3;
    crst.v.v3.connstr = connstr;
    lcb_INSTANCE *ret;
    lcb_STATUS rc = lcb_create(&ret, &crst);
    EXPECT_EQ(LCB_SUCCESS, rc);
    return ret;
}

TEST_F(CredsTest, testLegacyCreds)
{
    lcb_INSTANCE *instance;
    ASSERT_EQ(LCB_SUCCESS, lcb_create(&instance, NULL));
    lcb::Authenticator &auth = *instance->settings->auth;
    ASSERT_TRUE(auth.username().empty());
    ASSERT_EQ(LCBAUTH_MODE_CLASSIC, auth.mode());

    ASSERT_EQ(1, auth.buckets().size());
    ASSERT_TRUE(auth.buckets().find("default")->second.empty());
    ASSERT_EQ("", auth.password_for(NULL, NULL, "default"));
    ASSERT_EQ("default", auth.username_for(NULL, NULL, "default"));

    // Try to add another user/password:
    lcb_BUCKETCRED creds = {"user2", "pass2"};
    ASSERT_EQ(LCB_SUCCESS, lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_BUCKET_CRED, creds));
    ASSERT_EQ(2, auth.buckets().size());
    ASSERT_EQ("pass2", auth.buckets().find("user2")->second);
    ASSERT_EQ("user2", auth.username_for(NULL, NULL, "user2"));
    ASSERT_EQ("pass2", auth.password_for(NULL, NULL, "user2"));

    ASSERT_TRUE(auth.username().empty());
    ASSERT_TRUE(auth.password().empty());
    lcb_destroy(instance);
}

TEST_F(CredsTest, testRbacCreds)
{
    lcb_INSTANCE *instance = create("couchbase://localhost/default?username=mark");
    lcb::Authenticator &auth = *instance->settings->auth;
    ASSERT_EQ("mark", auth.username());
    ASSERT_EQ(LCBAUTH_MODE_RBAC, auth.mode());
    ASSERT_TRUE(auth.buckets().empty());
    ASSERT_EQ("mark", auth.username_for(NULL, NULL, "default"));
    ASSERT_EQ("", auth.password_for(NULL, NULL, "default"));
    ASSERT_EQ("mark", auth.username_for(NULL, NULL, "jane"));
    ASSERT_EQ("", auth.password_for(NULL, NULL, "jane"));

    // Try adding a new bucket, it should fail
    ASSERT_EQ(LCB_OPTIONS_CONFLICT, auth.add("users", "secret", LCBAUTH_F_BUCKET));

    // Try using "old-style" auth. It should fail:
    ASSERT_EQ(LCB_OPTIONS_CONFLICT, auth.add("users", "secret", LCBAUTH_F_BUCKET | LCBAUTH_F_CLUSTER));
    // Username/password should remain unchanged:
    ASSERT_EQ("mark", auth.username());
    ASSERT_EQ("", auth.password());

    // Try *changing* the credentials
    ASSERT_EQ(LCB_SUCCESS, auth.add("jane", "seekrit", LCBAUTH_F_CLUSTER));
    ASSERT_EQ("jane", auth.username_for(NULL, NULL, "default"));
    ASSERT_EQ("seekrit", auth.password_for(NULL, NULL, "default"));
    lcb_destroy(instance);
}

TEST_F(CredsTest, testSharedAuth)
{
    lcb_INSTANCE *instance1, *instance2;
    ASSERT_EQ(LCB_SUCCESS, lcb_create(&instance1, NULL));
    ASSERT_EQ(LCB_SUCCESS, lcb_create(&instance2, NULL));

    lcb_AUTHENTICATOR *auth = lcbauth_new();
    ASSERT_EQ(1, auth->refcount());

    lcb_set_auth(instance1, auth);
    ASSERT_EQ(2, auth->refcount());

    lcb_set_auth(instance2, auth);
    ASSERT_EQ(3, auth->refcount());

    ASSERT_EQ(instance1->settings->auth, instance2->settings->auth);
    lcb_destroy(instance1);
    lcb_destroy(instance2);
    ASSERT_EQ(1, auth->refcount());
    lcbauth_unref(auth);
}
