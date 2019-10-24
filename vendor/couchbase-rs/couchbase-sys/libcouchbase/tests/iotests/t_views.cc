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
#include <map>
#include <libcouchbase/pktfwd.h>
#include "contrib/cJSON/cJSON.h"

namespace
{

class ViewsUnitTest : public MockUnitTest
{
  protected:
    void SetUp() {}
    void TearDown() {}
    void connectBeerSample(HandleWrap &hw, lcb_INSTANCE **instance, bool first = true);
};

using std::string;
using std::vector;

extern "C" {
static void bktCreateCb(lcb_INSTANCE *, int, const lcb_RESPHTTP *resp)
{
    ASSERT_EQ(LCB_SUCCESS, lcb_resphttp_status(resp));
    uint16_t status;
    lcb_resphttp_http_status(resp, &status);
    ASSERT_TRUE(status > 199 && status < 300);
}
}

static const char *content_type = "application/json";

void ViewsUnitTest::connectBeerSample(HandleWrap &hw, lcb_INSTANCE **instance, bool first)
{
    lcb_create_st crparams;
    lcb_create_st crparamsAdmin;
    lcb_config_transport_t transports[] = {LCB_CONFIG_TRANSPORT_HTTP, LCB_CONFIG_TRANSPORT_LIST_END};
    MockEnvironment::getInstance()->makeConnectParams(crparams, NULL);
    crparamsAdmin = crparams;

    crparams.v.v2.bucket = "beer-sample";
    if (!CLUSTER_VERSION_IS_HIGHER_THAN(MockEnvironment::VERSION_50)) {
        // We could do CCCP if we really cared.. but it's simpler and makes
        // the logs cleaner.
        crparams.v.v2.user = "beer-sample";
        crparams.v.v2.mchosts = NULL;
        crparams.v.v2.transports = transports;
    }

    // See if we can connect:
    lcb_STATUS rv = tryCreateConnection(hw, instance, crparams);
    if (rv == LCB_SUCCESS) {
        return;
    } else if (!first) {
        ASSERT_EQ(LCB_SUCCESS, rv);
    }

    ASSERT_TRUE(rv == LCB_BUCKET_ENOENT || rv == LCB_AUTH_ERROR);
    hw.destroy(); // Should really be called clear(), since that's what it does

    // Use the management API to load the beer-sample database
    crparamsAdmin.v.v2.type = LCB_TYPE_CLUSTER;
    crparamsAdmin.v.v2.user = "Administrator";
    crparamsAdmin.v.v2.passwd = "password";
    crparamsAdmin.v.v2.bucket = NULL;
    crparamsAdmin.v.v2.transports = transports;

    rv = tryCreateConnection(hw, instance, crparamsAdmin);
    ASSERT_EQ(LCB_SUCCESS, rv);

    const char *path = "/sampleBuckets/install";
    const char *body = "[\"beer-sample\"]";

    lcb_CMDHTTP *htcmd;
    lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_MANAGEMENT);
    lcb_cmdhttp_path(htcmd, path, strlen(path));
    lcb_cmdhttp_body(htcmd, body, strlen(body));
    lcb_cmdhttp_content_type(htcmd, content_type, strlen(content_type));
    lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);

    lcb_install_callback3(*instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)bktCreateCb);
    lcb_sched_enter(*instance);
    rv = lcb_http(*instance, NULL, htcmd);
    lcb_cmdhttp_destroy(htcmd);
    ASSERT_EQ(LCB_SUCCESS, rv);
    lcb_sched_leave(*instance);
    lcb_wait(*instance);
    hw.destroy();

    // Now it should all be good, so we can call recursively..
    connectBeerSample(hw, instance, false);
}

struct ViewRow {
    string key;
    string value;
    string docid;

    struct {
        lcb_STATUS rc;
        const char *key;
        size_t nkey;
        const char *value;
        size_t nvalue;
        uint64_t cas;
    } docContents;

    void clear() {}

