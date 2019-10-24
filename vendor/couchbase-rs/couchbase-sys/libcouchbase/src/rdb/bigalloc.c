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
#include "rope.h"
#include "bigalloc.h"

#define MAXIMUM(a, b) (a) > (b) ? a : b

static void alloc_decref(rdb_ALLOCATOR *abase)
{
    lcb_list_t *llcur, *llnext;
    rdb_BIGALLOC *alloc = (rdb_BIGALLOC *)abase;
    if (--alloc->refcount) {
        return;
    }

    LCB_LIST_SAFE_FOR(llcur, llnext, (lcb_list_t *)&alloc->bufs)
    {
        rdb_ROPESEG *seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        lcb_clist_delete(&alloc->bufs, &seg->llnode);
        free(seg->root);
        free(seg);
    }
    free(alloc);
}

static void recheck_thresholds(rdb_BIGALLOC *alloc)
{
    if (++alloc->n_requests % RDB_BIGALLOC_RECHECK_RATE) {
        return;
    }

    alloc->total_requests += alloc->n_requests;
    alloc->total_toobig += alloc->n_toobig;
    alloc->total_toosmall += alloc->n_toosmall;

    if (alloc->n_toobig == alloc->n_toosmall) {
        /* all is ok */

    } else if (alloc->n_toobig > alloc->n_toosmall) {
        /* seems we need to allocate bigger chunks? */
        if (alloc->n_toobig * 2 > alloc->n_toosmall) {
            alloc->min_blk_alloc *= 2;
            alloc->max_blk_alloc *= 2;
        }
    } else if (alloc->n_toosmall > alloc->n_toobig) {
        if (alloc->n_toosmall * 2 > alloc->n_toobig) {
            alloc->min_blk_alloc /= 2;
            alloc->max_blk_alloc /= 2;
        }
    }

    alloc->n_requests = 0;
    alloc->n_toobig = 0;
    alloc->n_toosmall = 0;
}

static rdb_ROPESEG *seg_alloc(rdb_ALLOCATOR *abase, unsigned size)
{
    lcb_list_t *llcur;
    rdb_ROPESEG *newseg = NULL;
    rdb_BIGALLOC *alloc = (rdb_BIGALLOC *)abase;

    recheck_thresholds(alloc);
    /**
     * If the allocation reaches a certain threshold (for example, for a really
     * huge packet), then don't bother caching it.
     */
    if (size > alloc->max_blk_alloc) {
        alloc->n_toobig++;
        alloc->total_malloc++;
        newseg = calloc(1, sizeof(*newseg));
        newseg->root = malloc(size);
        newseg->nalloc = size;
        goto GT_RETNEW;
    } else if (size < alloc->min_blk_alloc) {
        alloc->n_toosmall++;
    }

    LCB_LIST_FOR(llcur, (lcb_list_t *)&alloc->bufs)
    {
        rdb_ROPESEG *cur = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        if (cur->nalloc < size) {
            continue;
        }

        newseg = cur;
        lcb_clist_delete(&alloc->bufs, llcur);
        break;
    }

    if (!newseg) {
        unsigned newsize = alloc->min_blk_alloc;
        if (LCB_CLIST_SIZE(&alloc->bufs) >= alloc->max_blk_count) {
            lcb_list_t *llold = lcb_clist_pop(&alloc->bufs);
            newseg = LCB_LIST_ITEM(llold, rdb_ROPESEG, llnode);
            free(newseg->root);
        } else {
            newseg = calloc(1, sizeof(*newseg));
            alloc->total_malloc++;
        }

        while (newsize < size) {
            newsize = (unsigned)((double)newsize * 1.5);
        }

        newseg->root = malloc(newsize);
        newseg->nalloc = newsize;
    }

GT_RETNEW:
    newseg->shflags = RDB_ROPESEG_F_LIB;
    newseg->allocator = abase;
    newseg->allocid = RDB_ALLOCATOR_CHUNKED;
    newseg->start = 0;
    newseg->nused = 0;
    alloc->refcount++;
    return newseg;
}

