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

/**
 * This is the default allocator which uses chunks via malloc/free
 */

#include <stdlib.h>
#include <stddef.h>
#include <string.h>
#include "rope.h"
#include "list.h"

static rdb_ROPESEG *seg_alloc(rdb_pALLOCATOR alloc, unsigned size)
{
    unsigned newsize;
    rdb_ROPESEG *ret;
    newsize = size + sizeof(*ret);
    ret = malloc(newsize);
    memset(ret, 0, sizeof(*ret));
    ret->nalloc = size;
    ret->root = ((char *)ret) + sizeof(*ret);
    ret->allocator = alloc;
    ret->allocid = RDB_ALLOCATOR_LIBCALLOC;

    return ret;
}

static rdb_ROPESEG *seg_realloc(rdb_pALLOCATOR alloc, rdb_ROPESEG *seg, unsigned size)
{
    unsigned newsize = size + sizeof(*seg);
    seg = realloc(seg, newsize);
    seg->root = ((char *)seg) + sizeof(*seg);

    (void)alloc;
    return seg;
}

static void seg_free(rdb_pALLOCATOR alloc, rdb_ROPESEG *seg)
{
    (void)alloc;
    free(seg);
}

static void buf_reserve(rdb_pALLOCATOR alloc, rdb_ROPEBUF *buf, unsigned cap)
{
    rdb_ROPESEG *newseg, *lastseg;
    unsigned to_alloc;
    lastseg = RDB_SEG_LAST(buf);
    if (lastseg && RDB_SEG_SPACE(lastseg) + buf->nused >= cap) {
        return;
    }

    to_alloc = cap;
    if (lastseg) {
        to_alloc -= lastseg->nalloc - lastseg->start;
    }
    newseg = alloc->s_alloc(alloc, to_alloc);
    lcb_list_append(&buf->segments, &newseg->llnode);
}

static void release_noop(rdb_pALLOCATOR alloc)
{
    (void)alloc;
}

static rdb_ALLOCATOR lcalloc = {buf_reserve, seg_alloc, seg_realloc, seg_free, release_noop};

rdb_ALLOCATOR *rdb_libcalloc_new(void)
{
    return &lcalloc;
}
