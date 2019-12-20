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

/*
 * BUILD:
 *   cc -o durability durability.c -lcouchbase
 *
 * RUN:
 *   ./durability [ CONNSTRING [ PASSWORD [ USERNAME ] ] ]
 *
 *   # use default durability check method
 *   ./durability couchbase://localhost
 *
 *   # force durability check method based on sequence numbers
 *   ./durability couchbase://localhost?fetch_mutation_tokens=true&dur_mutation_tokens=true
 */
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <libcouchbase/couchbase.h>

#define fail(msg)                                                                                                      \
    fprintf(stderr, "%s\n", msg);                                                                                      \
    exit(EXIT_FAILURE)

#define fail2(msg, err)                                                                                                \
    fprintf(stderr, "%s\n", msg);                                                                                      \
    fprintf(stderr, "Error was 0x%x (%s)\n", err, lcb_strerror(NULL, err));                                            \
    exit(EXIT_FAILURE)

static void store_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPSTORE *resp)
{
    lcb_STATUS rc = lcb_respstore_status(resp);
    int store_ok, exists_master, persisted_master;
    uint16_t num_responses, num_replicated, num_persisted;

    lcb_respstore_observe_stored(resp, &store_ok);
    lcb_respstore_observe_master_exists(resp, &exists_master);
    lcb_respstore_observe_master_persisted(resp, &persisted_master);
    lcb_respstore_observe_num_responses(resp, &num_responses);
    lcb_respstore_observe_num_replicated(resp, &num_replicated);
    lcb_respstore_observe_num_persisted(resp, &num_persisted);

    fprintf(stderr, "Got status of operation: 0x%02x, %s\n", rc, lcb_strerror_short(rc));
    fprintf(stderr, "Stored: %s\n", store_ok ? "true" : "false");
    fprintf(stderr, "Number of roundtrips: %d\n", (int)num_responses);
    fprintf(stderr, "In memory on master: %s\n", exists_master ? "true" : "false");
    fprintf(stderr, "Persisted on master: %s\n", persisted_master ? "true" : "false");
    fprintf(stderr, "Nodes have value replicated: %d\n", (int)num_replicated);
    fprintf(stderr, "Nodes have value persisted (including master): %d\n", (int)num_persisted);
}

int main(int argc, char *argv[])
{
    lcb_INSTANCE *instance;
    lcb_STATUS err;
    lcb_CMDSTORE *cmd;
    struct lcb_create_st create_options = {0};
    const char *key = "foo";
    const char *value = "{\"val\":42}";

    create_options.version = 3;
    if (argc > 1) {
        create_options.v.v3.connstr = argv[1];
    }
    if (argc > 2) {
        create_options.v.v3.passwd = argv[2];
    }
    if (argc > 3) {
        create_options.v.v3.username = argv[3];
    }

    if ((err = lcb_create(&instance, &create_options)) != LCB_SUCCESS) {
        fail2("cannot create connection instance", err);
    }
    if ((err = lcb_connect(instance)) != LCB_SUCCESS) {
        fail2("Couldn't schedule connection", err);
    }
    lcb_wait(instance);
    if ((err = lcb_get_bootstrap_status(instance)) != LCB_SUCCESS) {
        fail2("Couldn't get initial cluster configuration", err);
    }
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);

    lcb_cmdstore_create(&cmd, LCB_STORE_UPSERT);
    lcb_cmdstore_key(cmd, key, strlen(key));
    lcb_cmdstore_value(cmd, value, strlen(value));
    /* replicate and persist on all nodes */
    lcb_cmdstore_durability_observe(cmd, -1, -1);
    lcb_store(instance, NULL, cmd);
    lcb_cmdstore_destroy(cmd);

    lcb_wait(instance);

    lcb_destroy(instance);
    return EXIT_SUCCESS;
}
