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

#ifndef LCB_ROPE_H
#define LCB_ROPE_H
#include <libcouchbase/sysdefs.h>
#include <libcouchbase/visibility.h>
#include <netbuf/netbuf-defs.h>
#include <stdio.h>
#include "list.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup rdb Network Read Buffers
 * @brief Read buffer system.
 *
 * # Overview
 *
 * This subsystem provides an extensible means by which to deal with input
 * network buffers.
 *
 * Sequential network data is represented in something called an `rdb_ROPEBUF`
 * structure. The rope structure itself consists of one or more
 * `rdb_ROPESEG` structures; where a segment represents a contiguous block of
 * memory.
 *
 * Segments are thus the base allocation unit. They are chained together in
 * a ROPE structure to form a sequence of data, where the first segment contains
 * previous data and the last segment contains the newest data (and potentially
 * free space for additional reads).
 *
 * The size of each segment is determined by either the allocator or the
 * library. The allocator determines the size for "Estimated" read-aheads, where it
 * decides on the best form of fragmentation (if any) for the TCP buffers.
 * The library determines the size of a segment if a specific sequence of data
 * must be represented as contiguous bytes in memory.
 *
 * The differentiation between these two modes is expressed in the
 * r_reserve() function (where the allocator determines the best mode of
 * fragmentation) and the s_alloc/s_realloc functions where the library demands
 * a specific form of continuity.
 *
 * The API here is divided into two sections. One API deals with allocating
 * the relevant buffers (allocation API) while the other API deals with
 * extracting data from said buffers.
 *
 * # Extraction API
 *
 * The extraction API provides means for receiving buffers for network reads
 * and for extracting data once these reads have been completed.
 *
 * The read operation begins by the user providing an IOV array and passing it
 * to the rdb_rdstart() function. This function will populate the fields in
 * each IOV structure corresponding to the segments of the relevant IOVs.
 *
 * You may then forward these IOV buffers to the system-level read functions
 * (e.g. recvmsg).
 *
 * After data has been read into the buffers, call rdb_rdend() with the number
 * of bytes the last read operation received.
 *
 * Internally the rope structure contains a position at which data begins.
 * All functions which extract data begin from this offset.
 *
 * To extract the data you may call rdb_get_consolidate which will return
 * a pointer to a contiguous region of memory of the specified number of bytes.
 *
 * Finally there is rdb_copyread() which will _copy_ the contents of the read
 * buffer over to a user allocated buffer. This does _not_ consolidate the
 * buffers internally.
 *
 * Once you are done processing the data, you can call rdb_consumed() which
 * will advance the start position by the specified number of bytes, releasing
 * any buffers which do not contain data back to the allocator.
 *
 * ## Extended Extraction API
 *
 * The more advanced rdb_refread_ex() will populate an array of IOV and
 * rdb_SEGMENT pointers with the first such element containing the offsets of
 * the first data byte, and the last of these elements containing the end
 * of the specified length of data. This function is useful if the calling code
 * does not require the data to be contiguous. The two arrays are provided
 * so that the IOV array can easily be passed to a socket function (rather
 * than merely reconstructing them from the segments array).
 *
 * Using rdb_refread_ex() you may also use the rdb_seg_ref() function to "pin"
 * the the segments contained in the output array. When the segments are pinned
 * they are guaranteed not to be overwritten or released from memory until
 * the last call to rdb_seg_unref() is invoked.
 *
 * You may call rdb_consolidate() to ensure that a certain chunk of data
 * will be continuous when it is available to be read. This is useful in a
 * partial read where remaining data has not been received yet, but you know
 * that you will need the current segment as well as a specified length
 * of incoming data to be contiguous.
 *
 * # Allocator API
 *
 * The allocator APIs determine the granularity and fragmentation of the
 * read buffers. They can also perform pooling and other sorts of optimizations
 * that would depend on the application-specific use case and not determinable
 * by the library.
 *
 *
 * Buffers are initially allocated by RDB using the rdb_buf_reserve_fn callback.
 * This callback is typically invoked from an rdb_rdstart() function to
 * provide the underlying storage for buffers to be read into. It is here
 * that the allocator may choose to extend the buffer beyond the requested size
 * or provide fragmented segments instead of a large contiguous segment.
 *
 * If a specific length of contiguous data is required, the rdb_seg_alloc_fn
 * callback will be invoked (typically from one of the consolidation functions)
 * and here the allocator must return a segment of at least the specified length.
 * If an existing segment (which contains data) must be extended, the
 * rdb_seg_realloc_fn is called.
 *
 * When the segment is no longer needed (i.e. its refcount is 0 and is no
 * longer needed by the library) the rdb_seg_free_fn is called to release its
 * memory. The allocator may place the segment into a pool or release it through
 * other means.
 *
 * Finally, there is the a_release() field which acts as a destructor for the
 * allocator. It signals to the allocator that no _new_ data will be allocated
 * from it.
 */

