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

#define DESIGN_DOC_NAME "lcb_design_doc"
#define VIEW_NAME "lcb-test-view"

class HttpUnitTest : public MockUnitTest
{
};

class HttpCmdContext
{
  public:
    HttpCmdContext() : received(false), dumpIfEmpty(false), dumpIfError(false), cbCount(0) {}

    bool received;
    bool dumpIfEmpty;
    bool dumpIfError;
    unsigned cbCount;

    uint16_t status;
    lcb_STATUS err;
    std::string body;
};

static const char *view_common = "{ "
                                 " \"id\" : \"_design/" DESIGN_DOC_NAME "\","
                                 " \"language\" : \"javascript\","
                                 " \"views\" : { "
                                 " \"" VIEW_NAME "\" : {"
                                 "\"map\":"
                                 " \"function(doc) { "
                                 "if (doc.testid == 'lcb') { emit(doc.id) } "
                                 " } \" "
                                 " } "
                                 "}"
                                 "}";
static const char *content_type = "application/json";

static void dumpResponse(const lcb_RESPHTTP *resp)
{
    const char *const *headers;
    lcb_resphttp_headers(resp, &headers);
    if (headers) {
        for (const char *const *cur = headers; *cur; cur += 2) {
            std::cout << cur[0] << ": " << cur[1] << std::endl;
        }
    }
    const char *body;
    size_t nbody;
    lcb_resphttp_body(resp, &body, &nbody);
    if (body) {
        std::cout << "Data: " << std::endl;
        std::cout.write((const char *)body, nbody);
        std::cout << std::endl;
    }

    const char *path;
    size_t npath;
    lcb_resphttp_path(resp, &path, &npath);
    std::cout << "Path: " << std::endl;
    std::cout.write(path, npath);
    std::cout << std::endl;
}

extern "C" {
static void httpSimpleCallback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPHTTP *resp)
{
    HttpCmdContext *htctx;
    lcb_resphttp_cookie(resp, (void **)&htctx);
    lcb_STATUS rc = lcb_resphttp_status(resp);
    htctx->err = rc;
    lcb_resphttp_http_status(resp, &htctx->status);
    htctx->received = true;
    htctx->cbCount++;

    const char *body;
    size_t nbody;
    lcb_resphttp_body(resp, &body, &nbody);
    if (body) {
        htctx->body.assign(body, nbody);
    }

    if ((nbody == 0 && htctx->dumpIfEmpty) || (rc != LCB_SUCCESS && htctx->dumpIfError)) {
        std::cout << "Count: " << htctx->cbCount << std::endl
                  << "Code: " << rc << std::endl
                  << "nBytes: " << nbody << std::endl;
        dumpResponse(resp);
    }
}
}

/**
 * @test HTTP (Put)
 *
 * @pre Create a valid view document and store it on the server
 * @post Store succeeds and the HTTP result code is 201
 */
TEST_F(HttpUnitTest, testPut)
{
    SKIP_IF_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)httpSimpleCallback);

    std::string design_doc_path("/_design/" DESIGN_DOC_NAME);
    lcb_CMDHTTP *cmd;
    lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_VIEW);
    lcb_cmdhttp_path(cmd, design_doc_path.c_str(), design_doc_path.size());
    lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_PUT);
    lcb_cmdhttp_body(cmd, view_common, strlen(view_common));
    lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));

    lcb_HTTP_HANDLE *htreq;
    HttpCmdContext ctx;
    ctx.dumpIfError = true;
    lcb_cmdhttp_handle(cmd, &htreq);

    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, &ctx, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_wait(instance);

    ASSERT_EQ(true, ctx.received);
    ASSERT_EQ(LCB_SUCCESS, ctx.err);
    ASSERT_EQ(201, ctx.status); /* 201 Created */
    ASSERT_EQ(1, ctx.cbCount);
}

/**
 * @test HTTP (Get)
 * @pre Query a value view
 * @post HTTP Result is @c 200, and the view contents look like valid JSON
 * (i.e. the first non-whitespace char is a @c { and the last non-whitespace
 * char is a @c }
 */
