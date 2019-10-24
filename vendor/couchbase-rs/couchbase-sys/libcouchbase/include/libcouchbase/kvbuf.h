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

#ifndef LCB_KVBUF_H
#define LCB_KVBUF_H

/**
 * @file
 *
 * Low level structures used by commands for buffers.
 */

#ifdef __cplusplus
extern "C" {
#endif

/** @brief Flags indicating the storage policy for a buffer */
typedef enum {
    LCB_KV_COPY = 0, /**< The buffer should be copied */
    LCB_KV_CONTIG,   /**< The buffer is contiguous and should not be copied */
    LCB_KV_IOV,      /**< The buffer is not contiguous and should not be copied */
    /**Indicates that the precomputed vBucket ID should be used */
    LCB_KV_VBID,

    /**
     * The buffers are not contiguous (multi-part buffers) but should be
     * copied. This avoids having to make the buffers contiguous before
     * passing it into the library (only to have the library copy it again) */
    LCB_KV_IOVCOPY,
} lcb_KVBUFTYPE;

#define LCB_KV_HEADER_AND_KEY LCB_KV_CONTIG

/**
 * @brief simple buf/length structure for a contiguous series of bytes
 */
typedef struct lcb_CONTIGBUF {
    const void *bytes;
    /** Number of total bytes */
    lcb_size_t nbytes;
} lcb_CONTIGBUF;

/** @brief Common request header for all keys */
typedef struct lcb_KEYBUF {
    /**
     * The type of key to provide. This can currently be LCB_KV_COPY (Default)
     * to copy the key into the pipeline buffers, or LCB_KV_HEADER_AND_KEY
     * to provide a buffer with the header storage and the key.
     *
     * TODO:
     * Currently only LCB_KV_COPY should be used. LCB_KV_HEADER_AND_KEY is used
     * internally but may be exposed later on
     */
    lcb_KVBUFTYPE type;
    lcb_CONTIGBUF contig;
    lcb_U16 vbid; /**< precomputed vbucket id */
} lcb_KEYBUF;

/**@private*/
#define LCB_KEYBUF_IS_EMPTY(k) (k)->contig.nbytes == 0

/**
 * @brief Initialize a contiguous request backed by a buffer which should be
 * copied
 * @param req the key request to initialize
 * @param k the key to copy
 * @param nk the size of the key
 */
#define LCB_KREQ_SIMPLE(req, k, nk)                                                                                    \
    do {                                                                                                               \
        (req)->type = LCB_KV_COPY;                                                                                     \
        (req)->contig.bytes = k;                                                                                       \
        (req)->contig.nbytes = nk;                                                                                     \
    } while (0);

/**
 * Structure for an IOV buffer to be supplied as a buffer. This is currently
 * only used for value buffers
 */
typedef struct lcb_FRAGBUF {
    /** An IOV array */
    lcb_IOV *iov;

    /** Number of elements in iov array */
    unsigned int niov;

    /**
     * Total length of the items. This should be set, if known, to prevent the
     * library from manually traversing the iov array to calculate the length.
     */
    unsigned int total_length;
} lcb_FRAGBUF;

/** @brief Structure representing a value to be stored */
typedef struct lcb_VALBUF {
    /**
     * Value request type. This may be one of:
     * - LCB_KV_COPY: Copy over the value into LCB's own buffers
     *   Use the 'contig' field to supply the information.
     *
     * - LCB_KV_CONTIG: The buffer is a contiguous chunk of value data.
     *   Use the 'contig' field to supply the information.
     *
     * - LCB_KV_IOV: The buffer is a series of IOV elements. Use the 'multi'
     *   field to supply the information.
     */
    lcb_KVBUFTYPE vtype;
    union {
        lcb_CONTIGBUF contig;
        lcb_FRAGBUF multi;
    } u_buf;
} lcb_VALBUF;

#ifdef __cplusplus
}
#endif
#endif
