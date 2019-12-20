/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2016-2019 Couchbase, Inc.
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
 **/

#include <libcouchbase/couchbase.h>
#include <jsparse/parser.h>
#include "internal.h"
#include "http/http.h"
#include "logging.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <string>

#define LOGFMT "(FTR=%p) "
#define LOGID(req) static_cast<const void*>(req)
#define LOGARGS(req, lvl) req->instance->settings, "n1ql", LCB_LOG_##lvl, __FILE__, __LINE__

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_status(const lcb_RESPFTS *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_cookie(const lcb_RESPFTS *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_row(const lcb_RESPFTS *resp, const char **row, size_t *row_len)
{
    *row = resp->row;
    *row_len = resp->nrow;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_http_response(const lcb_RESPFTS *resp, const lcb_RESPHTTP **http)
{
    *http = resp->htresp;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respfts_handle(const lcb_RESPFTS *resp, lcb_FTS_HANDLE **handle)
{
    *handle = resp->handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_respfts_is_final(const lcb_RESPFTS *resp)
{
    return resp->rflags & LCB_RESP_F_FINAL;
}


LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_create(lcb_CMDFTS **cmd)
{
    *cmd = (lcb_CMDFTS *)calloc(1, sizeof(lcb_CMDFTS));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_destroy(lcb_CMDFTS *cmd)
{
    free(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_timeout(lcb_CMDFTS *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_parent_span(lcb_CMDFTS *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_callback(lcb_CMDFTS *cmd, lcb_FTS_CALLBACK callback)
{
    cmd->callback = callback;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_query(lcb_CMDFTS *cmd, const char *query, size_t query_len)
{
    cmd->query = query;
    cmd->nquery = query_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdfts_handle(lcb_CMDFTS *cmd, lcb_FTS_HANDLE **handle)
{
    cmd->handle = handle;
    return LCB_SUCCESS;
}

struct lcb_FTS_HANDLE_ : lcb::jsparse::Parser::Actions {
    const lcb_RESPHTTP *cur_htresp;
    lcb_HTTP_HANDLE *htreq;
    lcb::jsparse::Parser *parser;
    const void *cookie;
    lcb_FTS_CALLBACK callback;
    lcb_INSTANCE * instance;
    size_t nrows;
    lcb_STATUS lasterr;
    lcbtrace_SPAN *span;

    void invoke_row(lcb_RESPFTS *resp);
    void invoke_last();

    lcb_FTS_HANDLE_(lcb_INSTANCE *, const void *, const lcb_CMDFTS *);
    ~lcb_FTS_HANDLE_();
    void JSPARSE_on_row(const lcb::jsparse::Row& datum) {
        lcb_RESPFTS resp = { 0 };
        resp.row = static_cast<const char*>(datum.row.iov_base);
        resp.nrow = datum.row.iov_len;
        nrows++;
        invoke_row(&resp);
    }
    void JSPARSE_on_error(const std::string&) {
        lasterr = LCB_PROTOCOL_ERROR;
    }
    void JSPARSE_on_complete(const std::string&) {
        // Nothing
    }
};

static void
chunk_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPHTTP *rh = (const lcb_RESPHTTP *)rb;
    lcb_FTS_HANDLE_ *req = static_cast<lcb_FTS_HANDLE_*>(rh->cookie);

    req->cur_htresp = rh;
    if (rh->rc != LCB_SUCCESS || rh->htstatus != 200) {
        if (req->lasterr == LCB_SUCCESS || rh->htstatus != 200) {
            req->lasterr = rh->rc ? rh->rc : LCB_HTTP_ERROR;
        }
    }

    if (rh->rflags & LCB_RESP_F_FINAL) {
        req->invoke_last();
        delete req;

    } else if (req->callback == NULL) {
        /* Cancelled. Similar to the block above, except the http request
         * should remain alive (so we can cancel it later on) */
        delete req;
    } else {
        req->parser->feed(static_cast<const char*>(rh->body), rh->nbody);
    }
}

void
lcb_FTS_HANDLE_::invoke_row(lcb_RESPFTS *resp)
{
    resp->cookie = const_cast<void*>(cookie);
    resp->htresp = cur_htresp;
    resp->handle = this;

    if (callback) {
        callback(instance, -4, resp);
    }
}

void
lcb_FTS_HANDLE_::invoke_last()
{
    lcb_RESPFTS resp = { 0 };
    resp.rflags |= LCB_RESP_F_FINAL;
    resp.rc = lasterr;

    if (parser) {
        lcb_IOV meta;
        parser->get_postmortem(meta);
        resp.row = static_cast<const char*>(meta.iov_base);
        resp.nrow = meta.iov_len;
    }
    invoke_row(&resp);
    callback = NULL;
}

lcb_FTS_HANDLE_::lcb_FTS_HANDLE_(lcb_INSTANCE * instance_, const void *cookie_, const lcb_CMDFTS *cmd)
: lcb::jsparse::Parser::Actions(),
  cur_htresp(NULL), htreq(NULL),
  parser(new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_FTS, this)),
  cookie(cookie_), callback(cmd->callback), instance(instance_), nrows(0),
  lasterr(LCB_SUCCESS), span(NULL)
{
    if (!callback) {
        lasterr = LCB_EINVAL;
        return;
    }

    std::string content_type("application/json");

    lcb_CMDHTTP *htcmd;
    lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_FTS);
    lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);
    lcb_cmdhttp_handle(htcmd, &htreq);
    lcb_cmdhttp_content_type(htcmd, content_type.c_str(), content_type.size());
    lcb_cmdhttp_streaming(htcmd, true);

    Json::Value root;
    Json::Reader rr;
    if (!rr.parse(cmd->query, cmd->query + cmd->nquery, root)) {
        lasterr = LCB_EINVAL;
        return;
    }

    const Json::Value& constRoot = root;
    const Json::Value& j_ixname = constRoot["indexName"];
    if (!j_ixname.isString()) {
        lasterr = LCB_EINVAL;
        return;
    }

    std::string url;
    url.append("api/index/").append(j_ixname.asCString()).append("/query");
    lcb_cmdhttp_path(htcmd, url.c_str(), url.size());

    // Making a copy here to ensure that we don't accidentally create a new
    // 'ctl' field.
    const Json::Value& ctl = constRoot["value"];
    uint32_t timeout = cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, n1ql_timeout);
    if (ctl.isObject()) {
        const Json::Value& tmo = ctl["timeout"];
        if (tmo.isNumeric()) {
            timeout = tmo.asLargestUInt();
        }
    } else {
        root["ctl"]["timeout"] =  timeout / 1000;
    }
    lcb_cmdhttp_timeout(htcmd, timeout);

    std::string qbody(Json::FastWriter().write(root));
    lcb_cmdhttp_body(htcmd, qbody.c_str(), qbody.size());

    lasterr = lcb_http(instance, this, htcmd);
    lcb_cmdhttp_destroy(htcmd);
    if (lasterr == LCB_SUCCESS) {
        htreq->set_callback(chunk_callback);
        if (cmd->handle) {
            *cmd->handle = reinterpret_cast<lcb_FTS_HANDLE_*>(this);
        }
        if (instance->settings->tracer) {
            char id[20] = {0};
            snprintf(id, sizeof(id), "%p", (void *)this);
            span = lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_DISPATCH_TO_SERVER, LCBTRACE_NOW, NULL);
            lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_OPERATION_ID, id);
            lcbtrace_span_add_system_tags(span, instance->settings, LCBTRACE_TAG_SERVICE_SEARCH);
        }
    }
}

lcb_FTS_HANDLE_::~lcb_FTS_HANDLE_()
{
    if (htreq != NULL) {
        lcb_http_cancel(instance, htreq);
        htreq = NULL;
    }
    if (span) {
        if (htreq) {
            lcbio_CTX *ctx = htreq->ioctx;
            if (ctx) {
                std::string remote;
                if (htreq->ipv6) {
                    remote = "[" + std::string(htreq->host) + "]:" + std::string(htreq->port);
                } else {
                    remote = std::string(htreq->host) + ":" + std::string(htreq->port);
                }
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_PEER_ADDRESS, remote.c_str());
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_LOCAL_ADDRESS,
                                          lcbio__inet_ntop(&ctx->sock->info->sa_local).c_str());
            }
        }
        lcbtrace_span_finish(span, LCBTRACE_NOW);
        span = NULL;
    }
    if (parser) {
        delete parser;
        parser = NULL;
    }
}

LIBCOUCHBASE_API lcb_STATUS lcb_fts(lcb_INSTANCE *instance, void *cookie, const lcb_CMDFTS *cmd)
{
    lcb_FTS_HANDLE_ *req = new lcb_FTS_HANDLE_(instance, cookie, cmd);
    if (req->lasterr) {
        lcb_STATUS rc = req->lasterr;
        delete req;
        return rc;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_fts_cancel(lcb_INSTANCE *, lcb_FTS_HANDLE *handle)
{
    handle->callback = NULL;
    return LCB_SUCCESS;
}