TEST_F(HttpUnitTest, testGet)
{
    SKIP_IF_MOCK();

    HandleWrap hw;
    lcb_INSTANCE *instance;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)httpSimpleCallback);

    std::string view_path("/_design/" DESIGN_DOC_NAME "/_view/" VIEW_NAME);
    lcb_CMDHTTP *cmd;
    lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_VIEW);
    lcb_cmdhttp_path(cmd, view_path.c_str(), view_path.size());
    lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_GET);
    lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));

    lcb_HTTP_HANDLE *htreq;
    HttpCmdContext ctx;
    ctx.dumpIfEmpty = true;
    ctx.dumpIfError = true;
    lcb_cmdhttp_handle(cmd, &htreq);

    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, &ctx, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_wait(instance);

    ASSERT_EQ(true, ctx.received);
    ASSERT_EQ(200, ctx.status);
    ASSERT_GT(ctx.body.size(), 0U);
    ASSERT_EQ(ctx.cbCount, 1);

    unsigned ii;
    const char *pcur;

    for (ii = 0, pcur = ctx.body.c_str(); ii < ctx.body.size() && isspace(*pcur); ii++, pcur++) {
        /* no body */
    }

    /**
     * This is a view request. If all is in order, the content should be a
     * JSON object, first non-ws char is "{" and last non-ws char is "}"
     */
    ASSERT_NE(ctx.body.size(), ii);
    ASSERT_EQ(*pcur, '{');

    for (pcur = ctx.body.c_str() + ctx.body.size() - 1; ii >= 0 && isspace(*pcur); ii--, pcur--) {
        /* no body */
    }
    ASSERT_GE(ii, 0U);
    ASSERT_EQ('}', *pcur);
}

/**
 * @test HTTP (Connection Refused)
 * @bug CCBC-132
 * @pre Create a request of type RAW to @c localhost:1 - nothing should be
 * listening there
 * @post Command returns. Status code is one of CONNECT_ERROR or NETWORK_ERROR
 */
TEST_F(HttpUnitTest, testRefused)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)httpSimpleCallback);

    std::string path("non-exist-path");
    lcb_CMDHTTP *cmd;
    lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_RAW);
    lcb_cmdhttp_path(cmd, path.c_str(), path.size());
    const char *host = "localhost:1"; // should not have anything listening on it
    lcb_cmdhttp_host(cmd, host, strlen(host));
    lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_GET);
    lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));

    HttpCmdContext ctx;
    ctx.dumpIfEmpty = false;
    ctx.dumpIfError = false;
    lcb_HTTP_HANDLE *htreq;
    lcb_cmdhttp_handle(cmd, &htreq);

    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, &ctx, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_wait(instance);

    ASSERT_EQ(true, ctx.received);
    ASSERT_NE(0, LCB_EIFNET(ctx.err));
}

struct HtResult {
    std::string body;
    std::map< std::string, std::string > headers;

    bool gotComplete;
    bool gotChunked;
    lcb_STATUS rc;
    uint16_t http_status;

    void reset()
    {
        body.clear();
        gotComplete = false;
        gotChunked = false;
        rc = LCB_SUCCESS;
        http_status = 0;
    }
};

extern "C" {
static void http_callback(lcb_INSTANCE *, int, const lcb_RESPHTTP *resp)
{
    HtResult *me;
    lcb_resphttp_cookie(resp, (void **)&me);

    me->rc = lcb_resphttp_status(resp);
    lcb_resphttp_http_status(resp, &me->http_status);

    const char *body;
    size_t nbody;
    lcb_resphttp_body(resp, &body, &nbody);

    if (nbody) {
        me->body.append(body, body + nbody);
    }

    if (lcb_resphttp_is_final(resp)) {
        me->gotComplete = true;
        const char *const *cur = NULL;
        lcb_resphttp_headers(resp, &cur);
        for (; *cur; cur += 2) {
            me->headers[cur[0]] = cur[1];
        }
    } else {
        me->gotChunked = true;
    }
}
}

static void makeAdminReq(lcb_CMDHTTP **cmd, std::string &bkbuf)
{
    bkbuf.assign("/pools/default/buckets/default");

    lcb_cmdhttp_create(cmd, LCB_HTTP_TYPE_MANAGEMENT);
    lcb_cmdhttp_method(*cmd, LCB_HTTP_METHOD_GET);
    lcb_cmdhttp_path(*cmd, bkbuf.c_str(), bkbuf.size());
}

