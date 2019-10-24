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

#include <gtest/gtest.h>
#include "connspec.h"
using namespace lcb;

static size_t countHosts(const Connspec *spec)
{
    return spec->hosts().size();
}

class ConnstrTest : public ::testing::Test
{
  protected:
    Connspec params;
    const char *errmsg;
    void reinit()
    {
        params = Connspec();
    }
    void SetUp()
    {
        params = Connspec();
    }
};

const Spechost *findHost(const Connspec &params, const char *srch)
{
    for (size_t ii = 0; ii < params.hosts().size(); ++ii) {
        const Spechost *cur = &params.hosts()[ii];
        if (srch == cur->hostname) {
            return cur;
        }
    }
    return NULL;
}
const Spechost *findHost(const Connspec *params, const char *srch)
{
    return findHost(*params, srch);
}

struct OptionPair {
    std::string key;
    std::string value;
};

bool findOption(const Connspec &params, const char *srch, OptionPair &op)
{
    int iter = 0;
    Connspec::Options::const_iterator ii = params.options().begin();
    for (; ii != params.options().end(); ++ii) {
        if (ii->first == srch) {
            op.key = ii->first;
            op.value = ii->second;
            return true;
        }
    }
    return false;
}

bool findOption(const Connspec *params, const char *srch, OptionPair &op)
{
    return findOption(*params, srch, op);
}

size_t countHosts(const Connspec &params)
{
    return params.hosts().size();
}

