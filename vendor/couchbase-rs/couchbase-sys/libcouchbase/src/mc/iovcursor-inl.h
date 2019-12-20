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

#include "iovcursor.h"
#ifndef MINIMUM
#define MINIMUM(a, b) (a) < (b) ? a : b
#endif

/**Status code returned by the peek_ex() function. Codes below 0 are errors,
 * while codes above 0 are successes*/
typedef enum {
    /** Data would be fragmented and no `copytgt` parameter was provided */
    IOVCURSOR_STATUS_FRAGMENTED = -2,
    /** Pointer to data is referenced by the `contigptr` parameter */
    IOVCURSOR_STATUS_CONTIGPTR_OK = 0,
    /** Pointer to contiguous data cannot be obtained. It has been copied */
    IOVCURSOR_STATUS_BUFCOPY_OK
} iovcursor_STATUS;

/**
 * Obtain data from a cursor without advancing it
 * @param cursor The cursor to read from
 *
 * @param copytgt A buffer in which to copy the data, in case a contiguous
 * pointer to `size` bytes cannot be obtained.
 *
 * @param[out] contigref A pointer to be set to a contiguous region of memory
 * containing `size` bytes
 * @param size
 * @param offset The offset at which to start reading
 * @return A status code indicating success or failure of the operation
 * @note Either `copytgt` or `contigref` _must_ be specified. It is ok to pass
 * them both in the same function call (and the return status may be inspected
 * to see which one of the parameters contains the actual data).
 */
static iovcursor_STATUS iovcursor_peek_ex(const mc_IOVCURSOR *cursor, char *copytgt, const char **contigref,
                                          unsigned size, unsigned offset)
{
    unsigned ii;
    const nb_IOV *iov = cursor->iov;
    offset += cursor->offset;
    for (ii = 0; ii < cursor->niov && size > 0; ++ii) {
        unsigned contiglen, tmpoff;
        const char *srcbuf;
        const nb_IOV *cur = iov + ii;

        if (offset) {
            if (offset >= cur->iov_len) {
                offset -= cur->iov_len;
                continue;
            } else {
                tmpoff = offset;
                offset = 0;
            }
        } else {
            tmpoff = 0;
        }

        contiglen = cur->iov_len - tmpoff;
        srcbuf = (const char *)cur->iov_base + tmpoff;

        /* We always end up returning from these following blocks and _not_
         * the end of the loop:
         *
         * For contiguous buffers, contiglen >= size is true on the first
         * iteration.
         *
         * For fragmented buffers, contiglen >= size is _eventually_ true.
         * however during the first initial IOVs, the buffer is fragmented
         * but as the output is copied the size variable is decremented. Note
         * that during this process we also set contigref to NULL to avoid
         * accidentally thinking that the full data set is contiguous
         */
        if (size <= contiglen) {
            if (contigref) {
                *contigref = srcbuf;
                return IOVCURSOR_STATUS_CONTIGPTR_OK;
            } else {
                memcpy(copytgt, srcbuf, size);
                return IOVCURSOR_STATUS_BUFCOPY_OK;
            }
        } else if (copytgt == NULL) {
            *contigref = NULL;
            return IOVCURSOR_STATUS_FRAGMENTED;
        } else {
            /* copy and continue */
            unsigned to_copy = MINIMUM(size, cur->iov_len - tmpoff);
            memcpy(copytgt, srcbuf, to_copy);
            copytgt += to_copy;
            if (contigref) {
                *contigref = NULL;
                contigref = NULL;
            }

            /* We've copied, so ignore the 'contigref' */
            contigref = NULL;
            if (!(size -= to_copy)) {
                return IOVCURSOR_STATUS_BUFCOPY_OK;
            }
        }
    }

    lcb_assert(!size);
    *contigref = NULL;
    return IOVCURSOR_STATUS_FRAGMENTED;
}

/**
 * Copy data to the target buffer, without modifying the offset
 * @param cursor The cursror to read from
 * @param buf The target buffer
 * @param size The number of bytes to copy
 * @param offset Position in the input at which to start copying
 * @return true if there were sufficient bytes to copy, false otherwise.
 */
static int iovcursor_peek(const mc_IOVCURSOR *cursor, char *buf, unsigned size, unsigned offset)
{
    int rv = iovcursor_peek_ex(cursor, buf, NULL, size, offset);
    return rv == IOVCURSOR_STATUS_BUFCOPY_OK;
}

