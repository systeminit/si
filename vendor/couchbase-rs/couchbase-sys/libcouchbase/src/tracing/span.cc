/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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
#include "sllist-inl.h"
#ifdef HAVE__FTIME64_S
#include <sys/timeb.h>
#endif

typedef enum { TAGVAL_STRING, TAGVAL_UINT64, TAGVAL_DOUBLE, TAGVAL_BOOL } tag_type;
typedef struct tag_value {
    sllist_node slnode;
    struct {
        char *p;
        int need_free;
    } key;
    tag_type t;
    union {
        struct {
            char *p;
            size_t l;
        } s;
        lcb_U64 u64;
        double d;
        int b;
    } v;
} tag_value;

LIBCOUCHBASE_API
uint64_t lcbtrace_now()
{
    uint64_t ret;
#ifdef HAVE__FTIME64_S
    struct __timeb64 tb;
    _ftime64_s(&tb);
    ret = (uint64_t)tb.time * 1000000; /* sec */
    ret += (uint64_t)tb.millitm * 1000;
#else
    struct timeval tv;
    if (gettimeofday(&tv, NULL) == -1) {
        return -1;
    }
    ret = (uint64_t)tv.tv_sec * 1000000;
    ret += (uint64_t)tv.tv_usec;
#endif
    return ret;
}

LIBCOUCHBASE_API
void lcbtrace_span_finish(lcbtrace_SPAN *span, uint64_t now)
{
    if (!span) {
        return;
    }

    span->finish(now);
    delete span;
}

LIBCOUCHBASE_API
void lcbtrace_span_add_tag_str(lcbtrace_SPAN *span, const char *name, const char *value)
{
    if (!span || name == NULL || value == NULL) {
        return;
    }
    span->add_tag(name, 1, value);
}

LIBCOUCHBASE_API
void lcbtrace_span_add_tag_uint64(lcbtrace_SPAN *span, const char *name, uint64_t value)
{
    if (!span || name == NULL) {
        return;
    }
    span->add_tag(name, 1, value);
}

LIBCOUCHBASE_API
void lcbtrace_span_add_tag_double(lcbtrace_SPAN *span, const char *name, double value)
{
    if (!span || name == NULL) {
        return;
    }
    span->add_tag(name, 1, value);
}

LIBCOUCHBASE_API
void lcbtrace_span_add_tag_bool(lcbtrace_SPAN *span, const char *name, int value)
{
    if (!span || name == NULL) {
        return;
    }
    span->add_tag(name, 1, (bool)value);
}

LCB_INTERNAL_API
void lcbtrace_span_add_system_tags(lcbtrace_SPAN *span, lcb_settings *settings, const char *service)
{
    if (!span) {
        return;
    }
    span->add_tag(LCBTRACE_TAG_SERVICE, 0, service);
    std::string client_string(LCB_CLIENT_ID);
    if (settings->client_string) {
        client_string += " ";
        client_string += settings->client_string;
    }
    span->add_tag(LCBTRACE_TAG_COMPONENT, 0, client_string.c_str(), client_string.size());
    if (settings->bucket) {
        span->add_tag(LCBTRACE_TAG_DB_INSTANCE, 0, settings->bucket);
    }
}

LIBCOUCHBASE_API
lcbtrace_SPAN *lcbtrace_span_get_parent(lcbtrace_SPAN *span)
{
    if (!span) {
        return NULL;
    }
    return span->m_parent;
}

LCB_INTERNAL_API
void lcbtrace_span_set_parent(lcbtrace_SPAN *span, lcbtrace_SPAN *parent)
{
    if (!span) {
        return;
    }
    span->m_parent = parent;
}

LIBCOUCHBASE_API
uint64_t lcbtrace_span_get_start_ts(lcbtrace_SPAN *span)
{
    if (!span) {
        return 0;
    }
    return span->m_start;
}

LIBCOUCHBASE_API
uint64_t lcbtrace_span_get_finish_ts(lcbtrace_SPAN *span)
{
    if (!span) {
        return 0;
    }
    return span->m_finish;
}

LIBCOUCHBASE_API
int lcbtrace_span_is_orphaned(lcbtrace_SPAN *span)
{
    return span && span->m_orphaned;
}

LCB_INTERNAL_API
void lcbtrace_span_set_orphaned(lcbtrace_SPAN *span, int val)
{
    if (!span) {
        return;
    }
    span->m_orphaned = (val != 0);
}

LIBCOUCHBASE_API
uint64_t lcbtrace_span_get_span_id(lcbtrace_SPAN *span)
{
    if (!span) {
        return 0;
    }
    return span->m_span_id;
}

LIBCOUCHBASE_API
const char *lcbtrace_span_get_operation(lcbtrace_SPAN *span)
{
    if (!span) {
        return NULL;
    }
    return span->m_opname.c_str();
}

LIBCOUCHBASE_API
uint64_t lcbtrace_span_get_trace_id(lcbtrace_SPAN *span)
{
    if (!span) {
        return 0;
    }
    if (span->m_parent) {
        return span->m_parent->m_span_id;
    }
    return span->m_span_id;
}

LIBCOUCHBASE_API
lcb_STATUS lcbtrace_span_get_tag_str(lcbtrace_SPAN *span, const char *name, char **value, size_t *nvalue)
{
    if (!span || name == NULL || nvalue == NULL || value == NULL) {
        return LCB_EINVAL;
    }

    sllist_iterator iter;
    SLLIST_ITERFOR(&span->m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        if (strcmp(name, val->key.p) == 0) {
            if (val->t != TAGVAL_STRING) {
                return LCB_EINVAL;
            }
            *value = val->v.s.p;
            *nvalue = val->v.s.l;
            return LCB_SUCCESS;
        }
    }

    return LCB_KEY_ENOENT;
}

