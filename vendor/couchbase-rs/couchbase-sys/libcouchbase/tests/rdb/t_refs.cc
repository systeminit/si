#include "rdbtest.h"
#include "list.h"

class RefTest : public ::testing::Test
{
};

TEST_F(RefTest, testLifecycle)
{
    IORope *ior = new IORope(rdb_chunkalloc_new(4));
    ior->feed("12345678");
    ASSERT_EQ(8, ior->usedSize());

    nb_IOV iovs[32];
    rdb_ROPESEG *segs[32];

    unsigned nseg = rdb_refread_ex(ior, iovs, segs, 32, 8);
    ASSERT_EQ(2, nseg);

    for (unsigned ii = 0; ii < nseg; ii++) {
        rdb_seg_ref(segs[ii]);
        ASSERT_NE(0, segs[ii]->shflags & RDB_ROPESEG_F_USER);
        ASSERT_NE(0, segs[ii]->shflags & RDB_ROPESEG_F_LIB);
        ASSERT_EQ(1, segs[ii]->refcnt);
    }

    // Ensure the memory allocated is removed.
    delete ior;

    ASSERT_EQ(0, memcmp(iovs[0].iov_base, "1234", 4));
    ASSERT_EQ(0, memcmp(iovs[1].iov_base, "5678", 4));
    ASSERT_NE(0, segs[0]->shflags & RDB_ROPESEG_F_USER);
    ASSERT_EQ(0, segs[0]->shflags & RDB_ROPESEG_F_LIB);
    rdb_seg_unref(segs[0]);
    rdb_seg_unref(segs[1]);
}

// Some more blahblah
TEST_F(RefTest, testCycle2)
{
    IORope *ior = new IORope(rdb_chunkalloc_new(10));

    // Read first 3 bytes
    ior->feed("1234567890A");
    ReadPacket rp(ior, 3);
    ASSERT_EQ(1, rp.segments.size());
    rp.refSegment(0);
    rdb_consumed(ior, 3);
    ASSERT_EQ(rp.asString(), "123");

    // Read another 3 bytes
    ReadPacket rp2(ior, 3);
    rp2.refSegment(0);
    rdb_consumed(ior, 3);
    ASSERT_EQ(rp2.asString(), "456");
    ASSERT_EQ(1, rp2.segments.size());

    ASSERT_EQ(rp.segments[0], rp2.segments[0]);

    // Allocate a third one, consuming the rest
    ReadPacket rp3(ior, 5);
    ASSERT_EQ(2, rp3.segments.size());
    ASSERT_EQ(rp.segments[0], rp3.segments[0]);
    ASSERT_EQ(rp3.asString(), "7890A");
    rp3.refSegment(1);
    rdb_consumed(ior, 5);

    delete ior;

    rp.unrefSegment(0);
    rp2.unrefSegment(0);
    rp3.unrefSegment(1);
}

// See what happens when we try to consolidate buffers as part of an already
// referenced segment.
TEST_F(RefTest, testRefConsolidate)
{
    IORope *ior = new IORope(rdb_chunkalloc_new(3));
    ior->feed("123456789");
    ReadPacket rp(ior, 3);
    rp.refSegment(0);
    rdb_consumed(ior, 3);
    ASSERT_EQ(RDB_ROPESEG_F_USER, rp.segments[0]->shflags);

    rdb_consolidate(ior, 6);
    ASSERT_NE(RDB_SEG_FIRST(&ior->recvd), rp.segments[0]);
    ASSERT_EQ(6, ior->recvd.nused);
    rdb_consumed(ior, 6);
    rp.unrefSegment(0);
    delete ior;

    // however we should be using the same segment otherwise..
    ior = new IORope(rdb_chunkalloc_new(3));
    ior->feed("123456789");
    ReadPacket rp2(ior, 3);
    char *p = rdb_get_consolidated(ior, 6);
    ASSERT_EQ(0, memcmp(p, "123456", 6));
    ASSERT_EQ(RDB_SEG_FIRST(&ior->recvd), rp2.segments[0]);
    ASSERT_EQ(9, ior->recvd.nused);
    delete ior;

    ior = new IORope(rdb_chunkalloc_new(6));
    ior->feed("123456789");
    ReadPacket rp3(ior, 6);
    rp3.refSegment(0);

    p = rdb_get_consolidated(ior, 9);
    ASSERT_EQ(0, memcmp(p, "123456789", 9));
    ASSERT_EQ(9, ior->recvd.nused);
    rp3.unrefSegment(0);
    delete ior;
}