/**
 * @addtogroup rdb
 * @{
 */

typedef struct rdb_ALLOCATOR *rdb_pALLOCATOR;
typedef struct rdb_ROPESEG *rdb_pROPESEG;
typedef struct rdb_ROPEBUF *rdb_pROPEBUF;

typedef struct rdb_ROPEBUF {
    lcb_list_t segments;      /* linked list of buffer segments */
    unsigned nused;           /* bytes of data in use */
    rdb_pALLOCATOR allocator; /* pointer to allocator structure */
} rdb_ROPEBUF;

struct rdb_ROPESEG;

enum rdb_SEGFLAGS {
    RDB_ROPESEG_F_USER = 0x01, /* segment has pinned data */
    RDB_ROPESEG_F_LIB = 0x02   /* segment is in use by the library */
};

enum rdb_ALLOCID {
    RDB_ALLOCATOR_BIGALLOC = 1,
    RDB_ALLOCATOR_CHUNKED,
    RDB_ALLOCATOR_LIBCALLOC,

    /** use constants higher than this for your own allocator(s) */
    RDB_ALLOCATOR_MAX
};

/** Segment within a rope buffer */
typedef struct rdb_ROPESEG {
    lcb_list_t llnode;     /** linked list node */
    char *root;            /** Allocated buffer */
    unsigned char shflags; /** rdb_SEGFLAGS */
    unsigned char allocid; /** rdb_ALLOCID */
    unsigned nalloc;       /** total allocated length */
    unsigned nused;        /** number of bytes containing data */
    unsigned start;        /** offset where data begins */
    unsigned refcnt;       /** see ref/unref */
    rdb_pALLOCATOR allocator;
} rdb_ROPESEG;

typedef struct {
    rdb_ROPEBUF recvd; /** rope containing read data */
    rdb_ROPEBUF avail; /** rope used for subsequent network reads */
    unsigned rdsize;   /** preferred read size */
} rdb_IOROPE;

/**
 * @name Allocator API
 * @{
 */

/**
 * Extend an existing rope structure by adding additional space at the end.
 * @param allocator
 * @param buf the buffer to extend
 * @param total_capacity the number of bytes by which to extend. This should
 *        be the total new target capacity.
 *
 * It is assumed that this will only be called when it is safe to relocate
 * the contents of the underlying buffer.
 *
 * Each of the appended rdb_ROPESEG structures should initially have the
 * `RDB_ROPESEG_F_LIB` indicating they are in use by the library.
 */
typedef void (*rdb_buf_reserve_fn)(rdb_pALLOCATOR allocator, rdb_ROPEBUF *buf, unsigned total_capacity);

/**
 * Allocate a new segment.
 * The returned segment should have enough capacity for capacity bytes. The
 * returned shflags should contain RDB_ROPESEG_F_LIBUSED
 */
typedef rdb_ROPESEG *(*rdb_seg_alloc_fn)(rdb_pALLOCATOR allocator, unsigned capacity);

/**
 * This will resize the segment to be able to contain up to `capacity` bytes.
 * Any contents previously in the buffer should not be changed, though the
 * underlying `root` pointer may change.
 */
