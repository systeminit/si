#include <stdio.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <gtest/gtest.h>
#ifdef _WIN32
#include <windows.h>
#endif
#include "netbuf/netbuf.h"

#define BIG_BUF_SIZE 5000
#define SMALL_BUF_SIZE 50

class NetbufTest : public ::testing::Test
{
};

static void clean_check(nb_MGR *mgr)
{
    EXPECT_NE(0, netbuf_is_clean(mgr));
    netbuf_cleanup(mgr);
}

TEST_F(NetbufTest, testCleanCheck)
{
    nb_MGR mgr;
    netbuf_init(&mgr, NULL);
    nb_SPAN span;
    span.size = 500;
    int rv = netbuf_mblock_reserve(&mgr, &span);
    ASSERT_EQ(0, rv);
    ASSERT_EQ(0, netbuf_is_clean(&mgr));
    netbuf_mblock_release(&mgr, &span);
    ASSERT_NE(0, netbuf_is_clean(&mgr));

    nb_IOV iov;
    iov.iov_base = (void *)0x01;
    iov.iov_len = 500;
    netbuf_enqueue(&mgr, &iov, NULL);
    ASSERT_EQ(0, netbuf_is_clean(&mgr));

    unsigned toFlush = netbuf_start_flush(&mgr, &iov, 1, NULL);
    ASSERT_EQ(500, toFlush);
    netbuf_end_flush(&mgr, toFlush);
    ASSERT_NE(0, netbuf_is_clean(&mgr));

    clean_check(&mgr);
}

TEST_F(NetbufTest, testBasic)
{
    nb_MGR mgr;
    int rv;
    int ii;
    int n_bigspans = 20;
    int n_smallspans = 2000;

    nb_SPAN spans_big[20];
    nb_SPAN spans_small[2000];
    netbuf_init(&mgr, NULL);
    clean_check(&mgr);
    netbuf_init(&mgr, NULL);

    for (ii = 0; ii < n_bigspans; ii++) {
        int filler = 'a' + ii;
        nb_SPAN *span = spans_big + ii;
        span->size = BIG_BUF_SIZE;
        rv = netbuf_mblock_reserve(&mgr, span);
        ASSERT_EQ(0, rv);
        memset(SPAN_BUFFER(span), filler, span->size);
    }

    for (ii = 0; ii < n_smallspans; ii++) {
        nb_SPAN *span = spans_small + ii;
        int filler = ii;
        span->size = SMALL_BUF_SIZE;
        rv = netbuf_mblock_reserve(&mgr, span);
        ASSERT_EQ(0, rv);
        filler = ii;
        memset(SPAN_BUFFER(span), filler, span->size);
    }

    for (ii = 0; ii < n_bigspans; ii++) {
        char expected[BIG_BUF_SIZE];
        char *curbuf = SPAN_BUFFER(spans_big + ii);

        memset(expected, 'a' + ii, BIG_BUF_SIZE);
        ASSERT_EQ(0, memcmp(curbuf, expected, BIG_BUF_SIZE));

        netbuf_mblock_release(&mgr, spans_big + ii);
    }

    for (ii = 0; ii < n_smallspans; ii++) {
        char expected[SMALL_BUF_SIZE];
        char *curbuf = SPAN_BUFFER(spans_small + ii);
        memset(expected, ii, SMALL_BUF_SIZE);
        ASSERT_EQ(0, memcmp(curbuf, expected, SMALL_BUF_SIZE));
        netbuf_mblock_release(&mgr, spans_small + ii);
    }

    {
        nb_IOV iov[20];
        netbuf_start_flush(&mgr, iov, 20, NULL);
    }
    clean_check(&mgr);
}

TEST_F(NetbufTest, testFlush)
{
    nb_MGR mgr;
    nb_SETTINGS settings;
    nb_SPAN span;
    nb_SPAN spans[3];

    int ii;
    int rv;
    nb_IOV iov[10];
    unsigned int sz;

    netbuf_default_settings(&settings);
    settings.data_basealloc = 8;
    netbuf_init(&mgr, &settings);

    span.size = 32;
    rv = netbuf_mblock_reserve(&mgr, &span);
    ASSERT_EQ(rv, 0);

    netbuf_enqueue_span(&mgr, &span, NULL);
    sz = netbuf_start_flush(&mgr, iov, 1, NULL);
    ASSERT_EQ(32, sz);
    ASSERT_EQ(32, iov[0].iov_len);
    netbuf_end_flush(&mgr, 20);

    sz = netbuf_start_flush(&mgr, iov, 1, NULL);
    ASSERT_EQ(0, sz);
    netbuf_end_flush(&mgr, 12);
    netbuf_mblock_release(&mgr, &span);

    for (ii = 0; ii < 3; ii++) {
        spans[ii].size = 50;
        ASSERT_EQ(0, netbuf_mblock_reserve(&mgr, spans + ii));
    }

    for (ii = 0; ii < 3; ii++) {
        netbuf_enqueue_span(&mgr, spans + ii, NULL);
    }

    sz = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(150, sz);
    netbuf_end_flush(&mgr, 75);
    netbuf_reset_flush(&mgr);
    sz = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(75, sz);
    netbuf_end_flush(&mgr, 75);
    sz = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(0, sz);
    netbuf_mblock_release(&mgr, &spans[0]);

    spans[0].size = 20;
    rv = netbuf_mblock_reserve(&mgr, &spans[0]);
    ASSERT_EQ(0, rv);
    netbuf_mblock_release(&mgr, &spans[0]);

    for (ii = 1; ii < 3; ii++) {
        netbuf_mblock_release(&mgr, spans + ii);
    }

    netbuf_dump_status(&mgr, stdout);
    clean_check(&mgr);
}

