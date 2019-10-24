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

#undef NDEBUG
#include "socktest.h"
#include <lcbio/ssl.h>
using std::list;

extern "C" {
static void ctx_error(lcbio_CTX *ctx, lcb_STATUS err)
{
    ESocket *s = (ESocket *)lcbio_ctx_data(ctx);
    s->lasterr = err;
    s->actions->onError(s);
    s->parent->stop();
}

static void ctx_read(lcbio_CTX *ctx, unsigned nr)
{
    ESocket *s = (ESocket *)lcbio_ctx_data(ctx);
    s->actions->onRead(s, nr);
}

static void conn_cb(lcbio_SOCKET *sock, void *data, lcb_STATUS err, lcbio_OSERR oserr)
{
    ESocket *mysock = (ESocket *)data;
    mysock->assign(sock, err);
    mysock->parent->stop();
    mysock->syserr = oserr;
    mysock->callCount++;
}

static void ctx_flush_ready(lcbio_CTX *ctx)
{
    ESocket *s = (ESocket *)lcbio_ctx_data(ctx);
    s->actions->onFlushReady(s);
}

static void ctx_flush_done(lcbio_CTX *ctx, unsigned expected, unsigned nr)
{
    ESocket *s = (ESocket *)lcbio_ctx_data(ctx);
    s->actions->onFlushDone(s, expected, nr);
}
}

void IOActions::onError(ESocket *)
{
    // noop for now
}

void IOActions::onRead(ESocket *s, size_t nr)
{
    lcbio_CTXRDITER iter;
    LCBIO_CTX_ITERFOR(s->ctx, &iter, nr)
    {
        char *curbuf = (char *)lcbio_ctx_ribuf(&iter);
        unsigned nbuf = lcbio_ctx_risize(&iter);
        s->readbuf.insert(s->readbuf.end(), curbuf, curbuf + nbuf);
    }
}

IOActions ESocket::defaultActions;

void ESocket::assign(lcbio_SOCKET *s, lcb_STATUS err)
{
    creq = NULL;
    if (s == NULL) {
        lasterr = err;
        return;
    }

    err = lcbio_sslify_if_needed(s, s->settings);
    if (err != LCB_SUCCESS) {
        lasterr = err;
        return;
    }

    lcbio_CTXPROCS procs;
    procs.cb_err = ctx_error;
    procs.cb_read = ctx_read;
    procs.cb_flush_done = ctx_flush_done;
    procs.cb_flush_ready = ctx_flush_ready;
    sock = s;
    ctx = lcbio_ctx_new(s, this, &procs);
}

extern "C" {
static void close_cb(lcbio_SOCKET *s, int reusable, void *arg)
{
    (void)arg;
    if (reusable) {
        lcbio_ref(s);
        lcb::io::Pool::put(s);
    }
}
}

void ESocket::close()
{
    if (!ctx) {
        return;
    }

    if (lcb::io::Pool::is_from_pool(ctx->sock)) {
        lcbio_ctx_close(ctx, close_cb, NULL);
    } else {
        lcbio_ctx_close(ctx, NULL, NULL);
    }

    sock = NULL;
    ctx = NULL;
}

class BreakTimer : public Timer
{
  public:
    BreakTimer(Loop *l) : Timer(l->iot)
    {
        this->loop = l;
    }

    void expired()
    {
        if (!loop->bcond) {
            return;
        }
        if (loop->bcond->shouldBreak()) {
            loop->stop();
        } else {
            loop->scheduleBreak();
        }
    }
    virtual ~BreakTimer() {}

  private:
    Loop *loop;
};

Loop::Loop()
{
    io = NULL;

    lcb_STATUS rc = lcb_create_io_ops(&io, NULL);
    assert(rc == LCB_SUCCESS);
    assert(io != NULL);
    iot = lcbio_table_new(io);
    settings = lcb_settings_new();
    settings->logger = lcb_init_console_logger();
    server = new TestServer();
    breakTimer = new BreakTimer(this);
    bcond = NULL;
    sockpool = new lcb::io::Pool(settings, iot);
}

