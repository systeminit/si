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

#include "config.h"
#include "connect.h"
#include "ioutils.h"
#include "iotable.h"
#include "settings.h"
#include "timer-ng.h"
#include "timer-cxx.h"
#include "rnd.h"
#include <errno.h>

using namespace lcb::io;

/* win32 lacks EAI_SYSTEM */
#ifndef EAI_SYSTEM
#define EAI_SYSTEM 0
#endif
#define LOGARGS(conn, lvl) conn->settings, "connection", LCB_LOG_##lvl, __FILE__, __LINE__
static const lcb_host_t *get_loghost(lcbio_SOCKET *s)
{
    static lcb_host_t host = {"NOHOST", "NOPORT", 0};
    if (!s) {
        return &host;
    }
    if (!s->info) {
        return &host;
    }
    return &s->info->ep;
}

/** Format string arguments for %p%s:%s */
#define CSLOGID(sock)                                                                                                  \
    sock->settings->log_redaction ? LCB_LOG_SD_OTAG : "", get_loghost(sock)->ipv6 ? "[" : "", get_loghost(sock)->host, \
        get_loghost(sock)->ipv6 ? "]" : "", get_loghost(sock)->port,                                                   \
        sock->settings->log_redaction ? LCB_LOG_SD_CTAG : "", sock->id
#define CSLOGFMT "<" LCB_LOG_SPEC("%s%s%s:%s") "> (SOCK=%016" PRIx64 ") "

#define LOGARGS_T(lvl) LOGARGS(this->sock, lvl)
#define CSLOGID_T() CSLOGID(this->sock)

namespace lcb
{
namespace io
{
struct Connstart : ConnectionRequest {
    Connstart(lcbio_TABLE *, lcb_settings *, const lcb_host_t *, uint32_t, lcbio_CONNDONE_cb, void *);

    ~Connstart();
    void unwatch();
    void handler();
    void cancel();
    void C_connect();

    enum State { CS_PENDING, CS_CANCELLED, CS_CONNECTED, CS_ERROR };

    void state_signal(State next_state, lcb_STATUS status);
    void notify_success();
    void notify_error(lcb_STATUS err);
    bool ensure_sock();
    void clear_sock();

    lcbio_CONNDONE_cb user_handler;
    void *user_arg;

    lcbio_SOCKET *sock;
    lcbio_OSERR syserr;
    void *event;
    bool ev_active;   /* whether the event pointer is active (Event only) */
    bool in_uhandler; /* Whether we're inside the user-defined handler */
    addrinfo *ai_root;
    addrinfo *ai;
    State state;
    lcb_STATUS last_error;
    Timer< Connstart, &Connstart::handler > timer;
};
} // namespace io
} // namespace lcb

void Connstart::unwatch()
{
    if (sock && ev_active) {
        lcb_assert(sock->u.fd != INVALID_SOCKET);
        sock->io->E_event_cancel(sock->u.fd, event);
        ev_active = false;
    }
}

static void try_enable_sockopt(lcbio_SOCKET *sock, int cntl)
{
    lcb_STATUS rv = lcbio_enable_sockopt(sock, cntl);
    if (rv == LCB_SUCCESS) {
        lcb_log(LOGARGS(sock, DEBUG), CSLOGFMT "Successfully set %s", CSLOGID(sock), lcbio_strsockopt(cntl));
    } else {
        lcb_log(LOGARGS(sock, INFO), CSLOGFMT "Couldn't set %s", CSLOGID(sock), lcbio_strsockopt(cntl));
    }
}

/**
 * Handler invoked to deliver final status for a connection. This will invoke
 * the user supplied callback with the relevant status (if it has not been
 * cancelled) and then free the CONNSTART object.
 */
void Connstart::handler()
{
    lcb_STATUS err;

    if (sock && event) {
        unwatch();
        sock->io->E_event_destroy(event);
    }

    if (state == CS_PENDING) {
        /* state was not changed since initial scheduling */
        err = LCB_ETIMEDOUT;
    } else if (state == CS_CONNECTED) {
        /* clear pending error */
        err = LCB_SUCCESS;
    } else {
        if (sock != NULL && last_error == LCB_CONNECT_ERROR) {
            err = lcbio_mklcberr(syserr, sock->settings);
        } else {
            err = last_error;
        }
    }

    if (state == CS_CANCELLED) {
        /* ignore everything. Clean up resources */
        goto GT_DTOR;
    }

    if (sock) {
        lcbio__load_socknames(sock);
        if (err == LCB_SUCCESS) {
            lcb_log(LOGARGS_T(INFO), CSLOGFMT "Connected established", CSLOGID_T());

            if (sock->settings->tcp_nodelay) {
                try_enable_sockopt(sock, LCB_IO_CNTL_TCP_NODELAY);
            }
            if (sock->settings->tcp_keepalive) {
                try_enable_sockopt(sock, LCB_IO_CNTL_TCP_KEEPALIVE);
            }
        } else {
            lcb_log(LOGARGS_T(ERR), CSLOGFMT "Failed to establish connection: %s, os errno=%u", CSLOGID_T(),
                    lcb_strerror_short(err), syserr);
        }
    }

    /** Handler section */
    in_uhandler = true;
    user_handler(err == LCB_SUCCESS ? sock : NULL, user_arg, err, syserr);
    in_uhandler = false;

GT_DTOR:
    delete this;
}

