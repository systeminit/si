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
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include <mocksupport/server.h>
#include "mock-environment.h"
#include <sstream>
#include "internal.h" /* settings from lcb_INSTANCE *for logging */

#define LOGARGS(instance, lvl) instance->settings, "tests-ENV", LCB_LOG_##lvl, __FILE__, __LINE__

MockEnvironment *MockEnvironment::instance;

MockEnvironment *MockEnvironment::getInstance(void)
{
    if (instance == NULL) {
        instance = new MockEnvironment;
    }
    return instance;
}

void MockEnvironment::Reset()
{
    if (instance != NULL) {
        instance->TearDown();
        instance->SetUp();
    }
}

void MockEnvironment::init()
{
    mock = NULL;
    http = NULL;
    innerClient = NULL;
    argv = NULL;
    iops = NULL;

    numNodes = 4;
    realCluster = false;
    serverVersion = VERSION_UNKNOWN;
}

MockEnvironment::MockEnvironment()
{
    init();
}

MockEnvironment::MockEnvironment(const char **args, std::string bucketname)
{
    init();
    this->argv = args;
    this->bucketName = bucketname;
    SetUp();
}

void MockEnvironment::failoverNode(int index, std::string bucket, bool rebalance)
{
    MockBucketCommand bCmd(MockCommand::FAILOVER, index, bucket);
    bCmd.set("rebalance", rebalance);
    sendCommand(bCmd);
    getResponse();
}

void MockEnvironment::respawnNode(int index, std::string bucket)
{
    MockBucketCommand bCmd(MockCommand::RESPAWN, index, bucket);
    sendCommand(bCmd);
    getResponse();
}

void MockEnvironment::hiccupNodes(int msecs, int offset)
{
    MockCommand cmd(MockCommand::HICCUP);
    cmd.set("msecs", msecs);
    cmd.set("offset", offset);
    sendCommand(cmd);
    getResponse();
}

void MockEnvironment::regenVbCoords(std::string bucket)
{
    MockBucketCommand bCmd(MockCommand::REGEN_VBCOORDS, 0, bucket);
    MockResponse r;
    sendCommand(bCmd);
    getResponse(r);
    EXPECT_TRUE(r.isOk());
}

std::vector< int > MockEnvironment::getMcPorts(std::string bucket)
{
    MockCommand cmd(MockCommand::GET_MCPORTS);
    if (!bucket.empty()) {
        cmd.set("bucket", bucket);
    }

    sendCommand(cmd);
    MockResponse resp;
    getResponse(resp);
    EXPECT_TRUE(resp.isOk());
    const Json::Value &payload = resp.constResp()["payload"];

    std::vector< int > ret;

    for (int ii = 0; ii < (int)payload.size(); ii++) {
        ret.push_back(payload[ii].asInt());
    }
    return ret;
}

void MockEnvironment::setSaslMechs(std::vector< std::string > &mechanisms, std::string bucket,
                                   const std::vector< int > *nodes)
{
    MockCommand cmd(MockCommand::SET_SASL_MECHANISMS);
    Json::Value mechs(Json::arrayValue);
    for (std::vector< std::string >::const_iterator ii = mechanisms.begin(); ii != mechanisms.end(); ii++) {
        mechs.append(*ii);
    }
    cmd.set("mechs", mechs);

    if (!bucket.empty()) {
        cmd.set("bucket", bucket);
    }

    if (nodes != NULL) {
        const std::vector< int > &v = *nodes;
        Json::Value array(Json::arrayValue);

        for (std::vector< int >::const_iterator ii = v.begin(); ii != v.end(); ii++) {
            array.append(*ii);
        }

        cmd.set("servers", array);
    }

    sendCommand(cmd);
    getResponse();
}

void MockEnvironment::setCCCP(bool enabled, std::string bucket, const std::vector< int > *nodes)
{
    MockCommand cmd(MockCommand::SET_CCCP);
    cmd.set("enabled", enabled);

    if (!bucket.empty()) {
        cmd.set("bucket", bucket);
    }

    if (nodes != NULL) {
        const std::vector< int > &v = *nodes;
        Json::Value array(Json::arrayValue);

        for (std::vector< int >::const_iterator ii = v.begin(); ii != v.end(); ii++) {
            array.append(*ii);
        }

        cmd.set("servers", array);
    }

    sendCommand(cmd);
    getResponse();
}

