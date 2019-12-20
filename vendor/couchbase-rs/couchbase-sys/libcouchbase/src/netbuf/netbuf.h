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

#ifndef NETBUF_H
#define NETBUF_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file
 * @brief Netbuf write buffers
 */

/**
 * @defgroup netbufs Netbufs
 *
 * # Introduction
 *
 * ## GOALS
 *
 * 1.  provide a simple buffer allocation API
 *     From a logic perspective it's simplest to deal with a straight
 *     contiguous buffer per packet.
 *
 * 2.  provide an efficient way of sending multiple contiguous packets. This
 *     will reduce IOV fragmentation and reduce the number of trips to the
 *     I/O plugin for multiple writes. Currently this is done very efficiently
 *     with the ringbuffer - however this comes at the cost of copying all
 *     request data to the ringbuffer itself. Our aim is to reduce the
 *     number of copies while still maintaining a packed buffer.
 *
 * 3.  Allow a pluggable method by which user-provided data can be plugged
 *     into the span/cursor/flush architecture.
 *
 * @addtogroup netbufs
 * @{
 */

#include "sllist.h"
#include "netbuf-defs.h"
#include "netbuf-mblock.h"

/**
 * @brief Structure representing a buffer within netbufs
 *
 * @note It is recommended that you maintain the individual fields in your
 * own structure and then re-create them as needed. The span structure is 16
 * bytes on 64 bit systems, but can be reduced to 12 if needed. Additionally,
 * you may already have the 'size' field stored/calculated elsewhere.
 */
typedef struct {
    /** @private Parent block */
    nb_MBLOCK *parent;

    /** @private Offset from root at which this buffer begins */
    nb_SIZE offset;

    /** write-once: Allocation size */
    nb_SIZE size;
} nb_SPAN;

#define NETBUF_INVALID_OFFSET (nb_SIZE) - 1

/**
 * Creates a span from a buffer _not_ owned by netbufs.
 * @param span the span to initialize
 * @param buf the buffer
 * @param len the length of the buffer
 */
#define CREATE_STANDALONE_SPAN(span, buf, len)                                                                         \
    (span)->parent = (nb_MBLOCK *)(void *)buf;                                                                         \
    (span)->offset = NETBUF_INVALID_OFFSET;                                                                            \
    (span)->size = len;

/** @private */
typedef struct {
    sllist_node slnode;
    char *base;
    nb_SIZE len;
    /* Extra 4 bytes here. WHAT WE DO!!! */
    const void *parent; /* mc_PACKET */
} nb_SNDQELEM;

/** @private */
typedef struct {
    /** Linked list of pending spans to send */
    sllist_root pending;

    /**
     * List of PDUs to be flushed. A PDU is comprised of one or more IOVs
     * (or even a subsection thereof)
     */
    sllist_root pdus;

    /** The last window which was part of the previous fill call */
    nb_SNDQELEM *last_requested;

    /**
     * Number of bytes enqueued in the 'last request' element. This is needed
     * because it is possible for the last element to grow in length during
     * a subsequent flush.
     */
    nb_SIZE last_offset;

    /** Offset from last PDU which was partially flushed */
    nb_SIZE pdu_offset;

    /** Pool of elements to utilize */
    nb_MBPOOL elempool;
} nb_SENDQ;

struct netbuf_st {
    /** Send Queue */
    nb_SENDQ sendq;

    /** Pool for variable-size data */
    nb_MBPOOL datapool;

    nb_SETTINGS settings;
};

/**
 * Quick way to get the span from a buffer, when the buffer is *known* to
 * be standalone (i.e. CREATE_STANDALONE_SPAN()
 * @param span The span from which to extract the buffer
 * @return a pointer to the buffer
 */
#define SPAN_SABUFFER_NC(span) ((char *)(span)->parent)

/**
 * Quick way to get the span from a buffer when the buffer is known *not*
 * to be standalone
 * @param span The span from which to extract the buffer
 * @return A pointer to a buffer
 */
#define SPAN_MBUFFER_NC(span) ((span)->parent->root + (span)->offset)

/**
 * @brief Retrieves a pointer to the buffer related to this span.
 * @param span the span from which to extract the buffer
 * @return a pointer to the buffer.
 *
 * @see SPAN_SABUFFER_NC
 * @see SPAN_MBUFFER_NC
 */
#define SPAN_BUFFER(span) (((span)->offset == NETBUF_INVALID_OFFSET) ? SPAN_SABUFFER_NC(span) : SPAN_MBUFFER_NC(span))

/**
 * @brief allocate a span
 *
 * Reserve a contiguous region of memory, in-order for a given span. The
 * span will be reserved from the last block to be flushed to the network.
 *
 * The contents of the span are guaranteed to be contiguous (though not aligned)
 * and are available via the SPAN_BUFFER macro.
 *
 * @return 0 if successful, -1 on error
 */
