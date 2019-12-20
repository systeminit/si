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
#include "ringbuffer.h"

class Ringbuffer : public ::testing::Test
{
  protected:
    // Helper function used for debugging ;)
    void dump_buffer(ringbuffer_t *ring)
    {
        const char *begin = (const char *)ringbuffer_get_start(ring);
        const char *end = begin + ringbuffer_get_size(ring);
        const char *rd = (const char *)ringbuffer_get_read_head(ring);
        const char *wr = (const char *)ringbuffer_get_write_head(ring);
        const char *cur;

        /* write head */
        fprintf(stderr, " ");
        for (cur = begin; cur < end; cur++) {
            if (cur == wr) {
                fprintf(stderr, "w");
            } else {
                fprintf(stderr, " ");
            }
        }
        fprintf(stderr, "\n");

        /* the buffer contents */
        fprintf(stderr, "|");
        for (cur = begin; cur < end; cur++) {
            fprintf(stderr, "%c", *cur ? *cur : '-');
        }
        fprintf(stderr, "|\n");

        /* the read head */
        fprintf(stderr, " ");
        for (cur = begin; cur < end; cur++) {
            if (cur == rd) {
                fprintf(stderr, "r");
            } else {
                fprintf(stderr, " ");
            }
        }
        fprintf(stderr, "\n");
    }
};

TEST_F(Ringbuffer, basicTests)
{
    ringbuffer_t ring;
    char buffer[1024];
    int ii;

    EXPECT_NE(0, ringbuffer_initialize(&ring, 16));
    EXPECT_EQ(0, ringbuffer_read(&ring, buffer, 1));
    EXPECT_EQ(16, ringbuffer_write(&ring, "01234567891234567", 17));

    for (ii = 0; ii < 2; ++ii) {
        memset(buffer, 0, sizeof(buffer));
        EXPECT_EQ(16, ringbuffer_peek(&ring, buffer, 16));
        EXPECT_EQ(0, memcmp(buffer, "01234567891234567", 16));
        memset(buffer, 0, sizeof(buffer));
        EXPECT_EQ(10, ringbuffer_peek_at(&ring, 6, buffer, 10));
        EXPECT_EQ(0, memcmp(buffer, "67891234567", 10));
    }

    EXPECT_EQ(16, ringbuffer_read(&ring, buffer, 16));
    EXPECT_EQ(0, ringbuffer_read(&ring, buffer, 1));
    EXPECT_EQ(16, ringbuffer_write(&ring, "01234567891234567", 17));
    EXPECT_EQ(8, ringbuffer_read(&ring, buffer, 8));
    EXPECT_NE(0, ringbuffer_ensure_capacity(&ring, 9));
    EXPECT_EQ(32, ring.size);
    EXPECT_EQ(ring.root, ring.read_head);
    EXPECT_EQ(8, ringbuffer_read(&ring, buffer, 9));
    EXPECT_EQ(0, memcmp(buffer, "89123456", 8));

    ringbuffer_destruct(&ring);

    // wrapped_buffer_test();
    // my_regression_1_test();
}

TEST_F(Ringbuffer, wrappedBufferTest)
{
    ringbuffer_t ring;
    char buffer[128];

    EXPECT_NE(0, ringbuffer_initialize(&ring, 10));

    memset(ringbuffer_get_start(&ring), 0, 10);
    /*  w
     * |----------|
     *  r
     */

    /* put 8 chars into the buffer */
    EXPECT_EQ(8, ringbuffer_write(&ring, "01234567", 8));

    /*          w
     * |01234567--|
     *  r
     */

    /* consume first 5 chars */
    EXPECT_EQ(5, ringbuffer_read(&ring, buffer, 5));
    EXPECT_EQ(0, memcmp(buffer, "01234", 5));

    /*          w
     * |-----567--|
     *       r
     */
    EXPECT_EQ(0, ringbuffer_is_continous(&ring, RINGBUFFER_WRITE, 5));
    EXPECT_NE(0, ringbuffer_is_continous(&ring, RINGBUFFER_WRITE, 2));

    /* wrapped write: write 5 more chars */
    EXPECT_EQ(5, ringbuffer_write(&ring, "abcde", 5));

    /*     w
     * |cde--567ab|
     *       r
     */

    EXPECT_EQ(0, ringbuffer_is_continous(&ring, RINGBUFFER_READ, 7));
    EXPECT_NE(0, ringbuffer_is_continous(&ring, RINGBUFFER_READ, 2));

    /* wrapped read: read 6 chars */
    EXPECT_EQ(6, ringbuffer_read(&ring, buffer, 6));
    EXPECT_EQ(0, memcmp(buffer, "567abc", 6));
    /*     w
     * |-de-------|
     *   r
     */
    ringbuffer_destruct(&ring);
}

