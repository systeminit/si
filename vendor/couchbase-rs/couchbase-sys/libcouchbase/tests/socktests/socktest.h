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

/**
 * @file
 * Core routines and classes for @ref LCBIO tests.
 */
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <gtest/gtest.h>
#include <ioserver/ioserver.h>
#include <lcbio/lcbio.h>
#include <lcbio/iotable.h>
#include "settings.h"
#include "logging.h"
#include <list>
#include "internal.h"
#ifndef _WIN32
#include <signal.h>
#endif

using namespace LCBTest;

static inline void hostFromSockFD(SockFD *sfd, lcb_host_t *tgt)
{
    strcpy(tgt->host, sfd->getLocalHost().c_str());
    sprintf(tgt->port, "%d", sfd->getLocalPort());
}

class Loop;
struct ESocket;

/** Wrapper class for @ref lcbio_CTXPROCS */
class IOActions
{
  public:
    virtual void onRead(ESocket *s, size_t nr);
    virtual void onError(ESocket *s);
    virtual void onFlushDone(ESocket *s, size_t, size_t n) {}
    virtual void onFlushReady(ESocket *s) {}
    virtual ~IOActions() {}
};

/**
 * This class represents a connecting socket. It is used to wrap the
 * functionality of the @ref lcbio_SOCKET structure.
 *
 * To actually connect the socket, use one of the `connect` methods.
 *
 * Once the ESocket has been connected, it will also contain a pointer to the
 * _server's_ perspective of the connection (see #conn). This can then be
 * used to synchronize actions between the client (i.e. this object) and the
 * server (the @ref LCBTest::TestConnection object).
 *
 * To perform I/O on this object, you must coordinate manually between the
 * client (i.e. the ESocket object) and the server (i.e. the #conn member):
 *
 * Writing to the Server:
 *
 * 1. Add data to send using the put() method
 * 2. Schedule the data to be sent, using the schedule() method
 * 3. Create a @ref LCBTest::RecvFuture object, passing the number of bytes you expect
 *    the server to receive.
 * 4. Set the server's current future object, via LCBTest::TestConnection::setRecv
 * 5. Optionally set a @ref FutureBreakCondition on the @ref Loop object, so
 *    that the event loop will exit once the operation has been completed by
 *    the server
 * 6. Start the loop via Loop::start()
 *
 *
 * Reading from the Server:
 *
 * 1. Create a @ref LCBTest::SendFuture object which will contain the data the server
 *    should send to the client
 * 2. Set the @ref LCBTest::TestConnection (#conn)'s current future via
 *    TestConnection::setSend() method
 * 3. Indicate the minimum number of bytes the client expects to receive (this
 *    shoud be less or equal to the number of bytes passed to the
 *    @ref LCBTest::SendFuture object).
 * 4. Schedule the read via #schedule()
 * 5. Create either a @ref ReadBreakCondition, passing the number of bytes you
 *    want to read (at minimum) from the constructor
 * 6. Start the loop via Loop::start()
 * 7. The received data should be available via getReceived()
 *
 * @see lcbio_SOCKET
 * @see Loop
 * @see Loop::connect()
 */
struct ESocket {
    /** This contains the pending request. See the underlying C object for details */
    lcb::io::ConnectionRequest *creq;
    /** The underlying @ref lcbio_SOCKET */
    lcbio_SOCKET *sock;
    /** The current context */
    lcbio_CTX *ctx;

    /** If the initial Loop::connect() failed, this contains the last
     * `errno` (or equivalent) which was received on the connection attempt */
    lcbio_OSERR syserr;

    /** Contains the captured library error codes, if applicable */
    lcb_STATUS lasterr;
    Loop *parent;

    /** Actions implementation. The various callbacks in @ref lcbio_CTXPROCS
     * dispatch to this table */
    IOActions *actions;

    /** The *Server* state for this connection */
    TestConnection *conn;

    /** This is used internally by various tests, typically to check that a
     * callback was invoked a certain number of times (or at all) */
    int callCount;

    static IOActions defaultActions;

    ESocket()
    {
        sock = NULL;
        ctx = NULL;
        parent = NULL;
        lasterr = LCB_SUCCESS;
        syserr = 0;
        callCount = 0;
        conn = NULL;
        creq = NULL;
        actions = &defaultActions;
    }

    /** Closes the underlying @ref lcbio_SOCKET object */
    void close();

    /** Unsets the underlying @ref lcbio_SOCKET object */
    void clear()
    {
        ctx = NULL;
    }