void MockEnvironment::setEnhancedErrors(bool enabled, std::string bucket, const std::vector< int > *nodes)
{
    MockCommand cmd(MockCommand::SET_ENHANCED_ERRORS);
    cmd.set("enabled", enabled);

    if (!bucket.empty()) {
        cmd.set("bucket", bucket);
    }

    if (nodes != NULL) {
        const std::vector< int > &v = *nodes;
        Json::Value array(Json::arrayValue);

        for (std::vector< int >::const_iterator ii = v.begin(); ii != v.end(); ii++) {
            array.append(*ii);
        }

        cmd.set("servers", array);
    }

    sendCommand(cmd);
    getResponse();
}

void MockEnvironment::setCompression(std::string mode, std::string bucket, const std::vector< int > *nodes)
{
    MockCommand cmd(MockCommand::SET_COMPRESSION);
    cmd.set("mode", mode);

    if (!bucket.empty()) {
        cmd.set("bucket", bucket);
    }

    if (nodes != NULL) {
        const std::vector< int > &v = *nodes;
        Json::Value array(Json::arrayValue);

        for (std::vector< int >::const_iterator ii = v.begin(); ii != v.end(); ii++) {
            array.append(*ii);
        }

        cmd.set("servers", array);
    }

    sendCommand(cmd);
    getResponse();
}

const Json::Value MockEnvironment::getKeyInfo(std::string key, std::string bucket)
{
    MockKeyCommand cmd(MockCommand::KEYINFO, key);
    cmd.bucket = bucket;
    sendCommand(cmd);
    MockResponse resp;
    getResponse(resp);
    return resp.constResp()["payload"];
}

void MockEnvironment::sendCommand(MockCommand &cmd)
{
    std::string s = cmd.encode();
    lcb_ssize_t nw = send(mock->client, s.c_str(), (unsigned long)s.size(), 0);
    assert(nw == s.size());
}

void MockEnvironment::getResponse(MockResponse &ret)
{
    std::string rbuf;
    do {
        char c;
        int rv = recv(mock->client, &c, 1, 0);
        assert(rv == 1);
        if (c == '\n') {
            break;
        }
        rbuf += c;
    } while (true);

    ret.assign(rbuf);
    if (!ret.isOk()) {
        std::cerr << "Mock command failed!" << std::endl;
        std::cerr << ret.constResp()["error"].asString() << std::endl;
        std::cerr << ret;
    }
}

void MockEnvironment::postCreate(lcb_INSTANCE *instance)
{
    lcb_STATUS err;
    if (!isRealCluster()) {
        lcb_HTCONFIG_URLTYPE urltype = LCB_HTCONFIG_URLTYPE_COMPAT;
        err = lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_HTCONFIG_URLTYPE, &urltype);
        ASSERT_EQ(LCB_SUCCESS, err);
    }
    err = lcb_cntl_string(instance, "fetch_mutation_tokens", "true");
    ASSERT_EQ(LCB_SUCCESS, err);
}

void MockEnvironment::createConnection(HandleWrap &handle, lcb_INSTANCE **instance, const lcb_create_st &user_options)
{
    lcb_io_opt_t io;
    lcb_create_st options;
    memcpy(&options, &user_options, sizeof user_options);

    if (lcb_create_io_ops(&io, NULL) != LCB_SUCCESS) {
        fprintf(stderr, "Failed to create IO instance\n");
        exit(1);
    }

    options.v.v2.io = io;
    lcb_STATUS err = lcb_create(instance, &options);
    ASSERT_EQ(LCB_SUCCESS, err);
    postCreate(*instance);

    (void)lcb_set_cookie(*instance, io);

    handle.instance = *instance;
    handle.iops = io;
}

void MockEnvironment::createConnection(HandleWrap &handle, lcb_INSTANCE **instance)
{
    lcb_create_st options = {};
    makeConnectParams(options, NULL);
    createConnection(handle, instance, options);
}

void MockEnvironment::createConnection(lcb_INSTANCE **instance)
{
    HandleWrap handle;
    createConnection(handle, instance);

    handle.iops->v.base.need_cleanup = 1;
    handle.instance = NULL;
    handle.iops = NULL;
}

#define STAT_VERSION "version"

