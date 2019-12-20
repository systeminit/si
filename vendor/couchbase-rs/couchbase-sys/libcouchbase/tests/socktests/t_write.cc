#include "socktest.h"
using namespace LCBTest;
using std::string;
using std::vector;
class SockWriteTest : public SockTest
{
};

/**
 * This file tests the ability of the sockets to write various pieces
 * of data.
 */
TEST_F(SockWriteTest, testMultiWrite)
{
    ESocket sock;
    loop->connect(&sock);

    string expected("Hello World!");
    RecvFuture rf(expected.size());
    FutureBreakCondition wbc(&rf);

    sock.put("Hello ");
    sock.schedule();
    sock.put("World");
    sock.schedule();
    sock.put("!");
    sock.schedule();
    sock.schedule();

    sock.conn->setRecv(&rf);

    loop->setBreakCondition(&wbc);
    loop->start();
    rf.wait();
    ASSERT_TRUE(rf.isOk());
    ASSERT_EQ(expected, rf.getString());
}

/**
 * Test with a very big write
 */
TEST_F(SockWriteTest, testBigWrite)
{
    ESocket sock;
    loop->connect(&sock);
    string expected(1024 * 1024 * 2, '*');
    RecvFuture rf(expected.size());
    sock.conn->setRecv(&rf);
    sock.put(expected);
    sock.schedule();

    FutureBreakCondition wbc(&rf);
    loop->setBreakCondition(&wbc);
    loop->start();
    rf.wait();
    ASSERT_TRUE(rf.isOk());
    ASSERT_EQ(expected, rf.getString());
}

/**
 * Write to a broken socket. Because close is not synchronous on both ends
 * of the connection (even though in this case they are on the same host)
 * we need to loop until we get an error.
 */
TEST_F(SockWriteTest, testBrokenFirstWrite)
{
    ESocket sock;
    loop->connect(&sock);

    while (sock.lasterr == LCB_SUCCESS) {
        CloseFuture cf(CloseFuture::BEFORE_IO);
        FlushedBreakCondition fbc(&sock);
        sock.conn->setClose(&cf);
        sock.put("This should fail");
        sock.schedule();
        loop->setBreakCondition(&fbc);
        loop->start();
        cf.wait();
    }
}

TEST_F(SockWriteTest, testBrokenMultiWrites)
{
    ESocket sock;
    loop->connect(&sock);
    while (sock.lasterr == LCB_SUCCESS) {
        CloseFuture cf(CloseFuture::BEFORE_IO);
        FlushedBreakCondition fbc(&sock);
        sock.conn->setClose(&cf);

        for (int ii = 0; ii < 100; ii++) {
            sock.put("This message should fail");
            sock.schedule();
        }
        loop->setBreakCondition(&fbc);
        loop->start();
        cf.wait();
    }
}
