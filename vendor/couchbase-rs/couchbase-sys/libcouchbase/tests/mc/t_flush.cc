#include "mctest.h"
#include "mc/mcreq-flush-inl.h"

class McFlush : public ::testing::Test
{
};

struct MyCookie {
    int ncalled;
    void *exp_kbuf;
    MyCookie() : ncalled(0), exp_kbuf(NULL) {}
};

extern "C" {
static void buf_free_callback(mc_PIPELINE *, const void *cookie, void *kbuf, void *vbuf)
{
    MyCookie *ck = (MyCookie *)cookie;
    EXPECT_TRUE(kbuf == ck->exp_kbuf);
    ck->ncalled++;
}
}

TEST_F(McFlush, testBasicFlush)
{
    CQWrap cq;
    PacketWrap pw;

    cq.setBufFreeCallback(buf_free_callback);
    pw.setContigKey("1234");
    ASSERT_TRUE(pw.reservePacket(&cq));

    MyCookie cookie;
    cookie.exp_kbuf = pw.pktbuf;

    pw.setCookie(&cookie);
    pw.setHeaderSize();
    pw.copyHeader();
    mcreq_enqueue_packet(pw.pipeline, pw.pkt);
    mcreq_packet_handled(pw.pipeline, pw.pkt);

    nb_IOV iovs[10];

    unsigned toFlush = mcreq_flush_iov_fill(pw.pipeline, iovs, 10, NULL);
    EXPECT_EQ(28, toFlush);
    mcreq_flush_done(pw.pipeline, 8, toFlush);

    toFlush = mcreq_flush_iov_fill(pw.pipeline, iovs, 10, NULL);
    EXPECT_EQ(20, toFlush);
    mcreq_flush_done(pw.pipeline, toFlush, toFlush);

    toFlush = mcreq_flush_iov_fill(pw.pipeline, iovs, 10, NULL);
    ASSERT_EQ(0, toFlush);
    ASSERT_EQ(1, cookie.ncalled);
}

TEST_F(McFlush, testFlushedUnhandled)
{
    CQWrap cq;
    PacketWrap pw;
    cq.setBufFreeCallback(buf_free_callback);
    pw.setContigKey("1234");

    MyCookie cookie;
    cookie.exp_kbuf = pw.pktbuf;

    ASSERT_TRUE(pw.reservePacket(&cq));
    pw.setCookie(&cookie);
    pw.setHeaderSize();
    pw.copyHeader();

    mcreq_enqueue_packet(pw.pipeline, pw.pkt);

    nb_IOV iovs[10];
    unsigned toFlush = mcreq_flush_iov_fill(pw.pipeline, iovs, 10, NULL);
    ASSERT_EQ(28, toFlush);
    mcreq_flush_done(pw.pipeline, toFlush, toFlush);

    ASSERT_EQ(0, cookie.ncalled);
    ASSERT_NE(0, pw.pkt->flags & MCREQ_F_FLUSHED);

    ASSERT_EQ(pw.pkt, mcreq_pipeline_remove(pw.pipeline, pw.pkt->opaque));
    mcreq_packet_handled(pw.pipeline, pw.pkt);
    ASSERT_EQ(1, cookie.ncalled);
}

TEST_F(McFlush, testFlushCopy)
{
    CQWrap cq;
    PacketWrap pw;
    cq.setBufFreeCallback(buf_free_callback);
    pw.setCopyKey("Hello");
    ASSERT_TRUE(pw.reservePacket(&cq));

    MyCookie cookie;
    pw.setHeaderSize();
    pw.copyHeader();
    pw.setCookie(&cookie);
    mcreq_enqueue_packet(pw.pipeline, pw.pkt);

    nb_IOV iov[10];
    unsigned int toFlush = mcreq_flush_iov_fill(pw.pipeline, iov, 10, NULL);
    mcreq_flush_done(pw.pipeline, toFlush, toFlush);
    mcreq_pipeline_remove(pw.pipeline, pw.pkt->opaque);
    mcreq_packet_handled(pw.pipeline, pw.pkt);
    ASSERT_EQ(0, cookie.ncalled);
}

TEST_F(McFlush, testMultiFlush)
{
    CQWrap cq;
    int counter = 0;
    const int nitems = 10;
    MyCookie **cookies;

    cookies = new MyCookie *[nitems];
    PacketWrap **pws = new PacketWrap *[nitems];
    cq.setBufFreeCallback(buf_free_callback);

    for (int ii = 0; ii < nitems; ii++) {
        PacketWrap *pw = new PacketWrap;
        pws[ii] = pw;
        char curkey[128];
        sprintf(curkey, "Key_%d", ii);
        pw->setContigKey(curkey);

        cookies[ii] = new MyCookie;
        cookies[ii]->exp_kbuf = pw->pktbuf;

        ASSERT_TRUE(pw->reservePacket(&cq));

        pw->setCookie(cookies[ii]);
        mcreq_enqueue_packet(pw->pipeline, pw->pkt);
        pw->setHeaderSize();
        pw->copyHeader();
        mcreq_packet_handled(pw->pipeline, pw->pkt);
        mcreq_pipeline_remove(pw->pipeline, pw->pkt->opaque);
    }

    for (unsigned ii = 0; ii < cq.npipelines; ii++) {
        mc_PIPELINE *pipeline = cq.pipelines[ii];
        nb_IOV iov[10];
        unsigned toFlush = mcreq_flush_iov_fill(pipeline, iov, 10, NULL);
        if (toFlush) {
            mcreq_flush_done(pipeline, toFlush, toFlush);
        }
    }

    for (int ii = 0; ii < nitems; ii++) {
        ASSERT_EQ(1, cookies[ii]->ncalled);
        delete cookies[ii];
        delete pws[ii];
    }
    delete[] cookies;
    delete[] pws;
}

TEST_F(McFlush, testPartialFlush)
{
    CQWrap cq;
    PacketWrap pw;
    MyCookie cookie;

    cq.setBufFreeCallback(buf_free_callback);
    pw.setContigKey("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    ASSERT_TRUE(pw.reservePacket(&cq));
    pw.setCookie(&cookie);
    cookie.exp_kbuf = pw.pktbuf;
    pw.setHeaderSize();
    pw.copyHeader();
    mcreq_enqueue_packet(pw.pipeline, pw.pkt);

    nb_IOV iov[1];
    unsigned int toFlush = 0;
    do {
        toFlush = mcreq_flush_iov_fill(pw.pipeline, iov, 1, NULL);
        if (toFlush) {
            mcreq_flush_done(pw.pipeline, 1, toFlush);
        }
    } while (toFlush > 0);

    ASSERT_NE(0, pw.pkt->flags & MCREQ_F_FLUSHED);
    mcreq_pipeline_remove(pw.pipeline, pw.pkt->opaque);
    mcreq_packet_handled(pw.pipeline, pw.pkt);
    ASSERT_EQ(1, cookie.ncalled);
}