extern "C" {
static void statsCallback(lcb_INSTANCE *instance, lcb_CALLBACK_TYPE, const lcb_RESPSTATS *resp)
{
    MockEnvironment *me = (MockEnvironment *)resp->cookie;
    ASSERT_EQ(LCB_SUCCESS, resp->rc);

    if (resp->server == NULL) {
        return;
    }

    if (!resp->nkey) {
        return;
    }

    if (resp->nkey != sizeof(STAT_VERSION) - 1 || memcmp(resp->key, STAT_VERSION, sizeof(STAT_VERSION) - 1) != 0) {
        return;
    }
    MockEnvironment::ServerVersion version = MockEnvironment::VERSION_UNKNOWN;
    if (resp->nvalue > 2) {
        int major = ((const char *)resp->value)[0] - '0';
        int minor = ((const char *)resp->value)[2] - '0';
        switch (major) {
            case 4:
                switch (minor) {
                    case 0:
                        version = MockEnvironment::VERSION_40;
                        break;
                    case 1:
                        version = MockEnvironment::VERSION_41;
                        break;
                    case 5:
                        version = MockEnvironment::VERSION_45;
                        break;
                    case 6:
                        version = MockEnvironment::VERSION_46;
                        break;
                }
                break;
            case 5:
            case 6:
                version = MockEnvironment::VERSION_50;
                break;
        }
    }
    if (version == MockEnvironment::VERSION_UNKNOWN) {
        lcb_log(LOGARGS(instance, ERROR), "Unable to determine version from string '%.*s', assuming 4.0",
                (int)resp->nvalue, (const char *)resp->value);
        version = MockEnvironment::VERSION_40;
    }
    me->setServerVersion(version);
    lcb_log(LOGARGS(instance, INFO), "Using real cluster version %.*s (id=%d)", (int)resp->nvalue,
            (const char *)resp->value, version);
}
}

void MockEnvironment::bootstrapRealCluster()
{
    serverParams = ServerParams(mock->http, mock->bucket, mock->username, mock->password);

    lcb_INSTANCE *tmphandle;
    lcb_STATUS err;
    lcb_create_st options;
    serverParams.makeConnectParams(options, NULL);

    ASSERT_EQ(LCB_SUCCESS, lcb_create(&tmphandle, &options));
    postCreate(tmphandle);
    ASSERT_EQ(LCB_SUCCESS, lcb_connect(tmphandle));
    lcb_wait(tmphandle);

    lcb_install_callback3(tmphandle, LCB_CALLBACK_STATS, (lcb_RESPCALLBACK)statsCallback);
    lcb_CMDSTATS scmd = {0};
    err = lcb_stats3(tmphandle, this, &scmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_wait(tmphandle);

    const char *const *servers = lcb_get_server_list(tmphandle);
    int ii;
    for (ii = 0; servers[ii] != NULL; ii++) {
        // no body
    }

    featureRegistry.insert("observe");
    featureRegistry.insert("views");
    featureRegistry.insert("http");
    featureRegistry.insert("replica_read");
    featureRegistry.insert("lock");

    numNodes = ii;
    lcb_destroy(tmphandle);
}

extern "C" {
static void mock_flush_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *resp)
{
    ASSERT_EQ(LCB_SUCCESS, resp->rc);
}
}

void MockEnvironment::clearAndReset()
{
    if (is_using_real_cluster()) {
        return;
    }

    for (int ii = 0; ii < getNumNodes(); ii++) {
        respawnNode(ii, bucketName);
    }

    std::vector< int > mcPorts = getMcPorts(bucketName);
    serverParams.setMcPorts(mcPorts);
    setCCCP(true, bucketName);

    if (this != getInstance()) {
        return;
    }

    if (!innerClient) {
        lcb_create_st crParams;
        lcb_config_transport_t transports[] = {LCB_CONFIG_TRANSPORT_CCCP, LCB_CONFIG_TRANSPORT_LIST_END};
        memset(&crParams, 0, sizeof(crParams));
        // Use default I/O here..
        serverParams.makeConnectParams(crParams, NULL);
        crParams.v.v2.transports = transports;
        lcb_STATUS err = lcb_create(&innerClient, &crParams);
        if (err != LCB_SUCCESS) {
            printf("Error on create: 0x%x\n", err);
        }
        EXPECT_FALSE(NULL == innerClient);
        postCreate(innerClient);
        err = lcb_connect(innerClient);
        EXPECT_EQ(LCB_SUCCESS, err);
        lcb_wait(innerClient);
        lcb_install_callback3(innerClient, LCB_CALLBACK_CBFLUSH, mock_flush_callback);
    }

    lcb_CMDCBFLUSH fcmd = {0};
    lcb_STATUS err;

    err = lcb_cbflush3(innerClient, NULL, &fcmd);
    ASSERT_EQ(LCB_SUCCESS, err);
    lcb_wait(innerClient);
}

