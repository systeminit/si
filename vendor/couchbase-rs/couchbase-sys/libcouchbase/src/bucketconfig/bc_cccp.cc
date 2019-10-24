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

/**
 * This file contains the CCCP (Cluster Carrier Configuration Protocol)
 * implementation of the confmon provider. It utilizes a memcached connection
 * to retrieve configuration information.
 */

#include "internal.h"
#include "clconfig.h"
#include "packetutils.h"
#include <mcserver/negotiate.h>
#include <lcbio/lcbio.h>
#include <lcbio/timer-cxx.h>
#include <lcbio/ssl.h>
#include "ctx-log-inl.h"

#include <cstring>

#define LOGFMT CTX_LOGFMT
#define LOGID(p) CTX_LOGID(p->ioctx)
#define LOGARGS(cccp, lvl) cccp->parent->settings, "cccp", LCB_LOG_##lvl, __FILE__, __LINE__

struct CccpCookie;

using namespace lcb::clconfig;

struct CccpProvider : public Provider {
    CccpProvider(Confmon *);
    ~CccpProvider();

    /**
     * Stops the current request.
     * @param is_clean Whether the state of the current request is 'clean',
     *        i.e. whether we are stopping because of an error condition, or
     *        because we have received a successful response.
     */
    void stop_current_request(bool is_clean);
    lcb_STATUS schedule_next_request(lcb_STATUS why, bool can_rollover);
    lcb_STATUS mcio_error(lcb_STATUS why);
    void on_timeout()
    {
        mcio_error(LCB_ETIMEDOUT);
    }
    lcb_STATUS update(const char *host, const char *data);
    void request_config();
    void on_io_read();

    bool pause();                                // Override
    void configure_nodes(const lcb::Hostlist &); // Override
    void config_updated(lcbvb_CONFIG *);         // Override;
    void dump(FILE *) const;                     // Override
    lcb_STATUS refresh();                        // Override

    ConfigInfo *get_cached() /* Override */
    {
        return config;
    }

    const lcb::Hostlist *get_nodes() const /* Override */
    {
        return nodes;
    }

    void enable(void *arg)
    {
        instance = reinterpret_cast< lcb_INSTANCE * >(arg);
        Provider::enable();
    }

    // Whether there is a pending CCCP config request.
    bool has_pending_request() const
    {
        return creq != NULL || cmdcookie != NULL || ioctx != NULL;
    }

    lcb::Hostlist *nodes;
    ConfigInfo *config;
    lcb::io::Timer< CccpProvider, &CccpProvider::on_timeout > timer;
    lcb_INSTANCE *instance;
    lcb::io::ConnectionRequest *creq;
    lcbio_CTX *ioctx;
    CccpCookie *cmdcookie;
};

struct CccpCookie {
    CccpProvider *parent;
    bool active;
    lcb_STATUS select_rc;
    CccpCookie(CccpProvider *parent_) : parent(parent_), active(true), select_rc(LCB_SUCCESS) {}
};

static void io_error_handler(lcbio_CTX *, lcb_STATUS);
static void io_read_handler(lcbio_CTX *, unsigned nr);
static void on_connected(lcbio_SOCKET *, void *, lcb_STATUS, lcbio_OSERR);

static void pooled_close_cb(lcbio_SOCKET *sock, int reusable, void *arg)
{
    bool *ru_ex = reinterpret_cast< bool * >(arg);
    lcbio_ref(sock);
    if (reusable && *ru_ex) {
        lcb::io::Pool::put(sock);
    } else {
        lcb::io::Pool::discard(sock);
    }
}

void CccpProvider::stop_current_request(bool is_clean)
{
    if (cmdcookie) {
        cmdcookie->active = false;
        cmdcookie = NULL;
    }

    lcb::io::ConnectionRequest::cancel(&creq);

    if (ioctx) {
        lcbio_ctx_close(ioctx, pooled_close_cb, &is_clean);
        ioctx = NULL;
    }
}

lcb_STATUS CccpProvider::schedule_next_request(lcb_STATUS err, bool can_rollover)
{
    lcb_host_t *next_host = nodes->next(can_rollover);
    if (!next_host) {
        timer.cancel();
        parent->provider_failed(this, err);
        return err;
    }

    lcb::Server *server = instance->find_server(*next_host);
    if (server) {
        cmdcookie = new CccpCookie(this);
        lcb_log(LOGARGS(this, TRACE), "Re-Issuing CCCP Command on server struct %p (" LCB_HOST_FMT ")", (void *)server,
                LCB_HOST_ARG(this->parent->settings, next_host));
        timer.rearm(settings().config_node_timeout);
        if (settings().bucket && settings().bucket[0] != '\0' && config && config->vbc->bname == NULL) {
            instance->select_bucket(cmdcookie, server);
        }
        instance->request_config(cmdcookie, server);

    } else {

        lcb_log(LOGARGS(this, INFO), "Requesting connection to node " LCB_HOST_FMT " for CCCP configuration",
                LCB_HOST_ARG(this->parent->settings, next_host));
        creq = instance->memd_sockpool->get(*next_host, settings().config_node_timeout, on_connected, this);
    }

    return LCB_SUCCESS;
}