Connstart::~Connstart()
{
    timer.release();
    if (sock) {
        lcbio_unref(sock);
    }
    if (ai_root) {
        freeaddrinfo(ai_root);
    }
}

void Connstart::state_signal(State next_state, lcb_STATUS err)
{
    if (state != CS_PENDING) {
        /** State already set */
        return;
    }

    if (state == CS_CONNECTED) {
        /* clear last errors if we're successful */
        last_error = LCB_SUCCESS;
    } else if (last_error == LCB_SUCCESS) {
        /* set error code only if previous code was not a failure */
        last_error = err;
    }

    state = next_state;
    timer.signal();
}

void Connstart::notify_success()
{
    state_signal(CS_CONNECTED, LCB_SUCCESS);
}

void Connstart::notify_error(lcb_STATUS err)
{
    state_signal(CS_ERROR, err);
}

/** Cancels and mutes any pending event */
void lcbio_connect_cancel(lcbio_pCONNSTART cs)
{
    cs->cancel();
}

void Connstart::cancel()
{
    if (in_uhandler) {
        /* already inside user-defined handler */
        return;
    }
    state = CS_CANCELLED;
    handler();
}

bool Connstart::ensure_sock()
{
    lcbio_TABLE *io = sock->io;
    int errtmp = 0;

    if (ai == NULL) {
        return false;
    }

    if (io->is_E()) {
        if (sock->u.fd != INVALID_SOCKET) {
            /* already have one? */
            return true;
        }

        while (sock->u.fd == INVALID_SOCKET && ai != NULL) {
            sock->u.fd = lcbio_E_ai2sock(io, &ai, &errtmp);
            if (sock->u.fd != INVALID_SOCKET) {
                lcb_log(LOGARGS_T(DEBUG), CSLOGFMT "Created new socket with FD=%d", CSLOGID_T(), sock->u.fd);
                return true;
            }
        }
    } else {
        if (sock->u.sd) {
            return true;
        }

        while (sock->u.sd == NULL && ai != NULL) {
            sock->u.sd = lcbio_C_ai2sock(io, &ai, &errtmp);
            if (sock->u.sd) {
                sock->u.sd->lcbconn = const_cast< lcbio_SOCKET * >(sock);
                sock->u.sd->parent = IOT_ARG(io);
                return true;
            }
        }
    }

    if (ai == NULL) {
        lcbio_mksyserr(IOT_ERRNO(io), &syserr);
        return false;
    }
    return true;
}

void Connstart::clear_sock()
{
    lcbio_TABLE *iot = sock->io;
    if (ai) {
        ai = ai->ai_next;
    }

    if (!ai) {
        return;
    }

    if (iot->is_E()) {
        unwatch();
        iot->E_close(sock->u.fd);
        sock->u.fd = INVALID_SOCKET;
    } else {
        if (sock->u.sd) {
            iot->C_close(sock->u.sd);
            sock->u.sd = NULL;
        }
    }
}

