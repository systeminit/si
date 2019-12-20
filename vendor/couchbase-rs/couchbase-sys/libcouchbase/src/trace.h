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

#ifndef LIBCOUCHBASE_TRACE_H
#define LIBCOUCHBASE_TRACE_H 1

#if defined(__clang__) || defined(__GNUC__)
#pragma GCC diagnostic push

#ifdef __clang__
#pragma GCC diagnostic ignored "-Wgnu-zero-variadic-macro-arguments"
#else
#pragma GCC diagnostic ignored "-Wvariadic-macros"
#endif
#endif

#ifdef HAVE_DTRACE
/* include the generated probes header and put markers in code */
#include "probes.h"
#define TRACE(probe) probe

#else
/* Wrap the probe to allow it to be removed when no systemtap available */
#define TRACE(probe)
#endif

#define TRACE_BEGIN_COMMON(TGT, instance, req, cmd, ...)                                                               \
    TGT(instance, (req)->request.opaque, lcb_ntohs((req)->request.vbucket), (req)->request.opcode,                     \
        (const char *)((cmd)->key.contig.bytes), (cmd)->key.contig.nbytes, ##__VA_ARGS__)

#define TRACE_BEGIN_SIMPLE(TGT, instance, req, cmd)                                                                    \
    TGT(instance, (req)->request.opaque, lcb_ntohs((req)->request.vbucket), (req)->request.opcode,                     \
        (const char *)(cmd)->key.contig.bytes, (cmd)->key.contig.nbytes)

#define TRACE_END_COMMON(TGT, instance, pkt, mcresp, resp, ...)                                                        \
    TGT(instance, mcresp->opaque(), mcresp->opcode(),                                                                  \
        (MCREQ_PKT_RDATA(pkt)->dispatch) - (MCREQ_PKT_RDATA(pkt)->start), (resp)->rc, (const char *)(resp)->key,       \
        (resp)->nkey, ##__VA_ARGS__)

#define TRACE_END_SIMPLE(TGT, instance, pkt, mcresp, resp)                                                             \
    TGT(instance, mcresp->opaque(), mcresp->opcode(), MCREQ_PKT_RDATA(pkt)->dispatch - MCREQ_PKT_RDATA(pkt)->start,    \
        (resp)->rc, (const char *)(resp)->key, (resp)->nkey)

#define TRACE_GET_BEGIN(instance, req, cmd)                                                                            \
    TRACE(TRACE_BEGIN_COMMON(LIBCOUCHBASE_GET_BEGIN, instance, req, cmd, (cmd)->exptime))
#define TRACE_GET_END(instance, pkt, mcresp, resp)                                                                     \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_GET_END, instance, pkt, mcresp, resp, (const char *)(resp)->value,             \
                           (resp)->nvalue, (resp)->itmflags, (resp)->cas, mcresp->datatype()))

#define TRACE_UNLOCK_BEGIN(instance, req, cmd) TRACE(TRACE_BEGIN_SIMPLE(LIBCOUCHBASE_UNLOCK_BEGIN, instance, req, cmd))
#define TRACE_UNLOCK_END(instance, pkt, mcresp, resp)                                                                  \
    TRACE(TRACE_END_SIMPLE(LIBCOUCHBASE_UNLOCK_END, instance, pkt, mcresp, resp))

#define TRACE_EXISTS_BEGIN(instance, req, cmd) TRACE(TRACE_BEGIN_SIMPLE(LIBCOUCHBASE_EXISTS_BEGIN, instance, req, cmd))
#define TRACE_EXISTS_END(instance, pkt, mcresp, resp)                                                                  \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_EXISTS_END, instance, pkt, mcresp, resp, (resp)->cas))

