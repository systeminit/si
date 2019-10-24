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

#include <stdlib.h>
#include <stddef.h>
#include <string.h>
#include <assert.h>
#include <stdio.h>
#include "rope.h"

/**
 * This is a fixed size chunk allocator. It tries to allocate chunks of a
 * certain size. For constraints needing "Special" block sizes, it simply
 * defers to malloc/free (in truth, it could utilize the 'bigalloc'
 * allocator for malloc/frees, but that might not be necessary).
 */

typedef struct {
    rdb_ALLOCATOR base;
    lcb_clist_t chunks;
    unsigned refcount;
    unsigned chunksize;
    unsigned max_chunks;
} my_CHUNKALLOC;

static void alloc_decref(rdb_ALLOCATOR *abase)
{
    lcb_list_t *llcur, *llnext;
    my_CHUNKALLOC *alloc = (my_CHUNKALLOC *)abase;
    if (--alloc->refcount) {
        return;
    }

    LCB_LIST_SAFE_FOR(llcur, llnext, (lcb_list_t *)&alloc->chunks)
    {
        rdb_ROPESEG *seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        free(seg->root);
        free(seg);
    }
    free(alloc);
}

static void release_chunk(rdb_ALLOCATOR *abase, rdb_ROPESEG *seg)
{
    my_CHUNKALLOC *alloc = (my_CHUNKALLOC *)abase;
    if (seg->nalloc != alloc->chunksize || LCB_CLIST_SIZE(&alloc->chunks) > alloc->max_chunks) {
        free(seg->root);
        free(seg);
    } else {
        lcb_clist_prepend(&alloc->chunks, &seg->llnode);
    }
}

static rdb_ROPESEG *standalone_alloc(rdb_ALLOCATOR *abase, unsigned size)
{
    my_CHUNKALLOC *alloc = (my_CHUNKALLOC *)abase;
    rdb_ROPESEG *seg;

    seg = calloc(1, sizeof(*seg));
    seg->root = malloc(size);
    seg->nalloc = size;
    seg->shflags = RDB_ROPESEG_F_LIB;
    seg->allocid = RDB_ALLOCATOR_CHUNKED;
    seg->allocator = abase;
    alloc->refcount++;
    return seg;
}

static rdb_ROPESEG *chunked_alloc(my_CHUNKALLOC *alloc)
{
    rdb_ROPESEG *chunk = NULL;
    lcb_list_t *llcur, *llnext;

    LCB_LIST_SAFE_FOR(llcur, llnext, (lcb_list_t *)&alloc->chunks)
    {
        rdb_ROPESEG *cur = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);

        lcb_list_delete(&cur->llnode);
        if (cur->nalloc != alloc->chunksize) {
            release_chunk(&alloc->base, cur);
        } else {
            chunk = cur;
            break;
        }
    }

    if (chunk) {
        alloc->refcount++;
    } else {
        chunk = standalone_alloc(&alloc->base, alloc->chunksize);
    }

    chunk->start = 0;
    chunk->nused = 0;
    chunk->shflags = RDB_ROPESEG_F_LIB;
    return chunk;
}

static void buf_reserve(rdb_ALLOCATOR *abase, rdb_ROPEBUF *buf, unsigned n)
{
    my_CHUNKALLOC *alloc = (my_CHUNKALLOC *)abase;
    rdb_ROPESEG *lastseg = RDB_SEG_LAST(buf);
    unsigned allocated = 0;

    if (lastseg) {
        if (buf->nused + RDB_SEG_SPACE(lastseg) >= n) {
            return;
        }

        n -= (buf->nused + RDB_SEG_SPACE(lastseg));
    }

    while (allocated < n) {
        rdb_ROPESEG *seg = chunked_alloc(alloc);
        lcb_list_append(&buf->segments, &seg->llnode);
        allocated += seg->nalloc;
    }
}

static rdb_ROPESEG *seg_realloc(rdb_ALLOCATOR *abase, rdb_ROPESEG *seg, unsigned n)
{
    seg->nalloc = n;
    seg->root = realloc(seg->root, n);

    (void)abase;
    return seg;
}

static void seg_release(rdb_ALLOCATOR *abase, rdb_ROPESEG *seg)
{
    release_chunk(abase, seg);
    alloc_decref(abase);
}

LCB_INTERNAL_API
rdb_ALLOCATOR *rdb_chunkalloc_new(unsigned chunksize)
{
    rdb_ALLOCATOR *ret;
    my_CHUNKALLOC *alloc = calloc(1, sizeof(*alloc));
    alloc->refcount = 1;
    alloc->chunksize = chunksize;
    alloc->max_chunks = 512;

    lcb_clist_init(&alloc->chunks);
    ret = &alloc->base;

    ret->a_release = alloc_decref;
    ret->r_reserve = buf_reserve;
    ret->s_alloc = standalone_alloc;
    ret->s_realloc = seg_realloc;
    ret->s_release = seg_release;
    return ret;
}
