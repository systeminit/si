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

#include <libcouchbase/couchbase.h>
#include <libcouchbase/pktfwd.h>
#include <jsparse/parser.h>
#include <string>
#include "docreq/docreq.h"

struct VRDocRequest : lcb::docreq::DocRequest {
    lcb_VIEW_HANDLE *parent;
    lcb_IOV key;
    lcb_IOV value;
    lcb_IOV geo;
    std::string rowbuf;
};

struct lcb_VIEW_HANDLE_ : lcb::jsparse::Parser::Actions {
    lcb_VIEW_HANDLE_(lcb_INSTANCE *, const void *, const lcb_CMDVIEW *);
    ~lcb_VIEW_HANDLE_();
    void invoke_last(lcb_STATUS err);
    void invoke_last()
    {
        invoke_last(lasterr);
    }
    void invoke_row(lcb_RESPVIEW *);
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
    void cancel();

    /**
     * Perform the actual HTTP request
     * @param cmd User's command
     */
    inline lcb_STATUS request_http(const lcb_CMDVIEW *cmd);

    bool is_include_docs() const
    {
        return cmdflags & LCB_CMDVIEWQUERY_F_INCLUDE_DOCS;
    }
    bool is_no_rowparse() const
    {
        return cmdflags & LCB_CMDVIEWQUERY_F_NOROWPARSE;
    }
    bool is_spatial() const
    {
        return cmdflags & LCB_CMDVIEWQUERY_F_SPATIAL;
    }

    void JSPARSE_on_row(const lcb::jsparse::Row &);
    void JSPARSE_on_error(const std::string &);
    void JSPARSE_on_complete(const std::string &);

    /** Current HTTP response to provide in callbacks */
    const lcb_RESPHTTP *cur_htresp;
    /** HTTP request object, in case we need to cancel prematurely */
    lcb_HTTP_HANDLE *htreq;
    lcb::jsparse::Parser *parser;
    const void *cookie;
    lcb::docreq::Queue *docq;
    lcb_VIEW_CALLBACK callback;
    lcb_INSTANCE *instance;

    unsigned refcount;
    uint32_t cmdflags;
    lcb_STATUS lasterr;
    lcbtrace_SPAN *span;
};
