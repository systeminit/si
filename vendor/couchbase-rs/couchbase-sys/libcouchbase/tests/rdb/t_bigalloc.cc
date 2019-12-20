#include "rdbtest.h"
#include <rdb/bigalloc.h>
class BigallocTest : public ::testing::Test
{
};

TEST_F(BigallocTest, testBasic)
{
    RdbAllocator a(rdb_bigalloc_new());
    rdb_BIGALLOC *ba = (rdb_BIGALLOC *)a._inner;

    ASSERT_EQ(0, LCB_CLIST_SIZE(&ba->bufs));
    ASSERT_EQ(0, ba->n_requests);
    ASSERT_EQ(0, ba->n_toobig);
    ASSERT_EQ(0, ba->n_toosmall);
    ASSERT_EQ(RDB_BIGALLOC_ALLOCSZ_MAX, ba->max_blk_alloc);
    ASSERT_EQ(RDB_BIGALLOC_ALLOCSZ_MIN, ba->min_blk_alloc);
    ASSERT_EQ(RDB_BIGALLOC_BLKCNT_MAX, ba->max_blk_count);
    a.release();
}

TEST_F(BigallocTest, testTooSmall)
{
    RdbAllocator a(rdb_bigalloc_new());
    rdb_BIGALLOC *ba = (rdb_BIGALLOC *)a._inner;

    size_t first_size = RDB_BIGALLOC_ALLOCSZ_MIN * 2;
    rdb_ROPESEG *seg = a.alloc(first_size);
    a.free(seg);

    ASSERT_EQ(1, LCB_CLIST_SIZE(&ba->bufs));
    ASSERT_EQ(1, ba->n_requests);
    ASSERT_EQ(0, ba->n_toobig);
    ASSERT_EQ(0, ba->n_toosmall);

    rdb_ROPESEG *newseg = a.alloc(first_size);
    ASSERT_EQ(seg, newseg);
    a.free(newseg);

    size_t smallsize = RDB_BIGALLOC_ALLOCSZ_MIN / 2;
    while (ba->n_requests < RDB_BIGALLOC_RECHECK_RATE - 1) {
        newseg = a.alloc(smallsize);
        a.free(newseg);
        ASSERT_EQ(seg, newseg); // same pooled one
        ASSERT_EQ(0, seg->nused);
        ASSERT_EQ(0, seg->start);
        memset(seg->root, '*', seg->nalloc);
        seg->start = seg->nalloc;
    }

    size_t oldmin = ba->min_blk_alloc, oldmax = ba->max_blk_alloc;
    // Get one more:
    newseg = a.alloc(smallsize);
    ASSERT_EQ(oldmin / 2, ba->min_blk_alloc);
    ASSERT_EQ(oldmax / 2, ba->max_blk_alloc);
    a.free(newseg);

    rdb_bigalloc_dump(ba, stdout);
    a.release();
}

TEST_F(BigallocTest, testPooled)
{
    RdbAllocator a(rdb_bigalloc_new());
    rdb_BIGALLOC *ba = (rdb_BIGALLOC *)a._inner;
    std::vector< rdb_ROPESEG * > segs;
    size_t allocsize = 1;

    for (unsigned ii = 0; ii < (ba->max_blk_count * 2); ii++) {
        rdb_ROPESEG *seg = a.alloc(allocsize);
        segs.push_back(seg);
    }
    rdb_bigalloc_dump(ba, stdout);

    for (unsigned ii = 0; ii < segs.size(); ii++) {
        a.free(segs[ii]);
    }

    segs.clear();
    rdb_bigalloc_dump(ba, stdout);
    ASSERT_EQ(RDB_BIGALLOC_BLKCNT_MAX, LCB_CLIST_SIZE(&ba->bufs));
    a.release();
}

TEST_F(BigallocTest, testRealloc)
{
    RdbAllocator a(rdb_bigalloc_new());
    rdb_ROPESEG *seg = a.alloc(5);
    // well, we'll see what we have here
    size_t cursize = seg->nalloc;
    seg = a.realloc(seg, cursize + 1);
    ASSERT_GT(seg->nalloc, cursize);
    a.free(seg);
    a.release();
}