int netbuf_mblock_reserve(nb_MGR *mgr, nb_SPAN *span);

/**
 * @brief release a span
 *
 * Release a span previously allocated via reserve_span. It is assumed that the
 * contents of the span have either:
 *
 * 1. been successfully sent to the network
 * 2. have just been scheduled (and are being removed due to error handling)
 * 3. have been partially sent to a connection which is being closed.
 *
 * @param mgr the manager in which this span is reserved
 * @param span the span
 */
void netbuf_mblock_release(nb_MGR *mgr, nb_SPAN *span);

/**
 * @brief Enqueue a span for serialization
 *
 * Schedules an IOV to be placed inside the send queue. The storage of the
 * underlying buffer must not be freed or otherwise modified until it has
 * been sent.
 *
 * With the current usage model, flush status is implicitly completed once
 * a response has arrived.
 *
 * Note that you may create the IOV from a SPAN object like so:
 * @code{.c}
 * iov->iov_len = span->size;
 * iov->iov_base = SPAN_BUFFER(span);
 * @endcode
 */
void netbuf_enqueue(nb_MGR *mgr, const nb_IOV *bufinfo, const void *parent);

void netbuf_enqueue_span(nb_MGR *mgr, nb_SPAN *span, const void *parent);

/**
 * Gets the number of IOV structures required to flush the entire contents of
 * all buffers.
 */
unsigned int netbuf_get_niov(nb_MGR *mgr);

/**
 * @brief
 * Populates an iovec structure for flushing a set of bytes from the various
 * blocks.
 *
 * You may call this function mutltiple times, so long as each call to
 * start_flush is eventually mapped with a call to end_flush.
 *
 * @code{.c}
 * netbuf_start_flush(mgr, iov1, niov1);
 * netbuf_start_flush(mgr, iov2, niov2);
 * ...
 * netbuf_end_flush(mgr, nbytes1);
 * netbuf_end_flush(mgr, nbytes2);
 * @endcode
 *
 * Additionally, only the LAST end_flush call may be supplied an nflushed
 * parameter which is smaller than the size returned by start_flush. If the
 * entire contents of the `iovs` structure cannot be flushed immediately and
 * you do not wish to persist it until such a time that it may be flushed, then
 * netbuf_reset_flush() should be called. See that functions' documentation
 * for more details.
 *
 * This function may be thought of advancing a virtual cursor which is mapped
 * to a send queue. Each time this function is called the cursor is advanced
 * by the number of bytes that the library expects you to flush (i.e. the return
 * value of this function). Typically the cursor is never rewound and any
 * operation that advances the cursor places the burden on the user to
 * actually flush the data contained within the IOV objects.
 * The netbuf_end_flush() function merely has the task of releasing any memory
 * used for mapping of already-flushed data.
 *
 * From a different perspective, each call to netbuf_start_flush() establishes
 * a contract between the library and the user: The library guarantees that
 * this specific region (referred to within the IOVs) will not be flushed by
 * any other subsystem (i.e. nothing else will try to flush the same data).
 * The user guarantees that the data will eventually be flushed, and that the
 * data will be flushed in the order it was received via start_flush().
 *
 *
 * @param mgr the manager object
 * @param iovs an array of iovec structures
 * @param niov the number of iovec structures allocated.
 * @param[out] nused how many IOVs are actually required
 *
 * @return the number of bytes which can be flushed in this IOV. If the
 * return value is 0 then there are no more bytes to flush.
 *
 * Note that the return value is limited by the number of IOV structures
 * provided and should not be taken as an indicator of how many bytes are
 * used overall.
 */
nb_SIZE netbuf_start_flush(nb_MGR *mgr, nb_IOV *iovs, int niov, int *nused);

/**
 * @brief Indicate that a flush has completed.
 *
 * Indicate that a number of bytes have been flushed. This should be called after
 * the data retrieved by get_flushing_iov has been flushed to the TCP buffers.
 *
 * @param mgr the manager object
 * @param nflushed how much data in bytes was flushed to the network.
 */
void netbuf_end_flush(nb_MGR *mgr, nb_SIZE nflushed);

/**
 * Reset the flush context for the buffer. This should be called only when the
 * following is true:
 *
 * (1) There is only a single open call to netbuf_start_flush
 * (2) The last call to end_flush did not pass all the bytes which were to
 *     be flushed.
 *
 * In this case, the next call to start_flush() will return an IOV which begins
 * where the last end_flush() finished, rather than the last start_flush().
 * As a consequence it means that the previous IOV populated with start_flush
 * is no longer valid and start_flush should be called again.
 */
