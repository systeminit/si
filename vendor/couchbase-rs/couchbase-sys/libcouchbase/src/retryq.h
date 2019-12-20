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

#ifndef LCB_RETRYQ_H
#define LCB_RETRYQ_H

#include <lcbio/lcbio.h>
#include <lcbio/timer-ng.h>
#include <mc/mcreq.h>
#include "list.h"

#ifdef __cplusplus

/**
 * @file
 * @brief Retry Queue
 *
 * @defgroup lcb-retryq Retry Queue
 *
 * @details
 * Retry queue for operations. The retry queue accepts commands which have
 * previously failed and aims to retry them within a specified interval.
 *
 * @addtogroup lcb-retryq
 * @{
 */

namespace lcb
{

struct RetryOp;

class RetryQueue
{
  public:
    /**
     * @brief Create a new retry queue.
     * The retry queue serves as an asynchronous poller which will retry operations
     * with a certain throttle.
     *
     * @param cq The parent cmdqueue object
     * @param table used to create the timer
     * @param settings Used for logging and interval timeouts
     * @return A new retry queue object
     */
    RetryQueue(mc_CMDQUEUE *cq_, lcbio_pTABLE, lcb_settings *);
    ~RetryQueue();

    /**
     * @brief Enqueue a failed command
     * @param detchpkt A detached packet allocated with mcreq_renew_packet()
     * @param err the error code which caused the packet to be placed inside the
     * retry queue. Depending on the error code and subsequent errors, this code
     * will ultimately be sent back to the operation callback when the result is
     * final.
     *
     * @attention Only simple commands containing vBuckets may be placed here.
     * Complex commands such as OBSERVE or STAT may _not_ be retried through this
     * mechanism. Additionally since this relies on determining a vBucket master
     * it may _not_ be used for memcached buckets (which is typically OK, as we only
     * map things here as a response for a not-my-vbucket).
     */
    void add(mc_EXPACKET *detchpkt, lcb_STATUS err, errmap::RetrySpec *spec)
    {
        add(detchpkt, err, spec, 0);
    }

    /**
     * Retries the given packet as a result of a NOT_MY_VBUCKET failure. Currently
     * this is provided to allow for different behavior when handling these types
     * of responses.
     *
     * @param detchpkt The new packet
     */
    void nmvadd(mc_EXPACKET *detchpkt);
    void ucadd(mc_EXPACKET *pkt);

    /**
     * @brief Retry all queued operations
     *
     * This should normally be called when a new server connection is made or when
     * a new configuration update has arrived.
     *
     * @param rq The queue
     */
    void signal();

    /**
     * If this packet has been previously retried, this obtains the original error
     * which caused it to be enqueued in the first place. This eliminates spurious
     * timeout errors which mask the real cause of the error.
     *
     * @param pkt The packet to check for
     * @return An error code, or LCB_SUCCESS if the packet does not have an
     * original error.
     */
    static lcb_STATUS error_for(const mc_PACKET *);

    /**
     * Dumps the packets inside the queue
     * @param rq The request queue
     * @param fp The file to which the output should be written to
     */
    void dump(FILE *fp, mcreq_payload_dump_fn dumpfn);

    /**
     * @brief Check if there are operations to retry
     * @param ignore_cfgreq if true, consider queue with single 0xb5 request as empty
     * @return nonzero if there are pending operations
     */
    bool empty(bool ignore_cfgreq = false) const;

    /**
     * @brief Reset all timeouts on the retry queue.
     *
     * This will defer the timeout to start from the current time rather than
     * the time it was initially placed in the queue. Items are usually placed
     * in the queue after a network failure or similar; however one exception
     * is items which are placed in the queue via the scheduling APIs directly
     * (if there is no host for the command's vBucket)
     *
     * @param now The time to use
     */
    void reset_timeouts(uint64_t now = 0);

    /** Event loop tick */
    inline void tick();

    inline void add_fallback(mc_PACKET *pkt);

  private:
    void erase(RetryOp *);
    void fail(RetryOp *, lcb_STATUS);
    void schedule(hrtime_t now = 0);
    void flush(bool throttle);
    void update_trytime(RetryOp *op, hrtime_t now = 0);
    hrtime_t get_retry_interval() const;
    lcb_INSTANCE *get_instance() const
    {
        return reinterpret_cast< lcb_INSTANCE * >(cq->cqdata);
    }

    enum AddOptions { RETRY_SCHED_IMM = 0x01 };
    void add(mc_EXPACKET *pkt, lcb_STATUS, errmap::RetrySpec *, int options);

    /** List of operations in retry ordering. Sorted by 'crtime' */
    lcb_list_t schedops;
    /** List of operations in timeout ordering. Ordered by 'start_time' */
    lcb_list_t tmoops;
    /** Parent command queue */
    mc_CMDQUEUE *cq;
    lcb_settings *settings;
    lcbio_pTIMER timer;
};

} // namespace lcb
/**@}*/
#endif
#endif
