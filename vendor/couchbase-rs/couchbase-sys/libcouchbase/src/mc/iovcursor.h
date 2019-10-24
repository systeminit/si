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

#ifndef MC_IOVCURSOR_H
#define MC_IOVCURSOR_H
#ifdef __cplusplus
extern "C" {
#endif

/** Minimal cursor */
typedef struct {
    /**The IOV array containing the buffer offsets. This is initialized to the
     * first element of the array on input. As data is consumed by the
     * library, this pointer value will increment.*/
    nb_IOV *iov;

    /**Number of elements in the IOV array. This is decremented as the `iov`
     * field is incremented.*/
    unsigned niov;

    /**Offset into first IOV structure which contains data. This is used
     * if the IOV contains partially consumed data. The library sets this
     * field if a packet ends in the middle of an IOV buffer*/
    unsigned offset;
} mc_IOVCURSOR;

typedef struct {
    /** Cursor element */
    mc_IOVCURSOR c;

    /**The total number of bytes used by the library in the last packet
     * successfuly processed.*/
    unsigned consumed;

    /**Number of bytes wanted for next operation (OUT). This contains the
     * total number of bytes (including any within the buffer already).
     * The library does not read from this value. */
    unsigned wanted;

    /**The total amount of data within the IOV buffers. This is initialized
     * in the mc_iovinfo_init() function by traversing through all the elements
     * and adding their `iov_len` fields. If using the `IOVINFO` structure
     * in a read loop, you will want to increment this whenever new data has
     * been placed into buffers*/
    unsigned total;
} mc_IOVINFO;

#ifdef __cplusplus
}
#endif

#endif
