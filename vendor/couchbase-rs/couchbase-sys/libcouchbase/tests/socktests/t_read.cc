#include "socktest.h"
using namespace LCBTest;
using std::string;
using std::vector;
class SockReadTest : public SockTest
{
};
namespace
{

/**
 * Set a specific 'rdwant' value. Send data smaller than the want, and then
 * send some more data.
 */
TEST_F(SockReadTest, testWant)
{
    ESocket sock;
    loop->connect(&sock);
    string expected("Hello World!");
    SendFuture sf(expected);

    ReadBreakCondition cond(&sock, expected.size());

    sock.reqrd(expected.size() * 2);
    sock.schedule();
    sock.conn->setSend(&sf);
    loop->setBreakCondition(&cond);
    loop->start();
    sf.wait();
    ASSERT_TRUE(cond.didBreak());

    // If everything is right, we should have nothing inside the buffer
    ASSERT_TRUE(sock.getReceived().empty());

    SendFuture sf2(expected);
    sock.conn->setSend(&sf2);
    cond = ReadBreakCondition(&sock, expected.size() * 2);
    loop->setBreakCondition(&cond);
    loop->start();
    sf2.wait();
    expected += expected;
    ASSERT_FALSE(sock.getReceived().empty());
    ASSERT_EQ(expected, sock.getReceived());
}

// Ensure the 'rdwant' flag is reset when we invoke the callback
TEST_F(SockReadTest, testWantReset)
{
    ESocket sock;
    loop->connect(&sock);
    string expected("Hi!!!");
    sock.reqrd(expected.size());
    SendFuture sf(expected);
    ReadBreakCondition rbc(&sock, expected.size());
    loop->setBreakCondition(&rbc);
    ASSERT_EQ(expected.size(), sock.ctx->rdwant);
    sock.schedule();
    sock.conn->setSend(&sf);
    loop->start();
    ASSERT_EQ(0, sock.ctx->rdwant);
}

/**
 * We should get an error if the socket is closed before we have data
 */
TEST_F(SockReadTest, testBrokenRead)
{
    ESocket sock;
    loop->connect(&sock);
    CloseFuture cf(CloseFuture::BEFORE_IO);
    sock.conn->setClose(&cf);
    sock.reqrd(5000);
    sock.schedule();
    ErrorBreakCondition ebc(&sock);
    loop->setBreakCondition(&ebc);
    loop->start();
    cf.wait();
    ASSERT_TRUE(sock.lasterr == LCB_NETWORK_ERROR || sock.lasterr == LCB_ESOCKSHUTDOWN);
}

TEST_F(SockReadTest, testReadAhead)
{
    ESocket sock;
    loop->connect(&sock);
    string sendStr(200, '$');
    unsigned wantSize = sendStr.size() / 2;
    SendFuture sf(sendStr);
    ReadBreakCondition rbc(&sock, wantSize);

    sock.reqrd(wantSize);
    sock.conn->setSend(&sf);
    sock.schedule();

    loop->setBreakCondition(&rbc);
    loop->start();
    sf.wait();

    ASSERT_TRUE(sock.getReceived().size() >= wantSize);
    if (sock.getReceived().size() == wantSize) {
        fprintf(stderr, "!!! received exactly wantsize. Slow network?\n");
    }
}

/**
 * Test the behavior of an orderly close where all the required data is
 * consumed.
 */
TEST_F(SockReadTest, testOrderlyClose)
{
    ESocket sock;
    loop->connect(&sock);
    string expected(200, '$');

    SendFuture sf(expected);
    ReadBreakCondition rbc(&sock, expected.size());
    CloseFuture cf(CloseFuture::AFTER_IO);

    sock.conn->setSend(&sf);
    sock.conn->setClose(&cf);
    loop->setBreakCondition(&rbc);
    sock.reqrd(expected.size());
    sock.schedule();
    loop->start();

    cf.wait();
    sf.wait();

    ASSERT_EQ(expected, sock.getReceived());
}

class ChunkReadActions : public IOActions
{
  public:
    int numChunks;
    vector< char > buffer;
    void onRead(ESocket *s, size_t nr)
    {
        lcbio_CTXRDITER iter;
        LCBIO_CTX_ITERFOR(s->ctx, &iter, nr)
        {
            unsigned nbuf = lcbio_ctx_risize(&iter);
            void *buf = lcbio_ctx_ribuf(&iter);
            numChunks++;
            buffer.insert(buffer.end(), (char *)buf, (char *)buf + nbuf);
        }
    }

    ChunkReadActions() : IOActions()
    {
        numChunks = 0;
    }
};

class CRABreakCondition : public BreakCondition
{
  public:
    ChunkReadActions *cra;
    int expected;
    CRABreakCondition(ChunkReadActions *actions, int exp)
    {
        cra = actions;
        expected = exp;
    }

    bool shouldBreakImpl()
    {
        return cra->numChunks >= expected;
    }
};
/** Tests the iterator chunking mechanism */
TEST_F(SockReadTest, testChunkedIter)
{
    // Set the chunked allocator
    ESocket sock;
    loop->connect(&sock);
    rdb_challoc(&sock.ctx->ior, rdb_chunkalloc_new(1));

    string toSend(20, '+');
    SendFuture sf(toSend);
    sock.conn->setSend(&sf);
    ChunkReadActions cra;
    CRABreakCondition bc(&cra, 20);

    sock.setActions(&cra);
    sock.reqrd(toSend.size());
    sock.schedule();
    loop->setBreakCondition(&bc);
    loop->start();

    ASSERT_EQ(toSend.size(), cra.numChunks);
    ASSERT_EQ(toSend.size(), cra.buffer.size());
    ASSERT_EQ(toSend, string(cra.buffer.begin(), cra.buffer.end()));
    sf.wait();
}
} // namespace