#define TRACE_STORE_BEGIN(instance, req, cmd)                                                                          \
    TRACE(                                                                                                             \
        TRACE_BEGIN_COMMON(LIBCOUCHBASE_STORE_BEGIN, instance, req, cmd,                                               \
                           (const char *)((cmd)->value.vtype == LCB_KV_IOV ? NULL : (cmd)->value.u_buf.contig.bytes),  \
                           ((cmd)->value.vtype == LCB_KV_IOV ? 0 : (cmd)->value.u_buf.contig.nbytes), (cmd)->flags,    \
                           (cmd)->cas, (req)->request.datatype, (cmd)->exptime))

#define TRACE_STORE_END(instance, pkt, mcresp, resp)                                                                   \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_STORE_END, instance, pkt, mcresp, resp, (resp)->cas))

#define TRACE_ARITHMETIC_BEGIN(instance, req, cmd)                                                                     \
    TRACE(TRACE_BEGIN_COMMON(LIBCOUCHBASE_ARITHMETIC_BEGIN, instance, req, cmd, (cmd)->delta, (cmd)->initial,          \
                             (cmd)->exptime))
#define TRACE_ARITHMETIC_END(instance, pkt, mcresp, resp)                                                              \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_ARITHMETIC_END, instance, pkt, mcresp, resp, (resp)->value, (resp)->cas))

#define TRACE_TOUCH_BEGIN(instance, req, cmd)                                                                          \
    TRACE(TRACE_BEGIN_COMMON(LIBCOUCHBASE_TOUCH_BEGIN, instance, req, cmd, (cmd)->exptime))
#define TRACE_TOUCH_END(instance, pkt, mcresp, resp)                                                                   \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_TOUCH_END, instance, pkt, mcresp, resp, (resp)->cas))

#define TRACE_REMOVE_BEGIN(instance, req, cmd) TRACE(TRACE_BEGIN_SIMPLE(LIBCOUCHBASE_REMOVE_BEGIN, instance, req, cmd))
#define TRACE_REMOVE_END(instance, pkt, mcresp, resp)                                                                  \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_REMOVE_END, instance, pkt, mcresp, resp, (resp)->cas))

#define TRACE_OBSERVE_BEGIN(instance, req, body)                                                                       \
    TRACE(LIBCOUCHBASE_OBSERVE_BEGIN(instance, (req)->request.opaque, (req)->request.opcode, body,                     \
                                     ntohl((req)->request.bodylen)))
#define TRACE_OBSERVE_PROGRESS(instance, pkt, mcresp, resp)                                                            \
    TRACE(TRACE_END_COMMON(LIBCOUCHBASE_OBSERVE_PROGRESS, instance, pkt, mcresp, resp, (resp)->cas, (resp)->status,    \
                           (resp)->ismaster, (resp)->ttp, (resp)->ttr))
#define TRACE_OBSERVE_END(instance, pkt)                                                                               \
    TRACE(LIBCOUCHBASE_OBSERVE_END(instance, pkt->opaque, PROTOCOL_BINARY_CMD_OBSERVE,                                 \
                                   MCREQ_PKT_RDATA(pkt)->dispatch - MCREQ_PKT_RDATA(pkt)->start, LCB_SUCCESS))

#define TRACE_HTTP_BEGIN(req)                                                                                          \
    TRACE(LIBCOUCHBASE_HTTP_BEGIN((req)->instance, (req), (req)->reqtype, (req)->method, (req)->url.c_str(),           \
                                  (req)->host.c_str(), (req)->port.c_str()))
#define TRACE_HTTP_END(req, rc, htstatus)                                                                              \
    TRACE(LIBCOUCHBASE_HTTP_END((req)->instance, (req), (req)->reqtype, (req)->method, (req)->url.c_str(),             \
                                (req)->host.c_str(), (req)->port.c_str(), rc, htstatus, (gethrtime() - (req)->start)))

#define TRACE_NEW_CONFIG(instance, config)                                                                             \
    TRACE(LIBCOUCHBASE_NEW_CONFIG(instance, (config)->vbc->revid, (config)->vbc->bname, (config)->vbc->buuid, (config)))

#ifdef __clang__
#pragma GCC diagnostic pop
#endif /* __clang__ */

#endif /* TRACE_H */
