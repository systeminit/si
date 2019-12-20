#include "config.h"
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include "n1ql/n1ql-internal.h"

class N1qLStringTests : public ::testing::Test
{
};

TEST_F(N1qLStringTests, testParseTimeout)
{
    ASSERT_EQ(1500000, lcb_n1qlreq_parsetmo("1.5s"));
    ASSERT_EQ(1500000, lcb_n1qlreq_parsetmo("1500ms"));
    ASSERT_EQ(1500000, lcb_n1qlreq_parsetmo("1500000us"));
    ASSERT_EQ(0, lcb_n1qlreq_parsetmo("blahblah"));
    ASSERT_EQ(0, lcb_n1qlreq_parsetmo("124"));
    ASSERT_EQ(0, lcb_n1qlreq_parsetmo("99z"));
}
