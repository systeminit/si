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

#include "manager.h"
#include "hostlist.h"
#include "iotable.h"
#include "timer-ng.h"
#include "internal.h"

#define LOGARGS(mgr, lvl) mgr->settings, "lcbio_mgr", LCB_LOG_##lvl, __FILE__, __LINE__

using namespace lcb::io;

namespace lcb
{
namespace io
{

struct PoolHost {
    inline PoolHost(Pool *, const std::string &);
    inline void connection_available();
    inline void start_new_connection(uint32_t timeout);

    void ref()
    {
        refcount++;
    }

    void unref()
    {
        if (!--refcount) {
            delete this;
        }
    }

    ~PoolHost()
    {
        if (parent) {
            parent->unref();
            parent = NULL;
        }
    }

    inline void dump(FILE *fp) const;

    size_t num_pending() const
    {
        return LCB_CLIST_SIZE(&ll_pending);
    }
    size_t num_idle() const
    {
        return LCB_CLIST_SIZE(&ll_idle);
    }
    size_t num_requests() const
    {
        return LCB_CLIST_SIZE(&requests);
    }
    size_t num_leased() const
    {
        return n_total - (num_idle() + num_pending());
    }

    lcb_clist_t ll_idle;    /* idle connections */
    lcb_clist_t ll_pending; /* pending cinfo */
    lcb_clist_t requests;   /* pending requests */
    const std::string key;  /* host:port */
    Pool *parent;
    lcb::io::Timer< PoolHost, &PoolHost::connection_available > async;
    unsigned n_total; /* number of total connections */
    unsigned refcount;
};
} // namespace io
} // namespace lcb

struct CinfoNode : lcb_list_t {
};

namespace lcb
{
namespace io
{
struct PoolConnInfo : lcbio_PROTOCTX, CinfoNode {
    inline PoolConnInfo(PoolHost *parent, uint32_t timeout);
    inline ~PoolConnInfo();
    inline void on_idle_timeout();
    inline void on_connected(lcbio_SOCKET *sock, lcb_STATUS err);

    void set_leased()
    {
        lcb_assert(state == IDLE);
        state = LEASED;
        idle_timer.cancel();
    }

    static PoolConnInfo *from_llnode(lcb_list_t *node)
    {
        return static_cast< PoolConnInfo * >(static_cast< CinfoNode * >(node));
    }

    static PoolConnInfo *from_sock(lcbio_SOCKET *sock)
    {
        lcbio_PROTOCTX *ctx = lcbio_protoctx_get(sock, LCBIO_PROTOCTX_POOL);
        return static_cast< PoolConnInfo * >(ctx);
    }

    PoolHost *parent;
    lcbio_SOCKET *sock;
    lcbio_pCONNSTART cs;
    lcb::io::Timer< PoolConnInfo, &PoolConnInfo::on_idle_timeout > idle_timer;

    enum State { PENDING, IDLE, LEASED };
    State state;
};
} // namespace io
} // namespace lcb

struct ReqNode : lcb_list_t {
};
namespace lcb
{
namespace io
{
struct PoolRequest : ReqNode, ConnectionRequest {
    PoolRequest(PoolHost *host_, lcbio_CONNDONE_cb cb, void *cbarg)
        : host(host_), callback(cb), arg(cbarg), timer(host->parent->io, this), state(PENDING), sock(NULL),
          err(LCB_SUCCESS)
    {
    }

    virtual ~PoolRequest() {}
    virtual void cancel();
    inline void invoke();
    void invoke(lcb_STATUS err_)
    {
        err = err_;
        invoke();
    }

    inline void timer_handler();
    inline void set_ready(PoolConnInfo *cinfo)
    {
        cinfo->set_leased();
        sock = cinfo->sock;
        state = ASSIGNED;
        timer.signal();
    }

    inline void set_pending(uint32_t timeout)
    {
        timer.rearm(timeout);
    }

