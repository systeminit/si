/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2013-2019 Couchbase, Inc.
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

#include "internal.h"
#include "clconfig.h"
#include "bc_http.h"
#include "auth-priv.h"
#include <lcbio/ssl.h>
#include "ctx-log-inl.h"
#define LOGARGS(ht, lvlbase) ht->parent->settings, "htconfig", LCB_LOG_##lvlbase, __FILE__, __LINE__

#define LOGFMT CTX_LOGFMT
#define LOGID(p) CTX_LOGID(p->ioctx)

using namespace lcb::clconfig;

static void io_error_handler(lcbio_CTX *, lcb_STATUS);
static void on_connected(lcbio_SOCKET *, void *, lcb_STATUS, lcbio_OSERR);
static void read_common(lcbio_CTX *, unsigned);

/**
 * Determine if we're in compatibility mode with the previous versions of the
 * library - where the idle timeout is disabled and a perpetual streaming
 * connection will always remain open (regardless of whether it was triggered
 * by start_refresh/get_refresh).
 */
bool HttpProvider::is_v220_compat() const
{
    lcb_uint32_t setting = parent->settings->bc_http_stream_time;
    return setting == (lcb_uint32_t)-1;
}

void HttpProvider::close_current()
{
    disconn_timer.cancel();
    if (ioctx) {
        lcbio_ctx_close(ioctx, NULL, NULL);
    } else if (creq) {
        lcbio_connect_cancel(creq);
    }
    creq = NULL;
    ioctx = NULL;
}

/**
 * Call when there is an error in I/O. This includes read, write, connect
 * and timeouts.
 */
lcb_STATUS HttpProvider::on_io_error(lcb_STATUS origerr)
{
    close_current();

    creq = lcbio_connect_hl(parent->iot, &settings(), nodes, 0, settings().config_node_timeout, on_connected, this);
    if (creq) {
        return LCB_SUCCESS;
    }
    parent->provider_failed(this, origerr);
    io_timer.cancel();
    if (is_v220_compat() && parent->config != NULL) {
        lcb_log(LOGARGS(this, INFO), "HTTP node list finished. Trying to obtain connection from first node in list");
        as_reconnect.arm_if_disarmed(settings().grace_next_cycle);
    }
    return origerr;
}

void set_new_config(HttpProvider *http)
{
    const lcb_host_t *curhost;
    if (http->current_config) {
        http->current_config->decref();
    }

    curhost = lcbio_get_host(lcbio_ctx_sock(http->ioctx));
    http->current_config = http->last_parsed;
    http->current_config->incref();
    lcbvb_replace_host(http->current_config->vbc, curhost->host);
    http->parent->provider_got_config(http, http->current_config);
}

