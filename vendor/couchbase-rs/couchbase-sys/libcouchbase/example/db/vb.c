/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

/* View Benchmark. This application is using libcouchbase to store
 * single key and then get this key back infinitely through the view
 * through the views.
 *
 * BUILD:
 *
 *      cc -o vb vb.c -lcouchbase
 *      cl /DWIN32 /Iinclude vb.c lib\libcouchbase.lib
 *
 * RUN:
 *
 *      valgrind -v --tool=memcheck  --leak-check=full --show-reachable=yes ./vb
 *      ./vb key size <connstr> <passwd>
 *      vb.exe key size <connstr> <passwd>
 */

#include <stdio.h>
#include <libcouchbase/couchbase.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#ifdef _WIN32
#define PRIu64 "I64u"
#else
#include <signal.h>
#include <inttypes.h>
#endif

#ifndef _WIN32
static void handle_sigint(int sig)
{
    (void)sig;
    printf("Exiting on SIGINT\n");
    exit(0);
}

#define INSTALL_SIGINT_HANDLER() signal(SIGINT, handle_sigint)
#else
#define INSTALL_SIGINT_HANDLER()
#endif

static void store_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPSTORE *resp)
{
    lcb_STATUS rc = lcb_respstore_status(resp);
    if (rc == LCB_SUCCESS) {
        const char *key;
        size_t nkey;
        uint64_t cas;
        lcb_respstore_key(resp, &key, &nkey);
        lcb_respstore_cas(resp, &cas);
        fprintf(stderr, "STORED \"%.*s\" CAS: %" PRIu64 "\n", (int)nkey, key, cas);
    } else {
        fprintf(stderr, "STORE ERROR: %s (0x%x)\n", lcb_strerror(instance, rc), rc);
        exit(EXIT_FAILURE);
    }
    (void)cbtype;
}

const char *view;
const char *design;

static void do_query_view(lcb_INSTANCE *instance);

static void viewrow_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPVIEW *resp)
{
    if (lcb_respview_is_final(resp)) {
        lcb_STATUS rc = lcb_respview_status(resp);
        if (rc == LCB_SUCCESS) {
            do_query_view(instance);
        } else {
            const lcb_RESPHTTP *http;
            fprintf(stderr, "Couldn't query view: %s\n", lcb_strerror_short(rc));
            lcb_respview_http_response(resp, &http);
            if (http != NULL) {
                uint16_t status;
                const char *body;
                size_t nbody;
                lcb_resphttp_http_status(http, &status);
                fprintf(stderr, "HTTP Status: %u\n", status);
                lcb_resphttp_body(http, &body, &nbody);
                fprintf(stderr, "HTTP Body: %.*s\n", (int)nbody, body);
            }
            exit(EXIT_FAILURE);
        }
    }
    (void)cbtype;
}

static void http_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPHTTP *resp)
{
    const char *path;
    size_t npath;
    uint16_t status;
    lcb_STATUS rc;

    lcb_resphttp_path(resp, &path, &npath);
    lcb_resphttp_http_status(resp, &status);
    fprintf(stderr, "%.*s... %d\n", (int)npath, path, status);
    rc = lcb_resphttp_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Couldn't issue HTTP request: %s\n", lcb_strerror(NULL, rc));
        exit(EXIT_FAILURE);
    } else if (status != 201) {
        const char *body;
        size_t nbody;
        lcb_resphttp_body(resp, &body, &nbody);
        fprintf(stderr, "Negative reply from server!\n");
        fprintf(stderr, "%*.s\n", (int)nbody, body);
        exit(EXIT_FAILURE);
    }

    (void)cbtype;
}

static void do_query_view(lcb_INSTANCE *instance)
{
    lcb_CMDVIEW *cmd;
    lcb_STATUS err;
    lcb_cmdview_create(&cmd);
    lcb_cmdview_design_document(cmd, design, strlen(design));
    lcb_cmdview_view_name(cmd, view, strlen(view));
    lcb_cmdview_callback(cmd, viewrow_callback);
    lcb_cmdview_include_docs(cmd, 1);
    err = lcb_view(instance, NULL, cmd);
    lcb_cmdview_destroy(cmd);
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "Couldn't schedule view query: %s\n", lcb_strerror_short(err));
        exit(EXIT_FAILURE);
    }
}

