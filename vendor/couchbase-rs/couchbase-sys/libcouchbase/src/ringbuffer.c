/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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

static lcb_size_t minimum(lcb_size_t a, lcb_size_t b)
{
    return (a < b) ? a : b;
}

int ringbuffer_initialize(ringbuffer_t *buffer, lcb_size_t size)
{
    char *root = malloc(size);
    if (root == NULL) {
        return 0;
    }
    ringbuffer_take_buffer(buffer, root, size);
    return 1;
}

void ringbuffer_take_buffer(ringbuffer_t *buffer, char *buf, lcb_size_t size)
{
    memset(buffer, 0, sizeof(ringbuffer_t));
    buffer->root = buf;
    buffer->size = size;
    buffer->write_head = buffer->root;
    buffer->read_head = buffer->root;
}

void ringbuffer_reset(ringbuffer_t *buffer)
{
    ringbuffer_consumed(buffer, ringbuffer_get_nbytes(buffer));
}

void ringbuffer_destruct(ringbuffer_t *buffer)
{
    free(buffer->root);
    buffer->root = buffer->read_head = buffer->write_head = NULL;
    buffer->size = buffer->nbytes = 0;
}

int ringbuffer_ensure_capacity(ringbuffer_t *buffer, lcb_size_t size)
{
    char *new_root;
    lcb_size_t new_size = buffer->size << 1;
    if (new_size == 0) {
        new_size = 128;
    }

    if (size < (buffer->size - buffer->nbytes)) {
        /* we've got capacity! */
        return 1;
    }

    /* determine the new buffer size... */
    while ((new_size - buffer->nbytes) < size) {
        new_size <<= 1;
    }

    /* go ahead and allocate a bigger block */
    if ((new_root = malloc(new_size)) == NULL) {
        /* Allocation failed! */
        return 0;
    } else {
        /* copy the data over :) */
        char *old;
        lcb_size_t nbytes = buffer->nbytes;
        lcb_size_t nr = ringbuffer_read(buffer, new_root, nbytes);
        lcb_assert(nr == nbytes);
        old = buffer->root;
        buffer->size = new_size;
        buffer->root = new_root;
        buffer->nbytes = nbytes;
        buffer->read_head = buffer->root;
        buffer->write_head = buffer->root + nbytes;
        free(old);
        return 1;
    }
}

lcb_size_t ringbuffer_get_size(ringbuffer_t *buffer)
{
    return buffer->size;
}

void *ringbuffer_get_start(ringbuffer_t *buffer)
{
    return buffer->root;
}

void *ringbuffer_get_read_head(ringbuffer_t *buffer)
{
    return buffer->read_head;
}

void *ringbuffer_get_write_head(ringbuffer_t *buffer)
{
    return buffer->write_head;
}

lcb_size_t ringbuffer_write(ringbuffer_t *buffer, const void *src, lcb_size_t nb)
{
    const char *s = src;
    lcb_size_t nw = 0;
    lcb_size_t space;
    lcb_size_t toWrite;

    if (buffer->write_head >= buffer->read_head) {
        /* write up to the end with data.. */
        space = buffer->size - (lcb_size_t)(buffer->write_head - buffer->root);
        toWrite = minimum(space, nb);

        if (src != NULL) {
            memcpy(buffer->write_head, s, toWrite);
        }
        buffer->nbytes += toWrite;
        buffer->write_head += toWrite;
        nw = toWrite;

        if (buffer->write_head == (buffer->root + buffer->size)) {
            buffer->write_head = buffer->root;
        }

        if (nw == nb) {
            /* everything is written to the buffer.. */
            return nw;
        }

        nb -= toWrite;
        s += toWrite;
    }

    /* Copy data up until we catch up with the read head */
    space = (lcb_size_t)(buffer->read_head - buffer->write_head);
    toWrite = minimum(space, nb);
    if (src != NULL) {
        memcpy(buffer->write_head, s, toWrite);
    }
    buffer->nbytes += toWrite;
    buffer->write_head += toWrite;
    nw += toWrite;

    if (buffer->write_head == (buffer->root + buffer->size)) {
        buffer->write_head = buffer->root;
    }

    return nw;
}

lcb_size_t ringbuffer_strcat(ringbuffer_t *buffer, const char *str)
{
    lcb_size_t len = strlen(str);
    if (!ringbuffer_ensure_capacity(buffer, len)) {
        return 0;
    }
    return ringbuffer_write(buffer, str, len);
}