LIBCOUCHBASE_API lcb_STATUS lcbtrace_span_get_tag_uint64(lcbtrace_SPAN *span, const char *name, uint64_t *value)
{
    if (!span || name == NULL || value == NULL) {
        return LCB_EINVAL;
    }

    sllist_iterator iter;
    SLLIST_ITERFOR(&span->m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        if (strcmp(name, val->key.p) == 0) {
            if (val->t != TAGVAL_UINT64) {
                return LCB_EINVAL;
            }
            *value = val->v.u64;
            return LCB_SUCCESS;
        }
    }

    return LCB_KEY_ENOENT;
}

LIBCOUCHBASE_API lcb_STATUS lcbtrace_span_get_tag_double(lcbtrace_SPAN *span, const char *name, double *value)
{
    if (!span || name == NULL || value == NULL) {
        return LCB_EINVAL;
    }

    sllist_iterator iter;
    SLLIST_ITERFOR(&span->m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        if (strcmp(name, val->key.p) == 0) {
            if (val->t != TAGVAL_DOUBLE) {
                return LCB_EINVAL;
            }
            *value = val->v.d;
            return LCB_SUCCESS;
        }
    }

    return LCB_KEY_ENOENT;
}

LIBCOUCHBASE_API lcb_STATUS lcbtrace_span_get_tag_bool(lcbtrace_SPAN *span, const char *name, int *value)
{
    if (!span || name == NULL || value == NULL) {
        return LCB_EINVAL;
    }

    sllist_iterator iter;
    SLLIST_ITERFOR(&span->m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        if (strcmp(name, val->key.p) == 0) {
            if (val->t != TAGVAL_BOOL) {
                return LCB_EINVAL;
            }
            *value = val->v.b;
            return LCB_SUCCESS;
        }
    }

    return LCB_KEY_ENOENT;
}

LIBCOUCHBASE_API int lcbtrace_span_has_tag(lcbtrace_SPAN *span, const char *name)
{
    if (!span || name == NULL) {
        return 0;
    }

    sllist_iterator iter;
    SLLIST_ITERFOR(&span->m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        if (strcmp(name, val->key.p) == 0) {
            return 1;
        }
    }
    return 0;
}

using namespace lcb::trace;

Span::Span(lcbtrace_TRACER *tracer, const char *opname, uint64_t start, lcbtrace_REF_TYPE ref, lcbtrace_SPAN *other)
    : m_tracer(tracer), m_opname(opname)
{
    m_start = start ? start : lcbtrace_now();
    m_span_id = lcb_next_rand64();
    m_orphaned = false;
    memset(&m_tags, 0, sizeof(m_tags));
    add_tag(LCBTRACE_TAG_DB_TYPE, 0, "couchbase");
    add_tag(LCBTRACE_TAG_SPAN_KIND, 0, "client");

    if (other != NULL && ref == LCBTRACE_REF_CHILD_OF) {
        m_parent = other;
    } else {
        m_parent = NULL;
    }
}

Span::~Span()
{
    sllist_iterator iter;
    SLLIST_ITERFOR(&m_tags, &iter)
    {
        tag_value *val = SLLIST_ITEM(iter.cur, tag_value, slnode);
        sllist_iter_remove(&m_tags, &iter);
        if (val->key.need_free) {
            free(val->key.p);
        }
        if (val->t == TAGVAL_STRING) {
            free(val->v.s.p);
        }
        free(val);
    }
}

void Span::finish(uint64_t now)
{
    m_finish = now ? now : lcbtrace_now();
    if (m_tracer && m_tracer->version == 0 && m_tracer->v.v0.report) {
        m_tracer->v.v0.report(m_tracer, this);
    }
}

void Span::add_tag(const char *name, int copy, const char *value)
{
    if (name && value) {
        add_tag(name, copy, value, strlen(value));
    }
}

void Span::add_tag(const char *name, int copy, const char *value, size_t value_len)
{
    tag_value *val = (tag_value *)calloc(1, sizeof(tag_value));
    val->t = TAGVAL_STRING;
    val->key.need_free = copy;
    if (copy) {
        val->key.p = strdup(name);
    } else {
        val->key.p = (char *)name;
    }
    val->v.s.p = (char *)calloc(value_len, sizeof(char));
    val->v.s.l = value_len;
    memcpy(val->v.s.p, value, value_len);
    sllist_append(&m_tags, &val->slnode);
}

void Span::add_tag(const char *name, int copy, uint64_t value)
{
    tag_value *val = (tag_value *)calloc(1, sizeof(tag_value));
    val->t = TAGVAL_UINT64;
    val->key.need_free = copy;
    if (copy) {
        val->key.p = strdup(name);
    } else {
        val->key.p = (char *)name;
    }
    val->v.u64 = value;
    sllist_append(&m_tags, &val->slnode);
}

void Span::add_tag(const char *name, int copy, double value)
{
    tag_value *val = (tag_value *)calloc(1, sizeof(tag_value));
    val->t = TAGVAL_DOUBLE;
    val->key.need_free = copy;
    if (copy) {
        val->key.p = strdup(name);
    } else {
        val->key.p = (char *)name;
    }
    val->v.d = value;
    sllist_append(&m_tags, &val->slnode);
}

void Span::add_tag(const char *name, int copy, bool value)
{
    tag_value *val = (tag_value *)calloc(1, sizeof(tag_value));
    val->t = TAGVAL_BOOL;
    val->key.need_free = copy;
    if (copy) {
        val->key.p = strdup(name);
    } else {
        val->key.p = (char *)name;
    }
    val->v.b = value;
    sllist_append(&m_tags, &val->slnode);
}