#define netbuf_reset_flush(mgr)                                                                                        \
    do {                                                                                                               \
        (mgr)->sendq.last_requested = NULL;                                                                            \
        (mgr)->sendq.last_offset = 0;                                                                                  \
    } while (0);

/**
 * Informational function to get the total size of all data in the
 * buffers. This traverses all blocks, so call this for debugging only.
 */
nb_SIZE netbuf_get_size(const nb_MGR *mgr);

/**
 * Get the maximum size of a span which can be satisfied without using an
 * additional block.
 *
 * @param mgr
 *
 * @param allow_wrap
 * Whether to take into consideration wrapping. If this is true then the span
 * size will allow wrapping. If disabled, then only the packed size will be
 * available. Consider:
 * <pre>
 * [ ooooooo{S:10}xxxxxxxxx{C:10}ooooo{A:5} ]
 * </pre>
 * If wrapping is allowed, then the maximum span size will be 10, from 0..10
 * but the last 5 bytes at the end will be lost for the duration of the block.
 * If wrapping is not allowed then the maximum span size will be 5.
 *
 * @return
 * the maximum span size without requiring additional blocks.
 */
nb_SIZE netbuf_mblock_get_next_size(const nb_MGR *mgr, int allow_wrap);

/**
 * @brief Initializes an nb_MGR structure
 * @param mgr the manager to initialize
 * @param settings
 */
void netbuf_init(nb_MGR *mgr, const nb_SETTINGS *settings);

/**
 * @brief Frees up any allocated resources for a given manager
 * @param mgr the manager for which to release resources
 */
void netbuf_cleanup(nb_MGR *mgr);

/**
 * Populates the settings structure with the default settings. This structure
 * may then be modified or tuned and passed to netbuf_init()
 */
void netbuf_default_settings(nb_SETTINGS *settings);

/**
 * Dump the internal structure of the manager to the screen. Useful for
 * debugging.
 */
void netbuf_dump_status(nb_MGR *mgr, FILE *fp);

/**
 * Mark a PDU as being enqueued. This should be called whenever the final IOV
 * for a given PDU has just been enqueued.
 *
 * The PDU itself must remain valid and is assumed to contain an 'slnode'
 * structure which will point to the next PDU. The PDU will later be removed
 * from the queue when 'end_flush' has been called including its range.
 *
 * @param mgr The manager
 *
 * @param pdu An opaque pointer
 *
 * @param lloff The offset from the pdu pointer at which the slist_node
 *        structure may be found.
 */
void netbuf_pdu_enqueue(nb_MGR *mgr, void *pdu, nb_SIZE lloff);

/**
 * This callback is invoked during 'end_flush2'.
 *
 * @param pdu The PDU pointer enqueued via netbuf_pdu_enqueue()
 *
 * @param remaining A hint passed to the callback indicating how many bytes
 *        remain on the stream. IFF remaining is greater than the size of the
 *        PDU the callback may change internal state within the packet to mark
 *        it as flushed.
 *
 *        XXX:
 *        If nremaining < pdusize then it <b>must</b> be ignored. The value
 *        may have been previously passed to the same callback during a
 *        prior iteration.
 *
 *        This is done by design in order to avoid forcing each PDU to maintain
 *        a variable indicating "How much was flushed".
 *
 * @param arg A pointer passed to the start_flush call; used to correlate any
 *        data common to the queue itself.
 *
 * @return The size of the PDU. This will be used to determine future calls
 *         to the callback for subsequent PDUs.
 *
 *         If size <= remaining then this PDU will be popped off the PDU queue
 *         and is deemed to be no longer utilized by the send queue (and may
 *         be released from the mblock allocator; if it's being used).
 *
 *         If size > remaining then no further callbacks will be invoked on
 *         the relevant PDUs.
 */
typedef nb_SIZE (*nb_getsize_fn)(void *pdu, nb_SIZE remaining, void *arg);

void netbuf_end_flush2(nb_MGR *mgr, unsigned int nflushed, nb_getsize_fn callback, nb_SIZE lloff, void *arg);

/**
 * Ensures that the given internal structures of the manager are not allocated
 * or otherwise in use by other systems. This is useful for testing to ensure
 * that we wouldn't accidentally think everything is OK.
 *
 * Because resources are released at the block level, we might have had blocks
 * which were partially allocated.
 *
 * This also checks the PDU and send queues as well.
 */
int netbuf_is_clean(nb_MGR *mgr);

/**
 * Determines if there is any data to be flushed to the network
 */
int netbuf_has_flushdata(nb_MGR *mgr);

/**@}*/

#ifdef __cplusplus
}
#endif

#endif /* LCB_PACKET_H */