Loop::~Loop()
{
    delete breakTimer;
    delete server;
    sockpool->shutdown();
    lcbio_table_unref(iot);
    lcb_destroy_io_ops(io);
    lcb_settings_unref(settings);
}

void Loop::scheduleBreak()
{
    breakTimer->schedule(2); // 2ms
}

void Loop::cancelBreak()
{
    breakTimer->cancel();
}

void Loop::start()
{
    if (bcond) {
        scheduleBreak();
    }
    IOT_START(iot);
    cancelBreak();
    bcond = NULL;
}

void Loop::stop()
{
    cancelBreak();
    IOT_STOP(iot);
}

void Loop::initSockCommon(ESocket *sock)
{
    // Find the peer..
    struct sockaddr_in *addr = (struct sockaddr_in *)&sock->sock->info->sa_local;
    uint16_t port = ntohs(addr->sin_port);
    sock->conn = server->findConnection(port);
}

void Loop::connectPooled(ESocket *sock, lcb_host_t *host, unsigned mstmo)
{
    lcb_host_t tmphost = {0};
    sock->parent = this;
    if (!host) {
        populateHost(&tmphost);
        host = &tmphost;
    }
    sock->creq = sockpool->get(*host, LCB_MS2US(mstmo), conn_cb, sock);
    start();
    if (sock->sock) {
        initSockCommon(sock);
    }
}

void Loop::connect(ESocket *sock, lcb_host_t *host, unsigned mstmo)
{
    lcb_host_t tmphost = {0};

    if (host == NULL) {
        populateHost(&tmphost);
        host = &tmphost;
    }

    sock->parent = this;
    sock->creq = lcbio_connect(iot, settings, host, LCB_MS2US(mstmo), conn_cb, sock);

    start();

    if (sock->sock) {
        initSockCommon(sock);
    }
}

void Loop::populateHost(lcb_host_t *host)
{
    strcpy(host->host, server->getHostString().c_str());
    strcpy(host->port, server->getPortString().c_str());
}

extern "C" {
static void timerCallback(void *arg)
{
    Timer *tm = (Timer *)arg;
    tm->expired();
}
}

Timer::Timer(lcbio_TABLE *iot)
{
    timer = lcbio_timer_new(iot, this, timerCallback);
}

Timer::~Timer()
{
    destroy();
}

void Timer::destroy()
{
    lcbio_timer_destroy(timer);
    timer = NULL;
}

void Timer::cancel()
{
    lcbio_timer_disarm(timer);
}

void Timer::schedule(unsigned ms)
{
    lcbio_timer_rearm(timer, LCB_MS2US(ms));
}

void Timer::signal()
{
    schedule(0);
}

// Conditions
BreakCondition::BreakCondition()
{
    broke = false;
}

bool BreakCondition::shouldBreak()
{
    if (shouldBreakImpl()) {
        broke = true;
        return true;
    }
    return false;
}

bool FlushedBreakCondition::shouldBreakImpl()
{
    lcbio_CTX *ctx = sock->ctx;
    if (ctx->npending) {
        return false;
    }

    if (ctx->output && ctx->output->rb.nbytes == 0) {
        return true;
    }
    return false;
}

bool ReadBreakCondition::shouldBreakImpl()
{
    lcbio_CTX *ctx = sock->ctx;
    unsigned unread = rdb_get_nused(&ctx->ior);
    return unread + sock->getReceived().size() >= expected;
}

extern "C" {
static void dtor_cb(lcbio_CTX *ctx)
{
    CtxCloseBreakCondition *bc = (CtxCloseBreakCondition *)lcbio_ctx_data(ctx);
    bc->gotDtor();
}
}
void CtxCloseBreakCondition::closeCtx()
{
    lcbio_ctx_close_ex(s->ctx, NULL, NULL, dtor_cb, this);
    s->ctx = NULL;
}
