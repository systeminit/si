#include "socktest.h"
#include <lcbio/manager.h>
using namespace LCBTest;
using std::string;
using std::vector;
class SockMgrTest : public SockTest
{
    void SetUp()
    {
        SockTest::SetUp();
        loop->sockpool->get_options().maxidle = 2;
        loop->sockpool->get_options().tmoidle = LCB_MS2US(2000);
    }
};

TEST_F(SockMgrTest, testBasic)
{
    ESocket *sock1 = new ESocket();
    loop->connectPooled(sock1);
    lcbio_SOCKET *rawsock = sock1->sock;
    delete sock1;
    ESocket *sock2 = new ESocket();
    loop->connectPooled(sock2);
    ASSERT_EQ(rawsock, sock2->sock);

    ESocket *sock3 = new ESocket();
    loop->connectPooled(sock3);
    ASSERT_NE(rawsock, sock3->sock);
    delete sock3;
    delete sock2;
}

TEST_F(SockMgrTest, testCancellation)
{
    lcb_host_t host = {0};
    loop->populateHost(&host);
    lcb::io::ConnectionRequest *req = loop->sockpool->get(host, LCB_MS2US(1000), NULL, NULL);
    ASSERT_FALSE(req == NULL);
    req->cancel();
    loop->sockpool->get_options().tmoidle = LCB_MS2US(2);
    loop->start();
}

// See if a connection closed when it was idle is returned or not!
TEST_F(SockMgrTest, testIdleClosed)
{
    lcb_socket_t llfd;
    ESocket *sock1 = new ESocket();
    loop->connectPooled(sock1);
    TestConnection *tc = sock1->conn;

    if (loop->iot->model == LCB_IOMODEL_EVENT) {
        llfd = sock1->ctx->sock->u.fd;
    } else {
        llfd = sock1->ctx->sock->u.sd->socket;
    }

    // Wait unitl it's closed by the server
    CloseFuture cf(CloseFuture::BEFORE_IO);
    tc->setClose(&cf);
    cf.wait();

    delete sock1;
    // Since shutdown() is TCP level, while send/recv are socket level, we
    // might need to loop a bit until we get something!
    int rv = -1, attempts = 0;
    do {
        char buf = 0;
        rv = recv(llfd, &buf, 1, 0);
    } while (rv != 0 && ++attempts);

    if (attempts) {
        fprintf(stderr, "Needed to loop more than once! (%d times)\n", attempts);
    }

    ESocket *sock2 = new ESocket();
    loop->connectPooled(sock2);

    // We should be able to perform IO on this socket. Write a few bytes
    string msg("Hello World!");
    RecvFuture rf(msg.size());
    FutureBreakCondition fbc(&rf);

    sock2->conn->setRecv(&rf);
    sock2->put(msg);
    sock2->schedule();

    loop->setBreakCondition(&fbc);
    loop->start();
    rf.wait();
    ASSERT_TRUE(rf.isOk());
    delete sock2;
}

struct PCtxDummy : lcbio_PROTOCTX {
    int *cVar;
    bool invoked;
    bool shouldDelete;
};
extern "C" {
static void protoctx_dtor(lcbio_PROTOCTX *ctx)
{
    PCtxDummy *d = (PCtxDummy *)ctx;
    if (d->shouldDelete) {
        delete d;
    } else {
        *d->cVar += 1;
        d->invoked = true;
    }
}
}

TEST_F(SockMgrTest, testMaxIdle)
{
    // Assuming maxidle=2 as defined in the beginning
    int destroyCount = 0;
    ESocket *socks[4];
    PCtxDummy *ctxs[4];

    for (int ii = 0; ii < 4; ii++) {
        ESocket *s = new ESocket();
        PCtxDummy *pctx = new PCtxDummy();

        pctx->id = LCBIO_PROTOCTX_MAX;
        pctx->dtor = protoctx_dtor;
        pctx->cVar = &destroyCount;
        pctx->invoked = false;

        loop->connectPooled(s);
        lcbio_protoctx_add(s->sock, pctx);

        socks[ii] = s;
        ctxs[ii] = pctx;
    }

    // Assume they're all alive. Now delete them
    for (int ii = 0; ii < 4; ii++) {
        delete socks[ii];
    }

    // We need
    ASSERT_EQ(2, destroyCount);
    for (int ii = 0; ii < 4; ii++) {
        if (ctxs[ii]->invoked) {
            delete ctxs[ii];
        } else {
            ctxs[ii]->shouldDelete = true;
        }
    }

    // Ensure subsequent connections can also work!
    ESocket *otherSocks[8];
    for (int ii = 0; ii < 8; ii++) {
        ESocket *s = new ESocket();
        loop->connectPooled(s);
        otherSocks[ii] = s;
        ASSERT_TRUE(s->sock != NULL);
    }
    for (int ii = 0; ii < 8; ii++) {
        delete otherSocks[ii];
    }
}
