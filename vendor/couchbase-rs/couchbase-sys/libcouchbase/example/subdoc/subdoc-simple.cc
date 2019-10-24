/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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
#undef NDEBUG

#include <libcouchbase/couchbase.h>
#include <assert.h>
#include <string.h>

static void get_callback(lcb_INSTANCE *, int cbtype, const lcb_RESPGET *resp)
{
    fprintf(stderr, "Got callback for %s.. ", lcb_strcbtype(cbtype));

    lcb_STATUS rc = lcb_respget_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Operation failed (%s)\n", lcb_strerror(NULL, rc));
        return;
    }

    const char *value;
    size_t nvalue;
    lcb_respget_value(resp, &value, &nvalue);
    fprintf(stderr, "Value %.*s\n", (int)nvalue, value);
}

static void store_callback(lcb_INSTANCE *, int cbtype, const lcb_RESPSTORE *resp)
{
    fprintf(stderr, "Got callback for %s.. ", lcb_strcbtype(cbtype));

    lcb_STATUS rc = lcb_respstore_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Operation failed (%s)\n", lcb_strerror(NULL, rc));
        return;
    }

    fprintf(stderr, "OK\n");
}

static void subdoc_callback(lcb_INSTANCE *, int cbtype, const lcb_RESPSUBDOC *resp)
{
    lcb_STATUS rc = lcb_respsubdoc_status(resp);

    fprintf(stderr, "Got callback for %s.. ", lcb_strcbtype(cbtype));
    if (rc != LCB_SUCCESS && rc != LCB_SUBDOC_MULTI_FAILURE) {
        fprintf(stderr, "Operation failed (%s)\n", lcb_strerror_short(rc));
        return;
    }

    if (lcb_respsubdoc_result_size(resp) > 0) {
        const char *value;
        size_t nvalue;
        lcb_respsubdoc_result_value(resp, 0, &value, &nvalue);
        rc = lcb_respsubdoc_result_status(resp, 0);
        fprintf(stderr, "Status: %s. Value: %.*s\n", lcb_strerror_short(rc), (int)nvalue, value);
    } else {
        fprintf(stderr, "No result!\n");
    }
}

// Function to issue an lcb_get3() (and print the state of the document)
static void demoKey(lcb_INSTANCE *instance, const char *key)
{
    printf("Retrieving '%s'\n", key);
    printf("====\n");
    lcb_CMDGET *gcmd;
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key, strlen(key));
    lcb_STATUS rc = lcb_get(instance, NULL, gcmd);
    assert(rc == LCB_SUCCESS);
    lcb_cmdget_destroy(gcmd);
    lcb_wait(instance);
    printf("====\n\n");
}

// cluster_run mode
#define DEFAULT_CONNSTR "couchbase://localhost"
int main(int argc, char **argv)
{
    lcb_create_st crst = {0};
    crst.version = 3;
    if (argc > 1) {
        crst.v.v3.connstr = argv[1];
    } else {
        crst.v.v3.connstr = DEFAULT_CONNSTR;
    }
    if (argc > 2) {
        crst.v.v3.username = argv[2];
    } else {
        crst.v.v3.username = "Administrator";
    }
    if (argc > 3) {
        crst.v.v3.passwd = argv[3];
    } else {
        crst.v.v3.passwd = "password";
    }

    lcb_INSTANCE *instance;
    lcb_STATUS rc = lcb_create(&instance, &crst);
    assert(rc == LCB_SUCCESS);

    rc = lcb_connect(instance);
    assert(rc == LCB_SUCCESS);

    lcb_wait(instance);

    rc = lcb_get_bootstrap_status(instance);
    assert(rc == LCB_SUCCESS);

    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)get_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_SDLOOKUP, (lcb_RESPCALLBACK)subdoc_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_SDMUTATE, (lcb_RESPCALLBACK)subdoc_callback);

    // Store the initial document. Subdocument operations cannot create
    // documents
    printf("Storing the initial item..\n");
    // Store an item
    lcb_CMDSTORE *scmd;
    lcb_cmdstore_create(&scmd, LCB_STORE_SET);
    lcb_cmdstore_key(scmd, "key", 3);
    const char *initval = "{\"hello\":\"world\"}";
    lcb_cmdstore_value(scmd, initval, strlen(initval));
    rc = lcb_store(instance, NULL, scmd);
    lcb_cmdstore_destroy(scmd);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);

    lcb_CMDSUBDOC *cmd;
    lcb_SUBDOCOPS *ops;

    lcb_cmdsubdoc_create(&cmd);
    lcb_cmdsubdoc_key(cmd, "key", 3);

    /**
     * Retrieve a single item from a document
     */
    printf("Getting the 'hello' path from the document\n");
    lcb_subdocops_create(&ops, 1);
    lcb_subdocops_get(ops, 0, 0, "hello", 5);
    lcb_cmdsubdoc_operations(cmd, ops);
    rc = lcb_subdoc(instance, NULL, cmd);
    lcb_subdocops_destroy(ops);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);

    /**
     * Set a dictionary/object field
     */
    printf("Adding new 'goodbye' path to document\n");
    lcb_subdocops_create(&ops, 1);
    lcb_subdocops_dict_upsert(ops, 0, 0, "goodbye", 7, "\"hello\"", 7);
    lcb_cmdsubdoc_operations(cmd, ops);
    rc = lcb_subdoc(instance, NULL, cmd);
    lcb_subdocops_destroy(ops);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    demoKey(instance, "key");

    /**
     * Add new element to end of an array
     */
    // Options can also be used
    printf("Appending element to array (array might be missing)\n");
    lcb_subdocops_create(&ops, 1);
    // Create the array if it doesn't exist. This option can be used with
    // other commands as well..
    lcb_subdocops_array_add_last(ops, 0, LCB_SUBDOCOPS_F_MKINTERMEDIATES, "array", 5, "1", 1);
    lcb_cmdsubdoc_operations(cmd, ops);
    rc = lcb_subdoc(instance, NULL, cmd);
    lcb_subdocops_destroy(ops);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    demoKey(instance, "key");

    /**
     * Add element to the beginning of an array
     */
    printf("Prepending element to array (array must exist)\n");
    lcb_subdocops_create(&ops, 1);
    lcb_subdocops_array_add_first(ops, 0, 0, "array", 5, "1", 1);
    lcb_cmdsubdoc_operations(cmd, ops);
    rc = lcb_subdoc(instance, NULL, cmd);
    lcb_subdocops_destroy(ops);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    demoKey(instance, "key");

    /**
     * Get the first element back..
     */
    printf("Getting first array element...\n");
    lcb_subdocops_create(&ops, 1);
    lcb_subdocops_get(ops, 0, 0, "array[0]", 8);
    lcb_cmdsubdoc_operations(cmd, ops);
    rc = lcb_subdoc(instance, NULL, cmd);
    lcb_subdocops_destroy(ops);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);

    lcb_destroy(instance);
    return 0;
}