    static PoolRequest *from_llnode(lcb_list_t *node)
    {
        return static_cast< PoolRequest * >(static_cast< ReqNode * >(node));
    }

    PoolHost *host;
    lcbio_CONNDONE_cb callback;
    void *arg;
    Timer< PoolRequest, &PoolRequest::timer_handler > timer;

    enum State { ASSIGNED, PENDING };
    State state;
    lcbio_SOCKET *sock;
    lcb_STATUS err;
};
} // namespace io
} // namespace lcb

static const char *get_hehost(const PoolHost *h)
{
    if (!h) {
        return "NOHOST:NOPORT";
    }
    return h->key.c_str();
}

/** Format string arguments for %p%s:%s */
#define HE_LOGID(h)                                                                                                    \
    (h && h->parent && h->parent->settings->log_redaction) ? LCB_LOG_SD_OTAG : "", get_hehost(h),                      \
        (h && h->parent && h->parent->settings->log_redaction) ? LCB_LOG_SD_CTAG : "", (void *)h
#define HE_LOGFMT "<" LCB_LOG_SPEC("%s") "> (HE=%p) "

PoolConnInfo::~PoolConnInfo()
{
    parent->n_total--;
    if (state == IDLE) {
        lcb_clist_delete(&parent->ll_idle, this);

    } else if (state == PENDING && cs) {
        lcbio_connect_cancel(cs);
    }

    if (sock) {
        // Ensure destructor is not called!
        dtor = NULL;
        lcbio_protoctx_delptr(sock, this, 0);
        lcbio_unref(sock);
    }
    parent->unref();
}

static void cinfo_protoctx_dtor(lcbio_PROTOCTX *ctx)
{
    PoolConnInfo *info = reinterpret_cast< PoolConnInfo * >(ctx);
    info->sock = NULL;
    delete info;
}

Pool::Pool(lcb_settings *settings_, lcbio_pTABLE io_) : settings(settings_), io(io_), refcount(1) {}

typedef std::vector< PoolHost * > HeList;

void Pool::ref()
{
    refcount++;
}

void Pool::unref()
{
    if (!--refcount) {
        delete this;
    }
}

void Pool::shutdown()
{
    HeList hes;
    HostMap::iterator h_it;

    for (h_it = ht.begin(); h_it != ht.end(); ++h_it) {
        PoolHost *he = h_it->second;

        lcb_list_t *cur, *next;
        LCB_LIST_SAFE_FOR(cur, next, (lcb_list_t *)&he->ll_idle)
        {
            delete PoolConnInfo::from_llnode(cur);
        }

        LCB_LIST_SAFE_FOR(cur, next, (lcb_list_t *)&he->ll_pending)
        {
            delete PoolConnInfo::from_llnode(cur);
        }
        hes.push_back(he);
    }

    for (HeList::iterator it = hes.begin(); it != hes.end(); ++it) {
        PoolHost *he = *it;
        ht.erase(he->key);
        he->async.release();
        he->unref();
    }

    unref();
}

static void endpointToJSON(hrtime_t now, Json::Value &node, const PoolHost *host, const PoolConnInfo *info)
{
    Json::Value endpoint;
    char id[20] = {0};
    snprintf(id, sizeof(id), "%p", (void *)info->sock);
    endpoint["id"] = id;
    endpoint["remote"] = get_hehost(host);
    endpoint["local"] = lcbio__inet_ntop(&info->sock->info->sa_local);
    endpoint["last_activity_us"] = (Json::Value::UInt64)(now - info->sock->atime);
    endpoint["status"] = "connected";
    node[lcbio_svcstr(info->sock->service)].append(endpoint);
}

void Pool::toJSON(hrtime_t now, Json::Value &node)
{
    lcbio_MGR::HostMap::const_iterator it;
    for (it = ht.begin(); it != ht.end(); ++it) {
        const PoolHost *host = it->second;
        lcb_list_t *llcur;
        LCB_LIST_FOR(llcur, (lcb_list_t *)&host->ll_idle)
        {
            endpointToJSON(now, node, host, PoolConnInfo::from_llnode(llcur));
        }
        LCB_LIST_FOR(llcur, (lcb_list_t *)&host->ll_pending)
        {
            endpointToJSON(now, node, host, PoolConnInfo::from_llnode(llcur));
        }
    }
}

