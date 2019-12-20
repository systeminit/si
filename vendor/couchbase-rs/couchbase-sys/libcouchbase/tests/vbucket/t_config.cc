#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include <gtest/gtest.h>
#include <sstream>
#include <iostream>
#include <fstream>
#include <vector>
#include <map>
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"

using std::map;
using std::string;
using std::vector;

static string getConfigFile(const char *fname)
{
    // Determine where the file is located?
    string base = "";
    const char *prefix;

    // Locate source directory
    if ((prefix = getenv("CMAKE_CURRENT_SOURCE_DIR"))) {
        base = prefix;
    } else if ((prefix = getenv("srcdir"))) {
        base = prefix;
    } else {
        base = "./../";
    }
    base += "/tests/vbucket/confdata/";
    base += fname;

    // Open the file
    std::ifstream ifs;
    ifs.open(base.c_str());
    if (!ifs.is_open()) {
        std::cerr << "Couldn't open " << base << std::endl;
    }
    EXPECT_TRUE(ifs.is_open());
    std::stringstream ss;
    ss << ifs.rdbuf();
    return ss.str();
}

class ConfigTest : public ::testing::Test
{
  protected:
    void testConfig(const char *fname, bool checkNew = false);
};

void ConfigTest::testConfig(const char *fname, bool checkNew)
{
    string testData = getConfigFile(fname);
    lcbvb_CONFIG *vbc = lcbvb_create();
    ASSERT_TRUE(vbc != NULL);
    int rv = lcbvb_load_json(vbc, testData.c_str());
    ASSERT_EQ(0, rv);
    ASSERT_GT(vbc->nsrv, (unsigned int)0);

    if (vbc->dtype == LCBVB_DIST_VBUCKET) {
        ASSERT_GT(vbc->nvb, (unsigned int)0);

        for (unsigned ii = 0; ii < vbc->nvb; ii++) {
            lcbvb_vbmaster(vbc, ii);
            for (unsigned jj = 0; jj < vbc->nrepl; jj++) {
                lcbvb_vbreplica(vbc, ii, jj);
            }
        }
    }

    for (unsigned ii = 0; ii < vbc->nsrv; ++ii) {
        lcbvb_SERVER *srv = LCBVB_GET_SERVER(vbc, ii);
        ASSERT_TRUE(srv->authority != NULL);
        ASSERT_TRUE(srv->hostname != NULL);
        ASSERT_GT(srv->svc.data, 0);
        ASSERT_GT(srv->svc.mgmt, 0);
        if (vbc->dtype == LCBVB_DIST_VBUCKET) {
            ASSERT_GT(srv->svc.views, 0);
            if (checkNew) {
                ASSERT_GT(srv->svc_ssl.views, 0);
            }
        }
        if (checkNew) {
            ASSERT_GT(srv->svc_ssl.data, 0);
            ASSERT_GT(srv->svc_ssl.mgmt, 0);
        }
    }
    if (checkNew) {
        ASSERT_FALSE(NULL == vbc->buuid);
        ASSERT_GT(vbc->revid, -1);
    }

    const char *k = "Hello";
    size_t nk = strlen(k);
    // map the key
    int srvix, vbid;
    if (vbc->dtype == LCBVB_DIST_KETAMA) {
        if (testData.find("$HOST") != string::npos) {
            ASSERT_TRUE(vbc->continuum == NULL);
            ASSERT_EQ(0, vbc->ncontinuum);
            lcbvb_replace_host(vbc, "localhost");
        }

        ASSERT_TRUE(vbc->continuum != NULL);
        ASSERT_EQ(160 * vbc->nsrv, vbc->ncontinuum);
        lcbvb_map_key(vbc, k, nk, &vbid, &srvix);
        ASSERT_EQ(0, vbid);
    } else {
        lcbvb_map_key(vbc, k, nk, &vbid, &srvix);
        ASSERT_NE(0, vbid);
    }
    lcbvb_destroy(vbc);
}

