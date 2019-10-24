/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#include <stdio.h>
/*
 * The current test suite should not take more than 5 minutes to run.
 * If you're testing on a really slow system you may set the
 * environment variable LCB_MAX_TEST_DURATION to the maximum number of
 * seconds you'd like the tests to take.
 */
const int max_duration = 300;

#ifdef _WIN32
static HANDLE hTimer;
void CALLBACK test_timed_out(PVOID lpUnused, BOOLEAN bUnused)
{
    (void)lpUnused;
    (void)bUnused;
    fprintf(stderr, "Tests are taking too long to run. Aborting..\n");
    abort();
}
#endif

void setup_test_timeout_handler(void)
{
    char *ptr = getenv("LCB_MAX_TEST_DURATION");
    int duration = 0;
    if (ptr != NULL) {
        duration = atoi(ptr);
    }
    if (duration == 0) {
        duration = max_duration;
    }

#ifdef HAVE_SETITIMER
    struct itimerval timer = {.it_value = {.tv_sec = duration}};
    setitimer(ITIMER_REAL, &timer, NULL);
#elif defined(HAVE_ALARM)
    alarm(duration);
#elif defined(_WIN32)
    CreateTimerQueueTimer(&hTimer, NULL, test_timed_out, NULL, duration * 1000, 0, 0);
#else
    /* print an error message so that we're using the duration variable
     * and not generate a warning about unused variables ;) */
    fprintf(stderr, "Tests may run longer than %d due to lack of an alarm\n", duration);
#endif
}
