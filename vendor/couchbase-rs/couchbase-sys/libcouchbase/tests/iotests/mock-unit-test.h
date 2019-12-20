/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#ifndef TESTS_MOCK_UNIT_TESTS_H
#define TESTS_MOCK_UNIT_TESTS_H 1

#include "config.h"
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include "mock-environment.h"

class HandleWrap;

#define SKIP_IF_MOCK()                                                                                                 \
    if (!getenv(LCB_TEST_REALCLUSTER_ENV)) {                                                                           \
        MockEnvironment::printSkipMessage(__FILE__, __LINE__, "needs real cluster");                                   \
        return;                                                                                                        \
    }

#define SKIP_UNLESS_MOCK()                                                                                             \
    if (getenv(LCB_TEST_REALCLUSTER_ENV)) {                                                                            \
        MockEnvironment::printSkipMessage(__FILE__, __LINE__, "needs mock cluster");                                   \
        return;                                                                                                        \
    }

#define ASSERT_ERRISA(err, et) ASSERT_EQ(et, (int)lcb_get_errtype(err) & (int)et)

class MockUnitTest : public ::testing::Test
{
  protected:
    virtual void SetUp();
    virtual void createConnection(lcb_INSTANCE **instance);
    virtual void createConnection(HandleWrap &handle);
    virtual void createConnection(HandleWrap &handle, lcb_INSTANCE **instance);
    virtual void createClusterConnection(HandleWrap &handle, lcb_INSTANCE **instance);
    virtual lcb_STATUS tryCreateConnection(HandleWrap &hw, lcb_INSTANCE **instance, lcb_create_st &crparams);

    // A mock "Transaction"
    void doMockTxn(MockCommand &cmd)
    {
        MockEnvironment::getInstance()->sendCommand(cmd);
        MockResponse tmp;
        MockEnvironment::getInstance()->getResponse(tmp);
        ASSERT_TRUE(tmp.isOk());
    }
};

#endif
