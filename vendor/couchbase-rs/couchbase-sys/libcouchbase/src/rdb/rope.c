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
#include <libcouchbase/assert.h>
#include <stdio.h>
#include "rope.h"

#define MINIMUM(a, b) (a) < (b) ? a : b

#define SEG_RELEASE(seg) (seg)->allocator->s_release((seg)->allocator, seg)
#define SEG_REALLOC(seg, n) (seg)->allocator->s_realloc((seg)->allocator, seg, n)
#define ROPE_SALLOC(rope, n) (rope)->allocator->s_alloc((rope)->allocator, n)
static void wipe_rope(rdb_ROPEBUF *rope);

unsigned rdb_rdstart(rdb_IOROPE *ior, nb_IOV *iov, unsigned niov)
{
    unsigned orig_niov = niov;
    unsigned cur_rdsize = 0;

    lcb_list_t *ll;
    rdb_ROPESEG *seg = RDB_SEG_LAST(&ior->recvd);

    if (seg && RDB_SEG_SPACE(seg)) {
        iov->iov_base = RDB_SEG_WBUF(seg);
        iov->iov_len = RDB_SEG_SPACE(seg);
        cur_rdsize += iov->iov_len;
        ++iov;
        --niov;
        if (cur_rdsize >= ior->rdsize) {
            return 1;
        }
    }

    if (!niov) {
        return orig_niov - niov;
    }

    ior->avail.allocator->r_reserve(ior->avail.allocator, &ior->avail, ior->rdsize - cur_rdsize);

    lcb_assert(!LCB_LIST_IS_EMPTY(&ior->avail.segments));

    LCB_LIST_FOR(ll, &ior->avail.segments)
    {
        rdb_ROPESEG *cur = LCB_LIST_ITEM(ll, rdb_ROPESEG, llnode);
        iov->iov_base = RDB_SEG_WBUF(cur);
        iov->iov_len = RDB_SEG_SPACE(cur);
        ++iov;

        if (!--niov) {
            break;
        }
    }
    return orig_niov - niov;
}

void rdb_rdend(rdb_IOROPE *ior, unsigned nr)
{
    unsigned to_chop;
    lcb_list_t *llcur, *llnext;

    /** Chop the first segment at the end, if there's space */
    rdb_ROPESEG *seg = RDB_SEG_LAST(&ior->recvd);
    if (seg && RDB_SEG_SPACE(seg)) {
        to_chop = MINIMUM(nr, RDB_SEG_SPACE(seg));
        seg->nused += to_chop;
        ior->recvd.nused += to_chop;

        if (!(nr -= to_chop)) {
            wipe_rope(&ior->avail);
            return;
        }
    }

    LCB_LIST_SAFE_FOR(llcur, llnext, &ior->avail.segments)
    {
        seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        to_chop = MINIMUM(nr, RDB_SEG_SPACE(seg));

        seg->nused += to_chop;
        ior->recvd.nused += seg->nused;

        lcb_list_delete(&seg->llnode);
        lcb_list_append(&ior->recvd.segments, &seg->llnode);
        if (!(nr -= to_chop)) {
            wipe_rope(&ior->avail);
            return;
        }
    }

    /** Reads didn't fit into any segment */
    fprintf(stderr, "RDB: Tried to consume more than available in the buffer (n=%u)\n", nr);
    lcb_assert(0);
}

static void seg_consumed(rdb_ROPEBUF *rope, rdb_ROPESEG *seg, unsigned nr)
{
    lcb_assert(nr <= seg->nused);
    seg->nused -= nr;
    seg->start += nr;
    rope->nused -= nr;

    if (!seg->nused) {
        lcb_list_delete(&seg->llnode);
        seg->shflags &= ~RDB_ROPESEG_F_LIB;
        if (rdb_seg_recyclable(seg)) {
            SEG_RELEASE(seg);
        }
    }
}

static void rope_consumed(rdb_ROPEBUF *rope, unsigned nr)
{
    lcb_list_t *llcur, *llnext;
    lcb_assert(nr <= rope->nused);

    LCB_LIST_SAFE_FOR(llcur, llnext, &rope->segments)
    {
        unsigned to_chop;
        rdb_ROPESEG *seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        to_chop = MINIMUM(nr, seg->nused);
        seg_consumed(rope, seg, to_chop);

        if (!(nr -= to_chop)) {
            break;
        }
    }
}

void rdb_consumed(rdb_IOROPE *ior, unsigned nr)
{
    rope_consumed(&ior->recvd, nr);
}

