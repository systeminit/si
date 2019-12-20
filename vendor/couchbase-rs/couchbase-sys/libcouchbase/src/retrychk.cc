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

#include "internal.h"

int
lcb_should_retry(const lcb_settings *settings, const mc_PACKET *pkt, lcb_STATUS err)
{
    unsigned policy;
    unsigned mode;
    protocol_binary_request_header hdr;

    mcreq_read_hdr(pkt, &hdr);

    switch (hdr.request.opcode) {
    /* None of these commands can be 'redistributed' to other servers */
    case PROTOCOL_BINARY_CMD_GET_REPLICA:
    case PROTOCOL_BINARY_CMD_FLUSH:
    case PROTOCOL_BINARY_CMD_OBSERVE:
    case PROTOCOL_BINARY_CMD_OBSERVE_SEQNO:
    case PROTOCOL_BINARY_CMD_STAT:
    case PROTOCOL_BINARY_CMD_VERBOSITY:
    case PROTOCOL_BINARY_CMD_VERSION:
    case PROTOCOL_BINARY_CMD_NOOP:
        return 0;
    }

    if (err == LCB_ETIMEDOUT || err == LCB_MAP_CHANGED) {
        /* We can't exceed a timeout for ETIMEDOUT */
        /* MAP_CHANGED is sent after we've already called this function on the
         * packet once before */
        return 0;
    } else if (err == LCB_AUTH_ERROR) {
        /* spurious auth error */
        return 1;
    } else if (err == LCB_NOT_MY_VBUCKET) {
        mode = LCB_RETRY_ON_VBMAPERR;
    } else if (err == LCB_MAX_ERROR) {
        /* special, topology change */
        mode = LCB_RETRY_ON_TOPOCHANGE;
    } else if (LCB_EIFNET(err)) {
        mode = LCB_RETRY_ON_SOCKERR;
    } else {
        /* invalid mode */
        return 0;
    }
    policy = settings->retry[mode];

    if (policy == LCB_RETRY_CMDS_ALL) {
        return 1;
    } else if (policy == LCB_RETRY_CMDS_NONE) {
        return 0;
    }

    /** read the header */
    switch (hdr.request.opcode) {

    /* get is a safe operation which may be retried */
    case PROTOCOL_BINARY_CMD_GET:
    case PROTOCOL_BINARY_CMD_SUBDOC_GET:
    case PROTOCOL_BINARY_CMD_SUBDOC_EXISTS:
    case PROTOCOL_BINARY_CMD_SUBDOC_MULTI_LOOKUP:
        return policy & LCB_RETRY_CMDS_GET;

    case PROTOCOL_BINARY_CMD_ADD:
        return policy & LCB_RETRY_CMDS_SAFE;

    /* mutation operations are retriable so long as they provide a CAS */
    case PROTOCOL_BINARY_CMD_SET:
    case PROTOCOL_BINARY_CMD_REPLACE:
    case PROTOCOL_BINARY_CMD_APPEND:
    case PROTOCOL_BINARY_CMD_PREPEND:
    case PROTOCOL_BINARY_CMD_DELETE:
    case PROTOCOL_BINARY_CMD_UNLOCK_KEY:
    case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_ADD_UNIQUE:
    case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_FIRST:
    case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_LAST:
    case PROTOCOL_BINARY_CMD_SUBDOC_COUNTER:
    case PROTOCOL_BINARY_CMD_SUBDOC_DELETE:
    case PROTOCOL_BINARY_CMD_SUBDOC_DICT_UPSERT:
    case PROTOCOL_BINARY_CMD_SUBDOC_REPLACE:
    case PROTOCOL_BINARY_CMD_SUBDOC_DICT_ADD:
    case PROTOCOL_BINARY_CMD_SUBDOC_MULTI_MUTATION:
        if (hdr.request.cas) {
            return policy & LCB_RETRY_CMDS_SAFE;
        } else {
            return 0;
        }

    /* none of these commands accept a CAS, so they are not safe */
    case PROTOCOL_BINARY_CMD_INCREMENT:
    case PROTOCOL_BINARY_CMD_DECREMENT:
    case PROTOCOL_BINARY_CMD_TOUCH:
    case PROTOCOL_BINARY_CMD_GAT:
    case PROTOCOL_BINARY_CMD_GET_LOCKED:
    default:
        return 0;
    }
}
