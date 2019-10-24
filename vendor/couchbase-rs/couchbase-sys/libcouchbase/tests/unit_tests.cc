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
#include "config.h"
#include <gtest/gtest.h>

#ifdef NO_COUCHBASEMOCK
class MockTestsAreDisabled : public ::testing::Test
{
};
TEST_F(MockTestsAreDisabled, MockTestsAreDisabled)
{
    fprintf(stderr, "*** WARNING\n");
    fprintf(stderr, "*** libcouchbase Java Mock tests are disabled\n");
    fprintf(stderr, "*** Basic memcached functionality (get, set, etc.) \n");
    fprintf(stderr, "*** will NOT be tested\n");
}
#define SETUP_MOCK_ENV()
#define setup_test_timeout_handler()
#else
#include "iotests/iotests.h"
#define SETUP_MOCK_ENV() ::testing::AddGlobalTestEnvironment(MockEnvironment::getInstance())
#endif

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IOLBF, 2048);
    setvbuf(stderr, NULL, _IOLBF, 2048);
    SETUP_MOCK_ENV();
    ::testing::InitGoogleTest(&argc, argv);
    setup_test_timeout_handler();
    return RUN_ALL_TESTS();
}