    ~ESocket()
    {
        close();
    }

    /**
     * Add data to be sent. Call the schedule() method to schedule the operation.
     * @param b the buffer to send
     * @param n the number of bytes to send
     *
     * @see schedule()
     */
    void put(const void *b, size_t n)
    {
        lcbio_ctx_put(ctx, b, n);
    }

    void put(const std::string &s)
    {
        put(s.c_str(), s.size());
    }

    /**Request that a given number of bytes should be read from the remote. When
     * the data is received, it is placed into an internal buffer (#readbuf).
     *
     * @param n The minimum number of bytes to be read
     */
    void reqrd(size_t n)
    {
        lcbio_ctx_rwant(ctx, n);
    }

    /** Wraps lcbio_ctx_schedule() */
    void schedule()
    {
        lcbio_ctx_schedule(ctx);
    }

    /**Get the contents of the internal read buffer. The contents are not
     * removed.
     *
     * @return A copy of the contents.
     */
    std::string getReceived()
    {
        return std::string(readbuf.begin(), readbuf.end());
    }

    size_t getUnreadSize()
    {
        return rdb_get_nused(&ctx->ior);
    }

    void setActions(IOActions *ioa)
    {
        actions = ioa;
    }

    /** Internal method used to associate the socket (if any) with this object */
    void assign(lcbio_SOCKET *sock, lcb_STATUS err);

    /** Internal buffer to store read data */
    std::vector< char > readbuf;
};

/**
 * Base class for timers.
 *
 * The expired() method should be implemented by subclasses, and will be
 * called once the timer expires.
 */
class Timer
{
  public:
    /**Create a new timer
     * @param iot The parent IO Table */
    Timer(lcbio_TABLE *iot);

    virtual ~Timer();

    /**To be implemented by subclasses. Called whenever the timer expires;
     * or the signal() method is called.
     *
     * Note that this method is called from within the event loop (and never
     * directly).
     */
    virtual void expired() = 0;
    void destroy();

    /** Cancel this timer. The expired() membered function will not be invoked */
    void cancel();

    /**Set the expiration delay for this timer
     * @param ms The expiration delay, in milliseconds */
    void schedule(unsigned ms);

    /** Asynchronously call the #expired() method. */
    void signal();

  private:
    lcbio_pTIMER timer;
};

/**
 * This class checks if the loop should break or not. If it should, then
 * shouldBreak() returns true. This is required because some event loops are
 * in 'always run' mode and don't particularly break once no I/O handles
 * remain active.
 */
class BreakCondition
{
  public:
    BreakCondition();
    virtual ~BreakCondition() {}

    /** Called by the loop to determine if it should break.  */
    bool shouldBreak();

    /** Call this to determine if the loop actually exited because the condition
     * was satisfied. The loop may also exit after a period of general inactivity
     * depending on the implementation.
     *
     * @return true if the loop was forcefully broken because shouldBreak() returned
     * true. */
    bool didBreak()
    {
        return broke;
    }

  protected:
    /**Subclasses should implement this.
     * @return true if the loop should break.
     * @note This function is only called as long as #broke is false
     */
    virtual bool shouldBreakImpl() = 0;

    /** Set by the shouldBreak() wrapper */
    bool broke;
};

/**
 * This @ref BreakCondition implementation checks to see if the contained
 * @ref LCBTest::Future object is complete. If the underlying future is complete,
 * it will break the loop.
 */
class FutureBreakCondition : public BreakCondition
{
  public:
    /** @param ft The future to poll */
    FutureBreakCondition(Future *ft) : BreakCondition()
    {
        f = ft;
    }

    virtual ~FutureBreakCondition() {}

  protected:
    bool shouldBreakImpl()
    {
        return f->checkDone();
    }
    Future *f;
};

/**
 * This @ref BreakCondition implementation checks to see if the pending write
 * buffer from the underying @ref ESocket / @ref lcbio_SOCKET has been completely
 * flushed (and thus no longer needed by the underlying socket implementation).
 */
class FlushedBreakCondition : public BreakCondition
{
  public:
    /** @param s The socket to check */
    FlushedBreakCondition(ESocket *s)
    {
        sock = s;
    }
    virtual ~FlushedBreakCondition() {}

  protected:
    bool shouldBreakImpl();

  private:
    ESocket *sock;
};

/**
 * This @ref BreakCondition implementation checks to see if the given socket
 * has read at least a certain amount of data
 */