    ViewRow(const lcb_RESPVIEW *resp)
    {
        const char *p;
        size_t n;

        lcb_respview_key(resp, &p, &n);
        if (p != NULL) {
            key.assign(p, n);
        }
        lcb_respview_row(resp, &p, &n);
        if (p != NULL) {
            value.assign(p, n);
        }

        const lcb_RESPGET *rg;
        lcb_respview_document(resp, &rg);

        lcb_respview_doc_id(resp, &p, &n);
        if (p != NULL) {
            docid.assign(p, n);
            if (rg != NULL) {
                docContents.rc = lcb_respget_status(rg);
                lcb_respget_cas(rg, &docContents.cas);
                lcb_respget_key(rg, &docContents.key, &docContents.nkey);
                lcb_respget_value(rg, &docContents.value, &docContents.nvalue);

                string tmpId(docContents.key, docContents.nkey);
                EXPECT_EQ(tmpId, docid);
            } else {
                memset(&docContents, 0, sizeof docContents);
            }
        } else {
            EXPECT_TRUE(rg == NULL);
            memset(&docContents, 0, sizeof docContents);
        }
    }
};

struct ViewInfo {
    vector< ViewRow > rows;
    size_t totalRows;
    lcb_STATUS err;
    uint16_t http_status;

    void addRow(const lcb_RESPVIEW *resp)
    {
        lcb_STATUS rc = lcb_respview_status(resp);
        if (err == LCB_SUCCESS && rc != LCB_SUCCESS) {
            err = rc;
        }

        if (!lcb_respview_is_final(resp)) {
            rows.push_back(ViewRow(resp));
        } else {
            const char *row;
            size_t nrow;
            lcb_respview_row(resp, &row, &nrow);
            if (row != NULL) {
                // See if we have a 'value' for the final response
                string vBuf(row, nrow);
                cJSON *cj = cJSON_Parse(row);
                ASSERT_FALSE(cj == NULL);
                cJSON *jTotal = cJSON_GetObjectItem(cj, "total_rows");
                if (jTotal != NULL) {
                    totalRows = jTotal->valueint;
                } else {
                    // Reduce responses might skip total_rows
                    totalRows = rows.size();
                }
                cJSON_Delete(cj);
            }
            const lcb_RESPHTTP *http = NULL;
            lcb_respview_http_response(resp, &http);
            if (http) {
                lcb_resphttp_http_status(http, &http_status);
            }
        }
    }

    void clear()
    {
        for (size_t ii = 0; ii < rows.size(); ii++) {
            rows[ii].clear();
        }
        rows.clear();
        totalRows = 0;
        http_status = 0;
        err = LCB_SUCCESS;
    }

    ~ViewInfo()
    {
        clear();
    }

    ViewInfo()
    {
        clear();
    }
};

extern "C" {
static void viewCallback(lcb_INSTANCE *, int cbtype, const lcb_RESPVIEW *resp)
{
    EXPECT_EQ(LCB_CALLBACK_VIEWQUERY, cbtype);
    //    printf("View Callback invoked!\n");
    ViewInfo *info;
    lcb_respview_cookie(resp, (void **)&info);
    info->addRow(resp);
}
}

TEST_F(ViewsUnitTest, testSimpleView)
{
    SKIP_UNLESS_MOCK();
    // Requires beer-sample
    HandleWrap hw;
    lcb_INSTANCE *instance;
    connectBeerSample(hw, &instance);

    const char *ddoc = "beer", *view = "brewery_beers";

    lcb_CMDVIEW *vq;
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_callback(vq, viewCallback);

    ViewInfo vi;

    lcb_STATUS rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);

    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, vi.err);
    ASSERT_GT(vi.rows.size(), 0U);
    ASSERT_EQ(7303, vi.totalRows);
    // Check the row parses correctly:
    const ViewRow &row = vi.rows.front();
    // Unquoted docid
    ASSERT_EQ("21st_amendment_brewery_cafe", row.docid);
    ASSERT_EQ("[\"21st_amendment_brewery_cafe\"]", row.key);
    ASSERT_EQ("null", row.value);
    vi.clear();

    // apply limit
    const char *optstr = "limit=10";
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_option_string(vq, optstr, strlen(optstr));
    lcb_cmdview_callback(vq, viewCallback);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, vi.err);
    ASSERT_EQ(10, vi.rows.size());
    ASSERT_EQ(7303, vi.totalRows);
    vi.clear();

    // Set the limit to 0
    optstr = "limit=0";
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_option_string(vq, optstr, strlen(optstr));
    lcb_cmdview_callback(vq, viewCallback);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(0, vi.rows.size());
    ASSERT_EQ(7303, vi.totalRows);
}

