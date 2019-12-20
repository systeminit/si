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

#include "rdbtest.h"

class RopeTest : public ::testing::Test
{
};
using std::string;

TEST_F(RopeTest, testBasic)
{
    IORope rope;
    nb_IOV iovs;
    unsigned niov = rdb_rdstart(&rope, &iovs, 1);
    ASSERT_EQ(1, niov);
    ASSERT_FALSE(iovs.iov_base == NULL);
    ASSERT_GT(iovs.iov_len, 0U);

    memset(iovs.iov_base, 0x66, iovs.iov_len);
    rdb_rdend(&rope, iovs.iov_len);
    ASSERT_EQ(iovs.iov_len, rope.usedSize());

    for (unsigned ii = 0; ii < iovs.iov_len; ii++) {
        unsigned char tmp;
        rdb_copyread(&rope, &tmp, 1);
        ASSERT_EQ(0x66, tmp);
        rdb_consumed(&rope, 1);
    }
}

TEST_F(RopeTest, testFragmented)
{
    IORope rope(rdb_chunkalloc_new(1));
    nb_IOV iovs[32];
    unsigned niov;
    niov = rdb_rdstart(&rope, iovs, 32);
    ASSERT_EQ(32, niov);

    string hello("Hello World!");
    for (unsigned ii = 0; ii < hello.size(); ii++) {
        memcpy(iovs[ii].iov_base, &hello[ii], 1);
    }

    rdb_rdend(&rope, hello.size());
    ASSERT_EQ(hello.size(), rope.usedSize());

    // Now we should be able to extract it properly
    char tmpbuf[32] = {0};
    rdb_copyread(&rope, tmpbuf, hello.size());
    ASSERT_STREQ(tmpbuf, hello.c_str());

    /** Can we read contiguously? */
    nb_IOV iovs2[32];
    rdb_ROPESEG *backs[32];
    int nitems;

    nitems = rdb_refread_ex(&rope, iovs2, backs, 32, hello.size());
    ASSERT_EQ(hello.size(), nitems);
    for (unsigned ii = 0; ii < hello.size(); ii++) {
        nb_IOV *cur = iovs2 + ii;
        ASSERT_EQ(*(char *)cur->iov_base, hello.at(ii));
        ASSERT_EQ(1, cur->iov_len);
    }

    // Solidify them?
    rdb_consolidate(&rope, 5);
    memset(tmpbuf, 0, sizeof(tmpbuf));
    ASSERT_EQ(hello.size(), rope.usedSize());
    rdb_copyread(&rope, tmpbuf, hello.size());
    ASSERT_STREQ(tmpbuf, hello.c_str());

    nitems = rdb_refread_ex(&rope, iovs2, backs, 32, hello.size());
    ASSERT_EQ(hello.size() - 4, nitems);
}

// This tests the functionality where we want _subsequent_ reads to be
// consolidated into a single buffer.
TEST_F(RopeTest, testConsolidatedReadAhead)
{
    IORope ior(rdb_chunkalloc_new(1));
    ior.rdsize = 256;

    nb_IOV iovs[32];
    rdb_ROPESEG *segs[32];
    unsigned niov;

    // Feed 4 bytes into the buffer
    ior.feed("1234");

    // Make the next 6 bytes consolidated
    rdb_consolidate(&ior, 6);
    ior.feed("5678");
    niov = rdb_refread_ex(&ior, iovs, segs, 3, 8);

    ASSERT_EQ(3, niov);
    ASSERT_EQ(6, iovs[0].iov_len);
    ASSERT_EQ(0, memcmp(iovs[0].iov_base, "123456", 6));
    ASSERT_EQ(*(char *)iovs[1].iov_base, '7');
    ASSERT_EQ(*(char *)iovs[2].iov_base, '8');
}

// When I was integrating this into LCBIO, I realized this scenario. Trying to
// figure out what the intended outcome is.
// Apparently this cannot work because we can't consume a buffer which is also
// available for reading as this may result in the currently-being-read-into
// buffer being released.
TEST_F(RopeTest, DISABLED_testInterleavedReadConsume)
{
    IORope ior(rdb_bigalloc_new());
    ior.rdsize = 256;
    nb_IOV iov;
    unsigned niov;

    niov = rdb_rdstart(&ior, &iov, 1);
    ASSERT_EQ(1, niov);
    memset(iov.iov_base, '1', 29);

    rdb_rdend(&ior, 29);
    rdb_consumed(&ior, 24);
    ASSERT_EQ(5, rdb_get_nused(&ior));

    nb_IOV iov2;
    niov = rdb_rdstart(&ior, &iov2, 1);
    ASSERT_EQ(1, niov);
    ASSERT_EQ(5, rdb_get_nused(&ior));

    rdb_consumed(&ior, 5);
    ASSERT_EQ(0, rdb_get_nused(&ior));

    rdb_rdend(&ior, 100);
}
