/**
 * @file
 * This contains the API for the test socket server used to test the library's
 * core `lcbio` functionality.
 */

#ifndef NOMINMAX
#define NOMINMAX
#endif

#include "config.h"
#ifndef _WIN32
#include <sys/socket.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <unistd.h>
#define closesocket close
#else
#include "config.h"
#define sched_yield()
#define SHUT_RDWR SD_BOTH
#define ssize_t SSIZE_T
#endif

#include <cassert>
#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <cstdio>
#include <list>
#include <vector>
#include <string>
#include "threads.h"

namespace LCBTest
{
class TestServer;
class TestConnection;

/** Convenience class representing a numeric socket handle */
class SockFD
{
  public:
    SockFD(int sock);
    virtual ~SockFD();
    virtual void close();

    virtual int getFD() const
    {
        return fd;
    }
    operator int() const
    {
        return getFD();
    }

    void loadRemoteAddr();

    const struct sockaddr_in &localAddr4()
    {
        return *(struct sockaddr_in *)&sa_local;
    }

    const struct sockaddr_in &remoteAddr4()
    {
        return *(struct sockaddr_in *)&sa_remote;
    }

    uint16_t getLocalPort()
    {
        return ntohs(localAddr4().sin_port);
    }

    uint16_t getRemotePort()
    {
        return ntohs(remoteAddr4().sin_port);
    }

    std::string getLocalHost()
    {
        return getHostCommon(&sa_local);
    }

    std::string getRemoteHost()
    {
        return getHostCommon(&sa_remote);
    }

    template < typename T > bool setOption(int level, int option, T val)
    {
        int rv = setsockopt(fd, level, option, (char *)&val, sizeof val);
        return rv == 0;
    }

    bool setNodelay(bool enabled = true)
    {
        int isEnabled = enabled ? 1 : 0;
        return setOption< int >(IPPROTO_TCP, TCP_NODELAY, isEnabled);
    }

    SockFD *acceptClient();

    virtual size_t send(const void *buf, size_t n, int flags = 0)
    {
        return ::send(fd, (const char *)buf, n, flags);
    }
    virtual ssize_t recv(void *buf, size_t n, int flags = 0)
    {
        return ::recv(fd, (char *)buf, n, flags);
    }

    static SockFD *newListener();
    static SockFD *newClient(SockFD *server);

  private:
    static std::string getHostCommon(sockaddr_storage *ss);
    socklen_t naddr;
    struct sockaddr_storage sa_local;
    struct sockaddr_storage sa_remote;
#ifdef _WIN32
    SOCKET fd;
#else
    int fd;
#endif
    SockFD(SockFD &);
};

/**
 * A Future represents a certain action the server should take. Since the server
 * is essentially a dumb data handler, it relies on the test logic (in this case,
 * the client) to control what it does.
 *
 * Futures represent a certain action the server should take (see the various
 * subclasses). They can be waited on (via wait()), and their status can be
 * checked (via isOk()).
 *
 * Note that futures are executed in the context of the _server_'s thread,
 * so that a future may be done before the wait() method is called.
 *
 * See the specific subclasses of future for more usage details.
 *
 * @see SendFuture
 * @see RecvFuture
 */
class Future
{
  public:
    /**Wait until the task has been completed by the @ref TestConnection */
    void wait();

    /**Return if the task completed successfully. Only valid once wait() has
     * returned.
     * @return true on success, false on failure */
    bool isOk()
    {
        return !failed;
    }

    /**A non-blocking way to check if the task has completed
     * @return true if completed, false otherwise */
    bool checkDone();

    virtual ~Future();

  protected:
    friend class TestConnection;

    /**Locks the state of the Future. The action to be performed should be
     * done after this is called. When the action is done, call endUpdate() */
    void startUpdate();

    /**Closing bracket for startUpdate() */
    void endUpdate();

    void updateFailed()
    {
        startUpdate();
        bail();
        endUpdate();
    }

    /** Indicate this action has failed. Should only be called in an active
     * startUpdate()/endUpdate() block */
    void bail()
    {
        failed = true;
        last_errno = errno;
        printf("Bailing: Error=%d\n", last_errno);
    }