TEST_F(ConfigTest, testBasicConfigs)
{
    testConfig("full_25.json");
    testConfig("terse_25.json");
    testConfig("memd_25.json");
    testConfig("terse_30.json", true);
    testConfig("memd_30.json", true);
}

TEST_F(ConfigTest, testGeneration)
{
    lcbvb_CONFIG *cfg = lcbvb_create();
    lcbvb_genconfig(cfg, 4, 1, 1024);
    char *js = lcbvb_save_json(cfg);
    lcbvb_destroy(cfg);

    cfg = lcbvb_create();
    int rv = lcbvb_load_json(cfg, js);
    ASSERT_EQ(0, rv);
    ASSERT_EQ(4, cfg->nsrv);
    ASSERT_EQ(1, cfg->nrepl);
    ASSERT_EQ(LCBVB_DIST_VBUCKET, cfg->dtype);
    ASSERT_EQ(1024, cfg->nvb);
    lcbvb_destroy(cfg);
    free(js);
}

TEST_F(ConfigTest, testAltMap)
{
    lcbvb_CONFIG *cfg = lcbvb_create();
    lcbvb_genconfig(cfg, 4, 1, 64);
    lcbvb_genffmap(cfg);

    string key("Dummy Key");
    int vbix = lcbvb_k2vb(cfg, key.c_str(), key.size());
    int master = lcbvb_vbmaster(cfg, vbix);
    int oldmaster = master;

    int altix = lcbvb_nmv_remap(cfg, vbix, master);
    ASSERT_GT(altix, -1) << "Alternative index > -1";
    ASSERT_NE(altix, master) << "NMV Remap works with correct master";

    master = altix;
    altix = lcbvb_nmv_remap(cfg, vbix, oldmaster);
    ASSERT_EQ(master, altix) << "NMV Remap doesn't do anything with old master";
    lcbvb_destroy(cfg);
}

TEST_F(ConfigTest, testGetReplicaNode)
{
    lcbvb_CONFIG *cfg = lcbvb_create();
    lcbvb_genconfig(cfg, 4, 1, 2);

    // Select a random vbucket
    int srvix = cfg->vbuckets[0].servers[0];
    ASSERT_NE(-1, srvix);
    int rv = lcbvb_vbmaster(cfg, 0);
    ASSERT_EQ(srvix, rv);

    srvix = cfg->vbuckets[0].servers[1];
    ASSERT_NE(-1, srvix);
    rv = lcbvb_vbreplica(cfg, 0, 0);
    ASSERT_EQ(srvix, rv);

    rv = lcbvb_vbreplica(cfg, 0, 1);
    ASSERT_EQ(-1, rv);

    rv = lcbvb_vbreplica(cfg, 0, 9999);
    ASSERT_EQ(-1, rv);
    lcbvb_destroy(cfg);

    cfg = lcbvb_create();
    lcbvb_genconfig(cfg, 1, 0, 2);
    rv = lcbvb_vbmaster(cfg, 0);
    ASSERT_NE(-1, rv);
    rv = lcbvb_vbreplica(cfg, 0, 0);
    ASSERT_EQ(-1, rv);
    lcbvb_destroy(cfg);
}

TEST_F(ConfigTest, testBadInput)
{
    lcbvb_CONFIG *cfg = lcbvb_create();
    int rc = lcbvb_load_json(cfg, "{}");
    ASSERT_EQ(-1, rc);
    lcbvb_destroy(cfg);

    cfg = lcbvb_create();
    rc = lcbvb_load_json(cfg, "INVALIDJSON");
    ASSERT_EQ(-1, rc);
    lcbvb_destroy(cfg);

    cfg = lcbvb_create();
    rc = lcbvb_load_json(cfg, "");
    ASSERT_EQ(-1, rc);
    lcbvb_destroy(cfg);
}

TEST_F(ConfigTest, testEmptyMap)
{
    string emptyTxt = getConfigFile("bad.json");
    lcbvb_CONFIG *cfg = lcbvb_create();
    int rc = lcbvb_load_json(cfg, emptyTxt.c_str());
    ASSERT_EQ(-1, rc);
    lcbvb_destroy(cfg);
}

