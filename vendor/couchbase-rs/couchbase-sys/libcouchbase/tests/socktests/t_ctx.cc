#include "socktest.h"
using namespace LCBTest;
using std::string;
using std::vector;
class SockCtxTest : public SockTest
{
};

/**
 * While some of the previous tests also used the 'Easy' context implicitly,
 * this will try to test some of the more advanced free/destroy functionality.
 *
 * This mainly relates to freeing the context and inspecting whether any
 * callbacks are invoked afterwards.
 */

TEST_F(SockCtxTest, testClose)
{
    ESocket sock;

    loop->connect(&sock);
    for (unsigned ii = 0; ii < 100; ++ii) {
        sock.put("Hi");
        sock.schedule();
    }

    CtxCloseBreakCondition cbc(&sock);
    cbc.closeCtx();
    loop->setBreakCondition(&cbc);
    loop->start();
}

struct ReleaseInfo {
    lcbio_SOCKET *sock;
    bool reusable;
    ReleaseInfo()
    {
        sock = NULL;
        reusable = false;
    }
    void reset()
    {
        sock = NULL;
        reusable = false;
    }
};

extern "C" {
static void release_cb(lcbio_SOCKET *s, int reusable, void *arg)
{
    ReleaseInfo *info = (ReleaseInfo *)arg;
    if (reusable) {
        info->sock = s;
        lcbio_ref(s);
    }
    info->reusable = !!reusable;
}
}

TEST_F(SockCtxTest, testReleasable)
{
    ESocket sock;
    loop->connect(&sock);
    ReleaseInfo ri;
    // Release the socket
    lcbio_ctx_close(sock.ctx, release_cb, &ri);
    sock.clear();
    ASSERT_TRUE(ri.reusable);

    // Schedule some events on it. It should not be releaseable
    sock.assign(ri.sock, LCB_SUCCESS);
    sock.put("Hi!");
    sock.schedule();
    ri.reset();
    lcbio_ctx_close(sock.ctx, release_cb, &ri);
    lcbio_unref(sock.sock);
    sock.clear();
    ASSERT_FALSE(ri.reusable);
}