    /** Implemented by subclasses to determine if the action is done
     * @return true if done, false otherwise */
    virtual bool isDone() = 0;

    Future();

  private:
    Mutex mutex;
    Condvar cond;
    volatile bool failed;
    bool shouldEnd()
    {
        return isDone() || failed;
    }
    volatile int last_errno;
};

/** Future implementation that makes the server _send_ a buffer to the client */
class SendFuture : public Future
{
  public:
    /**@param bytes The buffer to send
     * @param nbytes The number of bytes to send */
    SendFuture(const void *bytes, size_t nbytes) : Future()
    {
        buf.insert(buf.begin(), (char *)bytes, (char *)bytes + nbytes);
        nsent = 0;
    }

    SendFuture(const std::string &ss)
    {
        buf.assign(ss.begin(), ss.end());
        nsent = 0;
    }

  protected:
    bool isDone()
    {
        return nsent == buf.size();
    }

  private:
    friend class TestConnection;
    /**Returns the beginning of the unsent buffer
     * @param[out] outbuf the pointer to contain the buffer */
    size_t getBuf(void **outbuf)
    {
        size_t ret = buf.size() - nsent;
        *outbuf = &buf[nsent];
        return ret;
    }

    /**Called to update the sent count.
     * @param n The number of bytes just sent. */
    void setSent(size_t n)
    {
        nsent += n;
    }

    volatile unsigned nsent;
    std::vector< char > buf;
};

/**
 * @ref Future implementation which instructs the server to receive a number
 * of bytes _sent_ by the client
 */
class RecvFuture : public Future
{
  public:
    /** @param n The number of bytes (exactly) to receive. */
    RecvFuture(size_t n) : Future()
    {
        reinit(n);
    }

    /**Discards the internal state and modifies the number of bytes to wait for
     * @param n The new number of bytes to wait for.
     * This is used by some tests to save on reinitialization */
    void reinit(size_t n)
    {
        required = n;
        buf.clear();
        buf.reserve(n);
    }

    /** Get the contents the server received as a `vector`
     * @return The received data */
    std::vector< char > getBuf()
    {
        return buf;
    }

    std::string getString()
    {
        return std::string(buf.begin(), buf.end());
    }

  protected:
    bool isDone()
    {
        return buf.size() == required;
    }

  private:
    friend class TestConnection;

    /**@return The number of bytes remaining to be received.
     * Used by @ref TestConnection */
    size_t getRequired()
    {
        return required - buf.size();
    }

    /**Call when new bytes are received on the connection.
     * @param rbuf The buffer containing the new contents
     * @param nbuf The length of the buffer
     * Used by @ref TestConnection */
    void setReceived(void *rbuf, size_t nbuf)
    {
        char *cbuf = (char *)rbuf;
        buf.insert(buf.end(), cbuf, cbuf + nbuf);
    }
    volatile size_t required;
    std::vector< char > buf;
};

/**
 * @ref Future implementation which makes the server _close_ the connection.
 * The wait() will wait until the connection has been closed. This is useful
 * if you wish to test behavior on a closed socket (i.e. a socket on which the
 * remote closed the connection).
 */
class CloseFuture : public Future
{
  public:
    /**
     * @enum CloseTime
     *
     * Indicates _when_ the close should take place. @ref CloseFuture objects
     * can be performed either before any I/O is pending (i.e. ignore any
     * outstanding I/O requests), or _after_ all pending I/O (i.e. any
     * @ref SendFuture or @ref RecvFuture objects) has completed.
     */
    enum CloseTime {
        BEFORE_IO /**< Close socket before any I/O is performed */,
        AFTER_IO /**< Closed socket once all pending I/O operations have successfully completed */
    };

    /**@param type See documentation for @ref CloseTime */
    CloseFuture(CloseTime type) : Future()
    {
        performed = false;
        closeTime = type;
    }

  protected:
    bool isDone()
    {
        return performed;
    }

  private:
    friend class TestConnection;
    void setDone()
    {
        performed = true;
    }
    CloseTime getType()
    {
        return closeTime;
    }

