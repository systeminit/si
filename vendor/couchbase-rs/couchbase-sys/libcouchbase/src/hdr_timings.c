/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

/* High dynamic range timings, using HdrHistogram
 * (http://hdrhistogram.github.io/HdrHistogram/), specifically the
 * C implementation (https://github.com/HdrHistogram/HdrHistogram_c)
 *
 * hdr_timings.c is a rrop-in replacment for timings.c - you only want one of
 * the two files.
 */

#include "internal.h"

#include "hdr_histogram.h"

struct lcb_histogram_st {
    struct hdr_histogram *hdr_histogram;
};

LCB_INTERNAL_API
lcb_HISTOGRAM *lcb_histogram_create(void)
{
    lcb_HISTOGRAM *histo = calloc(1, sizeof(struct lcb_histogram_st));

    if (histo != NULL) {
        hdr_init(/* minimum - 1 ns*/ 1,
                 /* maximum - 30 s*/ 30e9,
                 /* s   ignificant figures */ 3, &histo->hdr_histogram);
    }

    return histo;
}

LCB_INTERNAL_API
void lcb_histogram_destroy(lcb_HISTOGRAM *hg)
{
    free(hg->hdr_histogram);
    free(hg);
}

LCB_INTERNAL_API
void lcb_histogram_read(const lcb_HISTOGRAM *hg, const void *cookie, lcb_HISTOGRAM_CALLBACK callback)
{
    struct hdr_iter iter;
    hdr_iter_recorded_init(&iter, hg->hdr_histogram);

    while (hdr_iter_next(&iter)) {
        callback(cookie, LCB_TIMEUNIT_NSEC, iter.value_iterated_from, iter.value_iterated_to, iter.count,
                 hdr_max(hg->hdr_histogram));
    }
}

LCB_INTERNAL_API
lcb_STATUS lcb_histogram_print(lcb_HISTOGRAM *hg, FILE *stream)
{
    hdr_percentiles_print(hg->hdr_histogram, stream,
                          5,        // Granularity of printed values
                          1.0,      // Multiplier for results
                          CLASSIC); // Format CLASSIC/CSV supported.

    return LCB_SUCCESS;
}

LCB_INTERNAL_API
void lcb_histogram_record(lcb_HISTOGRAM *hg, lcb_U64 delta)
{
    hdr_record_value(hg->hdr_histogram, delta);
}
