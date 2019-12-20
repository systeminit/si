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

/* small utility for retrieving host/port information from the CTX */
static const lcb_host_t *get_ctx_host(const lcbio_CTX *ctx)
{
    static lcb_host_t host = {"NOHOST", "NOPORT", 0};
    if (!ctx) {
        return &host;
    }
    if (!ctx->sock) {
        return &host;
    }
    if (!ctx->sock->info) {
        return &host;
    }
    return &ctx->sock->info->ep;
}

#define CTX_LOGFMT_PRE "<" LCB_LOG_SPEC("%s%s%s:%s") "> (CTX=%p,%s"
#define CTX_LOGFMT CTX_LOGFMT_PRE ") "
#define CTX_LOGID(ctx)                                                                                                 \
    (ctx && ctx->sock && ctx->sock->settings->log_redaction) ? LCB_LOG_SD_OTAG : "",                                   \
        (get_ctx_host(ctx)->ipv6 ? "[" : ""), get_ctx_host(ctx)->host, (get_ctx_host(ctx)->ipv6 ? "]" : ""),           \
        get_ctx_host(ctx)->port, (ctx && ctx->sock && ctx->sock->settings->log_redaction) ? LCB_LOG_SD_CTAG : "",      \
        (void *)ctx, ctx ? ctx->subsys : ""
