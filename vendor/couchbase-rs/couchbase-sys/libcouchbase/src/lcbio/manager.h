/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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

#ifndef LCBIO_MANAGER_H
#define LCBIO_MANAGER_H
#include "connect.h"
#include "settings.h"
#include "list.h"
#include "ioutils.h"
#include <stdio.h>

/**
 * @file
 * @brief Socket pooling routines
 *
 * @details
 * General purpose connection manager for LCB sockets. This object is
 * responsible for maintaining and properly handling idle connections
 * and pooling them (optionally).
 */

/**
 * @ingroup lcbio
 * @defgroup lcbio-mgr Socket Pooling
 * @addtogroup lcbio-mgr
 * @{
 */

#ifdef __cplusplus
#include <map>

namespace lcb
{
namespace io
{

/** @brief Socket Pool */
class Pool;

/** @brief Pooled connection */
struct PoolConnInfo;

/** @brief Cancellable pool request */
struct PoolRequest;

struct PoolHost;
} // namespace io
} // namespace lcb

typedef lcb::io::Pool lcbio_MGR;
extern "C" {

#else
/* C only */
typedef struct lcbio_MGR_CDUMMY lcbio_MGR;
#endif

#ifdef __cplusplus
}

namespace lcb
{
namespace io
{
class Pool
{
  public:
    /**
     * Create a socket pool controlled by the given settings and IO structure.
     * This function will increment the refcount on both the settings and table
     * objects.
     */
    Pool(lcb_settings *, lcbio_pTABLE);

    /**
     * Destroy the socket pool. Note that internally this just decrements the
     * reference count. The object is only destroyed when its count hits zero.
     */
    void shutdown();

    /**
     * Request a connection from the socket pool. The semantics and prototype
     * of this function are by design similar to lcbio_connect() as they do the
     * same things.
     *
     * @param dest the host to connect to
     * @param timeout amount of time to wait for a connection to be estblished
     * @param handler a callback to invoke when the result is ready
     * @param arg an argument passed to the callback
     * @return a request handle which may be cancelled
     * @see lcbio_connect()
     */
    ConnectionRequest *get(const lcb_host_t &, uint32_t, lcbio_CONNDONE_cb, void *);

    /**
     * Release a socket back into the pool. This means the socket is no longer
     * used and shall be available for reuse for another request. To verify these
     * constraints, the socket's reference count must be one. Once the socket
     * has been released its reference count should not be modified.
     */
    static void put(lcbio_SOCKET *sock);

    /**
     * Mark a slot as available but discard the current connection. This should be
     * done if the connection itself is "dirty", i.e. has a protocol error on it
     * or is otherwise not suitable for reuse
     */
    static void discard(lcbio_SOCKET *sock);

    /**
     * Like lcbio_mgr_discard() except the source connection is left untouched. It
     * is removed from the pool instead.
     *
     * Because the lcbio_MGR object itself has internal limits and thresholds on how
     * many leased and/or open connections it can contain, when a connection receives
     * an error it must either be discarded back to the pool (in which case the
     * connection is cleaned up and is freed) or it must be detached (in which case
     * the connection object itself still remains valid, but the pool does not know
     * about it, and all its counters are restored, as with lcbio_mgr_discard()).
     *
     * lcbio_mgr_discard() itself is now implemented as the equivalent to:
     *  `lcbio_mgr_detach(mgr, conn)`;
     */
    static void detach(lcbio_SOCKET *sock);

    static bool is_from_pool(const lcbio_SOCKET *sock);

    /**
     * Dumps the connection manager state to stderr
     */
    void dump(FILE *) const;

    inline void ref();
    inline void unref();

    struct Options {
        Options() : maxtotal(0), maxidle(0), tmoidle(0) {}

        /** Maximum *total* number of connections opened by the pool. If this
         * number is exceeded, the pool will black hole future requests until
         * a new slot becomes available.
         */
        unsigned maxtotal;

        /**
         * Maximum number of idle connections to keep around
         */
        unsigned maxidle;

        /**
         * The amount of time the pool should wait before closing idle
         * connections. In microseconds
         */
        uint32_t tmoidle;
    };

    void set_options(const Options &opts)
    {
        options = opts;
    }

    Options &get_options()
    {
        return options;
    }

    void toJSON(hrtime_t now, Json::Value &node);

  private:
    friend struct PoolRequest;
    friend struct PoolConnInfo;
    friend struct PoolHost;

    typedef std::map< std::string, PoolHost * > HostMap;
    HostMap ht;
    lcb_settings *settings;
    lcbio_pTABLE io;
    Options options;
    unsigned refcount;
};
} // namespace io
} // namespace lcb
#endif
/**@}*/

#endif /* LCB_SOCKPOOL_H */