TEST_F(ViewsUnitTest, testIncludeDocs)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    lcb_STATUS rc;
    connectBeerSample(hw, &instance);

    ViewInfo vi;
    const char *ddoc = "beer", *view = "brewery_beers";
    lcb_CMDVIEW *vq;
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_callback(vq, viewCallback);
    lcb_cmdview_include_docs(vq, true);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);

    // Again, ensure everything is OK
    ASSERT_EQ(7303, vi.totalRows);
    ASSERT_EQ(7303, vi.rows.size());

    for (size_t ii = 0; ii < vi.rows.size(); ii++) {
        const ViewRow &row = vi.rows[ii];
        ASSERT_FALSE(row.docContents.key == NULL);
        ASSERT_EQ(row.docid.size(), row.docContents.nkey);
        ASSERT_EQ(LCB_SUCCESS, row.docContents.rc);
        ASSERT_NE(0, row.docContents.cas);
    }
}

TEST_F(ViewsUnitTest, testReduce)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    lcb_STATUS rc;
    connectBeerSample(hw, &instance);

    const char *ddoc = "beer", *view = "by_location";
    ViewInfo vi;
    lcb_CMDVIEW *vq;
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_callback(vq, viewCallback);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(1, vi.rows.size());
    ASSERT_STREQ("1411", vi.rows[0].value.c_str());

    vi.clear();
    // Try with include_docs
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_callback(vq, viewCallback);
    lcb_cmdview_include_docs(vq, true);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(1, vi.rows.size());

    vi.clear();
    // Try with reduce=false
    const char *optstr = "reduce=false&limit=10";
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_option_string(vq, optstr, strlen(optstr));
    lcb_cmdview_callback(vq, viewCallback);
    lcb_cmdview_include_docs(vq, true);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(10, vi.rows.size());
    ASSERT_EQ(1411, vi.totalRows);

    ViewRow *firstRow = &vi.rows[0];
    ASSERT_EQ("[\"Argentina\",\"\",\"Mendoza\"]", firstRow->key);
    ASSERT_EQ("1", firstRow->value);
    ASSERT_EQ("cervecera_jerome", firstRow->docid);

    // try with grouplevel
    vi.clear();
    optstr = "group_level=1";
    lcb_cmdview_create(&vq);
    lcb_cmdview_design_document(vq, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(vq, view, strlen(view));
    lcb_cmdview_option_string(vq, optstr, strlen(optstr));
    lcb_cmdview_callback(vq, viewCallback);
    lcb_cmdview_include_docs(vq, true);
    rc = lcb_view(instance, &vi, vq);
    lcb_cmdview_destroy(vq);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);

    firstRow = &vi.rows[0];
    ASSERT_EQ("[\"Argentina\"]", firstRow->key);
    ASSERT_EQ("2", firstRow->value);
    ASSERT_TRUE(firstRow->docid.empty());
}

