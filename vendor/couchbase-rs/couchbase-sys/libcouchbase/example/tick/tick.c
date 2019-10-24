/*
 *     Copyright 2015-2019 Couchbase, Inc.
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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <libcouchbase/couchbase.h>
#include <unistd.h>

#define VALUE_SIZE 1048576
static int counter = 0;
static const char *key = "Hello";
static char value[VALUE_SIZE];

static void store_cb(lcb_INSTANCE *instance, int cbtype, const lcb_RESPSTORE *resp)
{
    assert(lcb_respstore_status(resp) == LCB_SUCCESS);
    counter--;
    printf("-");
    fflush(stdout);
}

int main(int argc, char **argv)
{
    lcb_INSTANCE *instance;
    lcb_STATUS rc;
    int ii;
    struct lcb_create_st options = {0};
    lcb_CMDSTORE *cmd;

    if (argc != 2) {
        fprintf(stderr, "Must have connection string!\n");
        exit(EXIT_FAILURE);
    }

    options.version = 3;
    options.v.v3.connstr = argv[1];

    rc = lcb_create(&instance, &options);
    assert(rc == LCB_SUCCESS);

    rc = lcb_cntl_string(instance, "operation_timeout", "120");
    assert(rc == LCB_SUCCESS);

    rc = lcb_connect(instance);
    assert(rc == LCB_SUCCESS);

    lcb_wait(instance);
    rc = lcb_get_bootstrap_status(instance);
    assert(rc == LCB_SUCCESS);

    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_cb);

    // fill the value so valgrind doesn't warn about unitialized buffers
    for (ii = 0; ii < VALUE_SIZE; ii++) {
        value[ii] = '*';
    }

    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, key, strlen(key));
    lcb_cmdstore_value(cmd, value, VALUE_SIZE);

    printf("Running sample. This will schedule 1000 operations, invoking \n");
    printf("an event loop tick after each one. The tick is non-blocking\n");
    printf("It will sleep 500 microseconds between each operation to allow\n");
    printf("for the asynchronous sending of the buffer's contents to the\n");
    printf("server.\n\n");
    printf("LEGEND:\n");
    printf("  + => Operation Scheduled\n");
    printf("  - => Operation Completed\n");

    for (ii = 0; ii < 1000; ii++) {
        lcb_sched_enter(instance);

        // Note: lcb_store() implicitly does lcb_sched_enter(), lcb_store3(),
        // and lcb_sched_leave().
        rc = lcb_store(instance, NULL, cmd);
        assert(rc == LCB_SUCCESS);
        lcb_sched_leave(instance);
        counter++;

        // This is like lcb_wait(), except it does not block.
        lcb_tick_nowait(instance);

        // Sleep to demonstrate.. Naturally the longer the wait time, the
        // clearer the difference between the tick and non-tick versions
        usleep(100);
        printf("+");
        fflush(stdout);
    }
    lcb_cmdstore_destroy(cmd);

    printf("\nCalling lcb_wait()\n");
    lcb_wait(instance);
    printf("\n");
    lcb_destroy(instance);
}

/**
 * Sample output
 * +++++++++++++--+----+----+-++++++++++++++-++----------+++-+--+-------++++++++--+------++-+++++++++-+++++-++++-+++++-+++++-++++-+++++-++++-++++++++++-++-++++-++++-++++-++++++-+++++-+++++-+++++-+++++-+++++-++++++++---++-++-++-+-+++-+-+++-+-+++-+-++-+-+-+++-+++-+-++-++-++-+--+-+-++-+++-+--+-+-++-+-+++++++++++++++-+-+-+-+--++-++++++++++++-++++-+++++++++-++++-++-++++++-+++++-+++-+++++-+++-+++++-++++-++++--++-++-++-+-++++++++-+---+++--++-+-+-+++--+-+--+-++++--++--+-+-+-+++++-+-+--++-+++-+-+--+--+-+--+++-+-++-+--+-++++-+--++++++++--+-++---++-++-++---+-+--+-++++--+++--+-+-+--+-+++-++++++---+------------+-----------------------------------------------------------++++----------------------------------------------------------------------------------------------------++++-+++++++--+-+--+-+++++-+--+--+--++++-+-++--+-+-+--++++-+--+++-+-+--+-+--+-++++++++-+----++-+-------------------------+------------------------------++++++++++-+-+++-+-+--+-+---++-+-++++-+-+-+-+--+++-+--------+------++---++++-+++-+-+-+-+--+++-+++-+--+-+-+++-+-++-+-+-+--+-++----+---+------+++++++--+++-+++++-------+-++++++---------+++++-+---+-+-+++-++-----+-----+++++-+-+-++++++--+-+-+--+-+-+-++++--+-+--+-+++++--+--+-++-+----+-----++++++---++++-+++++-++++++-++++-++++-+++++-++++++++-+++++-+++++-+++++-++++-++++-+++++-+++++-+++++-+++++-++++-++++-++++++-+++++-++++-+++-++++-+++++-++++-+++-++++++-+++-++-+++-+--++-++-++-+--+++++----+++-+--+--+-+--++-+++-+-+--+--++-+++-+--+-++++--+-+-+-++-+++-+--+-+--+-+-++++--+-+--+-+-+-+++-+--+-+--+-++-+-++-+-+-+-+-+--+-++-+-+-------+-----+------+------------------+--------------------+------------------------------------------------------+---++++++++-++++-+++++++-++-+++++-+++++-++++-+++++-++++++-++++-+++-++-+-+-+++-++-+-+++-+-++-+++++++++------++-++-+--+-+--++++++++-----+-+++--+-+-+--+-+-+++-++-+-+-+--+-++-++++-+--+--+-+-+++-+++-+--+-+--+-++-+------+--------------+------------------------------------------------------+-------+++++++++-++++++-+++-++++-++++++-+
 * Calling lcb_wait()
 * ----------------------------
 *
 */
