/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018-2019 Couchbase, Inc.
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

#include <libcouchbase/couchbase.h>
#include <jsparse/parser.h>
#include "internal.h"
#include "auth-priv.h"
#include "http/http.h"
#include "logging.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <map>
#include <string>
#include <list>
#include "docreq/docreq.h"
#include "rnd.h"

#define LOGFMT "(NR=%p) "
#define LOGID(req) static_cast< const void * >(req)
#define LOGARGS(req, lvl) req->instance->settings, "analytics", LCB_LOG_##lvl, __FILE__, __LINE__

using namespace lcb;

struct lcb_INGEST_PARAM_ {
    lcb_INGEST_METHOD method;
    void *cookie;

    const char *row;
    size_t row_len;

    const char *id;
    size_t id_len;
    void (*id_dtor)(const char *);

    const char *out;
    size_t out_len;
    void (*out_dtor)(const char *);
};

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_cookie(lcb_INGEST_PARAM *param, void **cookie)
{
    *cookie = param->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_row(lcb_INGEST_PARAM *param, const char **row,
                                                               size_t *row_len)
{
    *row = param->row;
    *row_len = param->row_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_method(lcb_INGEST_PARAM *param, lcb_INGEST_METHOD *method)
{
    *method = param->method;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_set_id(lcb_INGEST_PARAM *param, const char *id,
                                                                  size_t id_len, void (*id_dtor)(const char *))
{
    param->id = id;
    param->id_len = id_len;
    param->id_dtor = id_dtor;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_dataconverter_param_set_out(lcb_INGEST_PARAM *param, const char *out,
                                                                   size_t out_len, void (*out_dtor)(const char *))
{
    param->out_dtor = out_dtor;
    param->out_len = out_len;
    param->out = out;
    return LCB_SUCCESS;
}

static lcb_INGEST_STATUS default_data_converter(lcb_INSTANCE *, lcb_INGEST_PARAM *param)
{
    param->id_dtor = (void (*)(const char *))free;
    char *buf = static_cast< char * >(calloc(34, sizeof(char)));
    param->id_len = snprintf(buf, 34, "%016" PRIx64 "-%016" PRIx64, lcb_next_rand64(), lcb_next_rand64());
    param->id = buf;
    return LCB_INGEST_STATUS_OK;
}

struct lcb_RESPANALYTICS_ {
    LCB_RESP_BASE
    const char *row;
    size_t nrow;
    const lcb_RESPHTTP *htresp;
    lcb_ANALYTICS_HANDLE *handle;
};

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_status(const lcb_RESPANALYTICS *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_cookie(const lcb_RESPANALYTICS *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_http_response(const lcb_RESPANALYTICS *resp, const lcb_RESPHTTP **http)
{
    *http = resp->htresp;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_row(const lcb_RESPANALYTICS *resp, const char **row, size_t *row_len)
{
    *row = resp->row;
    *row_len = resp->nrow;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_handle(const lcb_RESPANALYTICS *resp, lcb_ANALYTICS_HANDLE **handle)
{
    *handle = resp->handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_respanalytics_is_final(const lcb_RESPANALYTICS *resp)
{
    return resp->rflags & LCB_RESP_F_FINAL;
}

struct lcb_INGEST_OPTIONS_ {
    lcb_INGEST_METHOD method;
    uint32_t exptime;
    bool ignore_errors;
    lcb_INGEST_DATACONVERTER_CALLBACK data_converter;

    lcb_INGEST_OPTIONS_()
        : method(LCB_INGEST_METHOD_NONE), exptime(0), ignore_errors(false), data_converter(default_data_converter)
    {
    }
};

struct IngestRequest : docreq::DocRequest {
    lcb_ANALYTICS_HANDLE *parent;
    std::string row;
};

struct lcb_CMDANALYTICS_ {
    LCB_CMD_BASE;

    Json::Value root;
    lcb_ANALYTICS_CALLBACK callback;
    lcb_ANALYTICS_HANDLE **handle;
    lcb_INGEST_OPTIONS *ingest;

    lcb_CMDANALYTICS_() : root(Json::objectValue), callback(NULL), handle(NULL), ingest(NULL) {}
};

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_create(lcb_CMDANALYTICS **cmd)
{
    *cmd = new lcb_CMDANALYTICS();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_destroy(lcb_CMDANALYTICS *cmd)
{
    delete cmd;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_timeout(lcb_CMDANALYTICS *cmd, uint32_t timeout)
{
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_reset(lcb_CMDANALYTICS *cmd)
{
    cmd->root.clear();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_parent_span(lcb_CMDANALYTICS *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_handle(lcb_CMDANALYTICS *cmd, lcb_ANALYTICS_HANDLE **handle)
{
    cmd->handle = handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_callback(lcb_CMDANALYTICS *cmd, lcb_ANALYTICS_CALLBACK callback)
{
    if (cmd) {
        cmd->callback = callback;
        return LCB_SUCCESS;
    }
    return LCB_EINVAL;
}

#define fix_strlen(s, n)                                                                                               \
    if (n == (size_t)-1) {                                                                                             \
        n = strlen(s);                                                                                                 \
    }

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_query(lcb_CMDANALYTICS *cmd, const char *qstr, size_t nqstr)
{
    fix_strlen(qstr, nqstr);
    Json::Value value;
    if (!Json::Reader().parse(qstr, qstr + nqstr, value)) {
        return LCB_EINVAL;
    }
    cmd->root = value;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_option(lcb_CMDANALYTICS *cmd, const char *name, size_t name_len,
                                                    const char *value, size_t value_len)
{
    fix_strlen(value, value_len);
    fix_strlen(name, name_len);
    Json::Value jsonVal;
    if (!Json::Reader().parse(value, value + value_len, jsonVal)) {
        return LCB_EINVAL;
    }
    cmd->root[std::string(name, name_len)] = jsonVal;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_statement(lcb_CMDANALYTICS *cmd, const char *statement,
                                                       size_t statement_len)
{
    fix_strlen(statement, statement_len);
    cmd->root["statement"] = std::string(statement, statement_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_named_param(lcb_CMDANALYTICS *cmd, const char *name, size_t name_len,
                                                         const char *value, size_t value_len)
{
    return lcb_cmdanalytics_option(cmd, name, name_len, value, value_len);
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_positional_param(lcb_CMDANALYTICS *cmd, const char *value,
                                                              size_t value_len)
{
    fix_strlen(value, value_len);
    Json::Value jval;
    if (!Json::Reader().parse(value, value + value_len, jval)) {
        return LCB_EINVAL;
    }
    cmd->root["args"].append(jval);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_deferred(lcb_CMDANALYTICS *cmd, int deferred)
{
    if (deferred) {
        cmd->root["mode"] = std::string("async");
    } else {
        cmd->root.removeMember("mode");
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_ingest_options(lcb_CMDANALYTICS *cmd, lcb_INGEST_OPTIONS *options)
{
    cmd->ingest = options;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_create(lcb_INGEST_OPTIONS **options)
{
    *options = new lcb_INGEST_OPTIONS();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_destroy(lcb_INGEST_OPTIONS *options)
{
    delete options;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_method(lcb_INGEST_OPTIONS *options, lcb_INGEST_METHOD method)
{
    options->method = method;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_expiration(lcb_INGEST_OPTIONS *options, uint32_t expiration)
{
    options->exptime = expiration;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_ignore_error(lcb_INGEST_OPTIONS *options, int flag)
{
    options->ignore_errors = flag ? true : false;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_ingest_options_data_converter(lcb_INGEST_OPTIONS *options,
                                                              lcb_INGEST_DATACONVERTER_CALLBACK callback)
{
    options->data_converter = callback;
    return LCB_SUCCESS;
}

struct lcb_DEFERRED_HANDLE_ {
    std::string status;
    std::string handle;
    lcb_ANALYTICS_CALLBACK callback;

    lcb_DEFERRED_HANDLE_(std::string status_, std::string handle_) : status(status_), handle(handle_) {}
};

LIBCOUCHBASE_API lcb_STATUS lcb_respanalytics_deferred_handle_extract(const lcb_RESPANALYTICS *resp,
                                                                      lcb_DEFERRED_HANDLE **handle)
{
    if (resp == NULL || resp->rc != LCB_SUCCESS || ((resp->rflags & (LCB_RESP_F_FINAL | LCB_RESP_F_EXTDATA)) == 0) ||
        resp->nrow == 0 || resp->row == NULL) {
        return LCB_EINVAL;
    }
    Json::Value payload;
    if (!Json::Reader().parse(resp->row, resp->row + resp->nrow, payload)) {
        return LCB_EINVAL;
    }
    if (!payload.isObject()) {
        return LCB_EINVAL;
    }
    Json::Value status = payload["status"];
    Json::Value value = payload["handle"];
    if (status.isString() && value.isString()) {
        *handle = new lcb_DEFERRED_HANDLE_(status.asString(), value.asString());
        return LCB_SUCCESS;
    }
    return LCB_EINVAL;
}

LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_destroy(lcb_DEFERRED_HANDLE *handle)
{
    if (handle == NULL) {
        return LCB_EINVAL;
    }
    delete handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_status(lcb_DEFERRED_HANDLE *handle, const char **status,
                                                       size_t *status_len)
{
    if (handle == NULL) {
        return LCB_EINVAL;
    }
    *status = handle->status.c_str();
    *status_len = handle->status.size();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_callback(lcb_DEFERRED_HANDLE *handle, lcb_ANALYTICS_CALLBACK callback)
{
    if (handle) {
        handle->callback = callback;
        return LCB_SUCCESS;
    }
    return LCB_EINVAL;
}

typedef struct lcb_ANALYTICS_HANDLE_ : lcb::jsparse::Parser::Actions {
    const lcb_RESPHTTP *cur_htresp;
    lcb_HTTP_HANDLE *htreq;
    lcb::jsparse::Parser *parser;
    void *cookie;
    lcb_ANALYTICS_CALLBACK callback;
    lcb_INSTANCE *instance;
    lcb_STATUS lasterr;
    lcb_U32 timeout;
    // How many rows were received. Used to avoid parsing the meta
    size_t nrows;

    /** Request body as received from the application */
    Json::Value json;
    const Json::Value &json_const() const
    {
        return json;
    }

    /** String of the original statement. Cached here to avoid jsoncpp lookups */
    std::string statement;

    /** Whether we're retrying this */
    bool was_retried;

    /** Non-empty if this is deferred query check/fetch */
    std::string deferred_handle;

    lcb_INGEST_OPTIONS *ingest;
    docreq::Queue *docq;
    unsigned refcount;

    lcbtrace_SPAN *span;

    void unref()
    {
        if (!--refcount) {
            delete this;
        }
    }

    void ref()
    {
        refcount++;
    }

    /**
     * Issues the HTTP request for the query
     * @param payload The body to send
     * @return Error code from lcb's http subsystem
     */
    inline lcb_STATUS issue_htreq(const std::string &payload);

    lcb_STATUS issue_htreq()
    {
        std::string s = Json::FastWriter().write(json);
        return issue_htreq(s);
    }

    /**
     * Attempt to retry the query. This will inspect the meta (if present)
     * for any errors indicating that a failure might be a result of a stale
     * plan, and if this query was retried already.
     * @return true if the retry was successful.
     */
    inline bool maybe_retry();

    /**
     * Returns true if payload matches retry conditions.
     */
    inline bool has_retriable_error(const Json::Value &root);

    /**
     * Pass a row back to the application
     * @param resp The response. This is populated with state information
     *  from the current query
     * @param is_last Whether this is the last row. If this is the last, then
     *  the RESP_F_FINAL flag is set, and no further callbacks will be invoked
     */
    inline void invoke_row(lcb_RESPANALYTICS *resp, bool is_last);

    inline lcb_ANALYTICS_HANDLE_(lcb_INSTANCE *obj, void *user_cookie, const lcb_CMDANALYTICS *cmd);
    inline lcb_ANALYTICS_HANDLE_(lcb_INSTANCE *obj, void *user_cookie, lcb_DEFERRED_HANDLE *handle);
    inline ~lcb_ANALYTICS_HANDLE_();

    // Parser overrides:
    void JSPARSE_on_row(const lcb::jsparse::Row &row)
    {
        lcb_RESPANALYTICS resp = {0};
        resp.handle = this;
        resp.row = static_cast< const char * >(row.row.iov_base);
        resp.nrow = row.row.iov_len;
        nrows++;
        if (ingest && ingest->method != LCB_INGEST_METHOD_NONE) {
            IngestRequest *req = new IngestRequest();
            req->parent = this;
            req->row.assign(static_cast< const char * >(row.row.iov_base), row.row.iov_len);
            docq->add(req);
            ref();
        }
        invoke_row(&resp, false);
    }
    void JSPARSE_on_error(const std::string &)
    {
        lasterr = LCB_PROTOCOL_ERROR;
    }
    void JSPARSE_on_complete(const std::string &)
    {
        // Nothing
    }

} ANALYTICSREQ;

static bool parse_json(const char *s, size_t n, Json::Value &res)
{
    return Json::Reader().parse(s, s + n, res);
}

bool ANALYTICSREQ::has_retriable_error(const Json::Value &root)
{
    if (!root.isObject()) {
        return false;
    }
    const Json::Value &errors = root["errors"];
    if (!errors.isArray()) {
        return false;
    }
    Json::Value::const_iterator ii;
    for (ii = errors.begin(); ii != errors.end(); ++ii) {
        const Json::Value &cur = *ii;
        if (!cur.isObject()) {
            continue; // eh?
        }
        const Json::Value &jcode = cur["code"];
        unsigned code = 0;
        if (jcode.isNumeric()) {
            code = jcode.asUInt();
            switch (code) {
                case 23000:
                case 23003:
                case 23007:
                    lcb_log(LOGARGS(this, TRACE), LOGFMT "Will retry request. code: %d", LOGID(this), code);
                    return true;
                default:
                    break;
            }
        }
    }
    return false;
}

bool ANALYTICSREQ::maybe_retry()
{
    // Examines the buffer to determine the type of error
    Json::Value root;
    lcb_IOV meta;

    if (callback == NULL) {
        // Cancelled
        return false;
    }

    if (nrows) {
        // Has results:
        return false;
    }

    if (was_retried) {
        return false;
    }

    was_retried = true;
    parser->get_postmortem(meta);
    if (!parse_json(static_cast< const char * >(meta.iov_base), meta.iov_len, root)) {
        return false; // Not JSON
    }
    if (has_retriable_error(root)) {
        return true;
    }

    return false;
}

void ANALYTICSREQ::invoke_row(lcb_RESPANALYTICS *resp, bool is_last)
{
    resp->cookie = const_cast< void * >(cookie);
    resp->htresp = cur_htresp;

    if (is_last) {
        lcb_IOV meta;
        resp->rflags |= LCB_RESP_F_FINAL;
        resp->rc = lasterr;
        parser->get_postmortem(meta);
        resp->row = static_cast< const char * >(meta.iov_base);
        resp->nrow = meta.iov_len;
        if (!deferred_handle.empty()) {
            /* signal that response might have deferred handle */
            resp->rflags |= LCB_RESP_F_EXTDATA;
        }
    }

    if (callback) {
        callback(instance, LCB_CALLBACK_ANALYTICS, resp);
    }
    if (is_last) {
        callback = NULL;
    }
}

lcb_ANALYTICS_HANDLE_::~lcb_ANALYTICS_HANDLE_()
{
    if (htreq) {
        lcb_http_cancel(instance, htreq);
        htreq = NULL;
    }

    if (callback) {
        lcb_RESPANALYTICS resp = {0};
        invoke_row(&resp, 1);
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
    }

    if (docq != NULL) {
        docq->parent = NULL;
        docq->unref();
        lcb_aspend_del(&instance->pendops, LCB_PENDTYPE_COUNTER, NULL);
    }
}

static void chunk_callback(lcb_INSTANCE *instance, int ign, const lcb_RESPBASE *rb)
{
    const lcb_RESPHTTP *rh = (const lcb_RESPHTTP *)rb;
    ANALYTICSREQ *req = static_cast< ANALYTICSREQ * >(rh->cookie);

    (void)ign;
    (void)instance;

    req->cur_htresp = rh;
    if (rh->rc != LCB_SUCCESS || rh->htstatus != 200) {
        if (req->lasterr == LCB_SUCCESS || rh->htstatus != 200) {
            req->lasterr = rh->rc ? rh->rc : LCB_HTTP_ERROR;
        }
    }

    if (rh->rflags & LCB_RESP_F_FINAL) {
        req->htreq = NULL;
        if (!req->maybe_retry()) {
            req->unref();
        }
        return;
    } else if (req->callback == NULL) {
        /* Cancelled. Similar to the block above, except the http request
         * should remain alive (so we can cancel it later on) */
        req->unref();
        return;
    }
    req->parser->feed(static_cast< const char * >(rh->body), rh->nbody);
}

lcb_STATUS ANALYTICSREQ::issue_htreq(const std::string &body)
{
    lcb_CMDHTTP *htcmd;
    std::string content_type("application/json");

    lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_CBAS);
    lcb_cmdhttp_body(htcmd, body.c_str(), body.size());
    lcb_cmdhttp_content_type(htcmd, content_type.c_str(), content_type.size());

    if (deferred_handle.empty()) {
        lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);
    } else {
        lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_GET);
        lcb_cmdhttp_host(htcmd, deferred_handle.c_str(), deferred_handle.size());
    }
    lcb_cmdhttp_streaming(htcmd, true);
    lcb_cmdhttp_handle(htcmd, &htreq);
    lcb_cmdhttp_timeout(htcmd, timeout);

    lcb_STATUS rc = lcb_http(instance, this, htcmd);
    lcb_cmdhttp_destroy(htcmd);
    if (rc == LCB_SUCCESS) {
        htreq->set_callback(chunk_callback);
    }
    return rc;
}

lcb_U32 lcb_analyticsreq_parsetmo(const std::string &s)
{
    double num;
    int nchars, rv;

    rv = sscanf(s.c_str(), "%lf%n", &num, &nchars);
    if (rv != 1) {
        return 0;
    }
    std::string mults = s.substr(nchars);

    // Get the actual timeout value in microseconds. Note we can't use the macros
    // since they will truncate the double value.
    if (mults == "s") {
        return num * static_cast< double >(LCB_S2US(1));
    } else if (mults == "ms") {
        return num * static_cast< double >(LCB_MS2US(1));
    } else if (mults == "h") {
        return num * static_cast< double >(LCB_S2US(3600));
    } else if (mults == "us") {
        return num;
    } else if (mults == "m") {
        return num * static_cast< double >(LCB_S2US(60));
    } else if (mults == "ns") {
        return LCB_NS2US(num);
    } else {
        return 0;
    }
}

static void doc_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    lcb::docreq::DocRequest *dreq = reinterpret_cast< lcb::docreq::DocRequest * >(rb->cookie);
    lcb::docreq::Queue *q = dreq->parent;

    q->ref();

    q->n_awaiting_response--;
    dreq->ready = 1;

    q->check();

    q->unref();
}

static lcb_STATUS cb_op_schedule(lcb::docreq::Queue *q, lcb::docreq::DocRequest *dreq)
{
    IngestRequest *req = reinterpret_cast< IngestRequest * >(dreq);
    lcb_ANALYTICS_HANDLE_ *areq = req->parent;

    if (areq->ingest == NULL) {
        return LCB_EINTERNAL;
    }

    lcb_STORE_OPERATION op;
    switch (areq->ingest->method) {
        case LCB_INGEST_METHOD_INSERT:
            op = LCB_STORE_ADD;
            break;
        case LCB_INGEST_METHOD_REPLACE:
            op = LCB_STORE_REPLACE;
            break;
        case LCB_INGEST_METHOD_UPSERT:
        default:
            op = LCB_STORE_UPSERT;
            break;
    }

    lcb_INGEST_PARAM param;
    param.method = areq->ingest->method;
    param.row = req->row.c_str();
    param.row_len = req->row.size();
    param.cookie = areq->cookie;
    switch (areq->ingest->data_converter(q->instance, &param)) {
        case LCB_INGEST_STATUS_OK:
            /* continue */
            break;
        case LCB_INGEST_STATUS_IGNORE:
            /* assume that the user hasn't allocated anything */
            return LCB_SUCCESS;
            break;
        default:
            return LCB_EINTERNAL;
            break;
    }
    lcb_CMDSTORE *cmd;
    lcb_cmdstore_create(&cmd, op);
    lcb_cmdstore_expiration(cmd, areq->ingest->exptime);
    lcb_cmdstore_key(cmd, param.id, param.id_len);
    lcb_cmdstore_parent_span(cmd, areq->span);
    if (param.out) {
        lcb_cmdstore_value(cmd, param.out, param.out_len);
    } else {
        lcb_cmdstore_value(cmd, req->row.c_str(), req->row.size());
    }
    dreq->callback = doc_callback;
    cmd->cmdflags |= LCB_CMD_F_INTERNAL_CALLBACK;
    lcb_STATUS err = lcb_store(q->instance, &dreq->callback, cmd);
    lcb_cmdstore_destroy(cmd);
    if (param.id_dtor && param.id) {
        param.id_dtor(param.id);
    }
    if (param.out_dtor && param.out) {
        param.out_dtor(param.out);
    }
    return err;
}

static void cb_doc_ready(lcb::docreq::Queue *q, lcb::docreq::DocRequest *req_base)
{
    IngestRequest *req = (IngestRequest *)req_base;
    /* TODO: check if we should ignore errors */
    delete req;

    if (q->parent) {
        reinterpret_cast< lcb_ANALYTICS_HANDLE_ * >(q->parent)->unref();
    }
}

static void cb_docq_throttle(lcb::docreq::Queue *q, int enabled)
{
    lcb_ANALYTICS_HANDLE_ *req = reinterpret_cast< lcb_ANALYTICS_HANDLE_ * >(q->parent);
    if (req == NULL || req->htreq == NULL) {
        return;
    }
    if (enabled) {
        req->htreq->pause();
    } else {
        req->htreq->resume();
    }
}

lcb_ANALYTICS_HANDLE_::lcb_ANALYTICS_HANDLE_(lcb_INSTANCE *obj, void *user_cookie, const lcb_CMDANALYTICS *cmd)
    : cur_htresp(NULL), htreq(NULL), parser(new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_ANALYTICS, this)),
      cookie(user_cookie), callback(cmd->callback), instance(obj), lasterr(LCB_SUCCESS), timeout(0), nrows(0),
      was_retried(false), deferred_handle(""), ingest(cmd->ingest), docq(NULL), refcount(1), span(NULL)
{

    if (cmd->handle) {
        *cmd->handle = this;
    }

    std::string encoded = Json::FastWriter().write(cmd->root);
    if (!parse_json(encoded.c_str(), encoded.size(), json)) {
        lasterr = LCB_EINVAL;
        return;
    }

    const Json::Value &j_statement = json_const()["statement"];
    if (j_statement.isString()) {
        statement = j_statement.asString();
    } else if (!j_statement.isNull()) {
        lasterr = LCB_EINVAL;
        return;
    }

    Json::Value &tmoval = json["timeout"];
    if (tmoval.isNull()) {
        // Set the default timeout as the server-side query timeout if no
        // other timeout is used.
        char buf[64] = {0};
        sprintf(buf, "%uus", LCBT_SETTING(obj, n1ql_timeout));
        tmoval = buf;
        /* FIXME: use separate timeout for analytics */
        timeout = LCBT_SETTING(obj, n1ql_timeout);
    } else if (tmoval.isString()) {
        timeout = lcb_analyticsreq_parsetmo(tmoval.asString());
    } else {
        // Timeout is not a string!
        lasterr = LCB_EINVAL;
        return;
    }

    if (instance->settings->tracer) {
        char id[20] = {0};
        snprintf(id, sizeof(id), "%p", (void *)this);
        span = lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_DISPATCH_TO_SERVER, LCBTRACE_NOW, NULL);
        lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_OPERATION_ID, id);
        lcbtrace_span_add_system_tags(span, instance->settings, LCBTRACE_TAG_SERVICE_ANALYTICS);
    }

    if (ingest && ingest->method != LCB_INGEST_METHOD_NONE) {
        docq = new lcb::docreq::Queue(instance);
        docq->parent = this;
        docq->cb_schedule = cb_op_schedule;
        docq->cb_ready = cb_doc_ready;
        docq->cb_throttle = cb_docq_throttle;
        // TODO: docq->max_pending_response;
        lcb_aspend_add(&instance->pendops, LCB_PENDTYPE_COUNTER, NULL);
    }
}

lcb_ANALYTICS_HANDLE_::lcb_ANALYTICS_HANDLE_(lcb_INSTANCE *obj, void *user_cookie, lcb_DEFERRED_HANDLE *handle)
    : cur_htresp(NULL), htreq(NULL),
      parser(new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_ANALYTICS_DEFERRED, this)), cookie(user_cookie),
      callback(handle->callback), instance(obj), lasterr(LCB_SUCCESS), timeout(0), nrows(0), was_retried(false),
      deferred_handle(handle->handle), ingest(NULL), docq(NULL), refcount(1), span(NULL)
{
    /* FIXME: use separate timeout for analytics */
    timeout = LCBT_SETTING(obj, n1ql_timeout);

    if (instance->settings->tracer) {
        char id[20] = {0};
        snprintf(id, sizeof(id), "%p", (void *)this);
        span = lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_DISPATCH_TO_SERVER, LCBTRACE_NOW, NULL);
        lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_OPERATION_ID, id);
        lcbtrace_span_add_system_tags(span, instance->settings, LCBTRACE_TAG_SERVICE_ANALYTICS);
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_analytics(lcb_INSTANCE *instance, void *cookie, const lcb_CMDANALYTICS *cmd)
{
    lcb_STATUS err;
    ANALYTICSREQ *req = NULL;

    if (cmd->callback == NULL) {
        return LCB_EINVAL;
    }

    req = new lcb_ANALYTICS_HANDLE_(instance, cookie, cmd);
    if (!req) {
        err = LCB_CLIENT_ENOMEM;
        goto GT_DESTROY;
    }
    if ((err = req->lasterr) != LCB_SUCCESS) {
        goto GT_DESTROY;
    }

    if ((err = req->issue_htreq()) != LCB_SUCCESS) {
        goto GT_DESTROY;
    }

    return LCB_SUCCESS;

GT_DESTROY:
    if (cmd->handle) {
        *cmd->handle = NULL;
    }

    if (req) {
        req->callback = NULL;
        req->unref();
    }
    return err;
}

LIBCOUCHBASE_API lcb_STATUS lcb_deferred_handle_poll(lcb_INSTANCE *instance, void *cookie, lcb_DEFERRED_HANDLE *handle)
{
    lcb_STATUS err;
    ANALYTICSREQ *req = NULL;

    if (handle->callback == NULL || handle->handle.empty()) {
        return LCB_EINVAL;
    }

    req = new lcb_ANALYTICS_HANDLE_(instance, cookie, handle);
    if (!req) {
        err = LCB_CLIENT_ENOMEM;
        goto GT_DESTROY;
    }
    if ((err = req->lasterr) != LCB_SUCCESS) {
        goto GT_DESTROY;
    }

    if ((err = req->issue_htreq()) != LCB_SUCCESS) {
        goto GT_DESTROY;
    }

    return LCB_SUCCESS;

GT_DESTROY:
    if (req) {
        req->callback = NULL;
        req->unref();
    }
    return err;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdanalytics_cancel(lcb_INSTANCE *, lcb_ANALYTICS_HANDLE *handle)
{
    if (handle->callback) {
        handle->callback = NULL;
        if (handle->docq) {
            handle->docq->cancel();
        }
    }
    return LCB_SUCCESS;
}