/**
 * Populate an IOV structure with the effective offset of the first IOV contained
 * within the cursor
 * @param cursor
 * @param maxsize Maximum length of resultant IOV, if the cursor's first IOV
 * length is greated
 * @param iov The iov to be initialized
 */
static unsigned iovcursor_adv_first(mc_IOVCURSOR *cursor, unsigned maxsize, nb_IOV *iov)
{
    const char *srcbuf = (const char *)cursor->iov->iov_base + cursor->offset;

    /* Set the target */
    iov->iov_base = (void *)srcbuf;
    iov->iov_len = MINIMUM(cursor->iov->iov_len - cursor->offset, maxsize);

    if (iov->iov_len == (cursor->iov->iov_len - cursor->offset)) {
        /* did we swallow the entire source iov? - consume it */
        cursor->iov++;
        cursor->niov--;
        cursor->offset = 0;
    } else {
        /* increase the offset */
        cursor->offset += iov->iov_len;
    }

    return iov->iov_len;
}

/**
 * Copy data to the target buffer, advancing the cursor
 * @param cursor The cursor
 * @param tgt The buffer to copy to
 * @param size The number of bytes to copy
 *
 * @warning No check is made to see if `size` is greater than the amounf of
 * data contained within the cursor. Exceeding the amount of data will result
 * in undefined behavior.
 */
static void iovcursor_adv_copy(mc_IOVCURSOR *cursor, char *tgt, unsigned size)
{
    nb_IOV tmpiov;
    nb_IOV *iov;
    unsigned niov;

    size -= iovcursor_adv_first(cursor, size, &tmpiov);
    memcpy(tgt, tmpiov.iov_base, tmpiov.iov_len);
    tgt += tmpiov.iov_len;

    /* assign iov and iov now, since adv_first() may have modified them */
    iov = cursor->iov;
    niov = cursor->niov;

    while (size) {
        unsigned to_copy = MINIMUM(iov->iov_len, size);
        const char *srcbuf = (const char *)iov->iov_base;
        memcpy(tgt, srcbuf, to_copy);
        tgt += to_copy;
        size -= to_copy;

        if (to_copy != iov->iov_len) {
            cursor->offset = to_copy;
            lcb_assert(!size);
            break;
        }

        iov++;
        niov--;
    }

    /* modify the variables */
    cursor->iov = iov;
    cursor->niov = niov;
}

/**
 * Macro which determines if the specific cursor's first IOV has enough
 * data to ensure a contiguous memory region of a specific amount of bytes
 * @param mincur the cursor
 * @param n the number of bytes to check for
 * @return nonzero if the requested size is available
 */
#define IOVCURSOR_HAS_CONTIG(mincur, n) ((mincur)->iov->iov_len - (mincur)->offset) >= n

/**
 * Create an allocated array of IOVs which point to a subset of IOVs within
 * the current cursor. This function will also advance the cursor position.
 *
 * @param cursor The cursor to get the offsets from
 * @param size The size the resultant array should cover
 * @param[out] arr The IOV array which shall contain the offsets. The array
 * should be freed by free() when no longer required.
 * @param[out] narr Number of elements in the resultant array.
 */
static void iovcursor_adv_iovalloc(mc_IOVCURSOR *cursor, unsigned size, nb_IOV **p_arr, unsigned *p_narr)
{
    unsigned ii, narr;
    nb_IOV dummy, *arr;

    /* chop off the first IOV for convenience */
    size -= iovcursor_adv_first(cursor, size, &dummy);
    narr = 1;

    if (size) {
        unsigned cursz = size;
        for (ii = 0; cursz > 0; ++ii) {
            cursz -= MINIMUM(cursz, cursor->iov[ii].iov_len);
        }
        narr += ii;
    }

    arr = (nb_IOV *)malloc(sizeof(*arr) * narr);
    arr[0] = dummy;

    for (ii = 1; size > 0; ++ii) {
        unsigned to_adv = MINIMUM(size, cursor->iov->iov_len);
        const char *srcbuf = (const char *)cursor->iov->iov_base;

        arr[ii].iov_base = (void *)srcbuf;
        arr[ii].iov_len = MINIMUM(size, cursor->iov->iov_len);

        size -= to_adv;

        if (size == 0 && to_adv < cursor->iov->iov_len) {
            /* did we not copy the entire iov? */
            cursor->offset = to_adv;
        } else {
            cursor->iov++;
            cursor->niov--;
        }
    }

    *p_arr = arr;
    *p_narr = narr;
}
