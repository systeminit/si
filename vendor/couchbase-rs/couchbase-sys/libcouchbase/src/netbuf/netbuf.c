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

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
/* for ULONG */
#include <windows.h>
#endif

#include <stdio.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "netbuf.h"
#include "sllist-inl.h"

#include <libcouchbase/assert.h>

/******************************************************************************
 ******************************************************************************
 ** Handy Macros                                                             **
 ******************************************************************************
 ******************************************************************************/
#define MINIMUM(a, b) a < b ? a : b
#define MAXIMUM(a, b) a > b ? a : b

#define BLOCK_IS_EMPTY(block) ((block)->start == (block)->cursor)

#define FIRST_BLOCK(pool) (SLLIST_ITEM(SLLIST_FIRST(&(pool)->active), nb_MBLOCK, slnode))

#define LAST_BLOCK(mgr) (SLLIST_ITEM((mgr)->active_blocks.last, nb_BLOCKHDR, slnode))

#define NEXT_BLOCK(block) (SLLIST_ITEM((block)->slnode.next, nb_BLOCKHDR, slnode))

#define BLOCK_HAS_DEALLOCS(block) ((block)->deallocs && SLLIST_IS_EMPTY(&(block)->deallocs->pending))

/** Static forward decls */
static void mblock_release_data(nb_MBPOOL *, nb_MBLOCK *, nb_SIZE, nb_SIZE);
static void mblock_release_ptr(nb_MBPOOL *, char *, nb_SIZE);
static void mblock_init(nb_MBPOOL *);
static void mblock_cleanup(nb_MBPOOL *);
static void mblock_wipe_block(nb_MBLOCK *block);

/******************************************************************************
 ******************************************************************************
 ** Allocation/Reservation                                                   **
 ******************************************************************************
 ******************************************************************************/

/**
 * Determines whether the block is allocated as a standalone block, or if it's
 * part of a larger allocation
 */
static int mblock_is_standalone(nb_MBLOCK *block)
{
    return block->parent == NULL;
}

/**
 * Allocates a new block with at least the given capacity and places it
 * inside the active list.
 */
static nb_MBLOCK *alloc_new_block(nb_MBPOOL *pool, nb_SIZE capacity)
{
    unsigned int ii;
    nb_MBLOCK *ret = NULL;

    for (ii = 0; ii < pool->ncacheblocks; ii++) {
        if (!pool->cacheblocks[ii].nalloc) {
            ret = pool->cacheblocks + ii;
            break;
        }
    }

    if (!ret) {
        ret = calloc(1, sizeof(*ret));
    }

    if (!ret) {
        return NULL;
    }

    ret->nalloc = pool->basealloc;

    while (ret->nalloc < capacity) {
        ret->nalloc *= 2;
    }

    ret->wrap = 0;
    ret->cursor = 0;
    ret->root = malloc(ret->nalloc);

    if (!ret->root) {
        if (mblock_is_standalone(ret)) {
            free(ret);
        }
        return NULL;
    }

    return ret;
}

/**
 * Finds an available block within the available list. The block will have
 * room for at least capacity bytes.
 */
static nb_MBLOCK *find_free_block(nb_MBPOOL *pool, nb_SIZE capacity)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(&pool->avail, &iter)
    {
        nb_MBLOCK *cur = SLLIST_ITEM(iter.cur, nb_MBLOCK, slnode);
        if (cur->nalloc >= capacity) {
            sllist_iter_remove(&pool->avail, &iter);
            pool->curblocks--;
            return cur;
        }
    }

    return NULL;
}

/**
 * Find a new block for the given span and initialize it for a reserved size
 * correlating to the span.
 * The block may either be popped from the available section or allocated
 * as a standalone depending on current constraints.
 */
