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

#ifndef LCB_TRACING_INTERNAL_H
#define LCB_TRACING_INTERNAL_H

#include <libcouchbase/tracing.h>
#include "rnd.h"

#ifdef __cplusplus

#include <queue>

namespace lcb
{
namespace trace
{

class Span
{
  public:
    Span(lcbtrace_TRACER *tracer, const char *opname, uint64_t start, lcbtrace_REF_TYPE ref, lcbtrace_SPAN *other);
    ~Span();

    void finish(uint64_t finish);
    uint64_t duration()
    {
        return m_finish - m_start;
    }

    void add_tag(const char *name, int copy, const char *value);
    void add_tag(const char *name, int copy, const char *value, size_t value_len);
    void add_tag(const char *name, int copy, uint64_t value);
    void add_tag(const char *name, int copy, double value);
    void add_tag(const char *name, int copy, bool value);

    lcbtrace_TRACER *m_tracer;
    std::string m_opname;
    uint64_t m_span_id;
    uint64_t m_start;
    uint64_t m_finish;
    bool m_orphaned;
    Span *m_parent;
    sllist_root m_tags;
};

struct ReportedSpan {
    uint64_t duration;
    std::string payload;

    bool operator<(const ReportedSpan &rhs) const
    {
        return duration < rhs.duration;
    }
};

template < typename T > class FixedQueue : private std::priority_queue< T >
{
  public:
    explicit FixedQueue(size_t capacity) : m_capacity(capacity) {}

    void push(const T &item)
    {
        std::priority_queue< T >::push(item);
        if (this->size() > m_capacity) {
            this->c.pop_back();
        }
    }
    using std::priority_queue< T >::empty;
    using std::priority_queue< T >::top;
    using std::priority_queue< T >::pop;
    using std::priority_queue< T >::size;

  private:
    size_t m_capacity;
};

typedef ReportedSpan QueueEntry;
typedef FixedQueue< QueueEntry > FixedSpanQueue;
class ThresholdLoggingTracer
{
    lcbtrace_TRACER *m_wrapper;
    lcb_settings *m_settings;

    FixedSpanQueue m_orphans;
    FixedSpanQueue m_threshold;

    void flush_queue(FixedSpanQueue &queue, const char *message, bool warn);
    QueueEntry convert(lcbtrace_SPAN *span);

  public:
    ThresholdLoggingTracer(lcb_INSTANCE *instance);

    lcbtrace_TRACER *wrap();
    void add_orphan(lcbtrace_SPAN *span);
    void check_threshold(lcbtrace_SPAN *span);

    void flush_orphans();
    void flush_threshold();
    void do_flush_orphans();
    void do_flush_threshold();

    lcb::io::Timer< ThresholdLoggingTracer, &ThresholdLoggingTracer::flush_orphans > m_oflush;
    lcb::io::Timer< ThresholdLoggingTracer, &ThresholdLoggingTracer::flush_threshold > m_tflush;
};

} // namespace trace
} // namespace lcb

extern "C" {
#endif
LCB_INTERNAL_API
void lcbtrace_span_add_system_tags(lcbtrace_SPAN *span, lcb_settings *settings, const char *service);
LCB_INTERNAL_API
void lcbtrace_span_set_parent(lcbtrace_SPAN *span, lcbtrace_SPAN *parent);
LCB_INTERNAL_API
void lcbtrace_span_set_orphaned(lcbtrace_SPAN *span, int val);

#define LCBTRACE_KV_START(settings, cmd, operation_name, opaque, outspan)                                              \
    if ((settings)->tracer) {                                                                                          \
        lcbtrace_REF ref;                                                                                              \
        char opid[20] = {};                                                                                            \
        snprintf(opid, sizeof(opid), "0x%x", (int)opaque);                                                             \
        ref.type = LCBTRACE_REF_CHILD_OF;                                                                              \
        ref.span = cmd->pspan;                                                                                         \
        outspan = lcbtrace_span_start((settings)->tracer, operation_name, LCBTRACE_NOW, &ref);                         \
        lcbtrace_span_add_tag_str(outspan, LCBTRACE_TAG_OPERATION_ID, opid);                                           \
        lcbtrace_span_add_system_tags(outspan, (settings), LCBTRACE_TAG_SERVICE_KV);                                   \
    }

#define LCBTRACE_KV_COMPLETE(pipeline, request, response)                                                              \
    do {                                                                                                               \
        lcbtrace_SPAN *span = MCREQ_PKT_RDATA(request)->span;                                                          \
        if (span) {                                                                                                    \
            lcbtrace_span_add_tag_uint64(span, LCBTRACE_TAG_PEER_LATENCY, (response)->duration());                     \
            lcb::Server *server = static_cast< lcb::Server * >(pipeline);                                              \
            const lcb_host_t *remote = server->curhost;                                                                \
            if (remote) {                                                                                              \
                std::string hh;                                                                                        \
                if (remote->ipv6) {                                                                                    \
                    hh.append("[").append(remote->host).append("]:").append(remote->port);                             \
                } else {                                                                                               \
                    hh.append(remote->host).append(":").append(remote->port);                                          \
                }                                                                                                      \
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_PEER_ADDRESS, hh.c_str());                                \
            }                                                                                                          \
            lcbio_CTX *ctx = server->connctx;                                                                          \
            if (ctx) {                                                                                                 \
                char local_id[34] = {};                                                                                \
                snprintf(local_id, sizeof(local_id), "%016" PRIx64 "/%016" PRIx64,                                     \
                         (lcb_U64)server->get_settings()->iid, ctx->sock->id);                                         \
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_LOCAL_ID, local_id);                                      \
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_LOCAL_ADDRESS,                                            \
                                          lcbio__inet_ntop(&ctx->sock->info->sa_local).c_str());                       \
            }                                                                                                          \
        }                                                                                                              \
    } while (0);

#define LCBTRACE_KV_CLOSE(request)                                                                                     \
    do {                                                                                                               \
        lcbtrace_SPAN *span = MCREQ_PKT_RDATA(request)->span;                                                          \
        if (span) {                                                                                                    \
            lcbtrace_span_finish(span, LCBTRACE_NOW);                                                                  \
            MCREQ_PKT_RDATA(request)->span = NULL;                                                                     \
        }                                                                                                              \
    } while (0);

#define LCBTRACE_KV_FINISH(pipeline, request, response)                                                                \
    LCBTRACE_KV_COMPLETE(pipeline, request, response)                                                                  \
    LCBTRACE_KV_CLOSE(request)

#ifdef __cplusplus
}
#endif

#else

#define LCBTRACE_KV_START(settings, cmd, operation_name, opaque, outspan)
#define LCBTRACE_KV_COMPLETE(pipeline, request, response)
#define LCBTRACE_KV_CLOSE(request)
#define LCBTRACE_KV_FINISH(pipeline, request, response)

#endif /* LCB_TRACING_INTERNAL_H */
