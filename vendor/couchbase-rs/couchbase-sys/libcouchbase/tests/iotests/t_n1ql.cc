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
#include <libcouchbase/couchbase.h>
#include "internal.h"

using std::string;
using std::vector;

struct N1QLResult {
    vector< string > rows;
    string meta;
    uint16_t htcode;
    lcb_STATUS rc;
    bool called;

    N1QLResult()
    {
        reset();
    }

    void reset()
    {
        called = false;
        rc = LCB_SUCCESS;
        meta.clear();
        rows.clear();
        htcode = 0;
    }
};

#define SKIP_QUERY_TEST()                                                                                              \
    fprintf(stderr, "Requires recent mock with query support");                                                        \
    return

#define SKIP_CLUSTER_QUERY_TEST()                                                                                      \
    fprintf(stderr, "Requires recent server with query support");                                                      \
    return

extern "C" {
static void rowcb(lcb_INSTANCE *, int, const lcb_RESPN1QL *resp)
{
    N1QLResult *res;
    lcb_respn1ql_cookie(resp, (void **)&res);

    const char *row;
    size_t nrow;
    lcb_respn1ql_row(resp, &row, &nrow);

    if (lcb_respn1ql_is_final(resp)) {
        res->rc = lcb_respn1ql_status(resp);
        if (row) {
            res->meta.assign(row, nrow);
        }
        const lcb_RESPHTTP *http = NULL;
        lcb_respn1ql_http_response(resp, &http);
        if (http) {
            lcb_resphttp_http_status(http, &res->htcode);
        }
    } else {
        res->rows.push_back(string(row, nrow));
    }
    res->called = true;
}
}

class QueryUnitTest : public MockUnitTest
{
  protected:
    lcb_CMDN1QL *cmd;
    void SetUp()
    {
        lcb_cmdn1ql_create(&cmd);
    }
    void TearDown()
    {
        lcb_cmdn1ql_destroy(cmd);
    }
    bool createQueryConnection(HandleWrap &hw, lcb_INSTANCE **instance)
    {
        if (MockEnvironment::getInstance()->isRealCluster()) {
            return false;
        }
        createConnection(hw, instance);
        const lcbvb_CONFIG *vbc;
        lcb_STATUS rc;
        rc = lcb_cntl(*instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
        EXPECT_EQ(LCB_SUCCESS, rc);
        int ix = lcbvb_get_randhost(vbc, LCBVB_SVCTYPE_N1QL, LCBVB_SVCMODE_PLAIN);
        return ix > -1;
    }

    bool createClusterQueryConnection(HandleWrap &hw, lcb_INSTANCE **instance)
    {
        if (!MockEnvironment::getInstance()->isRealCluster()) {
            return false;
        }
        createClusterConnection(hw, instance);
        const lcbvb_CONFIG *vbc = NULL;
        lcb_STATUS rc;
        rc = lcb_cntl(*instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
        EXPECT_TRUE(vbc != NULL);
        EXPECT_EQ(LCB_SUCCESS, rc);
        int ix = lcbvb_get_randhost(vbc, LCBVB_SVCTYPE_N1QL, LCBVB_SVCMODE_PLAIN);
        return ix > -1;
    }

    void makeCommand(const char *query, bool prepared = false)
    {
        lcb_cmdn1ql_reset(cmd);
        lcb_cmdn1ql_statement(cmd, query, strlen(query));
        lcb_cmdn1ql_callback(cmd, rowcb);
        lcb_cmdn1ql_adhoc(cmd, !prepared);
    }
};

TEST_F(QueryUnitTest, testSimple)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }

    N1QLResult res;
    makeCommand("SELECT mockrow");
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, res.rc);
    ASSERT_EQ(1, res.rows.size());
}

TEST_F(QueryUnitTest, testQueryError)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT blahblah FROM blahblah");
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_TRUE(res.rows.empty());
}

