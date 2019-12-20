/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *       http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

#include "socktest.h"

#ifndef LCB_NO_SSL

#include <lcbio/ssl.h>
using namespace LCBTest;
using std::string;
using std::vector;

class SSLTest : public SockTest
{
  protected:
    void SetUp()
    {
        lcbio_ssl_global_init();
        lcb_STATUS errp = LCB_SUCCESS;
        // Initialize the SSL stuff

        SockTest::SetUp();
        loop->settings->sslopts = LCB_SSL_ENABLED | LCB_SSL_NOVERIFY;
        loop->settings->ssl_ctx = lcbio_ssl_new(NULL, NULL, NULL, 1, &errp, loop->settings);
        loop->server->factory = TestServer::sslSocketFactory;
        EXPECT_FALSE(loop->settings->ssl_ctx == NULL) << lcb_strerror(NULL, errp);
    }

    void TearDown()
    {
        lcbio_ssl_free(loop->settings->ssl_ctx);
        loop->settings->ssl_ctx = NULL;
        SockTest::TearDown();
    }
};

TEST_F(SSLTest, testBasic)
{
    // Copy/pasted from SockConnTest::testBasic

    ESocket sock;

    // We can connect
    loop->connect(&sock);
    ASSERT_FALSE(sock.sock == NULL);
    ASSERT_TRUE(sock.creq == NULL);
    ASSERT_EQ(1, sock.sock->refcount);

    // We can send data
    string sendStr("Hello World");
    RecvFuture rf(sendStr.size());
    FutureBreakCondition wbc(&rf);

    sock.conn->setRecv(&rf);
    sock.put(sendStr);
    sock.schedule();
    loop->setBreakCondition(&wbc);
    loop->start();
    rf.wait();
    ASSERT_TRUE(rf.isOk());
    ASSERT_EQ(rf.getString(), sendStr);

    // We can receive data
    string recvStr("Goodbye World!");
    SendFuture sf(recvStr);
    ReadBreakCondition rbc(&sock, recvStr.size());
    sock.conn->setSend(&sf);
    sock.reqrd(recvStr.size());
    sock.schedule();
    loop->setBreakCondition(&rbc);
    loop->start();
    sf.wait();
    ASSERT_TRUE(sf.isOk());
    ASSERT_EQ(sock.getReceived(), recvStr);

    // Clean it all up
    sock.close();
}

#else
class SSLTest : public ::testing::Test
{
};
TEST_F(SSLTest, DISABLED_testBasic)
{
    EXPECT_FALSE(true);
}
#endif