TEST_F(ConnstrTest, testParseBasic)
{
    lcb_STATUS err;
    err = params.parse("couchbase://1.2.3.4", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);

    const Spechost *tmphost;
    tmphost = findHost(params, "1.2.3.4");
    ASSERT_EQ(1, countHosts(params));
    ASSERT_FALSE(NULL == tmphost);
    ASSERT_EQ(0, tmphost->port);
    ASSERT_EQ(0, tmphost->type); // Nothing

    reinit();
    // test with bad scheme
    err = params.parse("blah://foo.com", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err) << "Error on bad scheme";

    reinit();
    err = params.parse("couchbase://", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with scheme only";

    reinit();
    err = params.parse("couchbase://?", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with only '?'";

    reinit();
    err = params.parse("couchbase://?&", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with only '?&'";

    reinit();
    err = params.parse("1.2.3.4", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok without scheme";
    ASSERT_EQ(LCB_CONFIG_HTTP_PORT, params.default_port());

    reinit();
    err = params.parse("1.2.3.4:999", &errmsg);
    ASSERT_EQ(1, countHosts(&params));
    tmphost = findHost(&params, "1.2.3.4");
    ASSERT_FALSE(tmphost == NULL);
    ASSERT_EQ(999, tmphost->port);
    ASSERT_TRUE(tmphost->isHTTP());
}

TEST_F(ConnstrTest, testParseHosts)
{
    lcb_STATUS err;
    err = params.parse("couchbase://foo.com,bar.com,baz.com", &errmsg);
    ASSERT_EQ(3, countHosts(&params));
    ASSERT_FALSE(NULL == findHost(&params, "foo.com"));
    ASSERT_FALSE(NULL == findHost(&params, "bar.com"));
    ASSERT_FALSE(NULL == findHost(&params, "baz.com"));

    // Parse with 'legacy' format
    reinit();
    err = params.parse("couchbase://foo.com:8091", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    const Spechost *dh = findHost(&params, "foo.com");
    ASSERT_FALSE(NULL == dh);
    ASSERT_EQ("foo.com", dh->hostname);
    // CCBC-599
    ASSERT_EQ(0, dh->port);
    ASSERT_EQ(0, dh->type);

    // parse with invalid port, without specifying protocol
    reinit();
    err = params.parse("couchbase://foo.com:4444", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    dh = findHost(&params, "foo.com");
    ASSERT_EQ(4444, dh->port);
    ASSERT_TRUE(dh->isMCD());

    reinit();
    err = params.parse("couchbases://foo.com:4444", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    dh = findHost(&params, "foo.com");
    ASSERT_EQ(LCB_SSL_ENABLED, params.sslopts());
    ASSERT_EQ(4444, dh->port);
    ASSERT_TRUE(dh->isMCDS());

    // Parse with recognized format
    reinit();
    err = params.parse("couchbase://foo.com:4444=mcd", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    dh = findHost(&params, "foo.com");
    ASSERT_EQ("foo.com", dh->hostname);
    ASSERT_EQ(4444, dh->port);
    ASSERT_TRUE(dh->isMCD());

    // Parse multiple hosts with ports
    reinit();
    err = params.parse("couchbase://foo.com:4444=mcd,bar.com:5555=mcd", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);

    dh = findHost(&params, "foo.com");
    ASSERT_FALSE(dh == NULL);
    ASSERT_EQ("foo.com", dh->hostname);
    ASSERT_EQ(4444, dh->port);
    ASSERT_TRUE(dh->isMCD());

    dh = findHost(&params, "bar.com");
    ASSERT_FALSE(dh == NULL);
    ASSERT_EQ("bar.com", dh->hostname);
    ASSERT_EQ(5555, dh->port);
    ASSERT_TRUE(dh->isMCD());

    reinit();
    err = params.parse("couchbase://foo.com,bar.com:4444", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    dh = findHost(&params, "bar.com");
    ASSERT_EQ(4444, dh->port);
    ASSERT_TRUE(dh->isMCD());
    dh = findHost(&params, "foo.com");
    ASSERT_TRUE(dh->isTypeless());

    reinit();
    err = params.parse("couchbase://foo.com;bar.com;baz.com", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Can parse old-style semicolons";
    ASSERT_EQ(3, countHosts(&params));
    ASSERT_FALSE(NULL == findHost(&params, "foo.com"));
    ASSERT_FALSE(NULL == findHost(&params, "bar.com"));
    ASSERT_FALSE(NULL == findHost(&params, "baz.com"));

    reinit();
    err = params.parse("couchbase://"
                       "::a15:f2df:3fef:51bb:212a:8cec,[::a15:f2df:3fef:51bb:212a:8ced],[::a15:f2df:3fef:51bb:212a:"
                       "8cee]:9001",
                       &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Cannot parse IPv6";
    ASSERT_EQ(3, countHosts(&params));
    ASSERT_FALSE(NULL == findHost(&params, "::a15:f2df:3fef:51bb:212a:8cec"));
    ASSERT_FALSE(NULL == findHost(&params, "::a15:f2df:3fef:51bb:212a:8ced"));
    dh = findHost(&params, "::a15:f2df:3fef:51bb:212a:8cee");
    ASSERT_FALSE(dh == NULL);
    ASSERT_EQ("::a15:f2df:3fef:51bb:212a:8cee", dh->hostname);
    ASSERT_EQ(9001, dh->port);
}

TEST_F(ConnstrTest, testParseBucket)
{
    lcb_STATUS err;
    err = params.parse("couchbase://foo.com/user", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ("user", params.bucket()) << "Basic bucket parse";

    reinit();
    err = params.parse("couchbase://foo.com/user/", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Bucket can have a slash";
    // We can have a bucket using a slash

    reinit();
    err = params.parse("couchbase:///default", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Bucket without host OK";
    ASSERT_EQ("default", params.bucket());

    reinit();
    err = params.parse("couchbase:///default?", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ("default", params.bucket());

    reinit();
    err = params.parse("couchbase:///%2FUsers%2F?", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ("/Users/", params.bucket());
}

TEST_F(ConnstrTest, testOptionsPassthrough)
{
    lcb_STATUS err;
    err = params.parse("couchbase://?foo=bar", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Options only";
    ASSERT_FALSE(params.options().empty());
    ASSERT_NE(0, params.options().size());

    OptionPair op;
    ASSERT_TRUE(findOption(&params, "foo", op));
    ASSERT_EQ("bar", op.value);

    reinit();
    err = params.parse("couchbase://?foo=bar", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_TRUE(findOption(&params, "foo", op));
    ASSERT_EQ("bar", op.value);

    reinit();
    err = params.parse("couchbase://?foo", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err) << "Option without value";

    // Multiple options
    reinit();
    err = params.parse("couchbase://?foo=fooval&bar=barval", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_TRUE(findOption(&params, "foo", op));
    ASSERT_EQ("fooval", op.value);

    ASSERT_TRUE(findOption(&params, "bar", op));
    ASSERT_EQ("barval", op.value);

    reinit();
    err = params.parse("couchbase:///protected?ssl=on&compression=off", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with bucket and no hosts";
    ASSERT_EQ(1, countHosts(&params));
    ASSERT_FALSE(NULL == findHost(&params, "localhost"));
    ASSERT_TRUE(findOption(&params, "compression", op));

    reinit();
    err = params.parse("couchbase://?foo=foo&bar=bar&", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with trailing '&'";

    reinit();
    err = params.parse("couchbase://?foo=foo&bootstrap_on=all&bar=bar", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "Ok with non-passthrough option";
    ASSERT_TRUE(findOption(&params, "foo", op));
    ASSERT_TRUE(findOption(&params, "bar", op));
    ASSERT_FALSE(findOption(&params, "bootstrap_on", op));
}

TEST_F(ConnstrTest, testRecognizedOptions)
{
    lcb_STATUS err;
    err = params.parse("couchbases://", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_SSL_ENABLED, params.sslopts());

    reinit();
    err = params.parse("couchbase://?ssl=on", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_SSL_ENABLED, params.sslopts());

    reinit();
    err = params.parse("couchbases://?ssl=no_verify", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_SSL_ENABLED | LCB_SSL_NOVERIFY, params.sslopts());

    reinit();
    err = params.parse("couchbases://?ssl=off", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err);

    // Loglevel
    reinit();
    err = params.parse("couchbase://?console_log_level=5", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(5, params.loglevel());

    reinit();
    err = params.parse("couchbase://?console_log_level=gah", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err);
}

TEST_F(ConnstrTest, testTransportOptions)
{
    lcb_STATUS err;
    err = params.parse("couchbase://", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_FALSE(params.is_bs_udef());

    reinit();
    err = params.parse("couchbase://?bootstrap_on=cccp", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "bootstrap_on=cccp";
    ASSERT_TRUE(params.has_bsmode(LCB_CONFIG_TRANSPORT_CCCP));
    ASSERT_FALSE(params.has_bsmode(LCB_CONFIG_TRANSPORT_HTTP));

    reinit();
    err = params.parse("couchbase://?bootstrap_on=http", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "bootstrap_on=http";
    ASSERT_TRUE(params.has_bsmode(LCB_CONFIG_TRANSPORT_HTTP));
    ASSERT_FALSE(params.has_bsmode(LCB_CONFIG_TRANSPORT_CCCP));

    reinit();
    err = params.parse("couchbase://?bootstrap_on=all", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err) << "bootstrap_on=all";
    ASSERT_TRUE(params.has_bsmode(LCB_CONFIG_TRANSPORT_CCCP));
    ASSERT_TRUE(params.has_bsmode(LCB_CONFIG_TRANSPORT_HTTP));

    reinit();
    err = params.parse("couchbase://?bootstrap_on=bleh", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err) << "Error on bad bootstrap_on value";
}

TEST_F(ConnstrTest, testCompatConversion)
{
    lcb_STATUS err;
    struct lcb_create_st cropts;
    memset(&cropts, 0, sizeof cropts);
    cropts.version = 0;
    cropts.v.v0.bucket = "users";
    cropts.v.v0.host = "foo.com;bar.com;baz.com";
    cropts.v.v0.passwd = "secret";

    err = params.load(cropts);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_FALSE(NULL == findHost(&params, "foo.com"));
    ASSERT_FALSE(NULL == findHost(&params, "bar.com"));
    ASSERT_FALSE(NULL == findHost(&params, "baz.com"));
    ASSERT_EQ(3, countHosts(&params));
    ASSERT_EQ("users", params.bucket());
    ASSERT_EQ("secret", params.password());

    // Ensure old-style port specifications are parsed and don't throw an
    // error. We'd also like to verify that these actually land within
    // the htport field, that's a TODO
    reinit();
    memset(&cropts, 0, sizeof cropts);
    cropts.version = 2;
    cropts.v.v2.host = "foo.com:9030;bar.com:9040;baz.com:9050";
    cropts.v.v2.mchosts = "foo.com:7030;bar.com:7040;baz.com:7050";
    err = params.load(cropts);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(6, countHosts(&params));

    // Ensure struct fields override the URI string
    reinit();
    memset(&cropts, 0, sizeof cropts);
    cropts.version = 3;
    cropts.v.v3.passwd = "secret";
    cropts.v.v3.connstr = "couchbase:///fluffle?password=bleh";
    err = params.load(cropts);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ("fluffle", params.bucket());
    ASSERT_EQ(cropts.v.v3.passwd, params.password());
}

TEST_F(ConnstrTest, testCertificateWithoutSSL)
{
    // Ensure we get an invalid input error for certificate paths without
    // couchbases://
    lcb_STATUS err;
    err = params.parse("couchbase://1.2.3.4/default?certpath=/foo/bar/baz", &errmsg);
    ASSERT_NE(LCB_SUCCESS, err);

    reinit();
    err = params.parse("couchbases://1.2.3.4/default?certpath=/foo/bar/baz", &errmsg);
    ASSERT_EQ(LCB_SUCCESS, err);
}

TEST_F(ConnstrTest, testDnsSrvExplicit)
{
    // Test various things relating to DNS SRV
    lcb_STATUS err;
    err = params.parse("couchbase+dnssrv://1.1.1.1", &errmsg);
    EXPECT_EQ(LCB_SUCCESS, err);
    EXPECT_TRUE(params.can_dnssrv());
    EXPECT_TRUE(params.is_explicit_dnssrv());

    reinit();
    err = params.parse("couchbase+dnssrv://1.1.1.1,2.2.2.2", &errmsg);
    EXPECT_NE(LCB_SUCCESS, err);

    reinit();
    err = params.parse("couchbases+dnssrv://1.1.1.1", &errmsg);
    EXPECT_EQ(LCB_SUCCESS, err);
    EXPECT_NE(0, params.sslopts());
    EXPECT_TRUE(params.can_dnssrv());
    EXPECT_TRUE(params.is_explicit_dnssrv());
}

TEST_F(ConnstrTest, testDnsSrvImplicit)
{
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbase://"));
    EXPECT_FALSE(params.can_dnssrv());
    EXPECT_FALSE(params.is_explicit_dnssrv());

    reinit();
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbase://1.1.1.1"));
    EXPECT_TRUE(params.can_dnssrv());
    EXPECT_FALSE(params.is_explicit_dnssrv());

    reinit();
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbase://1.1.1.1,2.2.2.2"));
    EXPECT_FALSE(params.can_dnssrv()) << "No implicit SRV on multiple hosts";

    reinit();
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbase://1.1.1.1:666"));
    EXPECT_FALSE(params.can_dnssrv());

    reinit();
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbase://1.1.1.1:11210"));
    EXPECT_TRUE(params.can_dnssrv());

    reinit();
    EXPECT_EQ(LCB_SUCCESS, params.parse("couchbases://1.1.1.1"));
    EXPECT_TRUE(params.can_dnssrv());
}