    volatile bool performed;
    CloseTime closeTime;
};

/**
 * Representation of a server side remote endpoint
 *
 * A TestConnection object is created whenever the @ref TestServer accepts a
 * new connection. It can be used by tests to coordinate various actions
 * between client and server, using the various @ref Future implementations.
 *
 * @note Futures of different kinds can exist concurrently within the same
 * TestConnection object; however, only _one_ future of a given type can
 * be active; so for example, a @ref SendFuture and a @ref RecvFuture may
 * be active concurrently, but two @ref CloseFuture objects may not.
 *
 * Additionally, note that any @ref Future object passed must remain valid
 * until it has completed (i.e. Future::wait() or Future::isDone() returns
 * true).
 */
class TestConnection
{
  public:
    /**
     * Set the @ref SendFuture object to indicate that the server should
     * send data
     * @param f
     */
    void setSend(SendFuture *f)
    {
        setCommon(f, (void **)&f_send);
    }

    /**
     * Indicate that the server should read data. The future object indicates
     * how much data to read
     * @param f
     */
    void setRecv(RecvFuture *f)
    {
        setCommon(f, (void **)&f_recv);
    }

    /**
     * Indicate that the connection should be closed, optionally before or
     * after outstanding IO (See @ref CloseFuture for more details).
     * @param f
     */
    void setClose(CloseFuture *f)
    {
        setCommon(f, (void **)&f_close);
    }

    /**
     * _Immediately_ close the underlying socket connection on the server side.
     * This is not the same as @ref CloseFuture which merely _schedules_
     * a close
     */
    void close()
    {
        datasock->close();
        ctlfd_loop->close();
        ctlfd_user->close();
        ctlfd_lsn->close();
    }

    /**
     * Return the remote port from which the client initiated the connection.
     * This is used to determine which the object should
     * be associated with a given client (i.e. @ref ESocket) object.
     * @return The port of the client.
     */
    uint16_t getPeerPort()
    {
        return datasock->getRemotePort();
    }

    inline void _doRun();

  protected:
    TestConnection(TestServer *server, SockFD *newsock);
    ~TestConnection();
    virtual void run();
    friend class TestServer;

  private:
    SockFD *datasock;
    SockFD *ctlfd_loop;
    SockFD *ctlfd_lsn;
    SockFD *ctlfd_user;
    Mutex mutex;
    Condvar initcond;
    Thread *thr;
    TestServer *parent;
    SendFuture *f_send;
    RecvFuture *f_recv;
    CloseFuture *f_close;
    void setCommon(void *src, void **target);
    void sendData();
    void recvData();
    void handleClose();
};

/**
 * Represents a listening socket for a test "Server". This server accepts
 * connections from clients, and for each new connection, creates a new
 * @ref TestConneciton object.
 */
class TestServer
{
  public:
    TestServer();
    ~TestServer();

    /** Run the server. This will open a new thread */
    void run();

    /** Stop the server. This will close the listening sokcet */
    void close()
    {
        closed = true;
        lsn->close();
    }

    bool isClosed()
    {
        return closed;
    }

    /**Find a connection with a given client port
     * @param cliport The client port to search for
     * @return The connection object, or `NULL` if no such connection exists.
     */
    TestConnection *findConnection(uint16_t cliport);

    /**
     * Get the listening port
     * @return The listening ports that clients may use to connect to this
     * object.
     */
    uint16_t getListenPort()
    {
        return lsn->getLocalPort();
    }

    /**
     * Get the IP address (usually `127.0.0.1` as a string)
     * @return The host the server is listening on
     */
    std::string getHostString()
    {
        return lsn->getLocalHost();
    }

    /**
     * Get the listening port, as a string
     * @return The listening port
     */
    std::string getPortString();

    typedef SockFD *(SocketFactory)(int);

    SocketFactory *factory;
    static SockFD *plainSocketFactory(int fd)
    {
        return new SockFD(fd);
    }
    static SockFD *sslSocketFactory(int fd);

  private:
    friend class TestConnection;
    bool closed;
    SockFD *lsn;
    Thread *thr;
    Mutex mutex;
    std::list< TestConnection * > conns;
    void startConnection(TestConnection *conn);
};

} // namespace LCBTest