static void E_conncb(lcb_socket_t, short events, void *arg)
{
    Connstart *cs = reinterpret_cast< Connstart * >(arg);
    lcbio_SOCKET *s = cs->sock;
    lcbio_TABLE *io = s->io;
    int retry_once = 0;
    lcbio_CSERR connstatus;
    int rv = 0;
    addrinfo *ai = NULL;

GT_NEXTSOCK:
    if (!cs->ensure_sock()) {
        cs->notify_error(LCB_CONNECT_ERROR);
        return;
    }

    if (events & LCB_ERROR_EVENT) {
        socklen_t errlen = sizeof(int);
        int sockerr = 0;
        lcb_log(LOGARGS(s, TRACE), CSLOGFMT "Received ERROR_EVENT", CSLOGID(s));
        getsockopt(s->u.fd, SOL_SOCKET, SO_ERROR, (char *)&sockerr, &errlen);
        lcbio_mksyserr(sockerr, &cs->syserr);
        cs->clear_sock();
        goto GT_NEXTSOCK;

    } else {
        ai = cs->ai;

    GT_CONNECT:
        rv = io->E_connect(s->u.fd, ai->ai_addr, ai->ai_addrlen);
        if (rv == 0) {
            cs->unwatch();
            cs->notify_success();
            return;
        }
    }

    connstatus = lcbio_mkcserr(io->get_errno());
    lcbio_mksyserr(io->get_errno(), &cs->syserr);

    switch (connstatus) {

        case LCBIO_CSERR_INTR:
            goto GT_CONNECT;

        case LCBIO_CSERR_CONNECTED:
            cs->unwatch();
            cs->notify_success();
            return;

        case LCBIO_CSERR_BUSY:
            lcb_log(LOGARGS(s, TRACE), CSLOGFMT "Scheduling I/O watcher for asynchronous connection completion.",
                    CSLOGID(s));
            io->E_event_watch(s->u.fd, cs->event, LCB_WRITE_EVENT, cs, E_conncb);
            cs->ev_active = 1;
            return;

        case LCBIO_CSERR_EINVAL:
            if (!retry_once) {
                retry_once = 1;
                goto GT_CONNECT;
            }
            /* fallthrough */

        case LCBIO_CSERR_EFAIL:
        default:
            /* close the current socket and try again */
            lcb_log(LOGARGS(s, TRACE), CSLOGFMT "connect() failed. errno=%d [%s]", CSLOGID(s), IOT_ERRNO(io),
                    strerror(IOT_ERRNO(io)));
            cs->clear_sock();
            goto GT_NEXTSOCK;
    }
}

static void C_conncb(lcb_sockdata_t *sock, int status)
{
    lcbio_SOCKET *s = reinterpret_cast< lcbio_SOCKET * >(sock->lcbconn);
    Connstart *cs = reinterpret_cast< Connstart * >(s->ctx);

    lcb_log(LOGARGS(s, TRACE), CSLOGFMT "Received completion handler. Status=%d. errno=%d [%s]", CSLOGID(s), status,
            IOT_ERRNO(s->io), strerror(IOT_ERRNO(s->io)));

    if (!--s->refcount) {
        lcbio__destroy(s);
        return;
    }

    if (!status) {
        if (cs->state == Connstart::CS_PENDING) {
            cs->state = Connstart::CS_CONNECTED;
        }
        cs->handler();
    } else {
        lcbio_mksyserr(s->io->get_errno(), &cs->syserr);
        cs->clear_sock();
        cs->C_connect();
    }
}

void Connstart::C_connect()
{
    int rv;
    bool retry_once = 0;
    lcbio_CSERR status;
    lcbio_TABLE *io = sock->io;

GT_NEXTSOCK:
    if (!ensure_sock()) {
        lcbio_mksyserr(IOT_ERRNO(io), &syserr);
        notify_error(LCB_CONNECT_ERROR);
        return;
    }

GT_CONNECT:
    rv = io->C_connect(sock->u.sd, ai->ai_addr, ai->ai_addrlen, C_conncb);
    if (rv == 0) {
        lcbio_ref(sock);
        return;
    }

    lcbio_mksyserr(io->get_errno(), &syserr);
    status = lcbio_mkcserr(io->get_errno());
    switch (status) {

        case LCBIO_CSERR_INTR:
            goto GT_CONNECT;

        case LCBIO_CSERR_CONNECTED:
            notify_success();
            return;

        case LCBIO_CSERR_BUSY:
            return;

        case LCBIO_CSERR_EINVAL:
            if (!retry_once) {
                retry_once = 1;
                goto GT_CONNECT;
            }
            /* fallthrough */

        case LCBIO_CSERR_EFAIL:
        default:
            clear_sock();
            goto GT_NEXTSOCK;
    }
}

ConnectionRequest *lcbio_connect(lcbio_TABLE *iot, lcb_settings *settings, const lcb_host_t *dest, uint32_t timeout,
                                 lcbio_CONNDONE_cb handler, void *arg)
{
    return new Connstart(iot, settings, dest, timeout, handler, arg);
}

