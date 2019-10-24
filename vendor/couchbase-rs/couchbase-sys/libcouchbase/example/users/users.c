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

/**
 * @file
 *
 * This is an example file showing how to create and list user accounts
 * on Couchbase Cluster 5.
 *
 * https://developer.couchbase.com/documentation/server/current/rest-api/rbac.html
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libcouchbase/couchbase.h>

static void die(lcb_INSTANCE *instance, const char *msg, lcb_STATUS err)
{
    fprintf(stderr, "%s. Received code 0x%X (%s)\n", msg, err, lcb_strerror(instance, err));
    exit(EXIT_FAILURE);
}

void http_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPHTTP *resp)
{
    uint16_t status;
    lcb_resphttp_http_status(resp, &status);
    printf("HTTP status: %d\n", status);
    const char *const *headers;
    lcb_resphttp_headers(resp, &headers);
    if (headers) {
        for (const char *const *cur = headers; *cur; cur += 2) {
            printf("%s: %s\n", cur[0], cur[1]);
        }
    }
    const char *body;
    size_t nbody;
    lcb_resphttp_body(resp, &body, &nbody);
    if (nbody) {
        printf("%*s\n", (int)nbody, body);
    }
    lcb_STATUS rc = lcb_resphttp_status(resp);
    if (rc != LCB_SUCCESS) {
        die(instance, "Failed to execute HTTP request", rc);
    }
}

int main(int argc, char *argv[])
{
    lcb_STATUS err;
    lcb_INSTANCE *instance;
    struct lcb_create_st create_options = {0};

    create_options.version = 3;

    if (argc < 3) {
        fprintf(stderr, "Usage: %s couchbase://host/bucket ADMIN_NAME ADMIN_PASSWORD\n", argv[0]);
        exit(EXIT_FAILURE);
    }

    create_options.v.v3.connstr = argv[1];
    create_options.v.v3.username = argv[2];
    create_options.v.v3.passwd = argv[3];

    err = lcb_create(&instance, &create_options);
    if (err != LCB_SUCCESS) {
        die(NULL, "Failed create couchbase handle", err);
    }

    err = lcb_connect(instance);
    if (err != LCB_SUCCESS) {
        die(instance, "Failed schedule connection", err);
    }

    lcb_wait(instance);

    err = lcb_get_bootstrap_status(instance);
    if (err != LCB_SUCCESS) {
        die(instance, "Failed bootstrap from cluster", err);
    }

    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)http_callback);

    printf("1. Create account 'cbtestuser' with predefined set of roles\n");
    {
        lcb_CMDHTTP *cmd;
        char *path = "/settings/rbac/users/local/cbtestuser";
        char *body = "name=TestUser&password=cbtestuserpwd&roles=cluster_admin,bucket_admin[default]";
        char *content_type = "application/x-www-form-urlencoded";

        lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_MANAGEMENT);
        lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_PUT);
        lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));
        lcb_cmdhttp_path(cmd, path, strlen(path));
        lcb_cmdhttp_body(cmd, body, strlen(body));

        err = lcb_http(instance, NULL, cmd);
        lcb_cmdhttp_destroy(cmd);
        if (err != LCB_SUCCESS) {
            die(instance, "Failed schedule command to upsert user", err);
        }
        lcb_wait(instance);
    }

    printf("2. Retrieve list of all accounts in the cluster\n");
    {
        lcb_CMDHTTP *cmd;
        char *path = "/settings/rbac/users/local";

        lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_MANAGEMENT);
        lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_GET);
        lcb_cmdhttp_path(cmd, path, strlen(path));

        err = lcb_http(instance, NULL, cmd);
        lcb_cmdhttp_destroy(cmd);
        if (err != LCB_SUCCESS) {
            die(instance, "Failed schedule command to upsert user", err);
        }
        lcb_wait(instance);
    }

    printf("3. Remove account 'cbtestuser'\n");
    {
        lcb_CMDHTTP *cmd;
        char *path = "/settings/rbac/users/local/cbtestuser";

        lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_MANAGEMENT);
        lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_DELETE);
        lcb_cmdhttp_path(cmd, path, strlen(path));

        err = lcb_http(instance, NULL, cmd);
        lcb_cmdhttp_destroy(cmd);
        if (err != LCB_SUCCESS) {
            die(instance, "Failed schedule command to upsert user", err);
        }
        lcb_wait(instance);
    }
    /* Now that we're all done, close down the connection handle */
    lcb_destroy(instance);
    return 0;
}