TEST_F(QueryUnitTest, testInvalidJson)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_CMDN1QL *cmd;
    lcb_cmdn1ql_create(&cmd);

    const char *bad_query = "blahblah";
    ASSERT_NE(LCB_SUCCESS, lcb_cmdn1ql_query(cmd, bad_query, strlen(bad_query)));
    lcb_cmdn1ql_destroy(cmd);
}

TEST_F(QueryUnitTest, testPrepareOk)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT mockrow", true);
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(res.rc, LCB_SUCCESS);
    ASSERT_EQ(1, res.rows.size());

    // Get the plan contents
    string query("SELECT mockrow");
    string plan;
    lcb_n1qlcache_getplan(instance->n1ql_cache, query, plan);
    // We have the plan!
    ASSERT_FALSE(plan.empty());

    // Issue it again..
    makeCommand("SELECT mockrow", true);
    res.reset();
    rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    string plan2;
    lcb_n1qlcache_getplan(instance->n1ql_cache, query, plan2);
    ASSERT_FALSE(plan2.empty());
    ASSERT_EQ(plan, plan2) << "Reused the same query (cache works!)";

    lcb_n1qlcache_clear(instance->n1ql_cache);
    plan.clear();
    lcb_n1qlcache_getplan(instance->n1ql_cache, query, plan);
    ASSERT_TRUE(plan.empty());

    // Issue it again!
    makeCommand("SELECT mockrow", true);
    res.reset();
    rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);

    ASSERT_EQ(1, res.rows.size());
    lcb_n1qlcache_getplan(instance->n1ql_cache, query, plan);
    ASSERT_FALSE(plan.empty());
}

TEST_F(QueryUnitTest, testPrepareStale)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT mockrow", true);
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(1, res.rows.size());

    // Reset the index "state"
    MockCommand mcmd(MockCommand::RESET_QUERYSTATE);
    doMockTxn(mcmd);

    // Ensure the previous plan fails
    string query("SELECT mockrow");

    string raw;
    lcb_n1qlcache_getplan(instance->n1ql_cache, query, raw);
    ASSERT_FALSE(raw.empty());

    lcb_cmdn1ql_reset(cmd);
    lcb_cmdn1ql_callback(cmd, rowcb);
    ASSERT_EQ(LCB_SUCCESS, lcb_cmdn1ql_query(cmd, raw.c_str(), raw.size()));

    res.reset();
    rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_TRUE(res.rows.empty());
    ASSERT_FALSE(res.meta.empty());
    ASSERT_NE(string::npos, res.meta.find("indexNotFound"));

    // Now that we've verified our current plan isn't working, let's try to
    // issue the cached plan again. lcb should get us a new plan
    makeCommand("SELECT mockrow", true);
    res.reset();
    rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(1, res.rows.size());
}

TEST_F(QueryUnitTest, testPrepareFailure)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT blahblah", true);
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_TRUE(res.called);
    ASSERT_NE(LCB_SUCCESS, res.rc);
    ASSERT_TRUE(res.rows.empty());
}

TEST_F(QueryUnitTest, testCancellation)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createQueryConnection(hw, &instance)) {
        SKIP_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT mockrow");
    lcb_N1QL_HANDLE *handle = NULL;
    lcb_cmdn1ql_handle(cmd, &handle);
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    ASSERT_TRUE(handle != NULL);
    lcb_n1ql_cancel(instance, handle);
    lcb_wait(instance);
    ASSERT_FALSE(res.called);
}

TEST_F(QueryUnitTest, testClusterwide)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    if (!createClusterQueryConnection(hw, &instance)) {
        SKIP_CLUSTER_QUERY_TEST();
    }
    N1QLResult res;
    makeCommand("SELECT 1");
    lcb_N1QL_HANDLE *handle = NULL;
    lcb_cmdn1ql_handle(cmd, &handle);
    lcb_STATUS rc = lcb_n1ql(instance, &res, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    ASSERT_TRUE(handle != NULL);
    lcb_n1ql_cancel(instance, handle);
    lcb_wait(instance);
    ASSERT_FALSE(res.called);
}