void MockEnvironment::SetUp()
{
    numNodes = 4;
    if (!mock) {
        mock = (struct test_server_info *)start_test_server((char **)argv);
    }

    realCluster = is_using_real_cluster() != 0;
    ASSERT_NE((const void *)(NULL), mock);
    http = get_mock_http_server(mock);
    ASSERT_NE((const char *)(NULL), http);

    if (realCluster) {
        bootstrapRealCluster();
        return;
    }

    if (bucketName.empty()) {
        const char *name = getenv("LCB_TEST_BUCKET");
        if (name != NULL) {
            bucketName = name;
        } else {
            bucketName = "default";
        }
    }
    serverParams = ServerParams(http, bucketName.c_str(), userName.c_str(), NULL);

    // Mock 0.6
    featureRegistry.insert("observe");
    featureRegistry.insert("views");
    featureRegistry.insert("replica_read");
    featureRegistry.insert("lock");

    clearAndReset();
}

void MockEnvironment::TearDown() {}

MockEnvironment::~MockEnvironment()
{
    shutdown_mock_server(mock);
    mock = NULL;
    if (innerClient != NULL) {
        lcb_destroy(innerClient);
        innerClient = NULL;
    }
}

void HandleWrap::destroy()
{
    if (instance) {
        lcb_destroy(instance);
    }
    if (iops) {
        lcb_destroy_io_ops(iops);
    }

    instance = NULL;
    iops = NULL;
}

HandleWrap::~HandleWrap()
{
    destroy();
}

MockCommand::MockCommand(Code code)
{
    this->code = code;
    name = GetName(code);
    command["command"] = name;
    payload = &(command["payload"] = Json::Value(Json::objectValue));
}

MockCommand::~MockCommand() {}

std::string MockCommand::encode()
{
    finalizePayload();
    return Json::FastWriter().write(command);
}

void MockKeyCommand::finalizePayload()
{
    MockCommand::finalizePayload();
    if (vbucket != -1) {
        set("vBucket", vbucket);
    }

    if (!bucket.empty()) {
        set("Bucket", bucket);
    }
    set("Key", key);
}

void MockMutationCommand::finalizePayload()
{
    MockKeyCommand::finalizePayload();
    set("OnMaster", onMaster);

    if (!replicaList.empty()) {
        Json::Value arr(Json::arrayValue);
        Json::Value &arrval = (*payload)["OnReplicas"] = Json::Value(Json::arrayValue);
        for (std::vector< int >::iterator ii = replicaList.begin(); ii != replicaList.end(); ii++) {
            arrval.append(*ii);
        }
    } else {
        set("OnReplicas", replicaCount);
    }

    if (cas != 0) {
        if (cas > (1LU << 30)) {
            fprintf(stderr, "Detected incompatible > 31 bit integer\n");
            abort();
        }
        set("CAS", static_cast< Json::UInt64 >(cas));
    }

    if (!value.empty()) {
        set("Value", value);
    }
}

void MockBucketCommand::finalizePayload()
{
    MockCommand::finalizePayload();
    set("idx", ix);
    set("bucket", bucket);
}

void MockResponse::assign(const std::string &resp)
{
    bool rv = Json::Reader().parse(resp, jresp);
    assert(rv);
}

MockResponse::~MockResponse() {}

std::ostream &operator<<(std::ostream &os, const MockResponse &resp)
{
    os << Json::FastWriter().write(resp.jresp) << std::endl;
    return os;
}

bool MockResponse::isOk()
{
    const Json::Value &status = static_cast< const Json::Value & >(jresp)["status"];
    if (!status.isString()) {
        return false;
    }
    return tolower(status.asString()[0]) == 'o';
}
