/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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

#include "config.h"
#include "settings.h"
#include <libcouchbase/couchbase.h>

#ifndef HAVE_GETHRTIME

#include <stdlib.h>
#include <time.h>
#if defined(__MACH__) && defined(__APPLE__)
#include <mach/mach_time.h>
#endif

#ifdef __linux__
#undef HAVE_CLOCK_GETTIME
#define HAVE_CLOCK_GETTIME 1
#endif

#define CLOCK_START_OFFSET (LCB_S2NS(3600 * 24))

hrtime_t gethrtime(void)
{
#ifdef __APPLE__

    /* Most things expect a pretty large timestamp - even though a smaller one
     * may be perfectly valid. Initialize the default with an offset of one day,
     * in nanoseconds
     */

    /* Use the various mach stuff:
     * https://developer.apple.com/library/mac/qa/qa1398/_index.html */

    static uint64_t start = 0;
    uint64_t now;
    static mach_timebase_info_data_t tmbi;

    if (start == 0) {
        start = mach_absolute_time();
        mach_timebase_info(&tmbi);
    }

    now = mach_absolute_time();
    return ((now - start) * tmbi.numer / tmbi.denom) + CLOCK_START_OFFSET;

#elif defined(HAVE_CLOCK_GETTIME)
    struct timespec tm;
    lcb_assert(clock_gettime(CLOCK_MONOTONIC, &tm) != -1);
    return (((hrtime_t)tm.tv_sec) * 1000000000) + (hrtime_t)tm.tv_nsec;
#elif defined(HAVE_GETTIMEOFDAY)

    hrtime_t ret;
    struct timeval tv;
    if (gettimeofday(&tv, NULL) == -1) {
        return -1;
    }

    ret = (hrtime_t)tv.tv_sec * 1000000000;
    ret += (hrtime_t)tv.tv_usec * 1000;
    return ret;
#elif defined(HAVE_QUERYPERFORMANCECOUNTER)
    double ret;
    // To fix the potential race condition for the local static variable,
    // gethrtime should be called in a global static variable first.
    // It will guarantee the local static variable will be initialized
    // before any thread calls the function.
    static LARGE_INTEGER pf = {0};
    static double freq;
    LARGE_INTEGER currtime;

    if (pf.QuadPart == 0) {
        lcb_assert(QueryPerformanceFrequency(&pf));
        lcb_assert(pf.QuadPart != 0);
        freq = 1.0e9 / (double)pf.QuadPart;
    }

    QueryPerformanceCounter(&currtime);

    ret = (double)currtime.QuadPart * freq;
    return (hrtime_t)ret + CLOCK_START_OFFSET;
#else
#error "I don't know how to build a highres clock..."
#endif
}
#endif /* HAVE_GETHRTIME */

/* Symbol usable so other subsystems can get the same idea of time the library
 * has. This will also allow us to stop shipping the 'gethrtime' file around.
 */

LCB_INTERNAL_API
lcb_U64 lcb_nstime(void)
{
    return gethrtime();
}