lcb_STATUS CccpProvider::mcio_error(lcb_STATUS err)
{
    if (err != LCB_NOT_SUPPORTED && err != LCB_UNKNOWN_COMMAND) {
        lcb_log(LOGARGS(this, ERR), LOGFMT "Could not get configuration: %s", LOGID(this), lcb_strerror_short(err));
    }

    stop_current_request(err == LCB_NOT_SUPPORTED);
    return schedule_next_request(err, false);
}

/** Update the configuration from a server. */
lcb_STATUS lcb::clconfig::cccp_update(Provider *provider, const char *host, const char *data)
{
    return static_cast< CccpProvider * >(provider)->update(host, data);
}

lcb_STATUS CccpProvider::update(const char *host, const char *data)
{
    lcbvb_CONFIG *vbc;
    int rv;
    ConfigInfo *new_config;
    vbc = lcbvb_create();

    if (!vbc) {
        return LCB_CLIENT_ENOMEM;
    }
    rv = lcbvb_load_json_ex(vbc, data, host, &LCBT_SETTING(this->parent, network));

    if (rv) {
        lcb_log(LOGARGS(this, ERROR), LOGFMT "Failed to parse config", LOGID(this));
        lcb_log_badconfig(LOGARGS(this, ERROR), vbc, data);
        lcbvb_destroy(vbc);
        return LCB_PROTOCOL_ERROR;
    }

    lcbvb_replace_host(vbc, host);
    new_config = ConfigInfo::create(vbc, CLCONFIG_CCCP);

    if (!new_config) {
        lcbvb_destroy(vbc);
        return LCB_CLIENT_ENOMEM;
    }

    if (config) {
        config->decref();
    }

    /** TODO: Figure out the comparison vector */
    config = new_config;
    parent->provider_got_config(this, new_config);
    return LCB_SUCCESS;
}

void lcb::clconfig::select_status(const void *cookie_, lcb_STATUS err)
{
    CccpCookie *cookie = reinterpret_cast< CccpCookie * >(const_cast< void * >(cookie_));
    cookie->select_rc = err;
}

void lcb::clconfig::cccp_update(const void *cookie_, lcb_STATUS err, const void *bytes, size_t nbytes,
                                const lcb_host_t *origin)
{
    CccpCookie *cookie = reinterpret_cast< CccpCookie * >(const_cast< void * >(cookie_));
    CccpProvider *cccp = cookie->parent;

    lcb_STATUS select_rc = cookie->select_rc;
    bool was_active = cookie->active;
    if (cookie->active) {
        cookie->active = false;
        cccp->timer.cancel();
        cccp->cmdcookie = NULL;
    }
    delete cookie;

    if (select_rc != LCB_SUCCESS) {
        cccp->mcio_error(select_rc);
        return;
    }

    if (err == LCB_SUCCESS) {
        std::string ss(reinterpret_cast< const char * >(bytes), nbytes);
        err = cccp->update(origin->host, ss.c_str());
    }

    if (err != LCB_SUCCESS && was_active) {
        cccp->mcio_error(err);
    }
}

static void on_connected(lcbio_SOCKET *sock, void *data, lcb_STATUS err, lcbio_OSERR)
{
    lcbio_CTXPROCS ioprocs;
    CccpProvider *cccp = reinterpret_cast< CccpProvider * >(data);
    lcb_settings *settings = cccp->parent->settings;
    cccp->creq = NULL;

    if (err != LCB_SUCCESS) {
        if (sock) {
            lcb::io::Pool::discard(sock);
        }
        cccp->mcio_error(err);
        return;
    }

    if (lcbio_protoctx_get(sock, LCBIO_PROTOCTX_SESSINFO) == NULL) {
        cccp->creq = lcb::SessionRequest::start(sock, settings, settings->config_node_timeout, on_connected, cccp);
        return;
    }

    ioprocs.cb_err = io_error_handler;
    ioprocs.cb_read = io_read_handler;
    cccp->ioctx = lcbio_ctx_new(sock, data, &ioprocs);
    cccp->ioctx->subsys = "bc_cccp";
    sock->service = LCBIO_SERVICE_CFG;
    cccp->request_config();
}

lcb_STATUS CccpProvider::refresh()
{
    if (has_pending_request()) {
        return LCB_BUSY;
    }

    return schedule_next_request(LCB_SUCCESS, true);
}

bool CccpProvider::pause()
{
    if (!has_pending_request()) {
        return true;
    }

    stop_current_request(false);
    timer.cancel();
    return true;
}

CccpProvider::~CccpProvider()
{
    stop_current_request(false);

    if (config) {
        config->decref();
    }
    if (nodes) {
        delete nodes;
    }
    timer.release();
}

void CccpProvider::configure_nodes(const lcb::Hostlist &nodes_)
{
    nodes->assign(nodes_);
    if (parent->settings->randomize_bootstrap_nodes) {
        nodes->randomize();
    }
}