static int reserve_empty_block(nb_MBPOOL *pool, nb_SPAN *span)
{
    nb_MBLOCK *block;

    if ((block = find_free_block(pool, span->size)) == NULL) {
        block = alloc_new_block(pool, span->size);
    }

    if (!block) {
        return -1;
    }

    span->parent = block;
    span->offset = 0;
    block->start = 0;
    block->wrap = span->size;
    block->cursor = span->size;

    block->deallocs = NULL;

    sllist_append(&pool->active, &block->slnode);
    return 0;
}

/**
 * Attempt to reserve space from the currently active block for the given
 * span.
 * @return 0 if the active block had enough space and the span was initialized
 * and nonzero otherwise.
 */
static int reserve_active_block(nb_MBLOCK *block, nb_SPAN *span)
{
    if (BLOCK_HAS_DEALLOCS(block)) {
        return -1;
    }

    if (block->cursor > block->start) {
        if (block->nalloc - block->cursor >= span->size) {
            span->offset = block->cursor;
            block->cursor += span->size;
            block->wrap = block->cursor;
            return 0;

        } else if (block->start >= span->size) {
            /** Wrap around the wrap */
            span->offset = 0;
            block->cursor = span->size;
            return 0;
        } else {
            return -1;
        }

    } else {
        /* Already wrapped */
        if (block->start - block->cursor >= span->size) {
            span->offset = block->cursor;
            block->cursor += span->size;
            return 0;
        } else {
            return -1;
        }
    }
}

static int mblock_reserve_data(nb_MBPOOL *pool, nb_SPAN *span)
{
    nb_MBLOCK *block;
    int rv;

#ifdef NETBUF_LIBC_PROXY
    block = malloc(sizeof(*block) + span->size);
    block->root = ((char *)block) + sizeof(*block);
    span->parent = block;
    span->offset = 0;
    return 0;
#endif

    if (SLLIST_IS_EMPTY(&pool->active)) {
        return reserve_empty_block(pool, span);

    } else {
        block = SLLIST_ITEM(pool->active.last, nb_MBLOCK, slnode);
        rv = reserve_active_block(block, span);

        if (rv != 0) {
            return reserve_empty_block(pool, span);
        }

        span->parent = block;
        return rv;
    }
}

/******************************************************************************
 ******************************************************************************
 ** Out-Of-Order Deallocation Functions                                      **
 ******************************************************************************
 ******************************************************************************/
static void ooo_queue_dealoc(nb_MGR *mgr, nb_MBLOCK *block, nb_SPAN *span)
{
    nb_QDEALLOC *qd;
    nb_DEALLOC_QUEUE *queue;
    nb_SPAN qespan;

    if (!block->deallocs) {
        queue = calloc(1, sizeof(*queue));
        queue->qpool.basealloc = sizeof(*qd) * mgr->settings.dea_basealloc;
        queue->qpool.ncacheblocks = mgr->settings.dea_cacheblocks;
        queue->qpool.mgr = mgr;
        mblock_init(&queue->qpool);
        block->deallocs = queue;
    }

    queue = block->deallocs;

    if (SLLIST_IS_EMPTY(&queue->pending)) {
        queue->min_offset = span->offset;
    }

    qespan.size = sizeof(*qd);
    mblock_reserve_data(&queue->qpool, &qespan);

    qd = (nb_QDEALLOC *)(void *)SPAN_MBUFFER_NC(&qespan);
    qd->offset = span->offset;
    qd->size = span->size;
    if (queue->min_offset > qd->offset) {
        queue->min_offset = qd->offset;
    }
    sllist_append(&queue->pending, &qd->slnode);
}

static INLINE void maybe_unwrap_block(nb_MBLOCK *block)
{
    if (!BLOCK_IS_EMPTY(block) && block->start == block->wrap) {
        block->wrap = block->cursor;
        block->start = 0;
    }
}