static void maybe_reset(ringbuffer_t *buffer)
{
    if (buffer->nbytes == 0) {
        buffer->write_head = buffer->root;
        buffer->read_head = buffer->root;
    }
}

lcb_size_t ringbuffer_read(ringbuffer_t *buffer, void *dest, lcb_size_t nb)
{
    char *d = dest;
    lcb_size_t nr = 0;
    lcb_size_t space;
    lcb_size_t toRead;

    if (buffer->nbytes == 0) {
        return 0;
    }
    if (buffer->read_head >= buffer->write_head) {
        /* read up to the wrap point */
        space = buffer->size - (lcb_size_t)(buffer->read_head - buffer->root);
        toRead = minimum(space, nb);

        if (dest != NULL) {
            memcpy(d, buffer->read_head, toRead);
        }
        buffer->nbytes -= toRead;
        buffer->read_head += toRead;
        nr = toRead;

        if (buffer->read_head == (buffer->root + buffer->size)) {
            buffer->read_head = buffer->root;
        }

        if (nr == nb) {
            maybe_reset(buffer);
            return nr;
        }

        nb -= toRead;
        d += toRead;
    }

    space = (lcb_size_t)(buffer->write_head - buffer->read_head);
    toRead = minimum(space, nb);

    if (dest != NULL) {
        memcpy(d, buffer->read_head, toRead);
    }
    buffer->nbytes -= toRead;
    buffer->read_head += toRead;
    nr += toRead;

    if (buffer->read_head == (buffer->root + buffer->size)) {
        buffer->read_head = buffer->root;
    }

    maybe_reset(buffer);
    return nr;
}

lcb_size_t ringbuffer_peek(ringbuffer_t *buffer, void *dest, lcb_size_t nb)
{
    ringbuffer_t copy = *buffer;
    return ringbuffer_read(&copy, dest, nb);
}

lcb_size_t ringbuffer_peek_at(ringbuffer_t *buffer, lcb_size_t offset, void *dest, lcb_size_t nb)
{
    ringbuffer_t copy = *buffer;
    lcb_size_t n = ringbuffer_read(&copy, NULL, offset);
    if (n != offset) {
        return -1;
    }
    return ringbuffer_read(&copy, dest, nb);
}

void ringbuffer_produced(ringbuffer_t *buffer, lcb_size_t nb)
{
    lcb_size_t n = ringbuffer_write(buffer, NULL, nb);
    lcb_assert(n == nb);
}

void ringbuffer_consumed(ringbuffer_t *buffer, lcb_size_t nb)
{
    lcb_size_t n = ringbuffer_read(buffer, NULL, nb);
    lcb_assert(n == nb);
}

lcb_size_t ringbuffer_get_nbytes(ringbuffer_t *buffer)
{
    return buffer->nbytes;
}

lcb_size_t ringbuffer_update(ringbuffer_t *buffer, ringbuffer_direction_t direction, const void *src, lcb_size_t nb)
{
    const char *s = src;
    lcb_size_t nw, ret = 0;

    if (direction == RINGBUFFER_READ) {
        if (buffer->read_head <= buffer->write_head) {
            nw = minimum(nb, buffer->nbytes);
            memcpy(buffer->read_head, s, nw);
            ret += nw;
        } else {
            nw = minimum(nb, buffer->size - (lcb_size_t)(buffer->read_head - buffer->root));
            memcpy(buffer->read_head, s, nw);
            nb -= nw;
            s += nw;
            ret += nw;
            if (nb) {
                nw = minimum(nb, (lcb_size_t)(buffer->write_head - buffer->root));
                memcpy(buffer->root, s, nw);
                ret += nw;
            }
        }
    } else {
        if (buffer->write_head >= buffer->read_head) {
            nw = minimum(nb, buffer->nbytes);
            memcpy(buffer->write_head - nw, s, nw);
            ret += nw;
        } else {
            nb = minimum(nb, buffer->nbytes);
            nw = minimum(nb, (lcb_size_t)(buffer->write_head - buffer->root));
            memcpy(buffer->write_head - nw, s + nb - nw, nw);
            nb -= nw;
            ret += nw;
            if (nb) {
                nw = minimum(nb, buffer->size - (lcb_size_t)(buffer->read_head - buffer->root));
                memcpy(buffer->root + buffer->size - nw, s, nw);
                ret += nw;
            }
        }
    }
    return ret;
}

