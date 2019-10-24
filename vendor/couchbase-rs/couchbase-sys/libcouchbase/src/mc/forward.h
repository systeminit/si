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

#ifndef MC_FORWARD_H
#define MC_FORWARD_H
#ifdef __cplusplus
extern "C" {
#endif

#include "iovcursor.h"

/**
 * Copy over the entire packet to the internal buffers. Input buffer is
 * temporary.
 */
#define MC_FWD_OPT_COPY 0x01

/**
 * The server to send to is already set as `pl`. Don't perform vbucket mapping.
 */
#define MC_FWD_OPT_NOMAP 0x02

void mc_iovinfo_init(mc_IOVINFO *info, const nb_IOV *iov, unsigned niov);

/**
 * Forward a packet to an upstream server.
 * @param[in] cq the command queue
 * @param[in] ctx a queue context used for scheduling/flushing the packets
 * @param[in] info an 'IOVINFO' structure. See the structure documentation for
 *        more details.
 * @param[out] pkt a pointer to a packet, set to the resultant packet structure
 * @param[out] pl the pipeline this packet is mapped to
 * @param options Options modifying the behavior of this operation
 * @return LCB_SUCCESS or an error code.
 *
 * Currently only some commands will successfully make sense to forward. This
 * function only handles the lower level aspect of actually allocating or
 * reserving the buffers required to forward the packet, but not actually
 * handling the received data for the callbacks themselves.
 *
 * Note that this function does not currently "Collapse" consectutive IOV
 * structures. Additionally the following should be noted:
 *
 * <ol>
 * <li>
 *     If the first IOV does not contain a contiguous buffer of the
 *     { header, extras, key }, then it will be copied into a library-based
 *     buffer.
 * </li>
 *
 * <li>
 *     If the total number of IOVs is greater than two, then niov-1 IOV
 *     structures will be allocated via 'malloc'. This may not always happen
 *     if the header _itself_ is fragmented
 * </li>
 *
 * <li>The first 24 bytes of the header WILL BE MODIFIED by the library.
 *     This will be used to modify the 'opaque' and 'vbucket' fields.
 *     Take this into note if you need to keep track of their original values.
 * </li>
 *
 * <li>Check the 'niov' counter to see if it is 0. If it's 0 then you should
 *     make sure to reset the counter.
 * </li>
 * </ol>
 */
lcb_STATUS mc_forward_packet(mc_CMDQUEUE *cq, mc_IOVINFO *info, mc_PACKET **pkt, mc_PIPELINE **pl, int options);

#ifdef __cplusplus
}
#endif
#endif