static lcb_STATUS process_chunk(HttpProvider *http, const void *buf, unsigned nbuf)
{
    namespace htp = lcb::htparse;
    lcb_STATUS err = LCB_SUCCESS;
    int rv;
    lcbvb_CONFIG *cfgh;
    unsigned state, oldstate, diff;
    lcb_host_t *host;
    htp::Response &resp = http->htp->get_cur_response();

    oldstate = resp.state;
    state = http->htp->parse(buf, nbuf);
    diff = state ^ oldstate;

    if (state & htp::Parser::S_ERROR) {
        return LCB_PROTOCOL_ERROR;
    }

    if (diff & htp::Parser::S_HEADER) {
        /* see that we got a success? */
        if (resp.status == 200) {
            /* nothing */
        } else if (resp.status == 404) {
            const int urlmode = http->settings().bc_http_urltype;
            err = LCB_BUCKET_ENOENT;

            if (++http->uritype > LCB_HTCONFIG_URLTYPE_COMPAT) {
                lcb_log(LOGARGS(http, ERR),
                        LOGFMT "Got 404 on config stream. Assuming bucket does not exist as we've tried both URL types",
                        LOGID(http));
                goto GT_HT_ERROR;

            } else if ((urlmode & LCB_HTCONFIG_URLTYPE_COMPAT) == 0) {
                lcb_log(LOGARGS(http, ERR),
                        LOGFMT "Got 404 on config stream for terse URI. Compat URI disabled, so not trying",
                        LOGID(http));

            } else {
                /* reissue the request; but wait for it to drain */
                lcb_log(LOGARGS(http, WARN),
                        LOGFMT "Got 404 on config stream. Assuming terse URI not supported on cluster", LOGID(http));
                http->try_nexturi = 1;
                goto GT_CHECKDONE;
            }
        } else if (resp.status == 401) {
            err = LCB_AUTH_ERROR;
        } else {
            err = LCB_ERROR;
        }

    GT_HT_ERROR:
        if (err != LCB_SUCCESS) {
            lcb_log(LOGARGS(http, ERR), LOGFMT "Got non-success HTTP status code %d", LOGID(http), resp.status);
            return err;
        }
    }

GT_CHECKDONE:
    if (http->try_nexturi) {
        if (!(state & htp::Parser::S_DONE)) {
            return LCB_SUCCESS;
        }
        host = lcbio_get_host(lcbio_ctx_sock(http->ioctx));
        http->try_nexturi = 0;
        if ((err = http->setup_request_header(*host)) != LCB_SUCCESS) {
            return err;
        }

        /* reset the state? */
        http->htp->reset();
        lcbio_ctx_put(http->ioctx, http->request_buf.c_str(), http->request_buf.size());
        return LCB_SUCCESS;
    }

    if (!(state & htp::Parser::S_BODY)) {
        /* nothing to parse yet */
        return LCB_SUCCESS;
    }

    /* seek ahead for strstr */
    size_t termpos = resp.body.find(CONFIG_DELIMITER);
    if (termpos == std::string::npos) {
        return LCB_SUCCESS;
    }
    resp.body[termpos] = '\0';
    cfgh = lcbvb_create();
    if (!cfgh) {
        return LCB_CLIENT_ENOMEM;
    }
    host = lcbio_get_host(lcbio_ctx_sock(http->ioctx));
    rv = lcbvb_load_json_ex(cfgh, resp.body.c_str(), host->host, &LCBT_SETTING(http->parent, network));
    if (rv != 0) {
        lcb_log(LOGARGS(http, ERR), LOGFMT "Failed to parse a valid config from HTTP stream", LOGID(http));
        lcb_log_badconfig(LOGARGS(http, ERR), cfgh, resp.body.c_str());
        lcbvb_destroy(cfgh);
        return LCB_PROTOCOL_ERROR;
    }
    if (http->last_parsed) {
        http->last_parsed->decref();
    }
    http->last_parsed = ConfigInfo::create(cfgh, CLCONFIG_HTTP);
    http->generation++;

    /** Relocate the stream */
    resp.body.erase(0, termpos + sizeof(CONFIG_DELIMITER) - 1);
    return LCB_SUCCESS;
}

/**
 * Common function to handle parsing the HTTP stream for both v0 and v1 io
 * implementations.
 */
static void read_common(lcbio_CTX *ctx, unsigned nr)
{
    lcbio_CTXRDITER riter;
    HttpProvider *http = reinterpret_cast< HttpProvider * >(lcbio_ctx_data(ctx));
    int old_generation = http->generation;

    lcb_log(LOGARGS(http, TRACE), LOGFMT "Received %d bytes on HTTP stream", LOGID(http), nr);
    http->io_timer.rearm(http->settings().config_node_timeout);

    LCBIO_CTX_ITERFOR(ctx, &riter, nr)
    {
        unsigned nbuf = lcbio_ctx_risize(&riter);
        void *buf = lcbio_ctx_ribuf(&riter);
        lcb_STATUS err = process_chunk(http, buf, nbuf);

        if (err != LCB_SUCCESS) {
            http->on_io_error(err);
            return;
        }
    }

    if (http->generation != old_generation) {
        lcb_log(LOGARGS(http, DEBUG), LOGFMT "Generation %d -> %d", LOGID(http), old_generation, http->generation);
        http->io_timer.cancel();
        set_new_config(http);
    }

    lcbio_ctx_rwant(ctx, 1);
    lcbio_ctx_schedule(ctx);
}

lcb_STATUS HttpProvider::setup_request_header(const lcb_host_t &host)
{
    request_buf.assign("GET ");
    if (settings().conntype == LCB_TYPE_BUCKET || settings().conntype == LCB_TYPE_CLUSTER) {
        if (uritype == LCB_HTCONFIG_URLTYPE_25PLUS) {
            request_buf.append(REQBUCKET_TERSE_PREFIX);
        } else {
            request_buf.append(REQBUCKET_COMPAT_PREFIX);
        }
        request_buf.append(settings().bucket);
    } else {
        return LCB_EINVAL;
    }

    request_buf.append(" HTTP/1.1\r\n");
    if (!settings().keypath) {
        // not using SSL client certificate to authenticate
        const std::string password = (settings().conntype == LCB_TYPE_BUCKET)
                                         ? settings().auth->password_for(host.host, host.port, settings().bucket)
                                         : settings().auth->password();
        if (!password.empty()) {
            const std::string username = (settings().conntype == LCB_TYPE_BUCKET)
                                             ? settings().auth->username_for(host.host, host.port, settings().bucket)
                                             : settings().auth->username();
            std::string cred;
            cred.append(username).append(":").append(password);
            char b64[256] = {0};
            if (lcb_base64_encode(cred.c_str(), cred.size(), b64, sizeof(b64)) == -1) {
                return LCB_EINTERNAL;
            }
            request_buf.append("Authorization: Basic ").append(b64).append("\r\n");
        }
    }

    request_buf.append("Host: ").append(host.host).append(":").append(host.port).append("\r\n");
    request_buf.append("User-Agent: ").append(LCB_CLIENT_ID);
    if (settings().client_string) {
        request_buf.append(" ").append(settings().client_string);
    }
    request_buf.append("\r\n");
    request_buf.append("\r\n");
    return LCB_SUCCESS;
}