static void try_compact(rdb_ROPESEG *seg)
{
    /** Can't move stuff around.. */
    char *cp_end;
    if (!rdb_seg_recyclable(seg)) {
        return;
    }

    /**
     * Copy only if:
     * (1) Waste in the beginning is >= nalloc/2
     * (3) There is no overlap between the two (i.e. memcpy)
     */
    if (seg->start < seg->nalloc / 2) {
        return;
    }

    cp_end = seg->root + seg->nused;
    if (seg->root + seg->start < cp_end) {
        /** Overlap */
        return;
    }

    memcpy(seg->root, seg->root + seg->start, seg->nused);
    seg->start = 0;
}

static void rope_consolidate(rdb_ROPEBUF *rope, unsigned nr)
{
    rdb_ROPESEG *seg, *newseg;
    lcb_list_t *llcur, *llnext;

    seg = RDB_SEG_FIRST(rope);
    if (seg->nused + RDB_SEG_SPACE(seg) >= nr || nr < 2) {
        return;
    }

    try_compact(seg);
    lcb_list_delete(&seg->llnode);

    if (rdb_seg_recyclable(seg)) {
        unsigned to_alloc = nr + seg->start;
        newseg = SEG_REALLOC(seg, to_alloc);
        /* We re-add it back after traversal */
    } else {
        newseg = ROPE_SALLOC(rope, nr);
        memcpy(RDB_SEG_WBUF(newseg), RDB_SEG_RBUF(seg), seg->nused);
        newseg->nused = seg->nused;
        /* "Free" it. Since this buffer is in use, we just unset our own flag */
        seg->shflags &= ~RDB_ROPESEG_F_LIB;
    }

    rope->nused -= newseg->nused;
    nr -= newseg->nused;

    LCB_LIST_SAFE_FOR(llcur, llnext, &rope->segments)
    {
        unsigned to_copy;
        seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        to_copy = MINIMUM(nr, seg->nused);

        memcpy(RDB_SEG_WBUF(newseg), RDB_SEG_RBUF(seg), to_copy);
        newseg->nused += to_copy;

        seg_consumed(rope, seg, to_copy);
        if (!(nr -= to_copy)) {
            break;
        }
    }

    lcb_list_prepend(&rope->segments, &newseg->llnode);
    rope->nused += newseg->nused;
    lcb_assert(rope->nused >= nr);
}

void rdb_consolidate(rdb_IOROPE *ior, unsigned nr)
{
    rope_consolidate(&ior->recvd, nr);
}

void rdb_copyread(rdb_IOROPE *ior, void *tgt, unsigned n)
{
    lcb_list_t *ll;
    char *p = tgt;

    LCB_LIST_FOR(ll, &ior->recvd.segments)
    {
        rdb_ROPESEG *seg = LCB_LIST_ITEM(ll, rdb_ROPESEG, llnode);
        unsigned to_copy = MINIMUM(seg->nused, n);
        memcpy(p, RDB_SEG_RBUF(seg), to_copy);
        p += to_copy;
        n -= to_copy;
        if (!n) {
            break;
        }
    }
}

int rdb_refread_ex(rdb_IOROPE *ior, nb_IOV *iov, rdb_ROPESEG **segs, unsigned nelem, unsigned ndata)
{
    unsigned orig_nelem = nelem;
    lcb_list_t *ll;
    LCB_LIST_FOR(ll, &ior->recvd.segments)
    {
        rdb_ROPESEG *seg = LCB_LIST_ITEM(ll, rdb_ROPESEG, llnode);
        unsigned cur_len = MINIMUM(ndata, seg->nused);
        iov->iov_len = cur_len;
        iov->iov_base = RDB_SEG_RBUF(seg);
        *segs = seg;

        ++iov;
        ++segs;
        --nelem;

        ndata -= cur_len;

        if (!ndata) {
            return orig_nelem - nelem;
        }

        if (!nelem) {
            return -1;
        }
    }

    /** Requested more data than we have */
    fprintf(stderr, "RDB: refread_ex was passed a size greater than our buffer (n=%u)\n", ndata);
    return -1;
}

unsigned rdb_get_contigsize(rdb_IOROPE *ior)
{
    rdb_ROPESEG *seg = RDB_SEG_FIRST(&ior->recvd);
    if (!seg) {
        return 0;
    }
    return seg->nused;
}

char *rdb_get_consolidated(rdb_IOROPE *ior, unsigned n)
{
    lcb_assert(ior->recvd.nused >= n);
    rdb_consolidate(ior, n);
    return RDB_SEG_RBUF(RDB_SEG_FIRST(&ior->recvd));
}

