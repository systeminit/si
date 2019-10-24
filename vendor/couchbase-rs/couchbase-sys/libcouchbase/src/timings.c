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
#include "internal.h"

/**
 * Timing data in libcouchbase is stored in a structure to make
 * it easy to work with. It ill consume a fair amount of data,
 * but it's only allocated when you enable it ;-)
 * I decided I'd rather just make it easy to work with...
 */
struct lcb_histogram_st {
    /** The highest value (number of ocurrences ) in all of the buckets */
    lcb_U32 max;

    /** Number of entries below a microsecond */
    lcb_U32 nsec;

    /**
     * Entries between 1-1000 microseconds. Each array element refers to a
     * 10 microsecond interval
     */
    lcb_U32 usec[100];

    /**
     * Entries between 1-10 milliseconds. Each array entry refers to a 100
     * microsecond interval.
     */
    lcb_U32 lt10msec[100];

    /**
     * Entries between 10-1000 milliseconds. Each entry refers to a 10 millisecond
     * interval
     */
    lcb_U32 msec[100];

    /**
     * Seconds are collected per sec
     */
    lcb_U32 sec[10];
};

LCB_INTERNAL_API
lcb_HISTOGRAM *lcb_histogram_create(void)
{
    return calloc(1, sizeof(lcb_HISTOGRAM));
}

LCB_INTERNAL_API
void lcb_histogram_destroy(lcb_HISTOGRAM *hg)
{
    free(hg);
}

LCB_INTERNAL_API
void lcb_histogram_read(const lcb_HISTOGRAM *hg, const void *cookie, lcb_HISTOGRAM_CALLBACK callback)
{
    lcb_U32 max, start, ii, end;

    max = hg->max;
    /*
    ** @todo I should merge "empty" sets.. currently I'm only going to
    ** report the nonzero ones...
    */
    if (hg->nsec) {
        callback(cookie, LCB_TIMEUNIT_NSEC, 0, 999, hg->nsec, max);
    }

    start = 1;
    for (ii = 0; ii < 100; ++ii) {
        end = (ii + 1) * 10 - 1;
        if (hg->usec[ii]) {
            callback(cookie, LCB_TIMEUNIT_USEC, start, end, hg->usec[ii], max);
        }
        start = end + 1;
    }

    start = 1000;
    for (ii = 0; ii < 100; ++ii) {
        end = (ii + 1) * 100 - 1;
        if (hg->lt10msec[ii]) {
            callback(cookie, LCB_TIMEUNIT_USEC, start, end, hg->lt10msec[ii], max);
        }
        start = end + 1;
    }

    start = 1;
    for (ii = 0; ii < 100; ++ii) {
        end = (ii + 1) * 10 - 1;
        if (hg->msec[ii]) {
            callback(cookie, LCB_TIMEUNIT_MSEC, start, end, hg->msec[ii], max);
        }
        start = end + 1;
    }

    for (ii = 1; ii < 9; ++ii) {
        start = ii * 1000;
        end = ((ii + 1) * 1000) - 1;
        if (hg->sec[ii]) {
            callback(cookie, LCB_TIMEUNIT_MSEC, start, end, hg->sec[ii], max);
        }
    }

    if (hg->sec[9]) {
        callback(cookie, LCB_TIMEUNIT_SEC, 9, 9999, hg->sec[9], max);
    }
}

static void default_timings_callback(const void *cookie, lcb_timeunit_t timeunit, lcb_uint32_t min_val,
                                     lcb_uint32_t max_val, lcb_uint32_t total, lcb_uint32_t maxtotal)
{
    FILE *stream = (FILE *)cookie;
    const char *unit = NULL;
    int ii;
    int num_hash;

    fprintf(stream, "[%-4u - %-4u]", min_val, max_val);
    if (timeunit == LCB_TIMEUNIT_NSEC) {
        unit = "ns";
    } else if (timeunit == LCB_TIMEUNIT_USEC) {
        unit = "us";
    } else if (timeunit == LCB_TIMEUNIT_MSEC) {
        unit = "ms";
    } else if (timeunit == LCB_TIMEUNIT_SEC) {
        unit = "s";
    } else {
        unit = "?";
    }
    fprintf(stream, "%s |", unit);

    num_hash = (int)((float)40.0 * (float)total / (float)maxtotal);

    for (ii = 0; ii < num_hash; ++ii) {
        putw('#', stream);
    }

    fprintf(stream, " - %u\n", total);
}

LCB_INTERNAL_API
void lcb_histogram_print(lcb_HISTOGRAM *hg, FILE *stream)
{
    lcb_histogram_read(hg, stream, default_timings_callback);
}

LCB_INTERNAL_API
void lcb_histogram_record(lcb_HISTOGRAM *hg, lcb_U64 delta)
{
    lcb_U32 num;

    if (delta < 1000) {
        /* nsec */
        if (++hg->nsec > hg->max) {
            hg->max = hg->nsec;
        }
    } else if (delta < LCB_US2NS(1000)) {
        /* micros */
        delta /= LCB_US2NS(1);
        if ((num = ++hg->usec[delta / 10]) > hg->max) {
            hg->max = num;
        }
    } else if (delta < LCB_US2NS(10000)) {
        /* 1-10ms */
        delta /= LCB_US2NS(1);
        lcb_assert(delta <= 10000);
        if ((num = ++hg->lt10msec[delta / 100]) > hg->max) {
            hg->max = num;
        }
    } else if (delta < LCB_S2NS(1)) {
        delta /= LCB_US2NS(1000);
        if ((num = ++hg->msec[delta / 10]) > hg->max) {
            hg->max = num;
        }
    } else {
        delta /= LCB_S2NS(1);
        if (delta > 9) {
            delta = 9;
        }

        if ((num = ++hg->sec[delta]) > hg->max) {
            hg->max = num;
        }
    }
}