void ringbuffer_get_iov(ringbuffer_t *buffer, ringbuffer_direction_t direction, struct lcb_iovec_st *iov)
{
    iov[1].iov_base = buffer->root;
    iov[1].iov_len = 0;

    if (direction == RINGBUFFER_READ) {
        iov[0].iov_base = buffer->read_head;
        iov[0].iov_len = buffer->nbytes;
        if (buffer->read_head >= buffer->write_head) {
            ptrdiff_t chunk = buffer->root + buffer->size - buffer->read_head;
            if (buffer->nbytes > (lcb_size_t)chunk) {
                iov[0].iov_len = (lcb_size_t)chunk;
                iov[1].iov_len = buffer->nbytes - (lcb_size_t)chunk;
            }
        }
    } else {
        lcb_assert(direction == RINGBUFFER_WRITE);
        iov[0].iov_base = buffer->write_head;
        iov[0].iov_len = buffer->size - buffer->nbytes;
        if (buffer->write_head >= buffer->read_head) {
            /* I may write all the way to the end! */
            iov[0].iov_len = (lcb_size_t)((buffer->root + buffer->size) - buffer->write_head);
            /* And all the way up to the read head */
            iov[1].iov_len = (lcb_size_t)(buffer->read_head - buffer->root);
        }
    }
}

int ringbuffer_is_continous(ringbuffer_t *buffer, ringbuffer_direction_t direction, lcb_size_t nb)
{
    int ret;

    if (direction == RINGBUFFER_READ) {
        ret = (nb <= buffer->nbytes);

        if (buffer->read_head >= buffer->write_head) {
            ptrdiff_t chunk = buffer->root + buffer->size - buffer->read_head;
            if (nb > (lcb_size_t)chunk) {
                ret = 0;
            }
        }
    } else {
        ret = (nb <= buffer->size - buffer->nbytes);
        if (buffer->write_head >= buffer->read_head) {
            ptrdiff_t chunk = buffer->root + buffer->size - buffer->write_head;
            if (nb > (lcb_size_t)chunk) {
                ret = 0;
            }
        }
    }
    return ret;
}

int ringbuffer_append(ringbuffer_t *src, ringbuffer_t *dest)
{
    char buffer[1024];
    lcb_size_t nr, nw;

    while ((nr = ringbuffer_read(src, buffer, sizeof(buffer))) != 0) {
        lcb_assert(ringbuffer_ensure_capacity(dest, nr));
        nw = ringbuffer_write(dest, buffer, nr);
        lcb_assert(nw == nr);
    }

    return 1;
}

int ringbuffer_memcpy(ringbuffer_t *dst, ringbuffer_t *src, lcb_size_t nbytes)
{
    ringbuffer_t copy = *src;
    struct lcb_iovec_st iov[2];
    int ii = 0;
    lcb_size_t towrite = nbytes;
    lcb_size_t toread, nb;

    if (nbytes > ringbuffer_get_nbytes(src)) {
        /* EINVAL */
        return -1;
    }

    if (!ringbuffer_ensure_capacity(dst, nbytes)) {
        /* Failed to allocate space */
        return -1;
    }

    ringbuffer_get_iov(dst, RINGBUFFER_WRITE, iov);
    toread = minimum(iov[ii].iov_len, nbytes);
    do {
        lcb_assert(ii < 2);
        nb = ringbuffer_read(&copy, iov[ii].iov_base, toread);
        toread -= nb;
        towrite -= nb;
        ++ii;
    } while (towrite > 0);
    ringbuffer_produced(dst, nbytes);
    return 0;
}

int ringbuffer_ensure_alignment(ringbuffer_t *c)
{
#if defined(__hpux__) || defined(__hpux) || defined(__sparc__) || defined(__sparc)
    intptr_t addr = (intptr_t)c->read_head;

    if (addr % 8 != 0) {
        ringbuffer_t copy;
        if (ringbuffer_initialize(&copy, c->size) == 0 || ringbuffer_memcpy(&copy, c, ringbuffer_get_nbytes(c)) == -1) {
            return -1;
        }
        ringbuffer_destruct(c);
        *c = copy;
    }
#else
    (void)c;
#endif
    return 0;
}
