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

#include "viewreq.h"
#include "sllist-inl.h"
#include "http/http.h"
#include "internal.h"

#define MAX_GET_URI_LENGTH 2048

LIBCOUCHBASE_API lcb_STATUS lcb_respview_status(const lcb_RESPVIEW *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_cookie(const lcb_RESPVIEW *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_key(const lcb_RESPVIEW *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_doc_id(const lcb_RESPVIEW *resp, const char **doc_id, size_t *doc_id_len)
{
    *doc_id = resp->docid;
    *doc_id_len = resp->ndocid;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_row(const lcb_RESPVIEW *resp, const char **row, size_t *row_len)
{
    *row = resp->value;
    *row_len = resp->nvalue;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_http_response(const lcb_RESPVIEW *resp, const lcb_RESPHTTP **http)
{
    *http = resp->htresp;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_document(const lcb_RESPVIEW *resp, const lcb_RESPGET **doc)
{
    *doc = resp->docresp;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respview_handle(const lcb_RESPVIEW *resp, lcb_VIEW_HANDLE **handle)
{
    *handle = resp->handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_respview_is_final(const lcb_RESPVIEW *resp)
{
    return resp->rflags & LCB_RESP_F_FINAL;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_create(lcb_CMDVIEW **cmd)
{
    *cmd = (lcb_CMDVIEW *)calloc(1, sizeof(lcb_CMDVIEW));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_destroy(lcb_CMDVIEW *cmd)
{
    free(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_timeout(lcb_CMDVIEW *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_parent_span(lcb_CMDVIEW *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_callback(lcb_CMDVIEW *cmd, lcb_VIEW_CALLBACK callback)
{
    cmd->callback = callback;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_design_document(lcb_CMDVIEW *cmd, const char *ddoc, size_t ddoc_len)
{
    cmd->ddoc = ddoc;
    cmd->nddoc = ddoc_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_view_name(lcb_CMDVIEW *cmd, const char *view, size_t view_len)
{
    cmd->view = view;
    cmd->nview = view_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_option_string(lcb_CMDVIEW *cmd, const char *optstr, size_t optstr_len)
{
    cmd->optstr = optstr;
    cmd->noptstr = optstr_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_post_data(lcb_CMDVIEW *cmd, const char *data, size_t data_len)
{
    cmd->postdata = data;
    cmd->npostdata = data_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_include_docs(lcb_CMDVIEW *cmd, int include_docs)
{
    if (include_docs) {
        cmd->cmdflags |= LCB_CMDVIEWQUERY_F_INCLUDE_DOCS;
    } else {
        cmd->cmdflags &= ~LCB_CMDVIEWQUERY_F_INCLUDE_DOCS;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_max_concurrent_docs(lcb_CMDVIEW *cmd, uint32_t num)
{
    cmd->docs_concurrent_max = num;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_no_row_parse(lcb_CMDVIEW *cmd, int flag)
{
    if (flag) {
        cmd->cmdflags |= LCB_CMDVIEWQUERY_F_NOROWPARSE;
    } else {
        cmd->cmdflags &= ~LCB_CMDVIEWQUERY_F_NOROWPARSE;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdview_handle(lcb_CMDVIEW *cmd, lcb_VIEW_HANDLE **handle)
{
    cmd->handle = handle;
    return LCB_SUCCESS;
}

static void chunk_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *);

template < typename value_type, typename size_type >
void IOV2PTRLEN(const lcb_IOV *iov, value_type *&ptr, size_type &len)
{
    ptr = reinterpret_cast< const value_type * >(iov->iov_base);
    len = iov->iov_len;
}

/* Whether the request (from the user side) is still ongoing */
#define CAN_CONTINUE(req) ((req)->callback != NULL)
#define LOGARGS(instance, lvl) instance->settings, "views", LCB_LOG_##lvl, __FILE__, __LINE__

void lcb_VIEW_HANDLE_::invoke_last(lcb_STATUS err)
{
    lcb_RESPVIEW resp = {0};
    if (callback == NULL) {
        return;
    }
    if (docq && docq->has_pending()) {
        return;
    }

    resp.rc = err;
    resp.htresp = cur_htresp;
    resp.cookie = const_cast< void * >(cookie);
    resp.rflags = LCB_RESP_F_FINAL;
    resp.handle = this;
    if (parser && parser->meta_complete) {
        resp.value = parser->meta_buf.c_str();
        resp.nvalue = parser->meta_buf.size();
    } else {
        resp.rflags |= LCB_RESP_F_CLIENTGEN;
    }
    callback(instance, LCB_CALLBACK_VIEWQUERY, &resp);
    cancel();
}

void lcb_VIEW_HANDLE_::invoke_row(lcb_RESPVIEW *resp)
{
    if (callback == NULL) {
        return;
    }
    resp->htresp = cur_htresp;
    resp->cookie = const_cast< void * >(cookie);
    callback(instance, LCB_CALLBACK_VIEWQUERY, resp);
}

static void chunk_callback(lcb_INSTANCE *instance, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPHTTP *rh = (const lcb_RESPHTTP *)rb;
    lcb_VIEW_HANDLE_ *req = reinterpret_cast< lcb_VIEW_HANDLE_ * >(rh->cookie);

    req->cur_htresp = rh;

    if (rh->rc != LCB_SUCCESS || rh->htstatus != 200 || (rh->rflags & LCB_RESP_F_FINAL)) {
        if (req->lasterr == LCB_SUCCESS && rh->htstatus != 200) {
            if (rh->rc != LCB_SUCCESS) {
                req->lasterr = rh->rc;
            } else {
                lcb_log(LOGARGS(instance, DEBUG), "Got not ok http status %d", rh->htstatus);
                req->lasterr = LCB_HTTP_ERROR;
            }
        }
        req->ref();
        req->invoke_last();
        if (rh->rflags & LCB_RESP_F_FINAL) {
            req->htreq = NULL;
            req->unref();
        }
        req->cur_htresp = NULL;
        req->unref();
        return;
    }

    if (!CAN_CONTINUE(req)) {
        return;
    }

    req->refcount++;
    req->parser->feed(reinterpret_cast< const char * >(rh->body), rh->nbody);
    req->cur_htresp = NULL;
    req->unref();
}

static void do_copy_iov(std::string &dstbuf, lcb_IOV *dstiov, const lcb_IOV *srciov)
{
    dstiov->iov_len = srciov->iov_len;
    dstiov->iov_base = const_cast< char * >(dstbuf.c_str() + dstbuf.size());
    dstbuf.append(reinterpret_cast< const char * >(srciov->iov_base), srciov->iov_len);
}

static VRDocRequest *mk_docreq(const lcb::jsparse::Row *datum)
{
    size_t extra_alloc = 0;
    VRDocRequest *dreq;
    extra_alloc = datum->key.iov_len + datum->value.iov_len + datum->geo.iov_len + datum->docid.iov_len;

    dreq = new VRDocRequest();
    dreq->rowbuf.reserve(extra_alloc);
    do_copy_iov(dreq->rowbuf, &dreq->key, &datum->key);
    do_copy_iov(dreq->rowbuf, &dreq->value, &datum->value);
    do_copy_iov(dreq->rowbuf, &dreq->docid, &datum->docid);
    do_copy_iov(dreq->rowbuf, &dreq->geo, &datum->geo);
    return dreq;
}

void lcb_VIEW_HANDLE_::JSPARSE_on_row(const lcb::jsparse::Row &datum)
{
    using lcb::jsparse::Row;
    if (!is_no_rowparse()) {
        parser->parse_viewrow(const_cast< Row & >(datum));
    }

    if (is_include_docs() && datum.docid.iov_len && callback) {
        VRDocRequest *dreq = mk_docreq(&datum);
        dreq->parent = this;
        docq->add(dreq);
        ref();

    } else {
        lcb_RESPVIEW resp = {0};
        if (is_no_rowparse()) {
            IOV2PTRLEN(&datum.row, resp.value, resp.nvalue);
        } else {
            IOV2PTRLEN(&datum.key, resp.key, resp.nkey);
            IOV2PTRLEN(&datum.docid, resp.docid, resp.ndocid);
            IOV2PTRLEN(&datum.value, resp.value, resp.nvalue);
            IOV2PTRLEN(&datum.geo, resp.geometry, resp.ngeometry);
        }
        resp.htresp = cur_htresp;
        invoke_row(&resp);
    }
}

void lcb_VIEW_HANDLE_::JSPARSE_on_error(const std::string &)
{
    invoke_last(LCB_PROTOCOL_ERROR);
}

void lcb_VIEW_HANDLE_::JSPARSE_on_complete(const std::string &)
{
    // Nothing
}

static void doc_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPGET *rg = (const lcb_RESPGET *)rb;
    lcb::docreq::DocRequest *dreq = reinterpret_cast< lcb::docreq::DocRequest * >(rb->cookie);
    lcb::docreq::Queue *q = dreq->parent;

    q->ref();

    q->n_awaiting_response--;
    dreq->docresp = *rg;
    dreq->ready = 1;
    dreq->docresp.key = dreq->docid.iov_base;
    dreq->docresp.nkey = dreq->docid.iov_len;

    /* Reference the response data, since we might not be invoking this right
     * away */
    if (rg->rc == LCB_SUCCESS) {
        lcb_backbuf_ref(reinterpret_cast< lcb_BACKBUF >(dreq->docresp.bufh));
    }
    q->check();

    q->unref();
}

static lcb_STATUS cb_op_schedule(lcb::docreq::Queue *q, lcb::docreq::DocRequest *dreq)
{
    lcb_CMDGET gcmd = {0};

    LCB_CMD_SET_KEY(&gcmd, dreq->docid.iov_base, dreq->docid.iov_len);
    if (dreq->parent->parent) {
        lcb_VIEW_HANDLE_ *req = reinterpret_cast< lcb_VIEW_HANDLE_ * >(dreq->parent->parent);
        if (req->span) {
            LCB_CMD_SET_TRACESPAN(&gcmd, req->span);
        }
    }
    dreq->callback = doc_callback;
    gcmd.cmdflags |= LCB_CMD_F_INTERNAL_CALLBACK;
    return lcb_get(q->instance, &dreq->callback, &gcmd);
}

static void cb_doc_ready(lcb::docreq::Queue *q, lcb::docreq::DocRequest *req_base)
{
    lcb_RESPVIEW resp = {0};
    VRDocRequest *dreq = (VRDocRequest *)req_base;
    resp.docresp = &dreq->docresp;
    IOV2PTRLEN(&dreq->key, resp.key, resp.nkey);
    IOV2PTRLEN(&dreq->value, resp.value, resp.nvalue);
    IOV2PTRLEN(&dreq->docid, resp.docid, resp.ndocid);
    IOV2PTRLEN(&dreq->geo, resp.geometry, resp.ngeometry);

    if (q->parent) {
        reinterpret_cast< lcb_VIEW_HANDLE_ * >(q->parent)->invoke_row(&resp);
    }

    delete dreq;

    if (q->parent) {
        reinterpret_cast< lcb_VIEW_HANDLE_ * >(q->parent)->unref();
    }
}

static void cb_docq_throttle(lcb::docreq::Queue *q, int enabled)
{
    lcb_VIEW_HANDLE_ *req = reinterpret_cast< lcb_VIEW_HANDLE_ * >(q->parent);
    if (req == NULL || req->htreq == NULL) {
        return;
    }
    if (enabled) {
        req->htreq->pause();
    } else {
        req->htreq->resume();
    }
}

lcb_VIEW_HANDLE_::~lcb_VIEW_HANDLE_()
{
    invoke_last();

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

    if (parser != NULL) {
        delete parser;
    }
    if (htreq != NULL) {
        lcb_http_cancel(instance, htreq);
    }
    if (docq != NULL) {
        docq->parent = NULL;
        docq->unref();
    }
}

lcb_STATUS lcb_VIEW_HANDLE_::request_http(const lcb_CMDVIEW *cmd)
{
    lcb_CMDHTTP *htcmd;

    lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_VIEW);
    lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_GET);
    lcb_cmdhttp_streaming(htcmd, true);

    std::string path;
    path.append("_design/");
    path.append(cmd->ddoc, cmd->nddoc);
    path.append(is_spatial() ? "/_spatial/" : "/_view/");
    path.append(cmd->view, cmd->nview);
    if (cmd->noptstr) {
        path.append("?").append(cmd->optstr, cmd->noptstr);
    }

    lcb_cmdhttp_path(htcmd, path.c_str(), path.size());
    lcb_cmdhttp_handle(htcmd, &htreq);

    if (cmd->npostdata) {
        std::string content_type("application/json");
        lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);
        lcb_cmdhttp_body(htcmd, cmd->postdata, cmd->npostdata);
        lcb_cmdhttp_content_type(htcmd, content_type.c_str(), content_type.size());
    }
    lcb_cmdhttp_timeout(htcmd, cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, views_timeout));

    lcb_STATUS err = lcb_http(instance, this, htcmd);
    lcb_cmdhttp_destroy(htcmd);
    if (err == LCB_SUCCESS) {
        htreq->set_callback(chunk_callback);
    }
    return err;
}

lcb_VIEW_HANDLE_::lcb_VIEW_HANDLE_(lcb_INSTANCE *instance_, const void *cookie_, const lcb_CMDVIEW *cmd)
    : cur_htresp(NULL), htreq(NULL), parser(new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_VIEWS, this)),
      cookie(cookie_), docq(NULL), callback(cmd->callback), instance(instance_), refcount(1), cmdflags(cmd->cmdflags),
      lasterr(LCB_SUCCESS), span(NULL)
{

    // Validate:
    if (cmd->nddoc == 0 || cmd->nview == 0 || callback == NULL) {
        lasterr = LCB_EINVAL;
    } else if (is_include_docs() && is_no_rowparse()) {
        lasterr = LCB_OPTIONS_CONFLICT;
    } else if (cmd->noptstr > MAX_GET_URI_LENGTH) {
        lasterr = LCB_E2BIG;
    }
    if (lasterr != LCB_SUCCESS) {
        return;
    }

    if (is_include_docs()) {
        docq = new lcb::docreq::Queue(instance);
        docq->parent = this;
        docq->cb_schedule = cb_op_schedule;
        docq->cb_ready = cb_doc_ready;
        docq->cb_throttle = cb_docq_throttle;
        if (cmd->docs_concurrent_max) {
            docq->max_pending_response = cmd->docs_concurrent_max;
        }
    }

    if (cmd->handle) {
        *cmd->handle = this;
    }

    lcb_aspend_add(&instance->pendops, LCB_PENDTYPE_COUNTER, NULL);

    lasterr = request_http(cmd);
    if (lasterr == LCB_SUCCESS && instance->settings->tracer) {
        char id[20] = {0};
        snprintf(id, sizeof(id), "%p", (void *)this);
        span = lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_DISPATCH_TO_SERVER, LCBTRACE_NOW, NULL);
        lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_OPERATION_ID, id);
        lcbtrace_span_add_system_tags(span, instance->settings, LCBTRACE_TAG_SERVICE_VIEW);
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_view(lcb_INSTANCE *instance, void *cookie, const lcb_CMDVIEW *cmd)
{
    lcb_VIEW_HANDLE_ *req = new lcb_VIEW_HANDLE_(instance, cookie, cmd);
    lcb_STATUS err = req->lasterr;
    if (err != LCB_SUCCESS) {
        req->cancel();
        delete req;
    }
    return err;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_view_cancel(lcb_INSTANCE *, lcb_VIEW_HANDLE *handle)
{
    handle->cancel();
    return LCB_SUCCESS;
}

void lcb_VIEW_HANDLE_::cancel()
{
    if (callback) {
        callback = NULL;
        lcb_aspend_del(&instance->pendops, LCB_PENDTYPE_COUNTER, NULL);
        if (docq) {
            docq->cancel();
        }
    }
}