void HttpProvider::reset_stream_state()
{
    const int urlmode = settings().bc_http_urltype;
    if (last_parsed) {
        last_parsed->decref();
        last_parsed = NULL;
    }
    if (urlmode & LCB_HTCONFIG_URLTYPE_25PLUS) {
        uritype = LCB_HTCONFIG_URLTYPE_25PLUS;
    } else {
        uritype = LCB_HTCONFIG_URLTYPE_COMPAT;
    }
    try_nexturi = false;
    htp->reset();
}

static void on_connected(lcbio_SOCKET *sock, void *arg, lcb_STATUS err, lcbio_OSERR syserr)
{
    HttpProvider *http = reinterpret_cast< HttpProvider * >(arg);
    lcb_host_t *host;
    lcbio_CTXPROCS procs;
    http->creq = NULL;

    if (err != LCB_SUCCESS) {
        lcb_log(LOGARGS(http, ERR), "Connection to REST API failed with %s (os errno = %d)", lcb_strerror_short(err),
                syserr);
        http->on_io_error(err);
        return;
    }
    host = lcbio_get_host(sock);
    lcb_log(LOGARGS(http, DEBUG), "Successfuly connected to REST API " LCB_HOST_FMT,
            LCB_HOST_ARG(http->parent->settings, host));

    lcbio_sslify_if_needed(sock, http->parent->settings);
    http->reset_stream_state();

    if ((err = http->setup_request_header(*host)) != LCB_SUCCESS) {
        lcb_log(LOGARGS(http, ERR), "Couldn't setup request header");
        http->on_io_error(err);
        return;
    }

    memset(&procs, 0, sizeof(procs));
    procs.cb_err = io_error_handler;
    procs.cb_read = read_common;
    http->ioctx = lcbio_ctx_new(sock, http, &procs);
    http->ioctx->subsys = "bc_http";
    sock->service = LCBIO_SERVICE_CFG;

    lcbio_ctx_put(http->ioctx, http->request_buf.c_str(), http->request_buf.size());
    lcbio_ctx_rwant(http->ioctx, 1);
    lcbio_ctx_schedule(http->ioctx);
    http->io_timer.rearm(http->settings().config_node_timeout);
}

void HttpProvider::on_timeout()
{
    lcb_log(LOGARGS(this, ERR), LOGFMT "HTTP Provider timed out waiting for I/O", LOGID(this));

    /**
     * If we're not the current provider then ignore the timeout until we're
     * actively requested to do so
     */
    if (this != parent->cur_provider || !parent->is_refreshing()) {
        lcb_log(LOGARGS(this, DEBUG),
                LOGFMT "Ignoring timeout because we're either not in a refresh or not the current provider",
                LOGID(this));
        return;
    }

    on_io_error(LCB_ETIMEDOUT);
}

lcb_STATUS HttpProvider::connect_next()
{
    lcb_log(LOGARGS(this, TRACE), "Starting HTTP Configuration Provider %p", (void *)this);
    close_current();
    as_reconnect.cancel();

    if (nodes->empty()) {
        lcb_log(LOGARGS(this, ERROR),
                "Not scheduling HTTP provider since no nodes have been configured for HTTP bootstrap");
        return LCB_CONNECT_ERROR;
    }

    creq = lcbio_connect_hl(parent->iot, &settings(), nodes, 1, settings().config_node_timeout, on_connected, this);
    if (creq) {
        return LCB_SUCCESS;
    }
    lcb_log(LOGARGS(this, ERROR), "%p: Couldn't schedule connection", (void *)this);
    return LCB_CONNECT_ERROR;
}

void HttpProvider::delayed_disconn()
{
    lcb_log(LOGARGS(this, DEBUG), "Stopping HTTP provider %p", (void *)this);

    /** closes the connection and cleans up the timer */
    close_current();
    io_timer.cancel();
}