Connstart::Connstart(lcbio_TABLE *iot_, lcb_settings *settings_, const lcb_host_t *dest, uint32_t timeout,
                     lcbio_CONNDONE_cb handler_, void *arg)
    : user_handler(handler_), user_arg(arg), sock(NULL), syserr(0), event(NULL), ev_active(false), in_uhandler(false),
      ai_root(NULL), ai(NULL), state(CS_PENDING), last_error(LCB_SUCCESS), timer(iot_, this)
{

    addrinfo hints;
    int rv;

    sock = reinterpret_cast< lcbio_SOCKET * >(calloc(1, sizeof(*sock)));

    /** Initialize the socket first */
    sock->io = iot_;
    sock->settings = settings_;
    sock->ctx = this;
    sock->refcount = 1;
    sock->id = lcb_next_rand64();
    sock->info = reinterpret_cast< lcbio_CONNINFO * >(calloc(1, sizeof(*sock->info)));
    sock->info->ep = *dest;
    lcbio_table_ref(sock->io);
    lcb_settings_ref(sock->settings);
    lcb_list_init(&sock->protos);

    if (iot_->is_E()) {
        sock->u.fd = INVALID_SOCKET;
        event = iot_->E_event_create();
    }

    timer.rearm(timeout);
    lcb_log(LOGARGS_T(INFO), CSLOGFMT "Starting. Timeout=%uus", CSLOGID_T(), timeout);

    /** Hostname lookup: */
    memset(&hints, 0, sizeof(hints));
    hints.ai_flags = AI_PASSIVE;
    hints.ai_socktype = SOCK_STREAM;
    if (settings_->ipv6 == LCB_IPV6_DISABLED) {
        hints.ai_family = AF_INET;
    } else if (settings_->ipv6 == LCB_IPV6_ONLY) {
        hints.ai_family = AF_INET6;
    } else {
        hints.ai_family = AF_UNSPEC;
    }

    if ((rv = getaddrinfo(dest->host, dest->port, &hints, &ai_root))) {
        const char *errstr = rv != EAI_SYSTEM ? gai_strerror(rv) : "";
        lcb_log(LOGARGS_T(ERR), CSLOGFMT "Couldn't look up %s (%s) [EAI=%d]", CSLOGID_T(), dest->host, errstr, rv);
        notify_error(LCB_UNKNOWN_HOST);
    } else {
        ai = ai_root;

        /** Figure out how to connect */
        if (iot_->is_E()) {
            E_conncb(-1, LCB_WRITE_EVENT, this);
        } else {
            C_connect();
        }
    }
}

ConnectionRequest *lcbio_connect_hl(lcbio_TABLE *iot, lcb_settings *settings, lcb::Hostlist *hl, int rollover,
                                    uint32_t timeout, lcbio_CONNDONE_cb handler, void *arg)
{
    const lcb_host_t *cur;
    unsigned ii = 0, hlmax = hl->size();

    while ((cur = hl->next(rollover)) && ii++ < hlmax) {
        ConnectionRequest *ret = lcbio_connect(iot, settings, cur, timeout, handler, arg);
        if (ret) {
            return ret;
        }
    }

    return NULL;
}

lcbio_SOCKET *lcbio_wrap_fd(lcbio_pTABLE iot, lcb_settings *settings, lcb_socket_t fd)
{
    lcbio_SOCKET *ret = reinterpret_cast< lcbio_SOCKET * >(calloc(1, sizeof(*ret)));
    lcbio_CONNDONE_cb *ci = reinterpret_cast< lcbio_CONNDONE_cb * >(calloc(1, sizeof(*ci)));

    if (ret == NULL || ci == NULL) {
        free(ret);
        free(ci);
        return NULL;
    }

    lcb_assert(iot->model = LCB_IOMODEL_EVENT);

    lcb_list_init(&ret->protos);
    ret->settings = settings;
    ret->io = iot;
    ret->refcount = 1;
    ret->u.fd = fd;
    ret->id = lcb_next_rand64();

    lcbio_table_ref(ret->io);
    lcb_settings_ref(ret->settings);
    lcbio__load_socknames(ret);
    return ret;
}

void lcbio_shutdown(lcbio_SOCKET *s)
{
    lcbio_TABLE *io = s->io;

    lcbio__protoctx_delall(s);
    if (IOT_IS_EVENT(io)) {
        if (s->u.fd != INVALID_SOCKET) {
            io->E_close(s->u.fd);
            s->u.fd = INVALID_SOCKET;
        }
    } else {
        if (s->u.sd) {
            io->C_close(s->u.sd);
            s->u.sd = NULL;
        }
    }
}

void lcbio__destroy(lcbio_SOCKET *s)
{
    lcbio_shutdown(s);
    if (s->info) {
        free(s->info);
    }
    lcbio_table_unref(s->io);
    lcb_settings_unref(s->settings);
    free(s);
}

const char *lcbio_svcstr(lcbio_SERVICE service)
{
    switch (service) {
        case LCBIO_SERVICE_CFG:
            return "config";
        case LCBIO_SERVICE_KV:
            return "kv";
        case LCBIO_SERVICE_MGMT:
            return "mgmt";
        case LCBIO_SERVICE_VIEW:
            return "view";
        case LCBIO_SERVICE_N1QL:
            return "n1ql";
        case LCBIO_SERVICE_FTS:
            return "fts";
        case LCBIO_SERVICE_CBAS:
            return "cbas";
        case LCBIO_SERVICE_UNSPEC:
            /* fallthrough */
        default:
            return "unspec";
    }
}
