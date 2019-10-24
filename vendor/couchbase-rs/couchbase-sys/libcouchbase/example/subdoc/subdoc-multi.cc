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
#include <cstdlib>
#include <string>
#include <vector>

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

static void subdoc_callback(lcb_INSTANCE *, int type, const lcb_RESPSUBDOC *resp)
{
    lcb_STATUS rc = lcb_respsubdoc_status(resp);

    fprintf(stderr, "Got callback for %s.. ", lcb_strcbtype(type));
    if (rc != LCB_SUCCESS && rc != LCB_SUBDOC_MULTI_FAILURE) {
        fprintf(stderr, "Operation failed (%s)\n", lcb_strerror_short(rc));
        return;
    }

    size_t total = lcb_respsubdoc_result_size(resp);
    for (size_t idx = 0; idx < total; idx++) {
        rc = lcb_respsubdoc_result_status(resp, idx);
        const char *value;
        size_t nvalue;
        lcb_respsubdoc_result_value(resp, idx, &value, &nvalue);
        printf("[%lu]: 0x%x. %.*s\n", idx, rc, (int)nvalue, value);
    }
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

    // Install generic callback
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)get_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_SDLOOKUP, (lcb_RESPCALLBACK)subdoc_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_SDMUTATE, (lcb_RESPCALLBACK)subdoc_callback);

    // Store an item
    lcb_CMDSTORE *scmd;
    lcb_cmdstore_create(&scmd, LCB_STORE_SET);
    lcb_cmdstore_key(scmd, "key", 3);
    const char *initval = "{\"hello\":\"world\"}";
    lcb_cmdstore_value(scmd, initval, strlen(initval));
    rc = lcb_store(instance, NULL, scmd);
    lcb_cmdstore_destroy(scmd);
    assert(rc == LCB_SUCCESS);

    lcb_SUBDOCOPS *specs;

    lcb_subdocops_create(&specs, 5);
    std::string bufs[10];
    // Add some mutations
    for (int ii = 0; ii < 5; ii++) {
        std::string &path = bufs[ii * 2];
        std::string &val = bufs[(ii * 2) + 1];
        char pbuf[24], vbuf[24];

        sprintf(pbuf, "pth%d", ii);
        sprintf(vbuf, "\"Value_%d\"", ii);
        path = pbuf;
        val = vbuf;

        lcb_subdocops_dict_upsert(specs, ii, 0, path.c_str(), path.size(), val.c_str(), val.size());
    }

    lcb_CMDSUBDOC *mcmd;
    lcb_cmdsubdoc_create(&mcmd);
    lcb_cmdsubdoc_key(mcmd, "key", 3);
    lcb_cmdsubdoc_operations(mcmd, specs);
    rc = lcb_subdoc(instance, NULL, mcmd);
    lcb_subdocops_destroy(specs);
    assert(rc == LCB_SUCCESS);

    // Reset the specs
    lcb_subdocops_create(&specs, 5);
    for (int ii = 0; ii < 5; ii++) {
        char pbuf[24];
        std::string &path = bufs[ii];
        sprintf(pbuf, "pth%d", ii);
        path = pbuf;

        lcb_subdocops_get(specs, ii, 0, path.c_str(), path.size());
    }

    lcb_subdocops_get(specs, 5, 0, "dummy", 5);
    lcb_cmdsubdoc_operations(mcmd, specs);
    rc = lcb_subdoc(instance, NULL, mcmd);
    lcb_subdocops_destroy(specs);
    lcb_cmdsubdoc_destroy(mcmd);
    assert(rc == LCB_SUCCESS);

    lcb_CMDGET *gcmd;
    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, "key", 3);
    rc = lcb_get(instance, NULL, gcmd);
    assert(rc == LCB_SUCCESS);
    lcb_cmdget_destroy(gcmd);

    lcb_wait(instance);
    lcb_destroy(instance);
}
