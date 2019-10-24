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

#ifndef LCBIO_TIMER_CXX
#define LCBIO_TIMER_CXX

#include <lcbio/timer-ng.h>
#include <cstdlib>

namespace lcb
{
namespace io
{

class SimpleTimer
{
  public:
    typedef void(Callback)(void *);
    SimpleTimer(lcbio_pTABLE iot, void *data, Callback cb) : inner(lcbio_timer_new(iot, data, cb)) {}
    ~SimpleTimer()
    {
        release();
    }
    void release()
    {
        if (inner != NULL) {
            lcbio_timer_destroy(inner);
            inner = NULL;
        }
    }
    void signal()
    {
        lcbio_async_signal(inner);
    }
    void cancel()
    {
        lcbio_timer_disarm(inner);
    }
    bool is_armed() const
    {
        return lcbio_timer_armed(inner);
    }
    void rearm(uint32_t usec)
    {
        lcbio_timer_rearm(inner, usec);
    }
    void arm_if_disarmed(uint32_t usec)
    {
        if (!is_armed()) {
            rearm(usec);
        }
    }
    void dump(FILE *fp) const
    {
        lcbio_timer_dump(inner, fp);
    }

  private:
    lcbio_pTIMER inner;
    SimpleTimer(const SimpleTimer &);
};

template < typename T, void (T::*M)(void) > class Timer : public SimpleTimer
{
  public:
    Timer(lcbio_pTABLE iot, T *ptr) : SimpleTimer(iot, ptr, cb) {}

    ~Timer()
    {
        release();
    }

  private:
    static void cb(void *arg)
    {
        T *obj = reinterpret_cast< T * >(arg);
        (obj->*M)();
    }
    Timer(const Timer &);
};

} // namespace io
} // namespace lcb
#endif