void PoolRequest::invoke()
{
    if (sock) {
        PoolConnInfo *info = PoolConnInfo::from_sock(sock);
        info->set_leased();
        state = ASSIGNED;
        lcb_log(LOGARGS(info->parent->parent, DEBUG), HE_LOGFMT "Assigning R=%p SOCKET=%p", HE_LOGID(info->parent),
                (void *)this, (void *)sock);
    }

    callback(sock, arg, err, 0);
    if (sock) {
        lcbio_unref(sock);
    }
    delete this;
}

/**
 * Called to notify that a connection has become available.
 */
void PoolHost::connection_available()
{
    while (LCB_CLIST_SIZE(&requests) && LCB_CLIST_SIZE(&ll_idle)) {
        lcb_list_t *reqitem = lcb_clist_shift(&requests);
        lcb_list_t *connitem = lcb_clist_pop(&ll_idle);

        PoolRequest *req = PoolRequest::from_llnode(reqitem);
        PoolConnInfo *info = PoolConnInfo::from_llnode(connitem);
        req->sock = info->sock;
        req->invoke();
    }
}

/**
 * Connection callback invoked from lcbio_connect() when a result is received
 */
static void on_connected(lcbio_SOCKET *sock, void *arg, lcb_STATUS err, lcbio_OSERR)
{
    reinterpret_cast< PoolConnInfo * >(arg)->on_connected(sock, err);
}

void PoolConnInfo::on_connected(lcbio_SOCKET *sock_, lcb_STATUS err)
{
    lcb_assert(state == PENDING);
    cs = NULL;

    lcb_log(LOGARGS(parent->parent, DEBUG), HE_LOGFMT "Received result for I=%p,C=%p; E=0x%x", HE_LOGID(parent),
            (void *)this, (void *)sock, err);
    lcb_clist_delete(&parent->ll_pending, this);

    if (err != LCB_SUCCESS) {
        /** If the connection failed, fail out all remaining requests */
        lcb_list_t *cur, *nxt;
        LCB_LIST_SAFE_FOR(cur, nxt, (lcb_list_t *)&parent->requests)
        {
            PoolRequest *req = PoolRequest::from_llnode(cur);
            lcb_clist_delete(&parent->requests, req);
            req->sock = NULL;
            req->invoke(err);
        }
        delete this;

    } else {
        state = IDLE;
        sock = sock_;
        lcbio_ref(sock);
        lcbio_protoctx_add(sock, this);

        lcb_clist_append(&parent->ll_idle, this);
        idle_timer.rearm(parent->parent->options.maxidle);
        parent->connection_available();
    }
}

PoolConnInfo::PoolConnInfo(PoolHost *he, uint32_t timeout)
    : parent(he), sock(NULL), cs(NULL), idle_timer(he->parent->io, this), state(PENDING)
{

    // protoctx fields
    id = LCBIO_PROTOCTX_POOL;
    dtor = cinfo_protoctx_dtor;

    lcb_host_t tmphost = {"", "", 0};
    lcb_STATUS err = lcb_host_parsez(&tmphost, he->key.c_str(), 80);
    if (err != LCB_SUCCESS) {
        lcb_log(LOGARGS(he->parent, ERROR), HE_LOGFMT "Could not parse host! Will supply dummy host (I=%p)",
                HE_LOGID(he), (void *)this);
        strcpy(tmphost.host, "BADHOST");
        strcpy(tmphost.port, "BADPORT");
    }
    lcb_log(LOGARGS(he->parent, TRACE), HE_LOGFMT "New pool entry: I=%p", HE_LOGID(he), (void *)this);

    cs = lcbio_connect(he->parent->io, he->parent->settings, &tmphost, timeout, ::on_connected, this);
}