static void ooo_apply_dealloc(nb_MBLOCK *block)
{
    nb_SIZE min_next = -1;
    sllist_iterator iter;
    nb_DEALLOC_QUEUE *queue = block->deallocs;

    SLLIST_ITERFOR(&queue->pending, &iter)
    {
        nb_QDEALLOC *cur = SLLIST_ITEM(iter.cur, nb_QDEALLOC, slnode);
        if (cur->offset == block->start) {
            block->start += cur->size;
            maybe_unwrap_block(block);

            sllist_iter_remove(&block->deallocs->pending, &iter);
            mblock_release_ptr(&queue->qpool, (char *)cur, sizeof(*cur));
        } else if (cur->offset < min_next) {
            min_next = cur->offset;
        }
    }
    queue->min_offset = min_next;
}

static INLINE void mblock_release_data(nb_MBPOOL *pool, nb_MBLOCK *block, nb_SIZE size, nb_SIZE offset)
{
    if (offset == block->start) {
        /** Removing from the beginning */
        block->start += size;

        if (block->deallocs && block->deallocs->min_offset == block->start) {
            ooo_apply_dealloc(block);
        }

        maybe_unwrap_block(block);

    } else if (offset + size == block->cursor) {
        /** Removing from the end */
        if (block->cursor == block->wrap) {
            /** Single region, no wrap */
            block->cursor -= size;
            block->wrap -= size;

        } else {
            block->cursor -= size;
            if (!block->cursor) {
                /** End has reached around */
                block->cursor = block->wrap;
            }
        }

    } else {
        nb_SPAN span;

        span.parent = block;
        span.offset = offset;
        span.size = size;
        ooo_queue_dealoc(pool->mgr, block, &span);
        return;
    }

    if (!BLOCK_IS_EMPTY(block)) {
        return;
    }

    {
        sllist_iterator iter;
        SLLIST_ITERFOR(&pool->active, &iter)
        {
            if (&block->slnode == iter.cur) {
                sllist_iter_remove(&pool->active, &iter);
                break;
            }
        }
    }

    if (pool->curblocks < pool->maxblocks) {
        sllist_append(&pool->avail, &block->slnode);
        pool->curblocks++;
    } else {
        mblock_wipe_block(block);
    }
}

static void mblock_release_ptr(nb_MBPOOL *pool, char *ptr, nb_SIZE size)
{
    nb_MBLOCK *block;
    nb_SIZE offset;
    sllist_node *ll;

#ifdef NETBUF_LIBC_PROXY
    block = (nb_MBLOCK *)(ptr - sizeof(*block));
    free(block);
    return;
#endif

    SLLIST_FOREACH(&pool->active, ll)
    {
        block = SLLIST_ITEM(ll, nb_MBLOCK, slnode);
        if (block->root > ptr) {
            continue;
        }
        if (block->root + block->nalloc <= ptr) {
            continue;
        }
        offset = ptr - block->root;
        mblock_release_data(pool, block, size, offset);
        return;
    }

    fprintf(stderr, "NETBUF: Requested to release pointer %p which was not allocated\n", (void *)ptr);
    lcb_assert(0);
}

static int mblock_get_next_size(const nb_MBPOOL *pool, int allow_wrap)
{
    nb_MBLOCK *block;
    if (SLLIST_IS_EMPTY(&pool->avail)) {
        return 0;
    }

    block = FIRST_BLOCK(pool);

    if (BLOCK_HAS_DEALLOCS(block)) {
        return 0;
    }

    if (!block->start) {
        /** Plain 'ole buffer */
        return block->nalloc - block->cursor;
    }

    if (block->cursor != block->wrap) {
        /** Already in second region */
        return block->start - block->cursor;
    }

    if (allow_wrap) {
        return MINIMUM(block->nalloc - block->wrap, block->start);
    }

    return block->nalloc - block->wrap;
}

