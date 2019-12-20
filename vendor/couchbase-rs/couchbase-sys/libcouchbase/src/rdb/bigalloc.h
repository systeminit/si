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

#ifndef RDB_BIGALLOC
#define RDB_BIGALLOC
#include "list.h"
#include <stdio.h>
#ifdef __cplusplus
extern "C" {
#endif

/**
 * Big block allocator. This allocator will allocate large chunks of memory with
 * the assumption that will typically not be wasted too quickly. It also keeps
 * track of an allocation history, so that it can adjust to the current
 * "climate".
 *
 * This header file exists for internal use. To create an allocator instance,
 * refer to rdb_bigalloc_new() in rope.h
 */

typedef struct {
    rdb_ALLOCATOR base;
    lcb_clist_t bufs; /* list of pooled segments */
    unsigned refcount;
    unsigned min_blk_alloc; /* minimum alloc size */
    unsigned max_blk_alloc; /* maximum alloc size (bigger than this is not pooled) */
    unsigned max_blk_count; /* maximum number of blocks to pool */
    unsigned n_requests;    /* number of requests. Reset every RECHECK_RATE */
    unsigned n_toobig;      /* number of requests > max_blk_alloc */
    unsigned n_toosmall;    /* number of requests < min_blk_alloc */

    /** counters updated at the end only */
    unsigned total_malloc;
    unsigned total_requests;
    unsigned total_toobig;
    unsigned total_toosmall;
} rdb_BIGALLOC;

#define RDB_BIGALLOC_ALLOCSZ_MAX 65536
#define RDB_BIGALLOC_ALLOCSZ_MIN 256
#define RDB_BIGALLOC_BLKCNT_MAX 8

/** Readjust thresholds every <n> requests. <n> is defined here */
#define RDB_BIGALLOC_RECHECK_RATE 15

/**
 * Dumps a textual representation of the specified allocator to a FILE
 * @param alloc
 * @param fp
 */
void rdb_bigalloc_dump(rdb_BIGALLOC *alloc, FILE *fp);

#ifdef __cplusplus
}
#endif

#endif