static void buf_reserve(rdb_pALLOCATOR abase, rdb_ROPEBUF *buf, unsigned size)
{
    rdb_BIGALLOC *alloc = (rdb_BIGALLOC *)abase;
    rdb_ROPESEG *newseg, *lastseg;

    lastseg = RDB_SEG_LAST(buf);
    if (lastseg && RDB_SEG_SPACE(lastseg) + buf->nused >= size) {
        return;
    }

    newseg = seg_alloc(&alloc->base, size);
    lcb_list_append(&buf->segments, &newseg->llnode);
}

static rdb_ROPESEG *seg_realloc(rdb_ALLOCATOR *abase, rdb_ROPESEG *seg, unsigned size)
{
    rdb_BIGALLOC *alloc = (rdb_BIGALLOC *)abase;

    if (size < alloc->n_toosmall) {
        alloc->n_toosmall++;
    } else if (size > alloc->n_toobig) {
        alloc->n_toobig++;
    }

    seg->root = realloc(seg->root, size);
    seg->nalloc = size;
    alloc->total_malloc++;
    recheck_thresholds((rdb_BIGALLOC *)abase);
    return seg;
}

static void seg_release(rdb_ALLOCATOR *abase, rdb_ROPESEG *seg)
{
    rdb_BIGALLOC *alloc = (rdb_BIGALLOC *)abase;
    if (LCB_CLIST_SIZE(&alloc->bufs) >= alloc->max_blk_count || seg->nalloc > alloc->max_blk_alloc ||
        seg->nalloc < alloc->min_blk_alloc) {
        free(seg->root);
        free(seg);
    } else {
        lcb_clist_prepend(&alloc->bufs, &seg->llnode);
    }
    alloc_decref(abase);
}

static void dump_wrap(rdb_pALLOCATOR alloc, FILE *fp)
{
    rdb_bigalloc_dump((rdb_BIGALLOC *)alloc, fp);
}

rdb_ALLOCATOR *rdb_bigalloc_new(void)
{
    rdb_ALLOCATOR *abase;
    rdb_BIGALLOC *alloc = calloc(1, sizeof(*alloc));
    lcb_clist_init(&alloc->bufs);
    alloc->max_blk_alloc = RDB_BIGALLOC_ALLOCSZ_MAX;
    alloc->min_blk_alloc = RDB_BIGALLOC_ALLOCSZ_MIN;
    alloc->max_blk_count = RDB_BIGALLOC_BLKCNT_MAX;
    alloc->refcount = 1;

    abase = &alloc->base;
    abase->r_reserve = buf_reserve;
    abase->s_release = seg_release;
    abase->s_alloc = seg_alloc;
    abase->s_realloc = seg_realloc;
    abase->a_release = alloc_decref;
    abase->dump = dump_wrap;
    return &alloc->base;
}

void rdb_bigalloc_dump(rdb_BIGALLOC *alloc, FILE *fp)
{
    static const char *indent = "  ";
    fprintf(fp, "BIGALLOC @%p\n", (void *)alloc);
    fprintf(fp, "%sPooled Blocks: %lu\n", indent, (unsigned long int)LCB_CLIST_SIZE(&alloc->bufs));
    fprintf(fp, "%sMinAlloc: %u\n", indent, alloc->min_blk_alloc);
    fprintf(fp, "%sMaxAlloc: %u\n", indent, alloc->max_blk_alloc);
    fprintf(fp, "%sMaxBlocks: %u\n", indent, alloc->max_blk_count);

    fprintf(fp, "%sTotalMalloc: %u\n", indent, alloc->total_malloc);
    fprintf(fp, "%sTotalRequests: %u\n", indent, alloc->total_requests);
    fprintf(fp, "%sTotalToobig: %u\n", indent, alloc->total_toobig);
    fprintf(fp, "%sTotalToosmall: %u\n", indent, alloc->total_toosmall);
}
