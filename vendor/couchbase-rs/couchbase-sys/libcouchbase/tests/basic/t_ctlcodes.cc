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

class CtlTest : public ::testing::Test
{
};

template < typename T > T getSetting(lcb_INSTANCE *instance, int ctl)
{
    T obj;
    lcb_STATUS err;
    err = lcb_cntl(instance, LCB_CNTL_GET, ctl, &obj);
    EXPECT_EQ(LCB_SUCCESS, err);
    return obj;
}

TEST_F(CtlTest, testExists)
{
    for (int ii = 0; ii < LCB_CNTL__MAX; ii++) {
        switch (ii) {
            case 0x0a: /* LCB_CNTL_SYNCMODE */
            case 0x2d: /* LCB_CNTL_RETRY_BACKOFF */
                ASSERT_FALSE(lcb_cntl_exists(ii));
                break;
            default:
                ASSERT_TRUE(lcb_cntl_exists(ii));
        }
    }
    ASSERT_EQ(0, lcb_cntl_exists(-1));
    ASSERT_EQ(0, lcb_cntl_exists(LCB_CNTL__MAX));
}

struct PairMap {
    const char *key;
    int opval;
};

TEST_F(CtlTest, testStringCtls)
{
    lcb_INSTANCE *instance;
    lcb_STATUS err;
    err = lcb_create(&instance, NULL);
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_FALSE(instance == NULL);

    // These are all U32
    PairMap ctlMap[] = {{"operation_timeout", LCB_CNTL_OP_TIMEOUT},
                        {"views_timeout", LCB_CNTL_VIEW_TIMEOUT},
                        {"durability_timeout", LCB_CNTL_DURABILITY_TIMEOUT},
                        {"durability_interval", LCB_CNTL_DURABILITY_INTERVAL},
                        {"http_timeout", LCB_CNTL_HTTP_TIMEOUT},
                        {"error_thresh_delay", LCB_CNTL_CONFDELAY_THRESH},
                        {"config_total_timeout", LCB_CNTL_CONFIGURATION_TIMEOUT},
                        {"config_node_timeout", LCB_CNTL_CONFIG_NODE_TIMEOUT},
                        {NULL, 0}};

    for (PairMap *cur = ctlMap; cur->key; cur++) {
        err = lcb_cntl_string(instance, cur->key, "50");
        ASSERT_EQ(LCB_SUCCESS, err);
        ASSERT_EQ(50000000, lcb_cntl_getu32(instance, cur->opval));
    }

    // try with a boolean
    err = lcb_cntl_string(instance, "randomize_nodes", "false");
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(0, getSetting< int >(instance, LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS));

    err = lcb_cntl_string(instance, "randomize_nodes", "true");
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(1, getSetting< int >(instance, LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS));

    // try with compression
    err = lcb_cntl_string(instance, "compression", "on");
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_COMPRESS_INOUT, getSetting< lcb_COMPRESSOPTS >(instance, LCB_CNTL_COMPRESSION_OPTS));

    err = lcb_cntl_string(instance, "compression", "off");
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_COMPRESS_NONE, getSetting< lcb_COMPRESSOPTS >(instance, LCB_CNTL_COMPRESSION_OPTS));

    err = lcb_cntl_string(instance, "compression", "inflate_only");
    ASSERT_EQ(LCB_SUCCESS, err);
    ASSERT_EQ(LCB_COMPRESS_IN, getSetting< lcb_COMPRESSOPTS >(instance, LCB_CNTL_COMPRESSION_OPTS));

    err = lcb_cntl_string(instance, "unsafe_optimize", "1");
    ASSERT_EQ(LCB_SUCCESS, err);
    err = lcb_cntl_string(instance, "unsafe_optimize", "0");
    ASSERT_NE(LCB_SUCCESS, err);

    lcb_destroy(instance);
}
