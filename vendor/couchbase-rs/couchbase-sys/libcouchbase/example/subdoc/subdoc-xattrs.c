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

#include <unistd.h>
#include <stdlib.h>

#include <libcouchbase/couchbase.h>
#include <assert.h>
#include <string.h>

static void subdoc_callback(lcb_INSTANCE *instance, int type, const lcb_RESPSUBDOC *resp)
{
    lcb_STATUS rc = lcb_respsubdoc_status(resp);
    size_t idx, total;
    if (rc != LCB_SUCCESS && rc != LCB_SUBDOC_MULTI_FAILURE) {
        printf("Failure: 0x%x, %s\n", lcb_strerror_short(rc));
        return;
    }

    total = lcb_respsubdoc_result_size(resp);
    for (idx = 0; idx < total; idx++) {
        const char *value;
        size_t nvalue;
        rc = lcb_respsubdoc_result_status(resp, idx);
        lcb_respsubdoc_result_value(resp, idx, &value, &nvalue);
        printf("[%lu]: 0x%x. %.*s\n", idx, rc, (int)nvalue, value);
    }
}

static void n1qlrow_callback(lcb_INSTANCE *instance, int type, const lcb_RESPN1QL *resp)
{
    lcb_STATUS rc = lcb_respn1ql_status(resp);
    const char *row;
    size_t nrow;

    lcb_respn1ql_row(resp, &row, &nrow);
    if (rc != LCB_SUCCESS) {
        const lcb_RESPHTTP *http;
        uint16_t status;

        lcb_respn1ql_http_response(resp, &http);
        printf("Failure: 0x%x, %s\n", rc, lcb_strerror(instance, rc));
        lcb_resphttp_http_status(http, &status);
        printf("HTTP status: %d\n", (int)status);
        {
            const char *const *hdr;
            lcb_resphttp_headers(http, &hdr);
            for (; hdr && *hdr; hdr++) {
                printf("%s", *hdr);
                if (hdr + 1) {
                    printf(" = %s", *(++hdr));
                }
                printf("\n");
            }
        }
        printf("%.*s\n", (int)nrow, row);
        return;
    }

    char *start = "{\"docID\":\"";
    char *stop = "\"";

    char *key = strstr(row, start);
    if (key == NULL || key != row) {
        return;
    }
    key += strlen(start);
    char *z = strstr(key, stop);
    if (z == NULL) {
        return;
    }
    *z = '\0';

    lcb_sched_enter(instance);
    {
        char *path = "discounts.jsmith123";

        lcb_SUBDOCOPS *specs;
        lcb_subdocops_create(&specs, 2);
        lcb_subdocops_exists(specs, 0, LCB_SUBDOCOPS_F_XATTRPATH, path, strlen(path));
        lcb_subdocops_exists(specs, 1, LCB_SUBDOCOPS_F_XATTRPATH, path, strlen(path));

        lcb_CMDSUBDOC *cmd;
        lcb_cmdsubdoc_create(&cmd);
        lcb_cmdsubdoc_key(cmd, key, strlen(key));
        lcb_cmdsubdoc_operations(cmd, specs);
        rc = lcb_subdoc(instance, NULL, cmd);
        lcb_subdocops_destroy(specs);
        lcb_cmdsubdoc_destroy(cmd);
        assert(rc == LCB_SUCCESS);
    }
    lcb_sched_leave(instance);
}

#define DEFAULT_CONNSTR "couchbase://localhost/travel-sample"

static lcb_INSTANCE *connect_as(char *username, char *password)
{
    struct lcb_create_st crst = {.version = 3};

    crst.v.v3.connstr = DEFAULT_CONNSTR;
    crst.v.v3.username = username;
    crst.v.v3.passwd = password;

    lcb_INSTANCE *instance;
    lcb_STATUS rc;

    rc = lcb_create(&instance, &crst);
    assert(rc == LCB_SUCCESS);
    rc = lcb_connect(instance);
    assert(rc == LCB_SUCCESS);
    lcb_wait(instance);
    rc = lcb_get_bootstrap_status(instance);
    assert(rc == LCB_SUCCESS);

    lcb_install_callback3(instance, LCB_CALLBACK_SDLOOKUP, (lcb_RESPCALLBACK)subdoc_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_SDMUTATE, (lcb_RESPCALLBACK)subdoc_callback);

    return instance;
}

