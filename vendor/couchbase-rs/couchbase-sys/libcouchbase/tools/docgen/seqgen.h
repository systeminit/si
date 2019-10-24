/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2016-2019 Couchbase, Inc.
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

#ifndef CBC_PILLOWFIGHT_SEQGEN_H
#define CBC_PILLOWFIGHT_SEQGEN_H

namespace Pillowfight
{

/**
 * Stateful sequence generator, divides sequences based on number of threads.
 *
 * On input, it takes a total range amount as well as a number of threads
 * to serve the range, and finally, the index of the current thread.
 *
 * There is one generator per thread, as it is stateful
 */
class SeqGenerator
{
  public:
    /** Constructor for sequential key generators */
    SeqGenerator(uint32_t start, uint32_t end, int num_workers, int cur_worker)
    {
        offset = start;
        uint32_t total = end - start;
        total_self = total / num_workers;
        locked = std::vector< bool >(total_self, false);
        lnum = 0;
        offset += total_self * cur_worker;
        rnum = 0;
        sequential = true;
        curr_seqno = 0;
    }

    /** Initialize as a random range */
    SeqGenerator(uint32_t start, uint32_t end)
    {
        total_self = end - start;
        locked = std::vector< bool >(total_self, false);
        lnum = 0;
        offset = start;
        rnum = 0;
        curr_seqno = 0;
        sequential = false;

        for (int ii = 0; ii < 8192; ii++) {
            seqpool.push_back(rand());
        }
    }

    /**
     * Get the next sequence in range
     * @return A number appropriate for the current sequence
     */
    uint32_t next()
    {
        if (sequential) {
            rnum++;
            rnum %= total_self;
            rnum += offset;
            return rnum;

        } else {
            rnum += seqpool[curr_seqno];
            curr_seqno++;
            if (curr_seqno >= seqpool.size()) {
                curr_seqno = 0;
            }
            uint32_t seq = rnum;
            seq %= total_self;
            seq += offset;
            return seq;
        }
    }

    uint32_t checkout()
    {
        uint32_t num;
        num = next();
        if (lnum == locked.size()) {
            lnum = 0;
            std::fill(locked.begin(), locked.end(), false);
        } else {
            while (locked[num - offset]) {
                num = next();
            }
        }
        locked[num - offset] = true;
        lnum++;
        return num;
    }

    void checkin(uint32_t num)
    {
        if (num - offset < locked.size()) {
            locked[num - offset] = 0;
        }
    }

    uint32_t maxItems() const
    {
        return total_self;
    }

  private:
    bool sequential;
    std::vector< uint32_t > seqpool;
    std::vector< bool > locked; // lock markers
    uint32_t lnum;              // number of locked keys
    uint32_t rnum;              // internal iterator
    uint32_t offset;            // beginning numerical offset
    uint32_t total_self;        // maximum value of iterator
    uint32_t curr_seqno;
};
} // namespace Pillowfight
#endif