class ReadBreakCondition : public BreakCondition
{
  public:
    /**
     * @param s The socket
     * @param nr The minimum number of bytes to read before breaking
     */
    ReadBreakCondition(ESocket *s, unsigned nr)
    {
        sock = s;
        expected = nr;
    }
    virtual ~ReadBreakCondition() {}

  protected:
    bool shouldBreakImpl();

  private:
    unsigned expected;
    ESocket *sock;
};

/**
 * This @ref BreakCondition implementation checks to see if the given socket
 * has an error.
 *
 * This is generally useful to use in tests where an error is expected, but where
 * the loop may terminate before the error is received.
 */
class ErrorBreakCondition : public BreakCondition
{
  public:
    /** @param s Socket to monitor for errors. */
    ErrorBreakCondition(ESocket *s) : BreakCondition()
    {
        sock = s;
    }

  protected:
    ESocket *sock;
    bool shouldBreakImpl()
    {
        return sock->lasterr != LCB_SUCCESS;
    }
};

/** TODO: Is this really a break condition? This seems to be used for testing
 * the invocation of lcbio_ctx_close_ex()'s callback argument. */
class CtxCloseBreakCondition : public BreakCondition
{
  public:
    CtxCloseBreakCondition(ESocket *sock) : BreakCondition()
    {
        s = sock;
        destroyed = false;
    }

    void gotDtor()
    {
        destroyed = true;
    }

    void closeCtx();

  protected:
    ESocket *s;
    bool destroyed;
    bool shouldBreakImpl()
    {
        return destroyed;
    }
};

/** This implementation will always break the loop. */
class NullBreakCondition : public BreakCondition
{
  public:
    bool shouldBreakImpl()
    {
        return true;
    }
};

/**Internal class used to allow the loop to 'poll' a @ref BreakCondition.
 * This will by default check every 2 milliseconds to see if the condition
 * has been satisfied or not. */
class BreakTimer;

/** Class representing the underlying libcouchbase event loop. */
class Loop
{
  public:
    Loop();
    ~Loop();

    /** Runs the loop */
    void start();

    /** Stops the loop */
    void stop();

    /**
     * This method will connect a newly created @ref ESocket object
     *
     * This method blocks until the socket is connected, or an error is
     * received. See the ESocket::syserr and ESocket::lasterr fields to
     * indicate status.
     *
     * @param sock The socket to connect
     * @param host the endpoint to connect to. If this is null, the value of
     * populateHost() will be used.
     * @param mstmo the timeout (in milliseconds) for the connection to succeed.
     */
    void connect(ESocket *sock, lcb_host_t *host = NULL, unsigned mstmo = 1000);

    /**
     * This method will connect a newly created @ref ESocket object using the
     * connection pool (i.e. @ref lcbio_MGR). The semantics should normally
     * be the same as the connect() method, except that if a previously created
     * (and available) connection in the socket pool exists, it will be used
     * rather than creating a new one.
     *
     * @param sock The socket to connect to
     * @param host The host to connect to. If NULL, the value of populateHost() will be used
     * @param mstmo The timeout (in milliseconds) for the connection to succeed
     */
    void connectPooled(ESocket *sock, lcb_host_t *host = NULL, unsigned mstmo = 1000);

    /**
     * Populate the host object with the host and port pointing to the current
     * @ref LCBTest::TestServer (#server)
     * @param host The host to populate
     */
    void populateHost(lcb_host_t *host);

    /**Sets the condition upon which the loop will terminate
     * @param bc the condition */
    void setBreakCondition(BreakCondition *bc)
    {
        bcond = bc;
    }

    /* These members are not really public, but are available for inspection */
    lcbio_MGR *sockpool;
    TestServer *server; /**< Underlying server object */
    lcb_settings *settings;
    lcb_io_opt_t io;
    lcbio_pTABLE iot;

  private:
    void scheduleBreak();
    void cancelBreak();
    void initSockCommon(ESocket *s);

    friend class BreakTimer;

    BreakTimer *breakTimer;
    std::list< Future * > pending;
    BreakCondition *bcond;
};

class SockTest : public ::testing::Test
{
  protected:
    Loop *loop;
    void SetUp()
    {
        lcb_initialize_socket_subsystem();
#ifndef _WIN32
        signal(SIGPIPE, SIG_IGN);
#endif
        loop = new Loop();
    }

    void TearDown()
    {
        delete loop;
    }
};