static void mblock_wipe_block(nb_MBLOCK *block)
{
    if (block->root) {
        free(block->root);
    }
    if (block->deallocs) {
        sllist_iterator dea_iter;
        nb_DEALLOC_QUEUE *queue = block->deallocs;

        SLLIST_ITERFOR(&queue->pending, &dea_iter)
        {
            nb_QDEALLOC *qd = SLLIST_ITEM(dea_iter.cur, nb_QDEALLOC, slnode);
            sllist_iter_remove(&queue->pending, &dea_iter);
            mblock_release_ptr(&queue->qpool, (char *)qd, sizeof(*qd));
        }

        mblock_cleanup(&queue->qpool);
        free(queue);
        block->deallocs = NULL;
    }

    if (mblock_is_standalone(block)) {
        free(block);
    }
}

static void free_blocklist(nb_MBPOOL *pool, sllist_root *list)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(list, &iter)
    {
        nb_MBLOCK *block = SLLIST_ITEM(iter.cur, nb_MBLOCK, slnode);
        sllist_iter_remove(list, &iter);
        mblock_wipe_block(block);
    }
    (void)pool;
}

static void mblock_cleanup(nb_MBPOOL *pool)
{
    free_blocklist(pool, &pool->active);
    free_blocklist(pool, &pool->avail);
    free(pool->cacheblocks);
}

static void mblock_init(nb_MBPOOL *pool)
{
    unsigned int ii;
    pool->cacheblocks = calloc(pool->ncacheblocks, sizeof(*pool->cacheblocks));
    for (ii = 0; ii < pool->ncacheblocks; ii++) {
        pool->cacheblocks[ii].parent = pool;
    }
    if (pool->ncacheblocks) {
        pool->maxblocks = pool->ncacheblocks * 2;
    }
}

int netbuf_mblock_reserve(nb_MGR *mgr, nb_SPAN *span)
{
    return mblock_reserve_data(&mgr->datapool, span);
}

/******************************************************************************
 ******************************************************************************
 ** Informational Routines                                                   **
 ******************************************************************************
 ******************************************************************************/
nb_SIZE netbuf_mblock_get_next_size(const nb_MGR *mgr, int allow_wrap)
{
    return mblock_get_next_size(&mgr->datapool, allow_wrap);
}

unsigned int netbuf_get_niov(nb_MGR *mgr)
{
    sllist_node *ll;
    unsigned int ret = 0;
    SLLIST_FOREACH(&mgr->sendq.pending, ll)
    {
        ret++;
    }

    return ret;
}

/******************************************************************************
 ******************************************************************************
 ** Flush Routines                                                           **
 ******************************************************************************
 ******************************************************************************/
static nb_SNDQELEM *get_sendqe(nb_SENDQ *sq, const nb_IOV *bufinfo)
{
    nb_SNDQELEM *sndqe;
    nb_SPAN span;
    span.size = sizeof(*sndqe);
    mblock_reserve_data(&sq->elempool, &span);
    sndqe = (nb_SNDQELEM *)(void *)SPAN_MBUFFER_NC(&span);

    sndqe->base = bufinfo->iov_base;
    sndqe->len = bufinfo->iov_len;
    return sndqe;
}

void netbuf_enqueue(nb_MGR *mgr, const nb_IOV *bufinfo, const void *parent)
{
    nb_SENDQ *q = &mgr->sendq;
    nb_SNDQELEM *win;

    if (SLLIST_IS_EMPTY(&q->pending)) {
        win = get_sendqe(q, bufinfo);
        sllist_append(&q->pending, &win->slnode);

    } else {
        win = SLLIST_ITEM(q->pending.last, nb_SNDQELEM, slnode);
        if (win->base + win->len == bufinfo->iov_base) {
            win->len += bufinfo->iov_len;

        } else {
            win = get_sendqe(q, bufinfo);
            sllist_append(&q->pending, &win->slnode);
        }
    }
    win->parent = parent;
}

