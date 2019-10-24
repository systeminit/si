/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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

#ifndef LCB_METRICS_H
#define LCB_METRICS_H

#ifdef __cplusplus
extern "C" {
#endif

struct lcb_METRICS_st;

typedef struct lcb_IOMETRICS_st {
    const char *hostport;
    lcb_SIZE io_close;
    lcb_SIZE io_error;
    lcb_SIZE bytes_sent;
    lcb_SIZE bytes_received;
} lcb_IOMETRICS;

typedef struct lcb_SERVERMETRICS_st {
    /** IO Metrics for the underlying socket */
    lcb_IOMETRICS iometrics;

    /** Number of packets sent on this server */
    lcb_SIZE packets_sent;

    /** Number of packets read on this server */
    lcb_SIZE packets_read;

    /** Total number of packets placed in send queue */
    lcb_SIZE packets_queued;

    /** Total number of bytes placed in send queue */
    lcb_SIZE bytes_queued;

    /**
     * Number of packets which failed on this server (i.e. as a result
     * of a timeout/network error or similar)
     */
    lcb_SIZE packets_errored;

    /** Number of packets which timed out. Subset of packets_errored */
    lcb_SIZE packets_timeout;

    /** Number of packets received which were timed out or otherwise cancelled */
    lcb_SIZE packets_ownerless;

    /** Number of NOT_MY_VBUCKET replies received */
    lcb_SIZE packets_nmv;
} lcb_SERVERMETRICS;

typedef struct lcb_METRICS_st {
    lcb_SIZE nservers;
    const lcb_SERVERMETRICS **servers;

    /** Number of times a packet entered the retry queue */
    lcb_SIZE packets_retried;
} lcb_METRICS;

#ifdef __cplusplus
}
#endif

#endif
