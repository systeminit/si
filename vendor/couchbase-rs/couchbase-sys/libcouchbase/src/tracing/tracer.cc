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

using namespace lcb::trace;

LIBCOUCHBASE_API void lcbtrace_destroy(lcbtrace_TRACER *tracer)
{
    if (tracer && tracer->destructor) {
        tracer->destructor(tracer);
    }
}

LIBCOUCHBASE_API
lcbtrace_SPAN *lcbtrace_span_start(lcbtrace_TRACER *tracer, const char *opname, uint64_t start, lcbtrace_REF *ref)
{
    lcbtrace_REF_TYPE type = LCBTRACE_REF_NONE;
    lcbtrace_SPAN *other = NULL;
    if (ref) {
        type = ref->type;
        other = ref->span;
    }
    return new Span(tracer, opname, start, type, other);
}

LIBCOUCHBASE_API
lcbtrace_TRACER *lcb_get_tracer(lcb_INSTANCE *instance)
{
    return (instance && instance->settings) ? instance->settings->tracer : NULL;
}

LIBCOUCHBASE_API
void lcb_set_tracer(lcb_INSTANCE *instance, lcbtrace_TRACER *tracer)
{
    if (instance && instance->settings) {
        instance->settings->tracer = tracer;
    }
}
