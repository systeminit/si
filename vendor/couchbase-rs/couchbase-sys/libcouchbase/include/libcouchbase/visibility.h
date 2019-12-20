/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright (C) 2011 Couchbase, Inc
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
#ifndef LIBCOUCHBASE_VISIBILITY_H
#define LIBCOUCHBASE_VISIBILITY_H

#ifdef LIBCOUCHBASE_STATIC
#define LIBCOUCHBASE_API
#define LCB_EXTERN_VAR extern
#define LCB_CLASS_EXPORT
#else

#ifdef LIBCOUCHBASE_INTERNAL
#ifdef __SUNPRO_C
#define LIBCOUCHBASE_API __global
#define LCB_CLASS_EXPORT LIBCOUCHBASE_API
#elif defined(HAVE_VISIBILITY) && HAVE_VISIBILITY
#define LIBCOUCHBASE_API __attribute__((visibility("default")))
#define LCB_CLASS_EXPORT LIBCOUCHBASE_API
#elif defined(_MSC_VER)
#define LIBCOUCHBASE_API extern __declspec(dllexport)
#define LCB_CLASS_EXPORT __declspec(dllexport)
#else
#define LIBCOUCHBASE_API
#define LCB_CLASS_EXPORT
#endif /* compiler version */

#else /* !LIBCOUCHBASE_INTERNAL */
#ifdef _MSC_VER
#define LIBCOUCHBASE_API extern __declspec(dllimport)
#define LCB_CLASS_EXPORT __declspec(dllimport)
#else
#define LIBCOUCHBASE_API
#define LCB_CLASS_EXPORT
#endif
#endif /* LIBCOUCHBASE_INTERNAL */

/* Define LCB_EXTERN_VAR only if !LIBCOUCHBASE_STATIC */
#ifdef _MSC_VER
/* Already includes 'extern' in LIBCOUCHBASE_API def, don't use it twice! */
#define LCB_EXTERN_VAR
#else
#define LCB_EXTERN_VAR extern
#endif /* _MSC_VER */
#endif /* !LIBCOUCHBASE_STATIC */

/**
 * This symbol declares internal APIs as accessible from other modules.
 * It should still not be used.
 */
#define LCB_INTERNAL_API LIBCOUCHBASE_API

#endif
