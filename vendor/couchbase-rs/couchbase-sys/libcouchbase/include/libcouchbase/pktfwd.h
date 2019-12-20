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

#ifndef LCB_PKTFWD_H
#define LCB_PKTFWD_H
#ifdef __cplusplus
extern "C" {
#endif

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-pktfwd Raw packet forwarding and dispatch routines
 * @brief These functions perform packet forwarding functions to send and
 * receive raw packets
 *
 * @addtogroup lcb-pktfwd
 * @{
 */

typedef struct rdb_ROPESEG *lcb_BACKBUF;

/**@brief Request for forwarding a packet
 * This structure is passed to the lcb_pktfwd3() function.
 */
typedef struct {
    int version;
    /**This structure should be initialized to a packet. The packet may be
     * in the form of a contiguous buffer to be copied (lcb_VALBUF::vtype should
     * be LCB_KV_COPY), a contiguous buffer to be maintained by the user
     * (lcb_VALBUF::vtype should be LCB_KV_CONTIG) or an array of lcb_IOV
     * structures (which should not be copied; lcb_VALBUF::vtype should be
     * LCB_KV_IOV).
     *
     * This field must contain a complete packet including any extras and body
     * associated with it.
     *
     * If the buffer(s) passed are not copied, you must wait for the
     * lcb_pktflushed_callback to be invoked to signal that the buffer is no
     * longer needed and may be released back to the application.
     *
     * @warning
     * The first 24 bytes of the buffer (i.e. the memcached header)
     * **will be modified**. Currently this is used to modify the `opaque` field
     * within the header.
     */
    lcb_VALBUF vb;

    /**
     * Whether to direct this command to a specific server. This should be
     * set if the packet itself doesn't contain any mapping information; and
     * should _not_ be used on normal key access commands, since key access
     * commands should be mapped to the appropriate server via the vbucket
     * mappet.
     *
     * The server should be specified in the #server_index field */
    char nomap;

    /**
     * @brief Specify server index for the command.
     * Only valid if #nomap is specified. */
    lcb_U16 server_index;
} lcb_CMDPKTFWD;

/**@brief Response structure containing the response for a packet */
typedef struct {
    /** Version of the response structure */
    int version;

    /**Pointer to the memcached header. This pointer is guaranteed to be
     * properly aligned as a protocol_binary_response_header structure and will
     * typically be quicker to access than analyzing the header as found
     * inside the #iovs field.
     *
     * This field may be NULL if the callback is invoked with an error.*/
    const lcb_U8 *header;

    /**Array of lcb_IOV structures containing the response packet. The number
     * of items in this array is contained in the #nitems field.
     *
     * Note that you may modify the contents of the buffers themselves (i.e.
     * the memory pointed to by lcb_IOV::iov_base.
     *
     * When a buffer is no longer needed, lcb_backbuf_unref() should be called
     * on its associated lcb_BACKBUF structure - which is located at the same
     * array index within the #bufs field (for example, `iovs[n]` will have
     * its associated lcb_BACKBUF structure at `bufs[n]`.
     */
    lcb_IOV *iovs;

    /**Contains the backing lcb_BACKBUF objects which control the allocation
     * lifespan of their associated elements in the #iovs field. */
    lcb_BACKBUF *bufs;

    /** The number of items in the #iovs and #bufs array. Currently this is
     * always `1` but may change in the future.*/
    unsigned nitems;
} lcb_PKTFWDRESP;

/**
 * @uncommitted
 *
 * @brief Forward a raw memcached packet to the cluster.
 *
 * @details
 * This function will process a buffer containing a raw complete memcached packet
 * to the cluster. Once the reply for the packet has been received, the
 * lcb_pktfwd_callback will be invoked with the `cookie` argument and the
 * response data.
 *
 * If using user-allocated buffers, an additional lcb_pktflushed_callback will
 * be invoked once the library no longer needs the buffers for the packet. Note
 * that no assumption should be made on the order of invocation for these two
 * callbacks - thus it is recommended to implement a reference counting scheme
 * on the buffer and decrement the count for each invocation of the callback.
 *
 * Note that not all memcached commands may be forwarded to this function.
 * Specifically, any packet passed to this function:
 *
 * 1. Must contain a single key after the `header` and `extras` fields. This
 *    means that commands like _OBSERVE_ and _GET_CLUSTER_CONFIG_ are not
 *    currently supported.
 * 2. The key must be mappable via the vBucket mapping algorithm. This means
 *    that commands such as _STATS_ are not currently supported.
 * 3. Must receive exactly one reply from the server. This means that "quiet"
 *    versions of commands, such as _GETQ_ and _SETQ_ are not currently
 *    supported.
 *
 * If you wish to forward one of the unsupported commands you may use the
 * higher level entry points (i.e. lcb_stats3(), lcb_observe3_ctxnew(), etc)
 * and manually reconstruct the output packet based on the callbacks received.
 *
 * Note additionally that the _opaque_ field within the packet will be modified
 * by the library. You should store the current opaque value in the structure
 * pointed to by the `cookie` parameter and then re-assign it once the packet
 * callback has been delivered.
 *
 * Forwarded packets are subject to the same lifecycle as normal commands. This
 * means they may be retried and remapped to other nodes upon topology changes,
 * and that they are subject to the same operation timeout semantics.
 *
 * @param instance the handle
 * @param cookie a pointer to be passed to the callbacks for this packet
 * @param cmd the command structure containing the buffer mappings for this
 * packet.
 *
 * @return LCB_SUCCESS on success, LCB_INCOMPLETE_PACKET if the packet passed
 * does not contain the full body. Other error codes may be returned as well
 *
 * @see lcb_set_pktfwd_callback lcb_set_pktflushed_callback, mc_forward_packet
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_pktfwd3(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDPKTFWD *cmd);

/**
 * Callback invoked when a response packet has arrived for a request
 * @param instance
 * @param cookie Opaque pointer associated with the request
 * @param err If a response packet could not be obtained, this contains the reason
 * @param resp Response structure. This is always present if there is a reply
 * from the server.
 *
 * The lcb_PKTFWDRESP::bufs structures are considered to be invalid after the
 * callback has exited because lcb_backbuf_unref() will be called on each of
 * them. To ensure they remain valid in your application outside the callback,
 * invoke lcb_backbuf_ref() on the required lcb_BACKBUF structures and then
 * once they are no longer needed use lcb_backbuf_unref()
 */
typedef void (*lcb_pktfwd_callback)(lcb_INSTANCE *instance, const void *cookie, lcb_STATUS err, lcb_PKTFWDRESP *resp);

/**
 * Callback invoked when the request buffer for a packet is no longer required.
 * @param instance
 * @param cookie The cookie associated with the request data
 */
typedef void (*lcb_pktflushed_callback)(lcb_INSTANCE *instance, const void *cookie);

/**
 * @uncommitted
 * @brief Set the callback to be invoked when a response to a
 * forwarded packet has been received.
 *
 * @param instance the handle
 * @param callback the callback to install
 * @return the old callback. If `callback` is NULL, this function just returns
 * the existing callback
 */
LIBCOUCHBASE_API
lcb_pktfwd_callback lcb_set_pktfwd_callback(lcb_INSTANCE *instance, lcb_pktfwd_callback callback);

/**
 * @uncommitted
 *
 * @brief Set the callback to be invoked when the buffer data supplied to the
 * packet forwarding function is no longer needed.
 *
 * @param instance the handle
 * @param callback the callback to install
 * @return the old callback. If `callback` is NULL then this function just
 * returns the existing callback.
 */
LIBCOUCHBASE_API
lcb_pktflushed_callback lcb_set_pktflushed_callback(lcb_INSTANCE *instance, lcb_pktflushed_callback callback);

/**
 * @name Response Buffer Handling
 *
 * @details
 *
 * The data received as part of a response buffer is _mapped_ by an lcb_IOV
 * structure, however the actual allocated data is held together by an
 * opaque lcb_BACKBUF structure. This structure allows multiple IOVs to exist
 * concurrently within the same block of allocated memory (with different
 * offsets and sizes). The lcb_BACKBUF structure functions as an opaque
 * reference counted object which controls the duration of the memmory to which
 * the IOV is mapped.
 *
 * From an API perspective, there is a one-to-one correlation between an IOV
 * and an lcb_BACKBUF
 *
 * @{
 */

/**
 * Indicate that the lcb_BACKBUF object which provides storage for an IOV's
 * data pointer will need to remain valid until lcb_backbuf_unref() is called.
 *
 * This function may be called from an lcb_pktfwd_callback handler to allow
 * the contents of the buffer to persist outside the specific callback
 * invocation.
 */
LIBCOUCHBASE_API
void lcb_backbuf_ref(lcb_BACKBUF buf);

/**
 * Indicate that the IOV backed by the specified `buf` is no longer required
 * @param buf the buffer which backs the IOV
 * After the buffer has been unreferenced, the relating IOV may no longer be
 * accessed
 */
LIBCOUCHBASE_API
void lcb_backbuf_unref(lcb_BACKBUF buf);
/**@}*/

/**@}*/
#ifdef __cplusplus
}
#endif
#endif