TEST_F(NetbufTest, testWrappingBuffers)
{
    nb_MGR mgr;
    nb_SETTINGS settings;
    int rv;
    nb_SPAN span1, span2, span3;

#ifdef NETBUFS_LIBC_PROXY
    return;
#endif

    netbuf_default_settings(&settings);
    settings.data_basealloc = 40;
    netbuf_init(&mgr, &settings);

    span1.size = 16;
    span2.size = 16;

    rv = netbuf_mblock_reserve(&mgr, &span1);
    ASSERT_EQ(0, rv);
    rv = netbuf_mblock_reserve(&mgr, &span2);
    ASSERT_EQ(0, rv);

    ASSERT_EQ(span1.parent, span2.parent);
    ASSERT_EQ(0, span1.offset);
    ASSERT_EQ(16, span2.offset);

    /* Wewease Wodewick! */
    netbuf_mblock_release(&mgr, &span1);
    ASSERT_EQ(16, span2.parent->start);

    /* So we have 8 bytes at the end.. */
    ASSERT_EQ(32, span2.parent->wrap);
    span3.size = 10;
    rv = netbuf_mblock_reserve(&mgr, &span3);

    ASSERT_EQ(0, rv);
    ASSERT_EQ(10, span2.parent->cursor);
    ASSERT_EQ(0, span3.offset);
    ASSERT_EQ(10, span3.parent->cursor);
    ASSERT_EQ(16, span3.parent->start);

    netbuf_mblock_release(&mgr, &span2);
    ASSERT_EQ(0, span3.parent->start);
    netbuf_mblock_release(&mgr, &span3);

    netbuf_dump_status(&mgr, stdout);

    span1.size = 20;
    rv = netbuf_mblock_reserve(&mgr, &span1);
    ASSERT_EQ(0, span1.offset);
    ASSERT_EQ(20, span1.parent->cursor);
    ASSERT_EQ(0, span1.parent->start);
    ASSERT_EQ(20, span1.parent->wrap);
    netbuf_dump_status(&mgr, stdout);

    netbuf_mblock_release(&mgr, &span1);

    clean_check(&mgr);
}

static void assert_iov_eq(nb_IOV *iov, nb_SIZE offset, char expected)
{
    char *buf = (char *)iov->iov_base;
    ASSERT_EQ(expected, buf[offset]);
}

TEST_F(NetbufTest, testMultipleFlush)
{
    nb_SETTINGS settings;
    nb_MGR mgr;
    int rv;
    nb_SIZE sz;
    nb_SPAN span1, span2, span3;
    nb_IOV iov[10];

    netbuf_default_settings(&settings);
    netbuf_init(&mgr, &settings);

    span1.size = 50;
    span2.size = 50;
    span3.size = 50;

    rv = netbuf_mblock_reserve(&mgr, &span1);
    ASSERT_EQ(0, rv);
    rv = netbuf_mblock_reserve(&mgr, &span2);
    ASSERT_EQ(0, rv);
    rv = netbuf_mblock_reserve(&mgr, &span3);
    ASSERT_EQ(0, rv);

    netbuf_enqueue_span(&mgr, &span1, NULL);
    netbuf_enqueue_span(&mgr, &span2, NULL);

    sz = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(100, sz);

    memset(SPAN_BUFFER(&span1), 'A', span1.size);
    memset(SPAN_BUFFER(&span2), 'B', span2.size);
    memset(SPAN_BUFFER(&span3), 'C', span3.size);

#ifndef NETBUFS_LIBC_PROXY
    ASSERT_EQ(100, iov->iov_len);
    assert_iov_eq(iov, 0, 'A');
    assert_iov_eq(iov, 50, 'B');

    netbuf_enqueue_span(&mgr, &span3, NULL);
    sz = netbuf_start_flush(&mgr, &iov[1], 0, NULL);
    ASSERT_EQ(sz, 50);
    assert_iov_eq(&iov[1], 0, 'C');
    ASSERT_EQ(50, iov[1].iov_len);

    netbuf_dump_status(&mgr, stdout);

    netbuf_end_flush(&mgr, 100);
    netbuf_dump_status(&mgr, stdout);

    netbuf_end_flush(&mgr, 50);
    sz = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(0, sz);
#endif

    netbuf_mblock_release(&mgr, &span1);
    netbuf_mblock_release(&mgr, &span2);
    netbuf_mblock_release(&mgr, &span3);
    clean_check(&mgr);
}