typedef rdb_ROPESEG *(*rdb_seg_realloc_fn)(rdb_pALLOCATOR allocator, rdb_ROPESEG *orig, unsigned capacity);

/** Release a previous segment allocated by rdb_seg_alloc_fn */
typedef void (*rdb_seg_free_fn)(rdb_pALLOCATOR allocator, rdb_ROPESEG *seg);

/** Allocator routines. This table is owned by the user. */
typedef struct rdb_ALLOCATOR {
    rdb_buf_reserve_fn r_reserve;
    rdb_seg_alloc_fn s_alloc;
    rdb_seg_realloc_fn s_realloc;
    rdb_seg_free_fn s_release;

    /**
     * This is called explicitly by an IOROPE structure when the allocator
     * is no longer needed. Note that the allocator should likely keep track
     * of any existing segments before freeing all its resources.
     */
    void (*a_release)(rdb_pALLOCATOR);
    void (*dump)(rdb_pALLOCATOR, FILE *);
} rdb_ALLOCATOR;

/**
 * @}
 */

/**
 * Initialize the IOROPE structure
 * @param rope a rope to use
 * @param allocator the allocator to use for allocation. The IOROPE structure
 * takes ownership of the allocator.
 */
void rdb_init(rdb_IOROPE *rope, rdb_ALLOCATOR *allocator);

/**
 * Change the allocator for the rope. This can be done at any time during
 * the application.
 * @param rope
 * @param allocator The new allocator to use
 */
void rdb_challoc(rdb_IOROPE *rope, rdb_ALLOCATOR *allocator);

void rdb_cleanup(rdb_IOROPE *ior);

/**
 * @name Basic Read API
 * @{
 */

/**
 * @brief Prepare a series of IOV structures for reading from the network.
 *
 * @param ior the IOROPE structure
 * @param[in,out] iov an array of IOV elements
 * @param niov the number of IOV elements
 */
unsigned rdb_rdstart(rdb_IOROPE *ior, nb_IOV *iov, unsigned niov);

/**
 * Indicate that some data was placed into the IOV structures populated with
 * rdstart()
 * @param ior the IOROPE structure
 * @param nr the number of total bytes placed into the IOV buffers
 */
void rdb_rdend(rdb_IOROPE *ior, unsigned nr);

/**
 * Indicate that some data at the beginning of the buffer is no longer needed
 * @param ior the rope
 * @param nr the number of bytes to discard.
 *
 * Note that if a segment was previously referenced, it is not invalidated
 * but is no longer used within the IOROPE structure itself
 */
void rdb_consumed(rdb_IOROPE *ior, unsigned nr);

/**
 * Ensure that a given chunk of data at the beginning of the rope structure
 * can fit contiguously within a single char buffer
 * @param ior the IOROPE structure
 * @param n number of bytes which must be contiguous
 */
void rdb_consolidate(rdb_IOROPE *ior, unsigned n);

/**
 * Convenience function to retrieve a pointer to the beginning of the
 * buffer. The pointer is guaranteed to point to n contiguous bytes.
 */
char *rdb_get_consolidated(rdb_IOROPE *ior, unsigned n);

/**
 * @}
 */

/**
 * @name Extended Read API
 * @{
 */

/**
 * Copy n bytes of data from the beginning of the rope structure into the
 * buffer pointed to by buf
 */
void rdb_copyread(rdb_IOROPE *ior, void *buf, unsigned n);

/** Get the pointer to the beginning of the buffer in the IOROPE. */
#define rdb_refread(ior)                                                                                               \
    ((LCB_LIST_ITEM(ior->recvd.segments.next, rdb_ROPESEG, llnode))->root +                                            \
     (LCB_LIST_ITEM(ior->recvd.segments.next, rdb_ROPESEG, llnode))->start)

/**
 * Populate an array of ROPESEG and IOV structures with data from the IOROPE.
 * @param ior
 * @param[out] iov the iov array containing buffer offsets
 * @param[out] segs an array to contain pointers to the segments for the IOVs
 * @param[in] nelem number of elements in the array
 * @param[in] ndata number of bytes to populate the arrays with
 * @return the number of IOV elements actually used, or -1 if the arrays
 *  did not contain enough elements to contain the IOVs completely.
 */