int main()
{
    lcb_STATUS rc;
    lcb_INSTANCE *instance;

    instance = connect_as("Administrator", "password");

    // Add key-value pairs to hotel_10138, representing traveller-Ids and associated discount percentages
    {
        lcb_SUBDOCOPS *specs;
        lcb_subdocops_create(&specs, 4);

        {
            char *path = "discounts.jsmith123";
            char *val = "20";
            lcb_subdocops_dict_upsert(specs, 0, LCB_SUBDOCOPS_F_MKINTERMEDIATES | LCB_SUBDOCOPS_F_XATTRPATH, path,
                                      strlen(path), val, strlen(val));
        }
        {
            char *path = "discounts.pjones356";
            char *val = "30";
            lcb_subdocops_dict_upsert(specs, 1, LCB_SUBDOCOPS_F_MKINTERMEDIATES | LCB_SUBDOCOPS_F_XATTRPATH, path,
                                      strlen(path), val, strlen(val));
        }
        // The following lines, "insert" and "remove", simply demonstrate insertion and
        // removal of the same path and value
        {
            char *path = "discounts.jbrown789";
            char *val = "25";
            lcb_subdocops_dict_add(specs, 2, LCB_SUBDOCOPS_F_MKINTERMEDIATES | LCB_SUBDOCOPS_F_XATTRPATH, path,
                                   strlen(path), val, strlen(val));
        }
        {
            char *path = "discounts.jbrown789";
            lcb_subdocops_remove(specs, 3, LCB_SUBDOCOPS_F_XATTRPATH, path, strlen(path));
        }

        char *key = "hotel_10138";

        lcb_CMDSUBDOC *cmd;
        lcb_cmdsubdoc_create(&cmd);
        lcb_cmdsubdoc_key(cmd, key, strlen(key));
        lcb_cmdsubdoc_operations(cmd, specs);
        rc = lcb_subdoc(instance, NULL, cmd);
        lcb_subdocops_destroy(specs);
        lcb_cmdsubdoc_destroy(cmd);
        assert(rc == LCB_SUCCESS);
    }

    // Add key - value pairs to hotel_10142, again representing traveller - Ids and associated discount percentages
    {
        lcb_SUBDOCOPS *specs;
        lcb_subdocops_create(&specs, 2);
        {
            char *path = "discounts.jsmith123";
            char *val = "15";
            lcb_subdocops_dict_upsert(specs, 0, LCB_SUBDOCOPS_F_MKINTERMEDIATES | LCB_SUBDOCOPS_F_XATTRPATH, path,
                                      strlen(path), val, strlen(val));
        }
        {
            char *path = "discounts.pjones356";
            char *val = "10";
            lcb_subdocops_dict_upsert(specs, 1, LCB_SUBDOCOPS_F_MKINTERMEDIATES | LCB_SUBDOCOPS_F_XATTRPATH, path,
                                      strlen(path), val, strlen(val));
        }

        char *key = "hotel_10142";

        lcb_CMDSUBDOC *cmd;
        lcb_cmdsubdoc_create(&cmd);
        lcb_cmdsubdoc_key(cmd, key, strlen(key));
        lcb_cmdsubdoc_operations(cmd, specs);
        rc = lcb_subdoc(instance, NULL, cmd);
        lcb_subdocops_destroy(specs);
        lcb_cmdsubdoc_destroy(cmd);
        assert(rc == LCB_SUCCESS);
    }

    lcb_wait(instance);

    // Create a user and assign roles. This user will search for their available discounts.
    {
        lcb_CMDHTTP *cmd;
        char *path = "/settings/rbac/users/local/jsmith123";
        char *payload = "password=jsmith123pwd&name=John+Smith"
                        "&roles=data_reader[travel-sample],query_select[travel-sample],data_writer[travel-sample]";
        char *content_type = "application/x-www-form-urlencoded";

        lcb_cmdhttp_create(&cmd, LCB_HTTP_TYPE_MANAGEMENT);
        lcb_cmdhttp_method(cmd, LCB_HTTP_METHOD_PUT);
        lcb_cmdhttp_path(cmd, path, strlen(path));
        lcb_cmdhttp_body(cmd, payload, strlen(payload));
        lcb_cmdhttp_content_type(cmd, content_type, strlen(content_type));
        lcb_http(instance, NULL, cmd);
        lcb_cmdhttp_destroy(cmd);
        lcb_wait(instance);
    }

    lcb_destroy(instance);

    // reconnect using new user
    instance = connect_as("jsmith123", "jsmith123pwd");

    // Perform a N1QL Query to return document IDs from the bucket. These IDs will be
    // used to reference each document in turn, and check for extended attributes
    // corresponding to discounts.
    {
        char *query = "SELECT id, meta(`travel-sample`).id AS docID FROM `travel-sample`";
        lcb_CMDN1QL *cmd;

        lcb_cmdn1ql_create(&cmd);
        lcb_cmdn1ql_statement(cmd, query, strlen(query));
        lcb_cmdn1ql_callback(cmd, n1qlrow_callback);

        printf("User \"jsmith123\" has discounts in the hotels below:\n");
        lcb_n1ql(instance, NULL, cmd);
        lcb_cmdn1ql_destroy(cmd);
        lcb_wait(instance);
    }

    lcb_destroy(instance);
}