void netbuf_enqueue_span(nb_MGR *mgr, nb_SPAN *span, const void *parent)
{
    nb_IOV spinfo;
    spinfo.iov_base = SPAN_BUFFER(span);
    spinfo.iov_len = span->size;
    netbuf_enqueue(mgr, &spinfo, parent);
}

nb_SIZE netbuf_start_flush(nb_MGR *mgr, nb_IOV *iovs, int niov, int *nused)
{
    nb_SIZE ret = 0;
    nb_IOV *iov_end = iovs + niov, *iov_start = iovs;
    nb_IOV *iov = iovs;
    sllist_node *ll;
    nb_SENDQ *sq = &mgr->sendq;
    nb_SNDQELEM *win = NULL;

    if (sq->last_requested) {
        if (sq->last_offset != sq->last_requested->len) {
            win = sq->last_requested;
            lcb_assert(win->len > sq->last_offset);

            iov->iov_len = win->len - sq->last_offset;
            iov->iov_base = win->base + sq->last_offset;
            ret += iov->iov_len;
            iov++;
        }

        ll = sq->last_requested->slnode.next;

    } else {
        ll = SLLIST_FIRST(&sq->pending);
    }

    while (ll && iov != iov_end) {
        win = SLLIST_ITEM(ll, nb_SNDQELEM, slnode);
        iov->iov_len = win->len;
        iov->iov_base = win->base;

        ret += iov->iov_len;
        iov++;
        ll = ll->next;
    }

    if (win) {
        sq->last_requested = win;
        sq->last_offset = win->len;
    }
    if (ret && nused) {
        *nused = iov - iov_start;
    }

    return ret;
}

void netbuf_end_flush(nb_MGR *mgr, unsigned int nflushed)
{
    nb_SENDQ *q = &mgr->sendq;
    sllist_iterator iter;
    SLLIST_ITERFOR(&q->pending, &iter)
    {
        nb_SNDQELEM *win = SLLIST_ITEM(iter.cur, nb_SNDQELEM, slnode);
        nb_SIZE to_chop = MINIMUM(win->len, nflushed);

        win->len -= to_chop;
        nflushed -= to_chop;

        if (!win->len) {
            sllist_iter_remove(&q->pending, &iter);
            mblock_release_ptr(&mgr->sendq.elempool, (char *)win, sizeof(*win));
            if (win == q->last_requested) {
                q->last_requested = NULL;
                q->last_offset = 0;
            }
        } else {
            win->base += to_chop;
            if (win == q->last_requested) {
                q->last_offset -= to_chop;
            }
        }

        if (!nflushed) {
            break;
        }
    }
    lcb_assert(!nflushed);
}

void netbuf_pdu_enqueue(nb_MGR *mgr, void *pdu, nb_SIZE lloff)
{
    nb_SENDQ *q = &mgr->sendq;
    sllist_append(&q->pdus, (sllist_node *)(void *)((char *)pdu + lloff));
}

void netbuf_end_flush2(nb_MGR *mgr, unsigned int nflushed, nb_getsize_fn callback, nb_SIZE lloff, void *arg)
{
    sllist_iterator iter;
    nb_SENDQ *q = &mgr->sendq;
    netbuf_end_flush(mgr, nflushed);

    /** Add to the nflushed overflow from last call */
    nflushed += q->pdu_offset;
    SLLIST_ITERFOR(&q->pdus, &iter)
    {
        nb_SIZE cursize;
        char *ptmp = (char *)iter.cur;
        cursize = callback(ptmp - lloff, nflushed, arg);

        if (cursize > nflushed) {
            break;
        }

        nflushed -= cursize;
        sllist_iter_remove(&q->pdus, &iter);

        if (!nflushed) {
            break;
        }
    }

    /** Store the remainder of data that wasn't processed for next call */
    q->pdu_offset = nflushed;
}

/******************************************************************************
 ******************************************************************************
 ** Release                                                                  **
 ******************************************************************************
 ******************************************************************************/
