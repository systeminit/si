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
using namespace LCBTest;
using std::string;
using std::vector;

class SockConnTest : public SockTest
{
};

TEST_F(SockConnTest, testBasic)
{
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

    //  We can receive data
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

static bool isRefused(lcbio_OSERR err)
{
    if (err == ECONNREFUSED || err == ECONNABORTED) {
        return true;
    }
#ifdef _WIN32
    if (err == WSAECONNREFUSED) {
        return true;
    }
#endif
    return false;
}
// Test a connect without an accept
TEST_F(SockConnTest, testRefused)
{
    ESocket sock;
    lcb_host_t host = {0};
    strcpy(host.host, "localhost");
    strcpy(host.port, "1");
    loop->connect(&sock, &host, 100000);
    ASSERT_TRUE(sock.sock == NULL);
    ASSERT_TRUE(isRefused(sock.syserr));
}

TEST_F(SockConnTest, testBadDomain)
{
    ESocket sock;
    lcb_host_t host = {0};
    strcpy(host.host, "domain-should-not-work.nonexist.com");
    strcpy(host.port, "123");
    loop->connect(&sock, &host, 1000);
    ASSERT_TRUE(sock.sock == NULL);
}

TEST_F(SockConnTest, testInvalidPort)
{
    ESocket sock;
    lcb_host_t host = {0};
    strcpy(host.host, "localhost");
    strcpy(host.port, "111111111");
    loop->connect(&sock, &host, 1000);
    ASSERT_TRUE(sock.sock == NULL);
}

TEST_F(SockTest, testEmptyHost)
{
    ESocket sock;
    lcb_host_t host = {0};
    host.host[0] = '\0';
    host.port[0] = '\0';
    loop->connect(&sock, &host, 1000);
    ASSERT_TRUE(sock.sock == NULL);
}

TEST_F(SockConnTest, testCancellation)
{
    ESocket sock;
    lcb_host_t host = {0};
    loop->populateHost(&host);
    sock.creq = lcbio_connect(loop->iot, loop->settings, &host, 100000, NULL, NULL);
    ASSERT_FALSE(sock.creq == NULL);
    lcb::io::ConnectionRequest::cancel(&sock.creq);

    NullBreakCondition nbc;
    loop->setBreakCondition(&nbc);
    loop->start();
}

extern "C" {
static void conncb_1(lcbio_SOCKET *sock, void *arg, lcb_STATUS err, lcbio_OSERR syserr)
{
    ESocket *es = (ESocket *)arg;
    es->creq = NULL;
    es->callCount++;
    es->ctx = NULL;
    es->lasterr = err;
    es->parent->stop();

    // unref is implicit
}
}
TEST_F(SockConnTest, testImmediateUnref)
{
    ESocket sock;
    lcb_host_t host = {0};
    sock.parent = loop;
    loop->populateHost(&host);
    sock.creq = lcbio_connect(loop->iot, loop->settings, &host, 1000000, conncb_1, &sock);
    loop->start();
    ASSERT_EQ(1, sock.callCount);
    ASSERT_TRUE(sock.sock == NULL);
}
