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
#ifndef CBC_PILLOWFIGHT_LOC_H
#define CBC_PILLOWFIGHT_LOC_H

namespace Pillowfight
{

// This class copy/pasted from Subdoc (which I also wrote)
class Loc
{
  public:
    const char *at;
    size_t length;

    Loc() : at(NULL), length(0) {}

    Loc(const lcb_IOV &iov) : at(reinterpret_cast< const char * >(iov.iov_base)), length(iov.iov_len) {}

    Loc(const char *s, size_t n)
    {
        assign(s, n);
    }

    lcb_IOV to_iov() const
    {
        lcb_IOV ret;
        ret.iov_base = const_cast< char * >(at);
        ret.iov_len = length;
        return ret;
    }

    enum OverlapMode { NO_OVERLAP = 0, OVERLAP = 1 };

    void assign(const char *s, size_t n)
    {
        at = s;
        length = n;
    }

    /**
     * Modifies the object so that it ends where `until` begins.
     *
     * The object will have a starting position of the base buffer, and will
     * span until the `until` buffer.
     *
     * Example:
     * @code
     * BASE     = ABCDEFGHIJ
     * UNTIL    =      FGH
     * THIS     = ABCDE
     * @endcode
     *
     * @param base The common buffer
     * @param until position at where this buffer should end
     * @param overlap Whether the end should overlap with the first byte of `until`
     */
    void end_at_begin(const Loc &base, const Loc &until, OverlapMode overlap)
    {
        at = base.at;
        length = until.at - base.at;
        if (overlap == OVERLAP) {
            length++;
        }
    }

    /**
     * Modifies the object so that it begins where `until` ends.
     *
     * The buffer will have an end position matching the end of the base buffer,
     * and will end where `from` ends.
     *
     * Example:
     * @code
     * BASE     = ABCDEFGHIJ
     * FROM     =   CDE
     * THIS     =      FGHIJ
     * @endcode
     *
     * @param base The common buffer
     * @param from A buffer whose end should be used as the beginning of the
     *        current buffer
     * @param overlap Whether the current buffer should overlap `until`'s last
     *        byte
     */
    void begin_at_end(const Loc &base, const Loc &from, OverlapMode overlap)
    {
        at = from.at + from.length;
        length = base.length - (at - base.at);
        if (overlap == OVERLAP) {
            at--;
            length++;
        }
    }

    /**
     * Modifies the object so that it begins where `from` begins.
     *
     * The buffer will have an end position matching the end of the base buffer
     * and will begin where `from` begins
     *
     * Example:
     * @code
     * BASE     = ABCDEFGHIJ
     * FROM     =    DEF
     * THIS     =    DEFGHIJ
     * @endcode
     *
     * @param base Common buffer
     * @param from The begin position
     */
    void begin_at_begin(const Loc &base, const Loc &from)
    {
        at = from.at;
        length = base.length - (from.at - base.at);
    }

    /**
     * Modifies the object so that it ends where `until` ends.
     *
     * The buffer will have a start position of `base` and an end position of
     * `until.
     *
     * Example
     * @code
     * BASE     = ABCDEFGHIJ
     * UNTIL    =     EFG
     * THIS     = ABCDEFG
     * @endcode
     *
     * @param base
     * @param until
     * @param overlap
     */
    void end_at_end(const Loc &base, const Loc &until, OverlapMode overlap)
    {
        at = base.at;
        length = (until.at + until.length) - base.at;
        if (overlap == NO_OVERLAP) {
            length--;
        }
    }

    bool empty() const
    {
        return length == 0;
    }

    std::string to_string() const
    {
        if (!empty()) {
            return std::string(at, length);
        } else {
            return std::string();
        }
    }

    // Move buffer start ahead n bytes
    void ltrim(size_t n)
    {
        at += n;
        length -= n;
    }

    // Move buffer end back n bytes
    void rtrim(size_t n)
    {
        length -= n;
    }

    // Added for pillowfight
    //
    // Set buffer to end where 'loc' begins, while not touching the beginning
    // of the buffer
    void rtrim_to(const Loc &loc)
    {
        assert(loc.at > at);
        size_t diff = loc.at - at;
        length = diff;
    }

    // Added for pillowfight
    bool contains(const Loc &sub) const
    {
        return sub.at >= at &&         // Begins at or after our beginning
               sub.at < at + length && // begins before or at the end
               sub.at + sub.length <= at + length;
    }

    static void dumpIovs(const std::vector< lcb_IOV > &vecs)
    {
        for (size_t ii = 0; ii < vecs.size(); ii++) {
            const lcb_IOV &iov = vecs[ii];
            printf("IOV[%lu]. Buf=%p. Len=%lu. Content=%.*s\n", (unsigned long int)ii, (void *)iov.iov_base,
                   (unsigned long int)iov.iov_len, (int)iov.iov_len, (const char *)iov.iov_base);
        }
    }
    static void dumpIovs(const std::vector< Loc > &vecs)
    {
        for (size_t ii = 0; ii < vecs.size(); ii++) {
            const Loc &loc = vecs[ii];
            std::string s = loc.to_string();
            printf("Loc[%lu]. Buf=%p. Len=%lu. Content=%s\n", (unsigned long int)ii, (void *)loc.at,
                   (unsigned long int)loc.length, s.c_str());
        }
    }
};
} // namespace Pillowfight
#endif