void PoolHost::start_new_connection(uint32_t tmo)
{
    PoolConnInfo *info = new PoolConnInfo(this, tmo);
    lcb_clist_append(&ll_pending, info);
    n_total++;
    refcount++;
}

void PoolRequest::timer_handler()
{
    if (state == ASSIGNED) {
        PoolConnInfo *cinfo = PoolConnInfo::from_sock(sock);
        // Note - invoke() checks to make sure we've been passed an IDLE
        // connection. We should probably add a dedicated state for this
        // in a separate commit.
        cinfo->state = PoolConnInfo::IDLE;
        invoke();
    } else {
        lcb_clist_delete(&host->requests, this);
        invoke(LCB_ETIMEDOUT);
    }
}

PoolHost::PoolHost(Pool *parent_, const std::string &key_)
    : key(key_), parent(parent_), async(parent->io, this), n_total(0), refcount(1)
{

    lcb_clist_init(&ll_idle);
    lcb_clist_init(&ll_pending);
    lcb_clist_init(&requests);
    parent->ref();
}

ConnectionRequest *Pool::get(const lcb_host_t &dest, uint32_t timeout, lcbio_CONNDONE_cb cb, void *cbarg)
{
    PoolHost *he;
    lcb_list_t *cur;

    std::string key;
    if (dest.ipv6) {
        key.append("[").append(dest.host).append("]:").append(dest.port);
    } else {
        key.append(dest.host).append(":").append(dest.port);
    }

    HostMap::iterator m = ht.find(key);
    if (m == ht.end()) {
        he = new PoolHost(this, key);
        ht.insert(std::make_pair(key, he));
    } else {
        he = m->second;
    }

    PoolRequest *req = new PoolRequest(he, cb, cbarg);

GT_POPAGAIN:

    cur = lcb_clist_pop(&he->ll_idle);
    if (cur) {
        int clstatus;
        PoolConnInfo *info = PoolConnInfo::from_llnode(cur);

        clstatus = lcbio_is_netclosed(info->sock, LCB_IO_SOCKCHECK_PEND_IS_ERROR);

        if (clstatus == LCB_IO_SOCKCHECK_STATUS_CLOSED) {
            lcb_log(LOGARGS(this, WARN), HE_LOGFMT "Pooled socket is dead. Continuing to next one", HE_LOGID(he));

            /* Set to LEASED, since it's not inside any of our lists */
            info->state = PoolConnInfo::LEASED;
            delete info;
            goto GT_POPAGAIN;
        }

        req->set_ready(info);
        lcb_log(LOGARGS(this, INFO),
                HE_LOGFMT "Found ready connection in pool. Reusing socket and not creating new connection",
                HE_LOGID(he));

    } else {
        req->set_pending(timeout);

        lcb_clist_append(&he->requests, req);
        if (he->num_pending() < he->num_requests()) {
            lcb_log(LOGARGS(this, DEBUG), HE_LOGFMT "Creating new connection because none are available in the pool",
                    HE_LOGID(he));
            he->start_new_connection(timeout);

        } else {
            lcb_log(LOGARGS(this, DEBUG), HE_LOGFMT "Not creating a new connection. There are still pending ones",
                    HE_LOGID(he));
        }
    }
    return req;
}

void PoolRequest::cancel()
{
    Pool *mgr = host->parent;

    if (sock) {
        lcb_log(LOGARGS(mgr, DEBUG), HE_LOGFMT "Cancelling request=%p with existing connection", HE_LOGID(host),
                (void *)this);
        Pool::put(sock);
        host->async.signal();
    } else {
        lcb_log(LOGARGS(mgr, DEBUG), HE_LOGFMT "Request=%p has no connection.. yet", HE_LOGID(host), (void *)this);
        lcb_clist_delete(&host->requests, this);
    }
    delete this;
}