void netbuf_mblock_release(nb_MGR *mgr, nb_SPAN *span)
{
#ifdef NETBUF_LIBC_PROXY
    free(span->parent);
    (void)mgr;
#else
    mblock_release_data(&mgr->datapool, span->parent, span->size, span->offset);
#endif
}

/******************************************************************************
 ******************************************************************************
 ** Init/Cleanup                                                             **
 ******************************************************************************
 ******************************************************************************/
void netbuf_default_settings(nb_SETTINGS *settings)
{
    settings->data_basealloc = NB_DATA_BASEALLOC;
    settings->data_cacheblocks = NB_DATA_CACHEBLOCKS;
    settings->dea_basealloc = NB_MBDEALLOC_BASEALLOC;
    settings->dea_cacheblocks = NB_MBDEALLOC_CACHEBLOCKS;
    settings->sndq_basealloc = NB_SNDQ_BASEALLOC;
    settings->sndq_cacheblocks = NB_SNDQ_CACHEBLOCKS;
}

void netbuf_init(nb_MGR *mgr, const nb_SETTINGS *user_settings)
{
    nb_MBPOOL *sqpool = &mgr->sendq.elempool;
    nb_MBPOOL *bufpool = &mgr->datapool;

    memset(mgr, 0, sizeof(*mgr));

    if (user_settings) {
        mgr->settings = *user_settings;
    } else {
        netbuf_default_settings(&mgr->settings);
    }

    /** Set our defaults */
    sqpool->basealloc = sizeof(nb_SNDQELEM) * mgr->settings.sndq_basealloc;
    sqpool->ncacheblocks = mgr->settings.sndq_cacheblocks;
    sqpool->mgr = mgr;
    mblock_init(sqpool);

    bufpool->basealloc = mgr->settings.data_basealloc;
    bufpool->ncacheblocks = mgr->settings.data_cacheblocks;
    bufpool->mgr = mgr;
    mblock_init(bufpool);
}

void netbuf_cleanup(nb_MGR *mgr)
{
    sllist_iterator iter;

    SLLIST_ITERFOR(&mgr->sendq.pending, &iter)
    {
        nb_SNDQELEM *e = SLLIST_ITEM(iter.cur, nb_SNDQELEM, slnode);
        sllist_iter_remove(&mgr->sendq.pending, &iter);
        mblock_release_ptr(&mgr->sendq.elempool, (char *)e, sizeof(*e));
    }

    mblock_cleanup(&mgr->sendq.elempool);
    mblock_cleanup(&mgr->datapool);
}

/******************************************************************************
 ******************************************************************************
 ** Block Dumping                                                            **
 ******************************************************************************
 ******************************************************************************/

static void dump_managed_block(nb_MBLOCK *block, FILE *fp)
{
    const char *indent = "  ";
    fprintf(fp, "%sBLOCK(MANAGED)=%p; BUF=%p, %uB\n", indent, (void *)block, (void *)block->root, block->nalloc);
    indent = "     ";

    fprintf(fp, "%sUSAGE:\n", indent);
    fprintf(fp, "%s", indent);
    if (BLOCK_IS_EMPTY(block)) {
        fprintf(fp, "EMPTY\n");
        return;
    }

    printf("[");

    if (block->cursor == block->wrap) {
        if (block->start) {
            fprintf(fp, "ooo{S:%u}xxx", block->start);
        } else {
            fprintf(fp, "{S:0}xxxxxx");
        }

        if (block->nalloc > block->cursor) {
            fprintf(fp, "{CW:%u}ooo{A:%u}", block->cursor, block->nalloc);
        } else {
            fprintf(fp, "xxx{CWA:%u)}", block->cursor);
        }
    } else {
        fprintf(fp, "xxx{C:%u}ooo{S:%u}xxx", block->cursor, block->start);
        if (block->wrap != block->nalloc) {
            fprintf(fp, "{W:%u}ooo{A:%u}", block->wrap, block->nalloc);
        } else {
            fprintf(fp, "xxx{WA:%u}", block->wrap);
        }
    }
    fprintf(fp, "]\n");
}