// Some more basic HTTP tests for the administrative API. We use the admin
// API since it's always available.
TEST_F(HttpUnitTest, testAdminApi)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    std::string pth;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)http_callback);

    // Make the request; this time we make it to the 'management' API
    lcb_CMDHTTP *cmd = NULL;

    makeAdminReq(&cmd, pth);
    HtResult htr;
    htr.reset();

    lcb_STATUS err;
    lcb_sched_enter(instance);
    err = lcb_http(instance, &htr, cmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_sched_leave(instance);
    lcb_wait(instance);

    ASSERT_TRUE(htr.gotComplete);
    ASSERT_EQ(LCB_SUCCESS, htr.rc);
    ASSERT_EQ(200, htr.http_status);
    ASSERT_FALSE(htr.body.empty());

    // Try with a chunked request
    htr.reset();
    lcb_cmdhttp_streaming(cmd, true);
    lcb_sched_enter(instance);
    err = lcb_http(instance, &htr, cmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_sched_leave(instance);
    lcb_wait(instance);

    ASSERT_TRUE(htr.gotComplete);
    ASSERT_TRUE(htr.gotChunked);

    // try another one, but this time cancelling it..
    lcb_HTTP_HANDLE *reqh;
    lcb_cmdhttp_handle(cmd, &reqh);
    lcb_sched_enter(instance);
    err = lcb_http(instance, NULL, cmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_FALSE(reqh == NULL);
    lcb_sched_leave(instance);
    lcb_http_cancel(instance, reqh);

    // Try another one, allocating a request body. Unfortunately, we need
    // to cancel this one too, as none of the mock's endpoints support a
    // request body
    lcb_cmdhttp_handle(cmd, &reqh);
    lcb_cmdhttp_body(cmd, "FOO", 3);
    lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_PUT);
    err = lcb_http(instance, NULL, cmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_FALSE(reqh == NULL);
    lcb_sched_leave(instance);
    lcb_http_cancel(instance, reqh);

    lcb_cmdhttp_destroy(cmd);
}

extern "C" {
static void doubleCancel_callback(lcb_INSTANCE *instance, int, const lcb_RESPHTTP *resp)
{
    if (lcb_resphttp_is_final(resp)) {
        lcb_HTTP_HANDLE *handle = NULL;
        lcb_resphttp_handle(resp, &handle);
        lcb_http_cancel(instance, handle);
        lcb_http_cancel(instance, handle);
    }
}
}

TEST_F(HttpUnitTest, testDoubleCancel)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)doubleCancel_callback);

    // Make the request; this time we make it to the 'management' API
    lcb_CMDHTTP *cmd = NULL;
    std::string bk;
    makeAdminReq(&cmd, bk);
    lcb_sched_enter(instance);
    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, NULL, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait(instance);
    // No crashes or errors here means we've done OK
}

extern "C" {
static void cancelVerify_callback(lcb_INSTANCE *instance, int, const lcb_RESPHTTP *resp)
{
    bool *bCancelled;
    lcb_resphttp_cookie(resp, (void **)&bCancelled);

    ASSERT_EQ(0, lcb_resphttp_is_final(resp));
    ASSERT_FALSE(*bCancelled);

    lcb_HTTP_HANDLE *handle;
    lcb_resphttp_handle(resp, &handle);
    lcb_http_cancel(instance, handle);
    *bCancelled = true;
}
}
// Ensure cancel actually does what it claims to do
TEST_F(HttpUnitTest, testCancelWorks)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    createConnection(hw, &instance);
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)cancelVerify_callback);
    lcb_CMDHTTP *cmd = NULL;
    std::string ss;
    makeAdminReq(&cmd, ss);
    // Make it chunked
    lcb_cmdhttp_streaming(cmd, true);
    bool cookie = false;
    lcb_sched_enter(instance);
    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, &cookie, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

extern "C" {
static void noInvoke_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *)
{
    EXPECT_FALSE(true) << "This callback should not be invoked!";
}
}
TEST_F(HttpUnitTest, testDestroyWithActiveRequest)
{
    lcb_INSTANCE *instance;
    // Note the one-arg form of createConnection which doesn't come with the
    // magical HandleWrap; this is because we destroy our instance explicitly
    // here.
    createConnection(&instance);

    lcb_CMDHTTP *cmd;
    std::string ss;
    makeAdminReq(&cmd, ss);

    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, noInvoke_callback);
    lcb_sched_enter(instance);
    ASSERT_EQ(LCB_SUCCESS, lcb_http(instance, NULL, cmd));
    lcb_cmdhttp_destroy(cmd);
    lcb_sched_leave(instance);
    lcb_destroy(instance);
}
