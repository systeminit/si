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

#include "internal.h"

#include <iostream>
#include <vector>

#define LOGARGS(tracer, lvl) tracer->m_settings, "tracer", LCB_LOG_##lvl, __FILE__, __LINE__

using namespace lcb::trace;

LIBCOUCHBASE_API lcbtrace_TRACER *lcbtrace_new(lcb_INSTANCE *instance, uint64_t flags)
{
    if (instance == NULL || (flags & LCBTRACE_F_THRESHOLD) == 0) {
        return NULL;
    }
    return (new ThresholdLoggingTracer(instance))->wrap();
}

extern "C" {
static void tlt_destructor(lcbtrace_TRACER *wrapper)
{
    if (wrapper == NULL) {
        return;
    }
    if (wrapper->cookie) {
        ThresholdLoggingTracer *tracer = reinterpret_cast< ThresholdLoggingTracer * >(wrapper->cookie);
        tracer->do_flush_orphans();
        tracer->do_flush_threshold();
        delete tracer;
        wrapper->cookie = NULL;
    }
    delete wrapper;
}

static void tlt_report(lcbtrace_TRACER *wrapper, lcbtrace_SPAN *span)
{
    if (wrapper == NULL || wrapper->cookie == NULL) {
        return;
    }

    ThresholdLoggingTracer *tracer = reinterpret_cast< ThresholdLoggingTracer * >(wrapper->cookie);
    char *value = NULL;
    size_t nvalue;
    if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_SERVICE, &value, &nvalue) == LCB_SUCCESS) {
        if (strncmp(value, LCBTRACE_TAG_SERVICE_KV, nvalue) == 0) {
            if (lcbtrace_span_is_orphaned(span)) {
                tracer->add_orphan(span);
            } else {
                tracer->check_threshold(span);
            }
        }
    }
}
}

lcbtrace_TRACER *ThresholdLoggingTracer::wrap()
{
    if (m_wrapper) {
        return m_wrapper;
    }
    m_wrapper = new lcbtrace_TRACER();
    m_wrapper->version = 0;
    m_wrapper->flags = 0;
    m_wrapper->cookie = this;
    m_wrapper->destructor = tlt_destructor;
    m_wrapper->v.v0.report = tlt_report;
    return m_wrapper;
}

QueueEntry ThresholdLoggingTracer::convert(lcbtrace_SPAN *span)
{
    QueueEntry orphan;
    orphan.duration = span->duration();
    Json::Value entry;
    char *value;
    size_t nvalue;

    if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_OPERATION_ID, &value, &nvalue) == LCB_SUCCESS) {
        entry["last_operation_id"] = std::string(span->m_opname) + ":" + std::string(value, value + nvalue);
    }
    if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_LOCAL_ID, &value, &nvalue) == LCB_SUCCESS) {
        entry["last_local_id"] = std::string(value, value + nvalue);
    }
    if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_LOCAL_ADDRESS, &value, &nvalue) == LCB_SUCCESS) {
        entry["last_local_address"] = std::string(value, value + nvalue);
    }
    if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_PEER_ADDRESS, &value, &nvalue) == LCB_SUCCESS) {
        entry["last_remote_address"] = std::string(value, value + nvalue);
    }
    uint64_t num;
    if (lcbtrace_span_get_tag_uint64(span, LCBTRACE_TAG_PEER_LATENCY, &num) == LCB_SUCCESS) {
        entry["server_us"] = (Json::UInt64)num;
    }
    entry["total_us"] = (Json::UInt64)orphan.duration;
    orphan.payload = Json::FastWriter().write(entry);
    return orphan;
}

void ThresholdLoggingTracer::add_orphan(lcbtrace_SPAN *span)
{
    m_orphans.push(convert(span));
}

void ThresholdLoggingTracer::check_threshold(lcbtrace_SPAN *span)
{
    if (span->duration() > m_settings->tracer_threshold[LCBTRACE_THRESHOLD_KV]) {
        m_threshold.push(convert(span));
    }
}

void ThresholdLoggingTracer::flush_queue(FixedSpanQueue &queue, const char *message, bool warn = false)
{
    Json::Value entries;
    entries["service"] = "kv";
    entries["count"] = (Json::UInt)queue.size();
    Json::Value top;
    while (!queue.empty()) {
        Json::Value entry;
        if (Json::Reader().parse(queue.top().payload, entry)) {
            top.append(entry);
        }
        queue.pop();
    }
    entries["top"] = top;
    std::string doc = Json::FastWriter().write(entries);
    if (doc.size() > 0 && doc[doc.size() - 1] == '\n') {
        doc[doc.size() - 1] = '\0';
    }
    if (warn) {
        lcb_log(LOGARGS(this, WARN), "%s: %s", message, doc.c_str());
    } else {
        lcb_log(LOGARGS(this, INFO), "%s: %s", message, doc.c_str());
    }
}

void ThresholdLoggingTracer::do_flush_orphans()
{
    if (m_orphans.empty()) {
        return;
    }
    flush_queue(m_orphans, "Orphan responses observed", true);
}

void ThresholdLoggingTracer::do_flush_threshold()
{
    if (m_threshold.empty()) {
        return;
    }
    flush_queue(m_threshold, "Operations over threshold");
}

void ThresholdLoggingTracer::flush_orphans()
{
    lcb_U32 tv = m_settings->tracer_orphaned_queue_flush_interval;
    if (tv == 0) {
        m_oflush.cancel();
    } else {
        m_oflush.rearm(tv);
    }
    do_flush_orphans();
}

void ThresholdLoggingTracer::flush_threshold()
{
    lcb_U32 tv = m_settings->tracer_threshold_queue_flush_interval;
    if (tv == 0) {
        m_tflush.cancel();
    } else {
        m_tflush.rearm(tv);
    }
    do_flush_threshold();
}

ThresholdLoggingTracer::ThresholdLoggingTracer(lcb_INSTANCE *instance)
    : m_wrapper(NULL), m_settings(instance->settings), m_orphans(LCBT_SETTING(instance, tracer_orphaned_queue_size)),
      m_threshold(LCBT_SETTING(instance, tracer_threshold_queue_size)), m_oflush(instance->iotable, this),
      m_tflush(instance->iotable, this)
{
    lcb_U32 tv = m_settings->tracer_orphaned_queue_flush_interval;
    if (tv > 0) {
        m_oflush.rearm(tv);
    }
    tv = m_settings->tracer_threshold_queue_flush_interval;
    if (tv > 0) {
        m_tflush.rearm(tv);
    }
}
