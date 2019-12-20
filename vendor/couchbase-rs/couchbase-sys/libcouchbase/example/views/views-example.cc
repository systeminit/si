/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2013-2019 Couchbase, Inc.
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

#include <libcouchbase/couchbase.h>
#include <string>
#include <cstring>
#include <cstdlib>
#include <cstdio>
#include <cassert>

static int cbCounter = 0;

extern "C" {
static void viewCallback(lcb_INSTANCE *, int, const lcb_RESPVIEW *rv)
{
    lcb_STATUS rc = lcb_respview_status(rv);

    if (lcb_respview_is_final(rv)) {
        const char *row;
        size_t nrow;
        lcb_respview_row(rv, &row, &nrow);
        printf("*** META FROM VIEWS ***\n");
        fprintf(stderr, "%.*s\n", (int)nrow, row);
        return;
    }

    const char *key, *docid;
    size_t nkey, ndocid;
    lcb_respview_key(rv, &key, &nkey);
    lcb_respview_doc_id(rv, &docid, &ndocid);
    printf("Got row callback from LCB: RC=0x%X, DOCID=%.*s. KEY=%.*s\n", rc, (int)ndocid, docid, (int)nkey, key);

    const lcb_RESPGET *doc = NULL;
    lcb_respview_document(rv, &doc);
    if (doc) {
        rc = lcb_respget_status(doc);
        uint64_t cas;
        lcb_respget_cas(doc, &cas);
        printf("   Document for response. RC=0x%X. CAS=0x%llx\n", rc, cas);
    }

    cbCounter++;
}
}

int main(int argc, const char **argv)
{
    lcb_INSTANCE *instance;
    lcb_create_st cropts;
    memset(&cropts, 0, sizeof cropts);
    const char *connstr = "couchbase://localhost/beer-sample";

    if (argc > 1) {
        if (strcmp(argv[1], "--help") == 0) {
            fprintf(stderr, "Usage: %s CONNSTR\n", argv[0]);
            exit(EXIT_SUCCESS);
        } else {
            connstr = argv[1];
        }
    }

    cropts.version = 3;
    cropts.v.v3.connstr = connstr;
    lcb_STATUS rc;
    rc = lcb_create(&instance, &cropts);
    assert(rc == LCB_SUCCESS);
    rc = lcb_connect(instance);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    assert(lcb_get_bootstrap_status(instance) == LCB_SUCCESS);

    // Nao, set up the views..
    lcb_CMDVIEW *vq;
    std::string dName = "beer";
    std::string vName = "by_location";
    std::string options = "reduce=false";

    lcb_cmdview_create(&vq);
    lcb_cmdview_callback(vq, viewCallback);
    lcb_cmdview_design_document(vq, dName.c_str(), dName.size());
    lcb_cmdview_view_name(vq, vName.c_str(), vName.size());
    lcb_cmdview_option_string(vq, options.c_str(), options.size());
    lcb_cmdview_include_docs(vq, true);

    rc = lcb_view(instance, NULL, vq);
    lcb_cmdview_destroy(vq);

    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    lcb_destroy(instance);
    printf("Total Invocations=%d\n", cbCounter);
    return 0;
}