int rdb_refread_ex(rdb_IOROPE *ior, nb_IOV *iov, rdb_ROPESEG **segs, unsigned nelem, unsigned ndata);

/**
 * Get the maximum contiguous size of the current input. This is the size of
 * data which may be read efficiently via 'get_consolidated' without actually
 * reallocating memory
 */
unsigned rdb_get_contigsize(rdb_IOROPE *ior);

/**
 * Increase the reference count on the segment. When a reference count is set
 * on a segment, internal functions will no longer be able to reuse any of its
 * contents (however further data may be read _into_ the segment). The contents
 * actually reserved are pinned to the contextual IOV structure which should
 * be available when receiving the segment.
 */
void rdb_seg_ref(rdb_ROPESEG *seg);

/**
 * When you are done with the segment (or the borrowed IOVs thereof) call this.
 * The segment may no longer be accessed by the caller.
 */
void rdb_seg_unref(rdb_ROPESEG *seg);

/** @private */
#define rdb_seg_recyclable(seg) (((seg)->shflags & RDB_ROPESEG_F_USER) == 0)

/**
 * Get the first segment. Useful for associating a contiguous consolidated
 * read without actually using refread
 * @param ior the rope structure
 * @return the segment of the contiguous read
 */
#define rdb_get_first_segment(ior) RDB_SEG_FIRST(&(ior)->recvd)

/**
 * @}
 */

/**
 * @name Utility Macros
 * @{
 */

/** number of unused bytes remaining in the segment */
#define RDB_SEG_SPACE(seg) (seg)->nalloc - ((seg)->nused + (seg)->start)

/** pointer to the first used byte in the segment */
#define RDB_SEG_RBUF(seg) (seg)->root + (seg)->start

/** pointer to the first available unused byte in the segment */
#define RDB_SEG_WBUF(seg) (seg)->root + (seg)->start + (seg)->nused

/** last segment in the rope structure */
#define RDB_SEG_LAST(rope)                                                                                             \
    (LCB_LIST_TAIL(&(rope)->segments)) ? LCB_LIST_ITEM(LCB_LIST_TAIL(&(rope)->segments), rdb_ROPESEG, llnode) : NULL

/** first segment in the rope structure */
#define RDB_SEG_FIRST(rope)                                                                                            \
    (LCB_LIST_HEAD(&(rope)->segments)) ? LCB_LIST_ITEM(LCB_LIST_HEAD(&(rope)->segments), rdb_ROPESEG, llnode) : NULL
/**
 * @}
 */

#define rdb_get_nused(ior) (ior)->recvd.nused

/**
 * Add data into the read buffer. This is primarily used for testing and does
 * the equivalent of a network "Read".
 * @param ior The iorope structure
 * @param buf The buffer to copy
 * @param nbuf Size of the buffer
 */
void rdb_copywrite(rdb_IOROPE *ior, void *buf, unsigned nbuf);

/**
 * Allocator APIs
 * Returns the big or "Default" allocator.
 */
LCB_INTERNAL_API
rdb_ALLOCATOR *rdb_bigalloc_new(void);

/**
 * Returns a chunked allocator which will attempt to allocated readahead buffers
 * of a specified size
 * @param chunksize the desired chunk/segment size
 */
LCB_INTERNAL_API
rdb_ALLOCATOR *rdb_chunkalloc_new(unsigned chunksize);

/**
 * Returns a simple allocator which merely proxies to malloc/calloc/realloc/free
 */
LCB_INTERNAL_API
rdb_ALLOCATOR *rdb_libcalloc_new(void);

/**
 * Dump information about the iorope structure to a file
 * @param ior The rope structure to dump
 * @param fp The destination file.
 */
void rdb_dump(const rdb_IOROPE *ior, FILE *fp);

#ifdef __cplusplus
}
#endif
#endif

/**
 * @}
 */
