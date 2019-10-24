#include "socktest.h"
using namespace LCBTest;
using std::string;
using std::vector;

class SockReentrantTest : public SockTest
{
};

/**
 * This file tests various reentrant actions within the socket handlers.
 */
class ReadAgainAction : public IOActions
{
  public:
    ReadAgainAction()
    {
        callCount = 0;
        sf = NULL;
    }

    virtual ~ReadAgainAction() {}

    int callCount;
    SendFuture *sf;
    void onRead(ESocket *s, size_t nr)
    {
        if (callCount++) {
            s->parent->stop();
            return;
        }

        s->reqrd(nr * 2);
        sf = new SendFuture(string(nr, '$'));
        s->conn->setSend(sf);
        s->schedule();
    }
    void onError(ESocket *)
    {
        // do nothing
    }
};

class CallCountBreakCondition : public BreakCondition
{
  public:
    ReadAgainAction *raa;
    CallCountBreakCondition(ReadAgainAction *action)
    {
        raa = action;
    }

  protected:
    bool shouldBreakImpl()
    {
        return raa->callCount >= 2;
    }
};

TEST_F(SockReentrantTest, testReadAgain)
{
    ESocket sock;
    loop->connect(&sock);
    SendFuture sf1(string(100, '#'));
    sock.conn->setSend(&sf1);
    sock.reqrd(100);
    sock.schedule();

    ReadAgainAction raa;
    CallCountBreakCondition bc(&raa);
    sock.setActions(&raa);
    loop->setBreakCondition(&bc);
    loop->start();
    ASSERT_TRUE(raa.callCount == 2);
    ASSERT_TRUE(sock.getUnreadSize() >= 200);
    ASSERT_TRUE(raa.sf != NULL);
    raa.sf->wait();
    delete raa.sf;
}

class CloseReadAction : public IOActions
{
  public:
    CloseReadAction()
    {
        wasCalled = false;
    }
    virtual ~CloseReadAction() {}
    void onRead(ESocket *s, size_t)
    {
        EXPECT_FALSE(wasCalled);
        wasCalled = true;
        s->parent->stop();
        s->close();
    }
    void onError(ESocket *) {}
    bool wasCalled;
};

class CRABreakCondition : public BreakCondition
{
  public:
    CloseReadAction *cra;
    CRABreakCondition(CloseReadAction *action)
    {
        cra = action;
    }

  protected:
    bool shouldBreakImpl()
    {
        return cra->wasCalled;
    }
};

TEST_F(SockReentrantTest, testCloseOnRead)
{
    ESocket sock;
    loop->connect(&sock);
    SendFuture sf(string(100, '#'));
    sock.conn->setSend(&sf);
    sock.reqrd(1);
    sock.schedule();
    CloseReadAction cra;
    CRABreakCondition bc(&cra);
    sock.setActions(&cra);
    loop->setBreakCondition(&bc);
    loop->start();
    sf.wait();
    ASSERT_TRUE(cra.wasCalled);
}

class CloseWriteAction : public IOActions
{
  public:
    bool wasCalled;
    CloseWriteAction()
    {
        wasCalled = false;
    }
    virtual ~CloseWriteAction() {}
    void onRead(ESocket *s, size_t)
    {
        EXPECT_FALSE(wasCalled);
        wasCalled = true;
        for (int ii = 0; ii < 100; ii++) {
            s->put("Hello!");
            s->schedule();
        }
        s->close();
        s->parent->stop();
    }
    void onError(ESocket *) {}
};
TEST_F(SockReentrantTest, testCloseOnWrite)
{
    ESocket sock;
    loop->connect(&sock);
    SendFuture sf(string(100, '#'));
    sock.conn->setSend(&sf);
    sock.reqrd(1);
    sock.schedule();
    CloseWriteAction cwa;
    sock.setActions(&cwa);

    RecvFuture rf(1);
    FutureBreakCondition fbc(&rf);
    loop->setBreakCondition(&fbc);
    loop->start();
    sf.wait();
    ASSERT_TRUE(cwa.wasCalled);
}