// This is a crash I noticed while I was debugging the tap code
TEST_F(Ringbuffer, regression1)
{
    ringbuffer_t ring;
    struct lcb_iovec_st iov[2];
    ring.root = (char *)0x477a80;
    ring.read_head = (char *)0x47b0a3;
    ring.write_head = (char *)0x47b555;
    ring.size = 16384;
    ring.nbytes = 1202;

    ringbuffer_get_iov(&ring, RINGBUFFER_WRITE, iov);
    // up to the end
    EXPECT_EQ(ring.write_head, iov[0].iov_base);
    EXPECT_EQ(1323, iov[0].iov_len);

    // then from the beginning
    EXPECT_EQ(ring.root, iov[1].iov_base);
    EXPECT_EQ(13859, iov[1].iov_len);
}

TEST_F(Ringbuffer, replace)
{
    ringbuffer_t rb;

    EXPECT_EQ(1, ringbuffer_initialize(&rb, 16));
    EXPECT_TRUE(memset(rb.root, 0, rb.size) != NULL);
    EXPECT_EQ(8, ringbuffer_write(&rb, "01234567", 8));
    EXPECT_EQ(0, memcmp(rb.root, "01234567\0\0\0\0\0\0\0\0", rb.size));
    /*          w
     * |01234567--------|
     *  r
     */

    EXPECT_EQ(2, ringbuffer_update(&rb, RINGBUFFER_READ, "ab", 2));
    EXPECT_EQ(8, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "ab234567\0\0\0\0\0\0\0\0", rb.size));
    /*          w
     * |ab234567--------|
     *  r
     */

    EXPECT_EQ(2, ringbuffer_update(&rb, RINGBUFFER_WRITE, "cd", 2));
    EXPECT_EQ(8, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "ab2345cd\0\0\0\0\0\0\0\0", rb.size));
    /*          w
     * |ab2345cd--------|
     *  r
     */

    ringbuffer_consumed(&rb, 3);
    EXPECT_EQ(5, rb.nbytes);
    EXPECT_EQ(rb.root + 3, rb.read_head);
    /*          w
     * |ab2345cd--------|
     *     r
     */

    EXPECT_EQ(5, ringbuffer_update(&rb, RINGBUFFER_READ, "efghij", 6));
    EXPECT_EQ(5, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "ab2efghi\0\0\0\0\0\0\0\0", rb.size));
    /*          w
     * |ab2efghi--------|
     *     r
     */

    EXPECT_EQ(5, ringbuffer_update(&rb, RINGBUFFER_WRITE, "klmnop", 6));
    EXPECT_EQ(5, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "ab2klmno\0\0\0\0\0\0\0\0", rb.size));
    /*          w
     * |ab2klmno--------|
     *     r
     */

    EXPECT_EQ(10, ringbuffer_write(&rb, "0123456789", 10));
    EXPECT_EQ(15, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "892klmno01234567", rb.size));
    /*    w
     * |892klmno01234567|
     *     r
     */

    EXPECT_EQ(10, ringbuffer_update(&rb, RINGBUFFER_WRITE, "abcdefghij", 10));
    EXPECT_EQ(15, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "ij2klmnoabcdefgh", rb.size));
    /*    w
     * |ij2klmnoabcdefgh|
     *     r
     */

    ringbuffer_consumed(&rb, 6);
    EXPECT_EQ(9, rb.nbytes);
    EXPECT_EQ(rb.root + 9, rb.read_head);
    /*    w
     * |ij2klmnoabcdefgh|
     *           r
     */

    EXPECT_EQ(8, ringbuffer_update(&rb, RINGBUFFER_READ, "12345678", 8));
    EXPECT_EQ(9, rb.nbytes);
    EXPECT_EQ(0, memcmp(rb.root, "8j2klmnoa1234567", rb.size));
    /*    w
     * |8j2klmnoa1234567|
     *           r
     */
    ringbuffer_destruct(&rb);
}

TEST_F(Ringbuffer, memcpy)
{
    char buffer[1024];
    ringbuffer_t src, dst;

    EXPECT_EQ(1, ringbuffer_initialize(&src, 16));
    EXPECT_EQ(8, ringbuffer_write(&src, "01234567", 8));

    EXPECT_EQ(1, ringbuffer_initialize(&dst, 16));

    EXPECT_EQ(0, ringbuffer_memcpy(&dst, &src, 4));
    EXPECT_EQ(4, dst.nbytes);
    EXPECT_EQ(4, ringbuffer_read(&dst, buffer, 4));
    EXPECT_EQ(0, memcmp(buffer, "0123", 4));

    ringbuffer_destruct(&src);
    ringbuffer_destruct(&dst);
}