TEST_F(ConfigTest, testLongHostNames)
{
    string emptyTxt = getConfigFile("terse_long_hostname.json");
    lcbvb_CONFIG *cfg = lcbvb_create();
    char *network = strdup("external");
    int rc = lcbvb_load_json_ex(cfg, emptyTxt.c_str(), NULL, &network);
    ASSERT_EQ(0, rc);
    const char *hostport = lcbvb_get_hostport(cfg, 0, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN);
    ASSERT_STREQ("ec2-35-165-248-26.us-west-2-long.long-long-name.compute.amazonaws.com:11210", hostport);
    free(network);
    lcbvb_destroy(cfg);
}

TEST_F(ConfigTest, testNondataNodes)
{
    // Tests the handling of nodes which don't have any data in them
    const size_t nservers = 6;
    const size_t ndatasrv = 3;
    const size_t nreplica = ndatasrv - 1;

    vector< lcbvb_SERVER > servers;
    servers.resize(nservers);

    size_t ii;
    for (ii = 0; ii < nservers - ndatasrv; ++ii) {
        lcbvb_SERVER &server = servers[ii];
        memset(&server, 0, sizeof server);
        server.svc.data = 1000 + ii;
        server.svc.views = 2000 + ii;
        server.hostname = const_cast< char * >("dummy.host.ru");
    }

    for (; ii < nservers; ii++) {
        lcbvb_SERVER &server = servers[ii];
        memset(&server, 0, sizeof server);
        server.svc.n1ql = 3000 + ii;
        server.hostname = const_cast< char * >("query.host.biz");
    }

    lcbvb_CONFIG *cfg_ex = lcbvb_create();
    int rv = lcbvb_genconfig_ex(cfg_ex, "default", NULL, &servers[0],
                                servers.size(), // include non-data servers
                                nreplica, 1024);
    ASSERT_EQ(0, rv);
    lcbvb_genffmap(cfg_ex);

    lcbvb_CONFIG *cfg_old = lcbvb_create();
    rv = lcbvb_genconfig_ex(cfg_old, "default", NULL, &servers[0], ndatasrv, nreplica, 1024);
    ASSERT_EQ(0, rv);
    lcbvb_genffmap(cfg_old);

    ASSERT_EQ(ndatasrv, cfg_ex->ndatasrv);
    ASSERT_EQ(nservers, cfg_ex->nsrv);

    ASSERT_EQ(ndatasrv, cfg_old->ndatasrv);
    ASSERT_EQ(ndatasrv, cfg_old->nsrv);

    // So far, so good.
    vector< string > keys;
    for (ii = 0; ii < 1024; ii++) {
        std::stringstream ss;
        ss << "Key_" << ii;
        keys.push_back(ss.str());
    }

    int vbid, ix_exp, ix_cur;
    // Ensure vBucket mapping, etc. is the same
    for (ii = 0; ii < keys.size(); ii++) {
        const string &s = keys[ii];

        lcbvb_map_key(cfg_old, s.c_str(), s.size(), &vbid, &ix_exp);
        lcbvb_map_key(cfg_ex, s.c_str(), s.size(), &vbid, &ix_cur);
        ASSERT_TRUE(ix_exp > -1 && ix_exp < (int)cfg_ex->ndatasrv);
        ASSERT_EQ(ix_exp, ix_cur);
    }

    // On the new config, ensure that remap never maps to a non-data node.
    for (ii = 0; ii < keys.size(); ii++) {
        const string &s = keys[ii];
        for (size_t jj = 0; jj < cfg_ex->nsrv * 2; jj++) {
            int ix;
            lcbvb_map_key(cfg_ex, s.c_str(), s.size(), &vbid, &ix);
            int newix = lcbvb_nmv_remap(cfg_ex, vbid, ix);
            if (newix == -1) {
                continue;
            } else {
                ASSERT_TRUE(newix < (int)cfg_ex->ndatasrv);
            }
        }
    }

    // Test with ketama
    lcbvb_make_ketama(cfg_ex);
    lcbvb_make_ketama(cfg_old);
    for (ii = 0; ii < keys.size(); ii++) {
        const string &s = keys[ii];
        lcbvb_map_key(cfg_old, s.c_str(), s.size(), &vbid, &ix_exp);
        lcbvb_map_key(cfg_ex, s.c_str(), s.size(), &vbid, &ix_cur);
        ASSERT_TRUE(ix_exp > -1 && ix_exp < (int)cfg_old->ndatasrv);
        ASSERT_EQ(ix_exp, ix_cur);
    }

    // destroy 'em
    lcbvb_destroy(cfg_ex);
    lcbvb_destroy(cfg_old);
}