TEST_F(ViewsUnitTest, testEngineErrors)
{
    SKIP_UNLESS_MOCK();
    // Tests various things which can go wrong; basically negative responses
    HandleWrap hw;
    lcb_INSTANCE *instance;
    connectBeerSample(hw, &instance);
    lcb_STATUS rc;

    const char *ddoc = "nonexist", *view = "nonexist";
    ViewInfo vi;
    lcb_CMDVIEW *cmd;
    lcb_cmdview_create(&cmd);
    lcb_cmdview_design_document(cmd, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_callback(cmd, viewCallback);
    rc = lcb_view(instance, &vi, cmd);
    lcb_cmdview_destroy(cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_HTTP_ERROR, vi.err);
    ASSERT_EQ(404, vi.http_status);

    vi.clear();
    ddoc = "beer";
    view = "badview";
    lcb_cmdview_create(&cmd);
    lcb_cmdview_design_document(cmd, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_callback(cmd, viewCallback);
    rc = lcb_view(instance, &vi, cmd);
    lcb_cmdview_destroy(cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_HTTP_ERROR, vi.err);
    ASSERT_EQ(404, vi.http_status);

    vi.clear();
    ddoc = "beer";
    view = "brewery_beers";
    const char *optstr = "reduce=true";
    lcb_cmdview_create(&cmd);
    lcb_cmdview_design_document(cmd, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_option_string(cmd, optstr, strlen(optstr));
    lcb_cmdview_callback(cmd, viewCallback);
    rc = lcb_view(instance, &vi, cmd);
    lcb_cmdview_destroy(cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_HTTP_ERROR, vi.err);
    ASSERT_EQ(400, vi.http_status);
}

TEST_F(ViewsUnitTest, testOptionValidation)
{
    HandleWrap hw;
    lcb_INSTANCE *instance;
    connectBeerSample(hw, &instance);

    lcb_CMDVIEW *cmd;
    lcb_cmdview_create(&cmd);
    ASSERT_EQ(LCB_EINVAL, lcb_view(instance, NULL, cmd));
    lcb_cmdview_destroy(cmd);

    lcb_cmdview_create(&cmd);
    lcb_cmdview_callback(cmd, viewCallback);
    ASSERT_EQ(LCB_EINVAL, lcb_view(instance, NULL, cmd));
    lcb_cmdview_destroy(cmd);

    const char *view = "view";
    lcb_cmdview_create(&cmd);
    lcb_cmdview_callback(cmd, viewCallback);
    lcb_cmdview_view_name(cmd, view, strlen(view));
    ASSERT_EQ(LCB_EINVAL, lcb_view(instance, NULL, cmd));
    lcb_cmdview_destroy(cmd);

    const char *ddoc = "design";
    lcb_cmdview_create(&cmd);
    lcb_cmdview_callback(cmd, viewCallback);
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_design_document(cmd, ddoc, strlen(ddoc));
    // Expect it to fail with flags
    lcb_cmdview_include_docs(cmd, true);
    lcb_cmdview_no_row_parse(cmd, true);
    ASSERT_EQ(LCB_OPTIONS_CONFLICT, lcb_view(instance, NULL, cmd));
    lcb_cmdview_destroy(cmd);
}

TEST_F(ViewsUnitTest, testBackslashDocid)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    lcb_STATUS rc;
    connectBeerSample(hw, &instance);

    string key("backslash\\docid");
    string doc("{\"type\":\"brewery\", \"name\":\"Backslash IPA\"}");
    storeKey(instance, key, doc);

    const char *ddoc = "beer", *view = "brewery_beers", *optstr = "stale=false&key=[\"backslash\\\\docid\"]";

    ViewInfo vi;
    lcb_CMDVIEW *cmd;

    lcb_cmdview_create(&cmd);
    lcb_cmdview_callback(cmd, viewCallback);
    lcb_cmdview_design_document(cmd, ddoc, strlen(ddoc));
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_option_string(cmd, optstr, strlen(optstr));
    rc = lcb_view(instance, &vi, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(LCB_SUCCESS, vi.err);
    ASSERT_EQ(1, vi.rows.size());
    ASSERT_EQ(key, vi.rows[0].docid);

    vi.clear();
    lcb_cmdview_include_docs(cmd, true);
    rc = lcb_view(instance, &vi, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(1, vi.rows.size());
    ASSERT_EQ(doc.size(), vi.rows[0].docContents.nvalue);

    removeKey(instance, key);
    vi.clear();
    rc = lcb_view(instance, &vi, cmd);
    ASSERT_EQ(LCB_SUCCESS, rc);
    lcb_wait(instance);
    ASSERT_EQ(0, vi.rows.size());
    lcb_cmdview_destroy(cmd);
}
} // namespace