static void dump_sendq(nb_SENDQ *q, FILE *fp)
{
    const char *indent = "  ";
    sllist_node *ll;
    fprintf(fp, "Send Queue\n");
    SLLIST_FOREACH(&q->pending, ll)
    {
        nb_SNDQELEM *e = SLLIST_ITEM(ll, nb_SNDQELEM, slnode);
        fprintf(fp, "%s[Base=%p, Len=%u]\n", indent, (void *)e->base, e->len);
        if (q->last_requested == e) {
            fprintf(fp, "%s<Current Flush Limit @%u^^^>\n", indent, q->last_offset);
        }
    }
}

void netbuf_dump_status(nb_MGR *mgr, FILE *fp)
{
    sllist_node *ll;
    fprintf(fp, "Status for MGR=%p\n", (void *)mgr);
    fprintf(fp, "ACTIVE:\n");

    SLLIST_FOREACH(&mgr->datapool.active, ll)
    {
        nb_MBLOCK *block = SLLIST_ITEM(ll, nb_MBLOCK, slnode);
        dump_managed_block(block, fp);
    }
    fprintf(fp, "AVAILABLE:\n");
    SLLIST_FOREACH(&mgr->datapool.avail, ll)
    {
        nb_MBLOCK *block = SLLIST_ITEM(ll, nb_MBLOCK, slnode);
        const char *indent = "    ";
        fprintf(fp, "%sBLOCK(AVAIL)=%p; BUF=%p, %uB\n", indent, (void *)block, (void *)block->root, block->nalloc);
    }
    dump_sendq(&mgr->sendq, fp);
}

static int is_pool_clean(const nb_MBPOOL *pool, int is_dealloc)
{
    int ret = 1;
    sllist_node *ll;

    SLLIST_FOREACH(&pool->active, ll)
    {
        nb_MBLOCK *block = SLLIST_ITEM(ll, nb_MBLOCK, slnode);

        if (!BLOCK_IS_EMPTY(block)) {
            printf("MBLOCK %p: Cursor (%u) != Start (%u)\n", (void *)block, block->cursor, block->start);
            ret = 0;
        }

        if (block->deallocs) {
            nb_DEALLOC_QUEUE *dq = block->deallocs;
            if (!SLLIST_IS_EMPTY(&dq->pending)) {
                printf("MBLOCK %p: Dealloc queue still has items\n", (void *)block);
                ret = 0;
            }

            if (!is_dealloc) {
                if (!is_pool_clean(&block->deallocs->qpool, 1)) {
                    ret = 0;
                }
            }
        }
    }
    return ret;
}

int netbuf_is_clean(nb_MGR *mgr)
{
    int ret = 1;

    if (!is_pool_clean(&mgr->datapool, 0)) {
        ret = 0;
    }

    if (!SLLIST_IS_EMPTY(&mgr->sendq.pending)) {
        printf("SENDQ @%p: Still have pending flush items\n", (void *)mgr);
        ret = 0;
    }

    if (!SLLIST_IS_EMPTY(&mgr->sendq.pdus)) {
        printf("PDUQ @%p: Still have pending PDU items\n", (void *)mgr);
        ret = 0;
    }

    if (!is_pool_clean(&mgr->sendq.elempool, 0)) {
        printf("SENDQ/MBLOCK @%p: Still have unfreed members in send queue\n", (void *)mgr);
        ret = 0;
    }

    return ret;
}

int netbuf_has_flushdata(nb_MGR *mgr)
{
    if (!SLLIST_IS_EMPTY(&mgr->sendq.pending)) {
        return 1;
    }
    if (!SLLIST_IS_EMPTY(&mgr->sendq.pdus)) {
        return 1;
    }
    return 0;
}
