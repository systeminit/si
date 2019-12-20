/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#include "bucketconfig/clconfig.h"
#include "http/http.h"
#include "http/http-priv.h"
#include "auth-priv.h"
#include "trace.h"

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_status(const lcb_RESPHTTP *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_cookie(const lcb_RESPHTTP *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_http_status(const lcb_RESPHTTP *resp, uint16_t *status)
{
    *status = resp->htstatus;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_path(const lcb_RESPHTTP *resp, const char **path, size_t *path_len)
{
    *path = (const char *)resp->key;
    *path_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_body(const lcb_RESPHTTP *resp, const char **body, size_t *body_len)
{
    *body = (const char *)resp->body;
    *body_len = resp->nbody;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_headers(const lcb_RESPHTTP *resp, const char *const **headers)
{
    *headers = resp->headers;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_resphttp_handle(const lcb_RESPHTTP *resp, lcb_HTTP_HANDLE **handle)
{
    *handle = resp->_htreq;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_resphttp_is_final(const lcb_RESPHTTP *resp)
{
    return resp->rflags & LCB_RESP_F_FINAL;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_create(lcb_CMDHTTP **cmd, lcb_HTTP_TYPE type)
{
    *cmd = (lcb_CMDHTTP *)calloc(1, sizeof(lcb_CMDHTTP));
    (*cmd)->type = type;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_destroy(lcb_CMDHTTP *cmd)
{
    free(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_parent_span(lcb_CMDHTTP *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_method(lcb_CMDHTTP *cmd, lcb_HTTP_METHOD method)
{
    cmd->method = method;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_path(lcb_CMDHTTP *cmd, const char *path, size_t path_len)
{
    LCB_CMD_SET_KEY(cmd, path, path_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_content_type(lcb_CMDHTTP *cmd, const char *content_type,
                                                     size_t content_type_len)
{
    cmd->content_type = content_type;
    (void)content_type_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_body(lcb_CMDHTTP *cmd, const char *body, size_t body_len)
{
    cmd->body = body;
    cmd->nbody = body_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_handle(lcb_CMDHTTP *cmd, lcb_HTTP_HANDLE **handle)
{
    cmd->reqhandle = handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_username(lcb_CMDHTTP *cmd, const char *username, size_t username_len)
{
    cmd->username = username;
    (void)username_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_password(lcb_CMDHTTP *cmd, const char *password, size_t password_len)
{
    cmd->password = password;
    (void)password_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_host(lcb_CMDHTTP *cmd, const char *host, size_t host_len)
{
    cmd->host = host;
    (void)host_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_streaming(lcb_CMDHTTP *cmd, int streaming)
{
    if (streaming) {
        cmd->cmdflags |= LCB_CMDHTTP_F_STREAM;
    } else {
        cmd->cmdflags &= ~LCB_CMDHTTP_F_STREAM;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_skip_auth_header(lcb_CMDHTTP *cmd, int skip_auth)
{
    if (skip_auth) {
        cmd->cmdflags |= LCB_CMDHTTP_F_NOUPASS;
    } else {
        cmd->cmdflags &= ~LCB_CMDHTTP_F_NOUPASS;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdhttp_timeout(lcb_CMDHTTP *cmd, uint32_t timeout)
{
    cmd->cmdflags |= LCB_CMDHTTP_F_CASTMO;
    cmd->cas = timeout;
    return LCB_SUCCESS;
}

using namespace lcb::http;

#define LOGFMT "<%s%s%s:%s> "
#define LOGID(req) ((req)->ipv6 ? "[" : ""), (req)->host.c_str(), ((req)->ipv6 ? "]" : ""), (req)->port.c_str()

#define LOGARGS(req, lvl) req->instance->settings, "http-io", LCB_LOG_##lvl, __FILE__, __LINE__

static const char *method_strings[] = {
    "GET ",   /* LCB_HTTP_METHOD_GET */
    "POST ",  /* LCB_HTTP_METHOD_POST */
    "PUT ",   /* LCB_HTTP_METHOD_PUT */
    "DELETE " /* LCB_HTTP_METHOD_DELETE */
};

void Request::decref()
{
    lcb_assert(refcount > 0);
    if (--refcount) {
        return;
    }

    close_io();

    if (parser) {
        delete parser;
    }

    if (timer) {
        lcbio_timer_destroy(timer);
        timer = NULL;
    }

    delete this;
}

void Request::finish_or_retry(lcb_STATUS rc)
{
    if (rc == LCB_ETIMEDOUT) {
        // No point on trying (or even logging) a timeout
        finish(rc);
        return;
    }
    if (passed_data) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "Not retrying. Data passed to callback", LOGID(this));
        finish(rc);
        return;
    }

    // Not a 'data API'. Request may be node-specific
    if (!is_data_request()) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "Not retrying non-data-api request", LOGID(this));
        finish(rc);
        return;
    }

    // See if we can find an API node.
    const char *nextnode = get_api_node();
    if (!nextnode) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "Not retrying. No nodes available", LOGID(this));
        finish(rc);
        return;
    }
    struct http_parser_url next_info;
    if (_lcb_http_parser_parse_url(nextnode, strlen(nextnode), 0, &next_info)) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "Not retrying. Invalid API endpoint", LOGID(this));
        finish(LCB_EINVAL);
        return;
    }

    // Reassemble URL:
    lcb_log(LOGARGS(this, DEBUG), LOGFMT "Retrying request on new node %s. Reason: 0x%02x (%s)", LOGID(this), nextnode,
            rc, lcb_strerror(NULL, rc));

    url.replace(url_info.field_data[UF_PORT].off, url_info.field_data[UF_PORT].len,
                nextnode + next_info.field_data[UF_PORT].off, next_info.field_data[UF_PORT].len);
    url.replace(url_info.field_data[UF_HOST].off, url_info.field_data[UF_HOST].len,
                nextnode + next_info.field_data[UF_HOST].off, next_info.field_data[UF_HOST].len);

    lcb_STATUS newrc;
    newrc = assign_url(NULL, 0, NULL, 0);
    if (newrc != LCB_SUCCESS) {
        lcb_log(LOGARGS(this, ERR), LOGFMT "Failed to assign URL for retry request on next endpoint (%s): 0x%02x (%s)",
                LOGID(this), nextnode, newrc, lcb_strerror(NULL, newrc));
        finish(rc);
        return;
    }

    newrc = submit();
    if (newrc != LCB_SUCCESS) {
        lcb_log(LOGARGS(this, WARN), LOGFMT "Failed to retry request on next endpoint (%s): 0x%02x (%s)", LOGID(this),
                nextnode, newrc, lcb_strerror(NULL, newrc));
        finish(rc);
    }
}

void Request::maybe_refresh_config(lcb_STATUS err)
{
    int htstatus_ok;
    if (!parser) {
        return;
    }

    if (!LCBT_SETTING(instance, refresh_on_hterr)) {
        return;
    }

    const lcb::htparse::Response &resp = parser->get_cur_response();
    htstatus_ok = resp.status >= 200 && resp.status < 299;

    if (err != LCB_SUCCESS && (err == LCB_ESOCKSHUTDOWN && htstatus_ok) == 0) {
        /* ignore graceful close */
        instance->bootstrap(BS_REFRESH_ALWAYS);
        return;
    }

    if (htstatus_ok) {
        return;
    }
    instance->bootstrap(BS_REFRESH_ALWAYS);
}

void Request::init_resp(lcb_RESPHTTP *res)
{
    const lcb::htparse::Response &htres = parser->get_cur_response();

    res->cookie = const_cast< void * >(command_cookie);
    res->key = url.c_str() + url_info.field_data[UF_PATH].off;
    res->nkey = url_info.field_data[UF_PATH].len;
    res->_htreq = static_cast< lcb_HTTP_HANDLE * >(this);
    if (!response_headers.empty()) {
        res->headers = &response_headers_clist[0];
    }
    res->htstatus = htres.status;
}

void Request::finish(lcb_STATUS error)
{
    /* This is always safe to execute */
    if (!(status & NOLCB)) {
        maybe_refresh_config(error);
    }

    /* And this one too */
    if ((status & CBINVOKED) == 0) {
        lcb_RESPHTTP resp = {0};
        init_resp(&resp);
        resp.rflags = LCB_RESP_F_FINAL;
        resp.rc = error;

        status |= CBINVOKED;
        callback(instance, LCB_CALLBACK_HTTP, (lcb_RESPBASE *)&resp);
    }

    if (status & FINISHED) {
        return;
    }

    TRACE_HTTP_END(this, error, parser->get_cur_response().status);
    status |= FINISHED;

    if (!(status & NOLCB)) {
        /* Remove from wait queue */
        lcb_aspend_del(&instance->pendops, LCB_PENDTYPE_HTTP, this);
        /* Break out from the loop (must be called after aspend_del) */
        lcb_maybe_breakout(instance);
    }

    /* Cancel the timeout */
    lcbio_timer_disarm(timer);

    /* Remove the initial refcount=1 (set from lcb_http3). Typically this will
     * also free the request (though this is dependent on pending I/O operations) */
    decref();
}

void Request::add_to_preamble(const char *s)
{
    preamble.insert(preamble.end(), s, s + strlen(s));
}
void Request::add_to_preamble(const std::string &s)
{
    preamble.insert(preamble.end(), s.c_str(), s.c_str() + s.size());
}
void Request::add_to_preamble(const Header &header)
{
    add_to_preamble(header.key);
    add_to_preamble(": ");
    add_to_preamble(header.value);
    add_to_preamble("\r\n");
}

lcb_STATUS Request::submit()
{
    lcb_STATUS rc;
    lcb_host_t reqhost = {"", "", 0};

    // Stop any pending socket/request
    close_io();

    if (host.size() > sizeof reqhost.host || port.size() > sizeof reqhost.port) {
        decref();
        return LCB_E2BIG;
    }

    preamble.clear();

    strncpy(reqhost.host, host.c_str(), host.size());
    strncpy(reqhost.port, port.c_str(), port.size());
    reqhost.host[host.size()] = '\0';
    reqhost.port[port.size()] = '\0';
    reqhost.ipv6 = ipv6;

    // Add the HTTP verb (e.g. "GET ") [note, the string contains a trailing space]
    add_to_preamble(method_strings[method]);

    // Add the path
    const char *url_s = url.c_str();
    size_t path_off = url_info.field_data[UF_PATH].off;
    size_t path_len = url.size() - path_off;
    preamble.insert(preamble.end(), url_s + path_off, url_s + path_off + path_len);
    lcb_log(LOGARGS(this, TRACE), LOGFMT "%s %s. Body=%lu bytes", LOGID(this), method_strings[method], url.c_str(),
            (unsigned long int)body.size());

    add_to_preamble(" HTTP/1.1\r\n");

    // Add the Host: header manually. If redirected to a different host then
    // we need to recalculate this, so don't make this part of the
    // global headers (which are typically not cleared)
    add_to_preamble("Host: ");
    add_to_preamble(host);
    add_to_preamble(":");
    add_to_preamble(port);
    add_to_preamble("\r\n");

    // Add the rest of the headers
    std::vector< Header >::const_iterator ii = request_headers.begin();
    for (; ii != request_headers.end(); ++ii) {
        add_to_preamble(*ii);
    }
    add_to_preamble("\r\n");
    // If there is a body, it is appended in the IO stage

    rc = start_io(reqhost);

    if (rc == LCB_SUCCESS) {
        // Only wipe old parser/response information if current I/O request
        // was a success
        if (parser) {
            parser->reset();
        } else {
            parser = new lcb::htparse::Parser(instance->settings);
        }
        response_headers.clear();
        response_headers_clist.clear();
        TRACE_HTTP_BEGIN(this);
    }

    return rc;
}

void Request::assign_from_urlfield(http_parser_url_fields field, std::string &target)
{
    target = url.substr(url_info.field_data[field].off, url_info.field_data[field].len);
}

lcb_STATUS Request::assign_url(const char *base, size_t nbase, const char *path, size_t npath)
{
    const char *htscheme;
    unsigned schemsize;

    if (LCBT_SETTING(instance, sslopts) & LCB_SSL_ENABLED) {
        htscheme = "https://";
        schemsize = sizeof("https://");
    } else {
        htscheme = "http://";
        schemsize = sizeof("http://");
    }

    schemsize--;
    if (base) {
        url.assign(htscheme, schemsize);
        if (nbase > schemsize && memcmp(base, htscheme, schemsize) == 0) {
            base += schemsize;
            nbase -= schemsize;
        }
        url.append(base, nbase);
        if (path) {
            if (*path != '/' && url[url.size() - 1] != '/') {
                url.append("/");
            }

            if (!lcb::strcodecs::urlencode(path, path + npath, url)) {
                return LCB_INVALID_CHAR;
            }
        }
    }

    bool redir_checked = false;
    static const unsigned required_fields = ((1 << UF_HOST) | (1 << UF_PORT) | (1 << UF_PATH));

GT_REPARSE:
    if (_lcb_http_parser_parse_url(url.c_str(), url.size(), 0, &url_info)) {
        return LCB_EINVAL;
    }

    if ((url_info.field_set & required_fields) != required_fields) {
        if (base == NULL && path == NULL && !redir_checked) {
            redir_checked = true;
            std::string first_part(htscheme, schemsize);
            first_part += host;
            first_part += ':';
            first_part += port;
            url = first_part + url;
            goto GT_REPARSE;
        }
        return LCB_EINVAL;
    }

    assign_from_urlfield(UF_HOST, host);
    assign_from_urlfield(UF_PORT, port);
    ipv6 = host.find(':') != std::string::npos;
    return LCB_SUCCESS;
}

void Request::redirect()
{
    lcb_STATUS rc;
    lcb_assert(!pending_redirect.empty());
    if (LCBT_SETTING(instance, max_redir) > -1) {
        if (LCBT_SETTING(instance, max_redir) < ++redircount) {
            finish(LCB_TOO_MANY_REDIRECTS);
            return;
        }
    }

    memset(&url_info, 0, sizeof url_info);
    url = pending_redirect;
    pending_redirect.clear();

    if ((rc = assign_url(NULL, 0, NULL, 0)) != LCB_SUCCESS) {
        lcb_log(LOGARGS(this, ERR), LOGFMT "Failed to add redirect URL (%s)", LOGID(this), url.c_str());
        finish(rc);
        return;
    }

    if ((rc = submit()) != LCB_SUCCESS) {
        finish(rc);
    }
}

static lcbvb_SVCTYPE httype2svctype(unsigned httype)
{
    switch (httype) {
        case LCB_HTTP_TYPE_VIEW:
            return LCBVB_SVCTYPE_VIEWS;
        case LCB_HTTP_TYPE_N1QL:
            return LCBVB_SVCTYPE_N1QL;
        case LCB_HTTP_TYPE_FTS:
            return LCBVB_SVCTYPE_FTS;
        case LCB_HTTP_TYPE_CBAS:
            return LCBVB_SVCTYPE_CBAS;
        default:
            return LCBVB_SVCTYPE__MAX;
    }
}

const char *Request::get_api_node(lcb_STATUS &rc)
{
    if (!is_data_request()) {
        return lcb_get_node(instance, LCB_NODE_HTCONFIG, 0);
    }

    if (!LCBT_VBCONFIG(instance)) {
        rc = LCB_CLIENT_ETMPFAIL;
        return NULL;
    }

    const lcbvb_SVCTYPE svc = httype2svctype(reqtype);
    const lcbvb_SVCMODE mode = LCBT_SETTING_SVCMODE(instance);

    lcbvb_CONFIG *vbc = LCBT_VBCONFIG(instance);

    if (last_vbcrev != vbc->revid) {
        used_nodes.clear();
        last_vbcrev = vbc->revid;
    }
    used_nodes.resize(LCBVB_NSERVERS(vbc));

    int ix = lcbvb_get_randhost_ex(vbc, svc, mode, &used_nodes[0]);
    if (ix < 0) {
        rc = LCB_NOT_SUPPORTED;
        return NULL;
    }
    used_nodes[ix] = 1;
    return lcbvb_get_resturl(vbc, ix, svc, mode);
}

lcb_STATUS Request::setup_inputs(const lcb_CMDHTTP *cmd)
{
    std::string username, password;
    const char *base = NULL;
    size_t nbase = 0;
    lcb_STATUS rc = LCB_SUCCESS;

    if (method > LCB_HTTP_METHOD_MAX) {
        return LCB_EINVAL;
    }

    if (cmd->username) {
        username = cmd->username;
    }
    if (cmd->password) {
        password = cmd->password;
    }

    if (reqtype == LCB_HTTP_TYPE_RAW) {
        if ((base = cmd->host) == NULL) {
            return LCB_EINVAL;
        }
    } else {
        if (cmd->host) {
            if (reqtype == LCB_HTTP_TYPE_CBAS || reqtype == LCB_HTTP_TYPE_PING) {
                /* might be a deferred CBAS URL or PING */
                base = cmd->host;
            } else {
                return LCB_EINVAL;
            }
        }
        if (base == NULL) {
            base = get_api_node(rc);
        }
        if (base == NULL || *base == '\0') {
            if (rc == LCB_SUCCESS) {
                return LCB_EINTERNAL;
            } else {
                return rc;
            }
        }

        if ((cmd->cmdflags & LCB_CMDHTTP_F_NOUPASS) || instance->settings->keypath) {
            // explicitly asked to skip Authorization header,
            // or using SSL client certificate to authenticate
            username.clear();
            password.clear();
        } else if (username.empty() && password.empty()) {
            const Authenticator &auth = *LCBT_SETTING(instance, auth);
            if (reqtype == LCB_HTTP_TYPE_MANAGEMENT) {
                username = auth.username();
                password = auth.password();
            } else {
                if (auth.mode() == LCBAUTH_MODE_DYNAMIC) {
                    struct http_parser_url info = {};
                    if (_lcb_http_parser_parse_url(base, strlen(base), 0, &info)) {
                        lcb_log(LOGARGS(this, WARN), LOGFMT "Failed to parse API endpoint", LOGID(this));
                        return LCB_EINTERNAL;
                    }
                    std::string hh(base + info.field_data[UF_HOST].off, info.field_data[UF_HOST].len);
                    std::string pp(base + info.field_data[UF_PORT].off, info.field_data[UF_PORT].len);
                    username = auth.username_for(hh.c_str(), pp.c_str(), LCBT_SETTING(instance, bucket));
                    password = auth.password_for(hh.c_str(), pp.c_str(), LCBT_SETTING(instance, bucket));
                } else {
                    username = auth.username_for(NULL, NULL, LCBT_SETTING(instance, bucket));
                    password = auth.password_for(NULL, NULL, LCBT_SETTING(instance, bucket));
                }
            }
        }
    }

    if (base) {
        nbase = strlen(base);
    }

    rc = assign_url(base, nbase, reinterpret_cast< const char * >(cmd->key.contig.bytes), cmd->key.contig.nbytes);
    if (rc != LCB_SUCCESS) {
        return rc;
    }

    std::string ua(LCB_CLIENT_ID);
    if (instance->settings->client_string) {
        ua.append(" ").append(instance->settings->client_string);
    }
    add_header("User-Agent", ua);

    if (instance->http_sockpool->get_options().maxidle == 0 || !is_data_request()) {
        add_header("Connection", "close");
    }

    add_header("Accept", "application/json");
    if (!username.empty()) {
        char auth[256];
        std::string upassbuf;
        upassbuf.append(username).append(":").append(password);
        if (lcb_base64_encode(upassbuf.c_str(), upassbuf.size(), auth, sizeof(auth)) == -1) {
            return LCB_EINVAL;
        }
        add_header("Authorization", std::string("Basic ") + auth);
    }

    if (!body.empty()) {
        char lenbuf[64];
        sprintf(lenbuf, "%lu", (unsigned long int)body.size());
        add_header("Content-Length", lenbuf);
        if (cmd->content_type) {
            add_header("Content-Type", cmd->content_type);
        }
    }

    return LCB_SUCCESS;
}

Request::Request(lcb_INSTANCE *instance_, const void *cookie, const lcb_CMDHTTP *cmd)
    : instance(instance_), body(cmd->body, cmd->body + cmd->nbody), method(cmd->method),
      chunked(cmd->cmdflags & LCB_CMDHTTP_F_STREAM), paused(false), command_cookie(cookie), refcount(1), redircount(0),
      passed_data(false), last_vbcrev(-1), reqtype(cmd->type), status(ONGOING),
      callback(lcb_find_callback(instance, LCB_CALLBACK_HTTP)), io(instance->iotable), ioctx(NULL), timer(NULL),
      parser(NULL), user_timeout(cmd->cmdflags & LCB_CMDHTTP_F_CASTMO ? cmd->cas : 0)
{
    memset(&creq, 0, sizeof creq);
}

uint32_t Request::timeout() const
{
    if (user_timeout) {
        return user_timeout;
    }
    switch (reqtype) {
        case LCB_HTTP_TYPE_N1QL:
        case LCB_HTTP_TYPE_FTS:
            return LCBT_SETTING(instance, n1ql_timeout);
        case LCB_HTTP_TYPE_VIEW:
            return LCBT_SETTING(instance, views_timeout);
        default:
            return LCBT_SETTING(instance, http_timeout);
    }
}

Request *Request::create(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDHTTP *cmd, lcb_STATUS *rc)
{
    Request *req = new Request(instance, cookie, cmd);
    if (!req) {
        *rc = LCB_CLIENT_ENOMEM;
        return NULL;
    }
    req->start = gethrtime();

    *rc = req->setup_inputs(cmd);
    if (*rc != LCB_SUCCESS) {
        req->decref();
        return NULL;
    }

    *rc = req->submit();
    if (*rc == LCB_SUCCESS) {
        if (cmd->reqhandle) {
            *cmd->reqhandle = static_cast< lcb_HTTP_HANDLE * >(req);
        }
        lcb_aspend_add(&instance->pendops, LCB_PENDTYPE_HTTP, req);
        return req;
    } else {
        // Do not call finish() as we don't want a callback
        req->decref();
        return NULL;
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_http(lcb_INSTANCE *instance, void *cookie, const lcb_CMDHTTP *cmd)
{
    lcb_STATUS rc;
    Request::create(instance, cookie, cmd, &rc);
    return rc;
}

void Request::cancel()
{
    if (status & (FINISHED | CBINVOKED)) {
        /* Nothing to cancel */
        return;
    }
    status |= CBINVOKED;
    finish(LCB_SUCCESS);
}

LIBCOUCHBASE_API lcb_STATUS lcb_http_cancel(lcb_INSTANCE *, lcb_HTTP_HANDLE *handle)
{
    handle->cancel();
    return LCB_SUCCESS;
}