void HttpProvider::delayed_reconnect()
{
    if (ioctx) {
        /* have a context already */
        return;
    }
    lcb_STATUS err = connect_next();
    if (err != LCB_SUCCESS) {
        on_io_error(err);
    }
}

bool HttpProvider::pause()
{
    if (is_v220_compat()) {
        return LCB_SUCCESS;
    }
    disconn_timer.arm_if_disarmed(settings().bc_http_stream_time);
    return LCB_SUCCESS;
}

lcb_STATUS HttpProvider::refresh()
{
    /**
     * We want a grace interval here because we might already be fetching a
     * connection. HOWEVER we don't want to indefinitely wait on a socket
     * so we issue a timer indicating how long we expect to wait for a
     * streaming update until we get something.
     */

    /** If we need a new socket, we do connect_next. */
    if (ioctx == NULL && creq == NULL) {
        as_reconnect.signal();
    }
    disconn_timer.cancel();
    if (ioctx) {
        io_timer.rearm(settings().config_node_timeout);
    }
    return LCB_SUCCESS;
}

ConfigInfo *HttpProvider::get_cached()
{
    return current_config;
}

void HttpProvider::config_updated(lcbvb_CONFIG *newconfig)
{
    lcbvb_SVCMODE mode = LCBT_SETTING_SVCMODE(parent);
    nodes->clear();

    for (size_t ii = 0; ii < newconfig->nsrv; ++ii) {
        const char *ss;
        lcb_STATUS status;
        ss = lcbvb_get_hostport(newconfig, ii, LCBVB_SVCTYPE_MGMT, mode);
        if (!ss) {
            /* not supported? */
            continue;
        }
        status = nodes->add(ss, LCB_CONFIG_HTTP_PORT);
        lcb_assert(status == LCB_SUCCESS);
    }
    if (nodes->empty()) {
        lcb_log(LOGARGS(this, FATAL), "New nodes do not contain management ports");
    }

    if (settings().randomize_bootstrap_nodes) {
        nodes->randomize();
    }
}

void HttpProvider::configure_nodes(const lcb::Hostlist &newnodes)
{
    nodes->assign(newnodes);
    if (settings().randomize_bootstrap_nodes) {
        nodes->randomize();
    }
}

const lcb::Hostlist *HttpProvider::get_nodes() const
{
    return nodes;
}

HttpProvider::~HttpProvider()
{
    reset_stream_state();
    close_current();
    delete htp;
    disconn_timer.release();
    io_timer.release();
    as_reconnect.release();

    if (current_config) {
        current_config->decref();
    }
    if (nodes) {
        delete nodes;
    }
}

void HttpProvider::dump(FILE *fp) const
{
    fprintf(fp, "## BEGIN HTTP PROVIDER DUMP\n");
    fprintf(fp, "NUMBER OF CONFIGS RECEIVED: %u\n", generation);
    fprintf(fp, "DUMPING I/O TIMER\n");
    io_timer.dump(fp);
    if (ioctx) {
        fprintf(fp, "DUMPING CURRENT CONNECTION:\n");
        lcbio_ctx_dump(ioctx, fp);
    } else if (creq) {
        fprintf(fp, "CURRENTLY CONNECTING..\n");
    } else {
        fprintf(fp, "NO CONNECTION ACTIVE\n");
    }
}

HttpProvider::HttpProvider(Confmon *parent_)
    : Provider(parent_, CLCONFIG_HTTP), ioctx(NULL), htp(new lcb::htparse::Parser(parent->settings)),
      disconn_timer(parent->iot, this), io_timer(parent->iot, this), as_reconnect(parent->iot, this),
      nodes(new Hostlist()), current_config(NULL), last_parsed(NULL), generation(0), try_nexturi(false), uritype(0)
{

    memset(&creq, 0, sizeof creq);
}

static void io_error_handler(lcbio_CTX *ctx, lcb_STATUS err)
{
    reinterpret_cast< HttpProvider * >(lcbio_ctx_data(ctx))->on_io_error(err);
}

const lcbio_SOCKET *lcb::clconfig::http_get_conn(const Provider *p)
{
    const HttpProvider *http = static_cast< const HttpProvider * >(p);
    if (!http->ioctx) {
        return NULL;
    }
    return lcbio_ctx_sock(http->ioctx);
}

const lcb_host_t *lcb::clconfig::http_get_host(const Provider *p)
{
    const lcbio_SOCKET *sock = http_get_conn(p);
    if (sock) {
        return lcbio_get_host(sock);
    }
    return NULL;
}

Provider *lcb::clconfig::new_http_provider(Confmon *mon)
{
    return new HttpProvider(mon);
}