TEST_F(ConfigTest, testKetamaUniformity)
{
    string txt = getConfigFile("memd_45.json");
    lcbvb_CONFIG *vbc = lcbvb_parse_json(txt.c_str());
    ASSERT_TRUE(vbc != NULL);
    ASSERT_EQ(4, vbc->nsrv);
    ASSERT_EQ(LCBVB_DIST_KETAMA, vbc->dtype);

    // Ensure the continuum stuff is NULL!
    ASSERT_TRUE(vbc->continuum == NULL);
    ASSERT_EQ(0, vbc->ncontinuum);
    lcbvb_replace_host(vbc, "localhost");

    ASSERT_STREQ("10.0.0.195:12000", vbc->servers[0].authority);
    ASSERT_STREQ("localhost:12002", vbc->servers[1].authority);
    ASSERT_STREQ("localhost:12004", vbc->servers[2].authority);
    ASSERT_STREQ("localhost:12006", vbc->servers[3].authority);
    lcbvb_destroy(vbc);
}

TEST_F(ConfigTest, testKetamaCompliance)
{
    string txt = getConfigFile("memd_ketama_config.json");
    lcbvb_CONFIG *vbc = lcbvb_parse_json(txt.c_str());
    ASSERT_TRUE(vbc != NULL);
    ASSERT_EQ(4, vbc->nsrv);
    ASSERT_EQ(LCBVB_DIST_KETAMA, vbc->dtype);

    lcbvb_replace_host(vbc, "192.168.1.104");
    // Now, load the hash file
    string expected_txt = getConfigFile("ketama_expected.json");
    Json::Value json;
    ASSERT_TRUE(Json::Reader().parse(expected_txt, json));

    ASSERT_EQ(json.size(), vbc->ncontinuum);

    // Iterate over the continuum in the vbuckets
    for (size_t ii = 0; ii < json.size(); ++ii) {
        const Json::Value &cur = json[static_cast< int >(ii)];
        unsigned exp_hash = cur["hash"].asUInt();
        string exp_server = cur["hostname"].asString();
        unsigned got_hash = vbc->continuum[ii].point;
        unsigned got_index = vbc->continuum[ii].index;

        ASSERT_EQ(exp_server, vbc->servers[got_index].authority);
        ASSERT_EQ(exp_hash, got_hash);
    }

    lcbvb_destroy(vbc);
}

TEST_F(ConfigTest, testPresentNodesextMissingNodesKetama)
{
    // Scenario when a node is in nodesext but not nodes
    string txt = getConfigFile("map_node_present_nodesext_missing_nodes.json");
    lcbvb_CONFIG *vbc = lcbvb_parse_json(txt.c_str());
    ASSERT_TRUE(vbc != NULL);
    ASSERT_EQ(4, vbc->nsrv);
    ASSERT_EQ(LCBVB_DIST_KETAMA, vbc->dtype);

    ASSERT_NE((const char *)NULL, lcbvb_get_hostport(vbc, 0, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN));
    ASSERT_NE((const char *)NULL, lcbvb_get_hostport(vbc, 1, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN));
    ASSERT_NE((const char *)NULL, lcbvb_get_hostport(vbc, 2, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN));
    ASSERT_EQ((const char *)NULL, lcbvb_get_hostport(vbc, 3, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN));
}
