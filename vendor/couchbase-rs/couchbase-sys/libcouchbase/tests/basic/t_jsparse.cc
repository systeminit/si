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
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include "jsparse/parser.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include "t_jsparse.h"

class JsonParseTest : public ::testing::Test
{
};

using namespace lcb::jsparse;

static std::string iov2s(const lcb_IOV &iov)
{
    return std::string(reinterpret_cast< const char * >(iov.iov_base), iov.iov_len);
}

struct Context : Parser::Actions {
    lcb_STATUS rc;
    bool received_done;
    std::string meta;
    std::vector< std::string > rows;
    Context()
    {
        reset();
    }
    void reset()
    {
        rc = LCB_SUCCESS;
        received_done = false;
        meta.clear();
        rows.clear();
    }
    void JSPARSE_on_row(const Row &row)
    {
        rows.push_back(iov2s(row.row));
    }
    void JSPARSE_on_complete(const std::string &s)
    {
        meta.assign(s);
        received_done = true;
    }
    void JSPARSE_on_error(const std::string &)
    {
        rc = LCB_PROTOCOL_ERROR;
        received_done = true;
    }
};

static bool validateJsonRows(const char *txt, size_t ntxt, Parser::Mode mode)
{
    Context cx;
    Parser parser(mode, &cx);

    for (size_t ii = 0; ii < ntxt; ii++) {
        parser.feed(txt + ii, 1);
    }
    EXPECT_EQ(LCB_SUCCESS, cx.rc);

    lcb_IOV out;
    parser.get_postmortem(out);
    EXPECT_EQ(cx.meta, iov2s(out));
    Json::Value root;
    EXPECT_TRUE(Json::Reader().parse(cx.meta, root));
    return true;
}

static bool validateBadParse(const char *txt, size_t ntxt, Parser::Mode mode)
{
    Context cx;
    Parser p(mode, &cx);
    p.feed(txt, ntxt);
    EXPECT_EQ(LCB_PROTOCOL_ERROR, cx.rc);
    return true;
}

TEST_F(JsonParseTest, testFTS)
{
    ASSERT_TRUE(validateJsonRows(JSON_fts_good, sizeof(JSON_fts_good), Parser::MODE_FTS));
    ASSERT_TRUE(validateBadParse(JSON_fts_bad, sizeof(JSON_fts_bad), Parser::MODE_FTS));
    ASSERT_TRUE(validateBadParse(JSON_fts_bad2, sizeof(JSON_fts_bad2), Parser::MODE_FTS));
}

TEST_F(JsonParseTest, testN1QL)
{
    ASSERT_TRUE(validateJsonRows(JSON_n1ql_nonempty, sizeof(JSON_n1ql_nonempty), Parser::MODE_N1QL));
    ASSERT_TRUE(validateJsonRows(JSON_n1ql_empty, sizeof(JSON_n1ql_empty), Parser::MODE_N1QL));
    ASSERT_TRUE(validateBadParse(JSON_n1ql_bad, sizeof(JSON_n1ql_bad), Parser::MODE_N1QL));
}

TEST_F(JsonParseTest, testAnalyticsDeferred)
{
    ASSERT_TRUE(validateJsonRows(JSON_ad_nonempty, sizeof(JSON_ad_nonempty), Parser::MODE_ANALYTICS_DEFERRED));
}