TEST_F(NetbufTest, testCyclicFlush)
{
    nb_SPAN spans[10];
    nb_IOV iov[4];
    nb_MGR mgr;
    nb_SETTINGS settings;
    int niov;
    unsigned nb;

    // Each call to netbuf_start_flush should be considered isolated; so that
    // the next call to start_flush _never_ overlaps any data from the previous
    // call to start_flush. Otherwise we might end up in a situation where
    // the same data ends up being sent out twice. netbuf_reset_flush() should
    // be called to invalidate any outstanding start_flush() calls, so that
    // the next call to start_flush() will begin from the beginning of the
    // send queue, rather than from the last call to start_flush().

    netbuf_default_settings(&settings);
    settings.data_basealloc = 50;
    netbuf_init(&mgr, &settings);

    for (size_t ii = 0; ii < 5; ii++) {
        spans[ii].size = 10;
        netbuf_mblock_reserve(&mgr, &spans[ii]);
        memset(SPAN_BUFFER(&spans[ii]), ii, 10);
        netbuf_enqueue_span(&mgr, &spans[ii], NULL);
        nb = netbuf_start_flush(&mgr, iov, 1, &niov);

        ASSERT_EQ(10, nb);
        ASSERT_EQ(1, niov);
    }
    // So far have 50 inside the span

    // flush the first span (should have 40 bytes remaining)
    netbuf_end_flush(&mgr, 10);
    for (size_t ii = 5; ii < 7; ii++) {
        spans[ii].size = 10;
        netbuf_mblock_reserve(&mgr, &spans[ii]);
        netbuf_enqueue_span(&mgr, &spans[ii], NULL);
        memset(SPAN_BUFFER(&spans[ii]), ii, 10);
    }

    nb = netbuf_start_flush(&mgr, iov, 4, &niov);
    ASSERT_EQ(20, nb);
    netbuf_end_flush(&mgr, 40);
    netbuf_end_flush(&mgr, nb);
    nb = netbuf_start_flush(&mgr, iov, 4, &niov);
    ASSERT_EQ(0, nb);
    for (size_t ii = 0; ii < 7; ii++) {
        netbuf_mblock_release(&mgr, &spans[ii]);
    }
    clean_check(&mgr);
}

typedef struct {
    sllist_node slnode;
    nb_SIZE size;
    int is_flushed;
    nb_SPAN spans[3];
    nb_SIZE nspans;
} my_PDU;

static nb_SIZE pdu_callback(void *p, nb_SIZE hint, void *arg)
{
    my_PDU *pdu = (my_PDU *)p;
    (void)arg;
    if (hint >= pdu->size) {
        pdu->is_flushed = 1;
    }
    return pdu->size;
}

TEST_F(NetbufTest, testPduEnqueue)
{
    nb_SETTINGS settings;
    nb_MGR mgr;
    my_PDU pdu;
    nb_IOV iov[10];
    nb_SIZE toflush;
    int ii;

    netbuf_default_settings(&settings);
    settings.data_basealloc = 1;
    netbuf_init(&mgr, &settings);

    memset(&pdu, 0, sizeof pdu);
    pdu.size = 24;

    for (ii = 0; ii < 3; ii++) {
        pdu.spans[ii].size = 8;
        netbuf_mblock_reserve(&mgr, pdu.spans + ii);
    }

    for (ii = 0; ii < 3; ii++) {
        netbuf_enqueue_span(&mgr, pdu.spans + ii, NULL);
    }

    netbuf_pdu_enqueue(&mgr, &pdu, offsetof(my_PDU, slnode));

    /** Start the flush */
    toflush = netbuf_start_flush(&mgr, iov, 2, NULL);
    ASSERT_EQ(16, toflush);
    netbuf_end_flush2(&mgr, toflush, pdu_callback, 0, NULL);
    ASSERT_EQ(0, pdu.is_flushed);

    toflush = netbuf_start_flush(&mgr, iov, 10, NULL);
    ASSERT_EQ(8, toflush);

    netbuf_end_flush2(&mgr, toflush, pdu_callback, 0, NULL);
    ASSERT_EQ(1, pdu.is_flushed);

    for (ii = 0; ii < 3; ii++) {
        netbuf_mblock_release(&mgr, pdu.spans + ii);
    }

    clean_check(&mgr);
}

TEST_F(NetbufTest, testOutOfOrder)
{
    nb_MGR mgr;
    nb_SPAN spans[3];
    int ii;

    netbuf_init(&mgr, NULL);

    for (ii = 0; ii < 3; ii++) {
        spans[ii].size = 10;
        int rv = netbuf_mblock_reserve(&mgr, spans + ii);
        ASSERT_EQ(0, rv);
    }

    netbuf_mblock_release(&mgr, &spans[1]);
    spans[1].size = 5;

    netbuf_mblock_reserve(&mgr, &spans[1]);
    ASSERT_EQ(30, spans[1].offset);

    for (ii = 0; ii < 3; ii++) {
        netbuf_mblock_release(&mgr, spans + ii);
    }

    clean_check(&mgr);
}