int main(int argc, char *argv[])
{
    lcb_STATUS err;
    lcb_INSTANCE *instance;
    struct lcb_create_st create_options;
    const char *key = "foo";
    size_t nkey = strlen(key);
    void *bytes;
    size_t nbytes = 6; /* the size of the value */

    memset(&create_options, 0, sizeof(create_options));
    create_options.version = 3;

    if (argc > 1) {
        key = argv[1];
        nkey = strlen(key);
    }
    if (argc > 2) {
        nbytes = atol(argv[2]);
    }
    if (argc > 3) {
        create_options.v.v3.connstr = argv[3];
    }
    if (argc > 4) {
        create_options.v.v3.passwd = argv[4];
    }

    INSTALL_SIGINT_HANDLER();

    err = lcb_create(&instance, &create_options);
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "Failed to create libcouchbase instance: %s\n", lcb_strerror(NULL, err));
        exit(EXIT_FAILURE);
    }
    /* Initiate the connect sequence in libcouchbase */
    if ((err = lcb_connect(instance)) != LCB_SUCCESS) {
        fprintf(stderr, "Failed to initiate connect: %s\n", lcb_strerror(NULL, err));
        lcb_destroy(instance);
        exit(EXIT_FAILURE);
    }
    lcb_wait(instance);
    if ((err = lcb_get_bootstrap_status(instance)) != LCB_SUCCESS) {
        fprintf(stderr, "Failed to establish connection to cluster: %s\n", lcb_strerror(NULL, err));
        exit(EXIT_FAILURE);
    }
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)http_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);

    fprintf(stderr, "key: \"%s\"\n", key);
    fprintf(stderr, "value size: %ld\n", nbytes);
    fprintf(stderr, "connection string: %s\n", create_options.v.v3.connstr ? create_options.v.v3.connstr : "");
    fprintf(stderr, "password: %s\n", create_options.v.v0.passwd ? create_options.v.v3.passwd : "");
    bytes = malloc(nbytes);

    {
        lcb_CMDSTORE *cmd;
        lcb_cmdstore_create(&cmd, LCB_STORE_SET);
        lcb_cmdstore_key(cmd, key, nkey);
        lcb_cmdstore_value(cmd, bytes, nbytes);
        err = lcb_store(instance, NULL, cmd);
        if (err != LCB_SUCCESS) {
            fprintf(stderr, "Failed to store: %s\n", lcb_strerror(NULL, err));
            exit(EXIT_FAILURE);
        }
        lcb_cmdstore_destroy(cmd);
    }
    lcb_wait(instance);

    /* Set view and design name: */
    view = "all";
    design = key;

    {
        char *content_type = "application/json";
        char design_path[64] = {0};
        char doc[256] = {0};
        lcb_CMDHTTP *cmd;
        sprintf(design_path, "_design/%s", design);
        sprintf(doc, "{\"views\":{\"all\":{\"map\":\"function(doc,meta){if(meta.id=='%s'){emit(meta.id)}}\"}}}", key);

        lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_VIEW);
        lcb_cmdhttp_path(cmd, design_path, strlen(design_path));
        lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));
        lcb_cmdhttp_body(cmd, doc, strlen(doc));
        lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_PUT);
        err = lcb_http(instance, NULL, cmd);
        lcb_cmdhttp_destroy(cmd);
        if (err != LCB_SUCCESS) {
            fprintf(stderr, "Failed to create design document: %s (0x%02x)\n", lcb_strerror(NULL, err), err);
            exit(EXIT_FAILURE);
        }
    }
    lcb_wait(instance);

    do_query_view(instance);
    lcb_wait(instance);
    lcb_destroy(instance);

    exit(EXIT_SUCCESS);
}
