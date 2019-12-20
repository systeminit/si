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

#ifndef LIBCOUCHBASE_RND_H
#define LIBCOUCHBASE_RND_H 1

#include "config.h"
#include <libcouchbase/couchbase.h>

#ifdef __cplusplus
extern "C" {
#endif

LCB_INTERNAL_API lcb_U32 lcb_next_rand32(void);
LCB_INTERNAL_API lcb_U64 lcb_next_rand64(void);

#if !defined(COMPILER_SUPPORTS_CXX11) || (defined(_MSC_VER) && _MSC_VER < 1600)
LCB_INTERNAL_API void lcb_rnd_global_init(void);
#endif

#ifdef __cplusplus
}
#endif

#endif