void CccpProvider::config_updated(lcbvb_CONFIG *vbc)
{
    lcbvb_SVCMODE mode = LCBT_SETTING_SVCMODE(parent);
    if (LCBVB_NSERVERS(vbc) < 1) {
        return;
    }

    nodes->clear();
    for (size_t ii = 0; ii < LCBVB_NSERVERS(vbc); ii++) {
        const char *mcaddr = lcbvb_get_hostport(vbc, ii, LCBVB_SVCTYPE_DATA, mode);
        if (!mcaddr) {
            lcb_log(LOGARGS(this, DEBUG), "Node %lu has no data service", (unsigned long int)ii);
            continue;
        }
        nodes->add(mcaddr, LCB_CONFIG_MCD_PORT);
    }

    if (settings().randomize_bootstrap_nodes) {
        nodes->randomize();
    }
}

static void io_error_handler(lcbio_CTX *ctx, lcb_STATUS err)
{
    CccpProvider *cccp = reinterpret_cast< CccpProvider * >(lcbio_ctx_data(ctx));
    cccp->mcio_error(err);
}

static void io_read_handler(lcbio_CTX *ioctx, unsigned)
{
    reinterpret_cast< CccpProvider * >(lcbio_ctx_data(ioctx))->on_io_read();
}

void CccpProvider::on_io_read()
{
    unsigned required;

#define return_error(e)                                                                                                \
    resp.release(ioctx);                                                                                               \
    mcio_error(e);                                                                                                     \
    return

    lcb::MemcachedResponse resp;
    if (!resp.load(ioctx, &required)) {
        lcbio_ctx_rwant(ioctx, required);
        lcbio_ctx_schedule(ioctx);
        return;
    }

    if (resp.status() != PROTOCOL_BINARY_RESPONSE_SUCCESS) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "CCCP Packet responded with 0x%x; nkey=%d, nbytes=%lu, cmd=0x%x, seq=0x%x",
                LOGID(this), resp.status(), resp.keylen(), (unsigned long)resp.bodylen(), resp.opcode(), resp.opaque());

        switch (resp.status()) {
            case PROTOCOL_BINARY_RESPONSE_NOT_SUPPORTED:
            case PROTOCOL_BINARY_RESPONSE_UNKNOWN_COMMAND:
                return_error(LCB_NOT_SUPPORTED);
            default:
                return_error(LCB_PROTOCOL_ERROR);
        }

        return;
    }

    if (!resp.bodylen()) {
        return_error(LCB_PROTOCOL_ERROR);
    }

    std::string jsonstr(resp.value(), resp.vallen());
    std::string hoststr(lcbio_get_host(lcbio_ctx_sock(ioctx))->host);

    resp.release(ioctx);
    stop_current_request(true);

    lcb_STATUS err = update(hoststr.c_str(), jsonstr.c_str());

    if (err == LCB_SUCCESS) {
        timer.cancel();
    } else {
        schedule_next_request(LCB_PROTOCOL_ERROR, 0);
    }

#undef return_error
}

void CccpProvider::request_config()
{
    lcb::MemcachedRequest req(PROTOCOL_BINARY_CMD_GET_CLUSTER_CONFIG);
    req.opaque(0xF00D);
    lcbio_ctx_put(ioctx, req.data(), req.size());
    lcbio_ctx_rwant(ioctx, 24);
    lcbio_ctx_schedule(ioctx);
    timer.rearm(settings().config_node_timeout);
}

void CccpProvider::dump(FILE *fp) const
{
    if (!enabled) {
        return;
    }

    fprintf(fp, "## BEGIN CCCP PROVIDER DUMP ##\n");
    fprintf(fp, "TIMER ACTIVE: %s\n", timer.is_armed() ? "YES" : "NO");
    fprintf(fp, "PIPELINE RESPONSE COOKIE: %p\n", (void *)cmdcookie);
    if (ioctx) {
        fprintf(fp, "CCCP Owns connection:\n");
        lcbio_ctx_dump(ioctx, fp);
    } else if (creq) {
        fprintf(fp, "CCCP Is connecting\n");
    } else {
        fprintf(fp, "CCCP does not have a dedicated connection\n");
    }

    for (size_t ii = 0; ii < nodes->size(); ii++) {
        const lcb_host_t &curhost = (*nodes)[ii];
        lcb_settings *dummy = NULL;
        fprintf(fp, "CCCP NODE: " LCB_HOST_FMT "\n", LCB_HOST_ARG(dummy, &curhost));
    }
    fprintf(fp, "## END CCCP PROVIDER DUMP ##\n");
}

CccpProvider::CccpProvider(Confmon *mon)
    : Provider(mon, CLCONFIG_CCCP), nodes(new lcb::Hostlist()), config(NULL), timer(mon->iot, this), instance(NULL),
      ioctx(NULL), cmdcookie(NULL)
{
    std::memset(&creq, 0, sizeof creq);
}

Provider *lcb::clconfig::new_cccp_provider(Confmon *mon)
{
    return new CccpProvider(mon);
}