void rdb_seg_ref(rdb_ROPESEG *seg)
{
    seg->refcnt++;
    seg->shflags |= RDB_ROPESEG_F_USER;
}

void rdb_seg_unref(rdb_ROPESEG *seg)
{
    if (--seg->refcnt) {
        return;
    }
    seg->shflags &= ~RDB_ROPESEG_F_USER;
    if (seg->shflags & RDB_ROPESEG_F_LIB) {
        return;
    }
    SEG_RELEASE(seg);
}

void rdb_init(rdb_IOROPE *ior, rdb_ALLOCATOR *alloc)
{
    memset(ior, 0, sizeof(*ior));
    lcb_list_init(&ior->recvd.segments);
    lcb_list_init(&ior->avail.segments);
    rdb_challoc(ior, alloc);
    ior->rdsize = 32768;
}

static void wipe_rope(rdb_ROPEBUF *rope)
{
    lcb_list_t *llcur, *llnext;
    LCB_LIST_SAFE_FOR(llcur, llnext, &rope->segments)
    {
        rdb_ROPESEG *seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        seg_consumed(rope, seg, seg->nused);
    }
}

void rdb_cleanup(rdb_IOROPE *ior)
{
    wipe_rope(&ior->recvd);
    wipe_rope(&ior->avail);
    ior->recvd.allocator->a_release(ior->recvd.allocator);
}

void rdb_challoc(rdb_IOROPE *ior, rdb_ALLOCATOR *alloc)
{
    if (ior->recvd.allocator) {
        ior->recvd.allocator->a_release(ior->recvd.allocator);
    }

    ior->recvd.allocator = alloc;
    ior->avail.allocator = alloc;
}

void rdb_copywrite(rdb_IOROPE *ior, void *buf, unsigned nbuf)
{
    char *cur = buf;

    while (nbuf) {
        unsigned ii;
        unsigned orig_nbuf = nbuf;
        nb_IOV iov[32];
        unsigned niov;

        niov = rdb_rdstart(ior, iov, 32);
        for (ii = 0; ii < niov && nbuf; ii++) {
            unsigned to_copy = MINIMUM(nbuf, iov[ii].iov_len);
            memcpy(iov[ii].iov_base, cur, to_copy);
            cur += to_copy;
            nbuf -= to_copy;
        }
        rdb_rdend(ior, orig_nbuf - nbuf);
    }
}

static void dump_ropebuf(const rdb_ROPEBUF *buf, FILE *fp)
{
    lcb_list_t *llcur;
    fprintf(fp, "TOTAL LENGTH: %u\n", buf->nused);
    fprintf(fp, "WILL DUMP SEGMENTS..\n");
    LCB_LIST_FOR(llcur, &buf->segments)
    {
        const char *indent = "    ";
        rdb_ROPESEG *seg = LCB_LIST_ITEM(llcur, rdb_ROPESEG, llnode);
        fprintf(fp, "%sSEG=%p\n", indent, (void *)seg);
        fprintf(fp, "%sALLOCATOR=%p [%u]\n", indent, (void *)seg->allocator, seg->allocid);
        fprintf(fp, "%sBUFROOT=%p\n", indent, (void *)seg->root);
        fprintf(fp, "%sALLOC SIZE: %u\n", indent, seg->nalloc);
        fprintf(fp, "%sDATA SIZE: %u\n", indent, seg->nused);
        fprintf(fp, "%sDATS OFFSET: %u\n", indent, seg->start);
        fprintf(fp, "%sSEG FLAGS: 0x%x\n", indent, seg->shflags);
        fprintf(fp, "%sSEG REFCNT: %u\n", indent, seg->refcnt);
        fprintf(fp, "\n");
    }
}

void rdb_dump(const rdb_IOROPE *ior, FILE *fp)
{
    fprintf(fp, "@@ DUMP IOROPE=%p\n", (void *)ior);
    fprintf(fp, "@@ ROPEBUF[AVAIL]=%p\n", (void *)&ior->avail);
    dump_ropebuf(&ior->avail, fp);
    fprintf(fp, "@@ ROPEBUF[ACTIVE]=%p\n", (void *)&ior->recvd);
    dump_ropebuf(&ior->recvd, fp);
    if (ior->avail.allocator && ior->avail.allocator->dump) {
        ior->avail.allocator->dump(ior->avail.allocator, fp);
    }
}
