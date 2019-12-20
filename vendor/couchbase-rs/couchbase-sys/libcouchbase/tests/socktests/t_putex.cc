#include "socktest.h"
#include <netbuf/netbuf.h>
#include <algorithm>
using namespace LCBTest;
using std::list;
using std::string;
using std::vector;

/**
 * These tests cover the various lcbio_PutEx() routines.
 */

struct WriteBuffer {
    sllist_node ll;
    char *buf;
    const size_t length;
    bool flushed;

    WriteBuffer(const std::string &s) : length(s.size())
    {
        buf = new char[length];
        memcpy(buf, s.data(), length);
        flushed = false;
    }

    ~WriteBuffer()
    {
        delete[] buf;
    }
};

extern "C" {
static nb_SIZE pdu_callback(void *pdu, nb_SIZE remaining, void *arg)
{
    WriteBuffer *wb = (WriteBuffer *)pdu;

    *(size_t *)arg += 1;
    if (wb->length <= remaining) {
        wb->flushed = true;
    }
    return wb->length;
}
}

struct BufList {
    nb_MGR mgr;
    BufList()
    {
        netbuf_init(&mgr, NULL);
    }

    ~BufList()
    {
        netbuf_cleanup(&mgr);
    }

    vector< lcb_IOV > getIOV(size_t *nbytes)
    {
        nb_IOV iovs[32];
        vector< lcb_IOV > ret;
        int niov = 0;
        *nbytes = netbuf_start_flush(&mgr, iovs, 32, &niov);
        for (int ii = 0; ii < niov; ii++) {
            lcb_IOV cur;
            cur.iov_base = iovs[ii].iov_base;
            cur.iov_len = iovs[ii].iov_len;
            ret.push_back(cur);
        }
        return ret;
    }

    void updateFlushed(size_t expected, unsigned n)
    {
        size_t pduCount = 0;
        netbuf_end_flush2(&mgr, n, pdu_callback, 0, &pduCount);
        size_t nremoved = 0;
        while (!bufs.empty()) {
            WriteBuffer *curBuf = bufs.front();
            if (curBuf->flushed) {
                bufs.pop_front();
                delete curBuf;
                nremoved++;
            } else {
                break;
            }
        }

        if (expected != n) {
            netbuf_reset_flush(&mgr);
        }
    }

    void append(const std::string &s)
    {
        WriteBuffer *wb = new WriteBuffer(s);
        nb_SPAN span;
        CREATE_STANDALONE_SPAN(&span, wb->buf, wb->length);
        netbuf_enqueue_span(&mgr, &span, NULL);
        netbuf_pdu_enqueue(&mgr, wb, 0);
        bufs.push_back(wb);
    }

    list< WriteBuffer * > bufs;
};

class BufActions : public IOActions
{
  public:
    unsigned totalFlushed;
    BufActions()
    {
        totalFlushed = 0;
    }
    BufList buflist;
    void onFlushReady(ESocket *s)
    {
        int ready = 0;
        size_t nbytes;

        do {
            vector< lcb_IOV > iovs = buflist.getIOV(&nbytes);
            if (!nbytes) {
                break; // nothing left to flush
            }
            ready = lcbio_ctx_put_ex(s->ctx, &iovs[0], iovs.size(), nbytes);
        } while (ready);

        if (nbytes) {
            lcbio_ctx_wwant(s->ctx);
            s->schedule();
        }
    }

    void onFlushDone(ESocket *s, size_t expected, size_t nr)
    {
        totalFlushed += nr;
        buflist.updateFlushed(expected, nr);
    }
};

class MyBreakCondition : public BreakCondition
{
  public:
    BufList *bl;
    RecvFuture *rf;
    MyBreakCondition(BufList *buflist, RecvFuture *ft)
    {
        bl = buflist;
        rf = ft;
    }

  protected:
    /** break only after we're fully flushed and all the data's been received */
    bool shouldBreakImpl()
    {
        return rf->checkDone() && bl->bufs.empty();
    }
};

class SockPutexTest : public SockTest
{
  public:
    ESocket sock;
    BufActions bufActions;
    BufList *buflist;
    void SetUp()
    {
        SockTest::SetUp();
        sock.setActions(&bufActions);
        loop->connect(&sock);
        buflist = &bufActions.buflist;
    }
    void TearDown()
    {
        sock.close();
        SockTest::TearDown();
    }
};

TEST_F(SockPutexTest, testBasic)
{
    RecvFuture rf(100);
    for (int ii = 0; ii < 100; ii++) {
        buflist->append("@");
    }

    sock.conn->setRecv(&rf);
    lcbio_ctx_wwant(sock.ctx);
    sock.schedule();
    MyBreakCondition mbc(buflist, &rf);
    loop->setBreakCondition(&mbc);
    loop->start();
    rf.wait();
    string expected(100, '@');
    ASSERT_EQ(rf.getString(), expected);
}

TEST_F(SockPutexTest, testBig)
{
    const size_t rchunk = 1000, niters = 1000, expected = rchunk * niters;

    // fill up the write buffers
    for (int ii = 0; ii < niters; ii++) {
        buflist->append(string(rchunk, '#'));
    }

    size_t nconsumed = 0;
    size_t nbufsOrig = buflist->bufs.size();

    /**
     * Iterate until all the buffers have been flushed and returned to the
     * caller, and until they have all been received
     */
    while (buflist->bufs.empty() == false || nconsumed != expected) {
        RecvFuture rf(100);
        FutureBreakCondition fbc(&rf);

        if (nconsumed != expected) {
            rf.reinit(std::min(rchunk, expected - nconsumed));
            sock.conn->setRecv(&rf);
            loop->setBreakCondition(&fbc);
        }

        if (bufActions.totalFlushed != expected) {
            // Make the tests run quicker and only send data when it's not
            // flushed
            lcbio_ctx_wwant(sock.ctx);
            sock.schedule();
            loop->start();
        }

        if (nconsumed != expected) {
            rf.wait();
            ASSERT_TRUE(rf.isOk());
            loop->setBreakCondition(NULL);
            nconsumed += rf.getBuf().size();
        }
        if (bufActions.totalFlushed == expected) {
            assert(buflist->bufs.empty());
        }
    }

    ASSERT_TRUE(buflist->bufs.empty());
}

class TClosedBreakCondition : public BreakCondition
{
  public:
    ESocket *s;
    BufList *bl;
    TClosedBreakCondition(ESocket *sock, BufList *buflist)
    {
        s = sock;
        bl = buflist;
    }

  protected:
    bool shouldBreakImpl()
    {
        return bl->bufs.empty();
    }
};
TEST_F(SockPutexTest, testClosed)
{
    sock.conn->close();
    TClosedBreakCondition tcb(&sock, buflist);

    while (sock.lasterr == LCB_SUCCESS) {
        buflist->append(string(100, '$'));
        lcbio_ctx_wwant(sock.ctx);
        sock.schedule();
        loop->setBreakCondition(&tcb);
        loop->start();
    }

    ASSERT_TRUE(buflist->bufs.empty());
}