void PoolConnInfo::on_idle_timeout()
{
    lcb_log(LOGARGS(parent->parent, DEBUG), HE_LOGFMT "Idle connection expired", HE_LOGID(parent));
    lcbio_unref(sock);
}

void Pool::put(lcbio_SOCKET *sock)
{
    PoolHost *he;
    Pool *mgr;
    PoolConnInfo *info = PoolConnInfo::from_sock(sock);

    if (!info) {
        fprintf(stderr, "Requested put() for non-pooled (or detached) socket=%p\n", (void *)sock);
        lcbio_unref(sock);
        return;
    }

    he = info->parent;
    mgr = he->parent;

    if (he->num_idle() >= mgr->options.maxidle) {
        lcb_log(LOGARGS(mgr, INFO), HE_LOGFMT "Closing idle connection. Too many in quota", HE_LOGID(he));
        lcbio_unref(info->sock);
        return;
    }

    lcb_log(LOGARGS(mgr, INFO), HE_LOGFMT "Placing socket back into the pool. I=%p,C=%p", HE_LOGID(he), (void *)info,
            (void *)sock);
    info->idle_timer.rearm(mgr->options.tmoidle);
    lcb_clist_append(&he->ll_idle, info);
    info->state = PoolConnInfo::IDLE;
}

void Pool::discard(lcbio_SOCKET *sock)
{
    lcbio_unref(sock);
}

void Pool::detach(lcbio_SOCKET *sock)
{
    lcbio_protoctx_delid(sock, LCBIO_PROTOCTX_POOL, 1);
}

bool Pool::is_from_pool(const lcbio_SOCKET *sock)
{
    return lcbio_protoctx_get(sock, LCBIO_PROTOCTX_POOL) != NULL;
}

#define CONN_INDENT "    "

static void write_he_list(const lcb_clist_t *ll, FILE *out)
{
    lcb_list_t *llcur;
    LCB_LIST_FOR(llcur, (lcb_list_t *)ll)
    {
        PoolConnInfo *info = PoolConnInfo::from_llnode(llcur);
        fprintf(out, "%sCONN [I=%p,C=%p ", CONN_INDENT, (void *)info, (void *)&info->sock);

        if (info->sock->io->model == LCB_IOMODEL_EVENT) {
            fprintf(out, "SOCKFD=%d", (int)info->sock->u.fd);
        } else {
            fprintf(out, "SOCKDATA=%p", (void *)info->sock->u.sd);
        }
        fprintf(out, " STATE=0x%x", info->state);
        fprintf(out, "]\n");
    }
}

void PoolHost::dump(FILE *out) const
{
    lcb_list_t *llcur;
    fprintf(out, "HOST=%s", key.c_str());
    fprintf(out, "Requests=%lu, Idle=%lu, Pending=%lu, Leased=%lu\n", (unsigned long int)num_requests(),
            (unsigned long int)num_idle(), (unsigned long int)num_pending(), (unsigned long int)num_leased());

    fprintf(out, CONN_INDENT "Idle Connections:\n");
    write_he_list(&ll_idle, out);
    fprintf(out, CONN_INDENT "Pending Connections: \n");
    write_he_list(&ll_pending, out);
    fprintf(out, CONN_INDENT "Pending Requests:\n");

    LCB_LIST_FOR(llcur, (lcb_list_t *)&requests)
    {
        PoolRequest *req = PoolRequest::from_llnode(llcur);
        union {
            lcbio_CONNDONE_cb cb;
            void *ptr;
        } u_cb;

        u_cb.cb = req->callback;

        fprintf(out, "%sREQ [R=%p, Callback=%p, Data=%p, State=0x%x]\n", CONN_INDENT, (void *)req, u_cb.ptr,
                (void *)req->arg, req->state);
    }

    fprintf(out, "\n");
}

void Pool::dump(FILE *out) const
{
    if (out == NULL) {
        out = stderr;
    }
    HostMap::const_iterator ii;
    for (ii = ht.begin(); ii != ht.end(); ++ii) {
        ii->second->dump(out);
    }
}
