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
#ifndef RINGBUFFER_H
#define RINGBUFFER_H 1

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ringbuffer_st {
    char *root;
    char *read_head;
    char *write_head;
    lcb_size_t size;
    lcb_size_t nbytes;
} ringbuffer_t;

typedef enum { RINGBUFFER_READ = 0x01, RINGBUFFER_WRITE = 0x02 } ringbuffer_direction_t;

int ringbuffer_initialize(ringbuffer_t *buffer, lcb_size_t size);

/**
 * Initialize a ringbuffer, taking ownership of an allocated char buffer.
 * This function always succeeds.
 * @param buffer a ringbuffer_t to be initialized
 * @param buf the buffer to steal
 * @param size the allocated size of the buffer
 */
void ringbuffer_take_buffer(ringbuffer_t *buffer, char *buf, lcb_size_t size);

void ringbuffer_reset(ringbuffer_t *buffer);

void ringbuffer_destruct(ringbuffer_t *buffer);
int ringbuffer_ensure_capacity(ringbuffer_t *buffer, lcb_size_t size);
lcb_size_t ringbuffer_get_size(ringbuffer_t *buffer);
void *ringbuffer_get_start(ringbuffer_t *buffer);
void *ringbuffer_get_read_head(ringbuffer_t *buffer);
void *ringbuffer_get_write_head(ringbuffer_t *buffer);
lcb_size_t ringbuffer_write(ringbuffer_t *buffer, const void *src, lcb_size_t nb);
lcb_size_t ringbuffer_strcat(ringbuffer_t *buffer, const char *str);
lcb_size_t ringbuffer_read(ringbuffer_t *buffer, void *dest, lcb_size_t nb);
lcb_size_t ringbuffer_peek(ringbuffer_t *buffer, void *dest, lcb_size_t nb);
lcb_size_t ringbuffer_peek_at(ringbuffer_t *buffer, lcb_size_t offset, void *dest, lcb_size_t nb);
/* replace +nb+ bytes on +direction+ end of the buffer with src */
lcb_size_t ringbuffer_update(ringbuffer_t *buffer, ringbuffer_direction_t direction, const void *src, lcb_size_t nb);
void ringbuffer_get_iov(ringbuffer_t *buffer, ringbuffer_direction_t direction, struct lcb_iovec_st *iov);
void ringbuffer_produced(ringbuffer_t *buffer, lcb_size_t nb);
void ringbuffer_consumed(ringbuffer_t *buffer, lcb_size_t nb);
lcb_size_t ringbuffer_get_nbytes(ringbuffer_t *buffer);
int ringbuffer_is_continous(ringbuffer_t *buffer, ringbuffer_direction_t direction, lcb_size_t nb);

int ringbuffer_append(ringbuffer_t *src, ringbuffer_t *dest);
int ringbuffer_memcpy(ringbuffer_t *dst, ringbuffer_t *src, lcb_size_t nbytes);

/* Align the read head of the ringbuffer for platforms where it's needed */
int ringbuffer_ensure_alignment(ringbuffer_t *src);

#ifdef __cplusplus
}
#endif

#endif
