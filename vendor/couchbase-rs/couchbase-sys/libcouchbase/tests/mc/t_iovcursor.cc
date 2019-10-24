#include <stdio.h>
#include "mctest.h"
#include "mc/iovcursor-inl.h"
#include "mc/forward.h"
class McIOVC : public ::testing::Test
{
};

TEST_F(McIOVC, testPeekCopy)
{
    nb_IOV iov;
    iov.iov_base = (void *)"ABCDEF";
    iov.iov_len = strlen((const char *)iov.iov_base);
    mc_IOVINFO cursor;
    mc_IOVCURSOR *mincur = &cursor.c;
    mc_iovinfo_init(&cursor, &iov, 1);

    // basic sanity
    ASSERT_EQ(0, mincur->offset);
    ASSERT_EQ(6, cursor.total);
    ASSERT_EQ(1, mincur->niov);
    ASSERT_EQ(&iov, mincur->iov);

    char buf[256] = {0};
    // test basic peek
    iovcursor_peek(mincur, buf, 3, 0);
    ASSERT_STREQ("ABC", buf);

    memset(buf, 0, sizeof buf);
    iovcursor_peek(mincur, buf, 3, 3);
    ASSERT_STREQ("DEF", buf);

    memset(buf, 0, sizeof buf);
    iovcursor_peek(mincur, buf, 1, 5);
    ASSERT_STREQ("F", buf);
}

TEST_F(McIOVC, testPeekEx)
{
    nb_IOV iov[3];
    iov[0].iov_base = (void *)"ABC";
    iov[0].iov_len = 3;
    iov[1].iov_base = (void *)"DEF";
    iov[1].iov_len = 3;
    iov[2].iov_base = (void *)"GHI";
    iov[2].iov_len = 3;

    iovcursor_STATUS status;
    mc_IOVINFO cursor;
    mc_IOVCURSOR *mincur = &cursor.c;
    mc_iovinfo_init(&cursor, iov, 3);

    const char *contigptr;
    char cptgt[256] = {0};

    // Simple case
    status = iovcursor_peek_ex(mincur, cptgt, NULL, 9, 0);
    ASSERT_EQ(IOVCURSOR_STATUS_BUFCOPY_OK, status);
    ASSERT_STREQ("ABCDEFGHI", cptgt);

    memset(cptgt, 0, sizeof cptgt);
    status = iovcursor_peek_ex(mincur, cptgt, &contigptr, 9, 0);
    ASSERT_EQ(IOVCURSOR_STATUS_BUFCOPY_OK, status);
    ASSERT_STREQ("ABCDEFGHI", cptgt);
    ASSERT_TRUE(contigptr == NULL);

    status = iovcursor_peek_ex(mincur, NULL, &contigptr, 9, 0);
    ASSERT_EQ(IOVCURSOR_STATUS_FRAGMENTED, status);
    ASSERT_TRUE(contigptr == NULL);

    // Contiguous pointer
    memset(cptgt, 0, sizeof cptgt);
    status = iovcursor_peek_ex(mincur, NULL, &contigptr, 3, 0);
    ASSERT_EQ(IOVCURSOR_STATUS_CONTIGPTR_OK, status);
    ASSERT_EQ('\0', cptgt[0]);
    memcpy(cptgt, contigptr, 3);
    ASSERT_STREQ("ABC", cptgt);

    // With offsets: CDE
    contigptr = (char *)0x1;
    memset(cptgt, 0, sizeof cptgt);
    status = iovcursor_peek_ex(mincur, cptgt, &contigptr, 3, 2);
    ASSERT_EQ(IOVCURSOR_STATUS_BUFCOPY_OK, status);
    ASSERT_STREQ("CDE", cptgt);
    ASSERT_TRUE(contigptr == NULL);
}

TEST_F(McIOVC, testAdvCopy)
{
    nb_IOV iov[3];
    iov[0].iov_base = (void *)"ABC";
    iov[0].iov_len = 3;
    iov[1].iov_base = (void *)"DEF";
    iov[1].iov_len = 3;
    iov[2].iov_base = (void *)"GHI";
    iov[2].iov_len = 3;
    mc_IOVINFO cursor;
    mc_IOVCURSOR *mincur = &cursor.c;

    mc_iovinfo_init(&cursor, iov, 3);
    char tgt[256];

    iovcursor_adv_copy(mincur, tgt, 1);
    ASSERT_EQ('A', tgt[0]);
    ASSERT_EQ(1, mincur->offset);
    memset(tgt, 0, sizeof tgt);

    iovcursor_adv_copy(mincur, tgt, 2);
    ASSERT_STREQ("BC", tgt);
    ASSERT_EQ(0, mincur->offset);
    ASSERT_EQ(2, mincur->niov);

    // Reset the iovs
    memset(tgt, 0, sizeof tgt);
    mc_iovinfo_init(&cursor, iov, 3);
    iovcursor_adv_copy(mincur, tgt, 4);
    ASSERT_STREQ("ABCD", tgt);
    ASSERT_EQ(2, mincur->niov);
    ASSERT_EQ(1, mincur->offset);

    // Read the rest?
    memset(tgt, 0, sizeof tgt);
    iovcursor_adv_copy(mincur, tgt, 5);
    ASSERT_STREQ("EFGHI", tgt);
    ASSERT_EQ(0, mincur->niov);
    ASSERT_EQ(0, mincur->offset);
}

TEST_F(McIOVC, testAdvIovalloc)
{
    nb_IOV *iov_p;
    unsigned niov;

    nb_IOV iov[5];
    iov[0].iov_base = (void *)"ABC";
    iov[0].iov_len = 3;

    iov[1].iov_base = (void *)"DEF";
    iov[1].iov_len = 3;

    iov[2].iov_base = (void *)"GHI";
    iov[2].iov_len = 3;

    iov[3].iov_base = (void *)"JKL";
    iov[3].iov_len = 3;

    iov[4].iov_base = (void *)"MNO";
    iov[4].iov_len = 3;

    mc_IOVINFO cursor;
    mc_IOVCURSOR *mincur = &cursor.c;
    mc_iovinfo_init(&cursor, iov, 5);

    char tgt[256] = {0};
    // Copy the first 4 fragments
    iovcursor_adv_copy(mincur, tgt, 4);
    ASSERT_STREQ("ABCD", tgt);

    // Now the IOVs payload: { EF, GHI, JK }
    iovcursor_adv_iovalloc(mincur, 7, &iov_p, &niov);
    ASSERT_FALSE(iov_p == NULL);
    ASSERT_EQ(3, niov);

    // make a small cursor based on the current one, why not?
    mc_IOVCURSOR mini;
    mini.offset = 0;
    mini.iov = iov_p;
    mini.niov = niov;
    memset(tgt, 0, sizeof tgt);
    iovcursor_adv_copy(&mini, tgt, 7);
    ASSERT_STREQ("EFGHIJK", tgt);
    free(iov_p);
}
