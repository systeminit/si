/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018-2019 Couchbase, Inc.
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
 * This is a minimal example file showing how to connect to a cluster and
 * set and retrieve a single item. This is copy of minimal.c, but with
 * tracing enabled.
 *
 *   docker run -d -p 9411:9411 openzipkin/zipkin
 *   make
 *   ./views couchbase://localhost/beer-sample password Administrator
 *
 *  open browser at http://localhost:9411
 */

#include <stdio.h>
#include <libcouchbase/couchbase.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h> /* strlen */
#ifdef _WIN32
#define PRIx64 "I64x"
#define PRId64 "I64d"
#else
#include <inttypes.h>
#endif

#include <netdb.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <arpa/inet.h>

#include "cJSON.h"

#define COMPONENT_NAME "demo"

struct zipkin_payload;
typedef struct zipkin_payload {
    char *data;
    struct zipkin_payload *next;
} zipkin_payload;

typedef struct zipkin_state {
    char *json_api_host;
    char *json_api_port;
    /* [0, 100], where 0 is "never", 100 is "always" */
    int sample_rate;
    zipkin_payload *root;
    zipkin_payload *last;
    size_t content_length;
} zipkin_state;

void zipkin_destructor(lcbtrace_TRACER *tracer)
{
    if (tracer) {
        if (tracer->cookie) {
            free(tracer->cookie);
            tracer->cookie = NULL;
        }
        free(tracer);
    }
}

void zipkin_report(lcbtrace_TRACER *tracer, lcbtrace_SPAN *span)
{
    zipkin_state *state = NULL;

    if (tracer == NULL) {
        return;
    }
    state = tracer->cookie;
    if (state == NULL) {
        return;
    }
    if (rand() % 100 > state->sample_rate) {
        return;
    }

    {
#define BUFSZ 1000
        size_t nbuf = BUFSZ;
        char *buf;
        lcbtrace_SPAN *parent;
        uint64_t start;
        zipkin_payload *payload = calloc(1, sizeof(zipkin_payload));
        cJSON *json = cJSON_CreateObject();

        buf = calloc(nbuf, sizeof(char));
        cJSON_AddItemToObject(json, "name", cJSON_CreateString(lcbtrace_span_get_operation(span)));
        snprintf(buf, nbuf, "%" PRIx64, lcbtrace_span_get_span_id(span));
        cJSON_AddItemToObject(json, "id", cJSON_CreateString(buf));
        snprintf(buf, nbuf, "%" PRIx64, lcbtrace_span_get_trace_id(span));
        cJSON_AddItemToObject(json, "traceId", cJSON_CreateString(buf));
        parent = lcbtrace_span_get_parent(span);
        if (parent) {
            snprintf(buf, nbuf, "%" PRIx64, lcbtrace_span_get_trace_id(parent));
            cJSON_AddItemToObject(json, "parentId", cJSON_CreateString(buf));
        }
        start = lcbtrace_span_get_start_ts(span);
        cJSON_AddItemToObject(json, "timestamp", cJSON_CreateNumber(start));
        cJSON_AddItemToObject(json, "duration", cJSON_CreateNumber(lcbtrace_span_get_finish_ts(span) - start));

        {
            cJSON *endpoint = cJSON_CreateObject();

            nbuf = BUFSZ;
            if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_DB_TYPE, &buf, &nbuf) == LCB_SUCCESS) {
                buf[nbuf] = '\0';
                cJSON_AddItemToObject(endpoint, "serviceName", cJSON_CreateString(buf));
            }
            cJSON_AddItemToObject(json, "localEndpoint", endpoint);
        }

        {
            cJSON *tags = cJSON_CreateObject();
            uint64_t latency, operation_id;
            if (lcbtrace_span_get_tag_uint64(span, LCBTRACE_TAG_PEER_LATENCY, &latency) == LCB_SUCCESS) {
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_PEER_LATENCY, cJSON_CreateNumber(latency));
            }
            if (lcbtrace_span_get_tag_uint64(span, LCBTRACE_TAG_OPERATION_ID, &operation_id) == LCB_SUCCESS) {
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_OPERATION_ID, cJSON_CreateNumber(operation_id));
            }
            nbuf = BUFSZ;
            if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_COMPONENT, &buf, &nbuf) == LCB_SUCCESS) {
                buf[nbuf] = '\0';
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_COMPONENT, cJSON_CreateString(buf));
            }
            nbuf = BUFSZ;
            if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_PEER_ADDRESS, &buf, &nbuf) == LCB_SUCCESS) {
                buf[nbuf] = '\0';
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_PEER_ADDRESS, cJSON_CreateString(buf));
            }
            nbuf = BUFSZ;
            if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_LOCAL_ADDRESS, &buf, &nbuf) == LCB_SUCCESS) {
                buf[nbuf] = '\0';
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_LOCAL_ADDRESS, cJSON_CreateString(buf));
            }
            nbuf = BUFSZ;
            if (lcbtrace_span_get_tag_str(span, LCBTRACE_TAG_DB_INSTANCE, &buf, &nbuf) == LCB_SUCCESS) {
                buf[nbuf] = '\0';
                cJSON_AddItemToObject(tags, LCBTRACE_TAG_DB_INSTANCE, cJSON_CreateString(buf));
            }
            if (cJSON_GetArraySize(tags) > 0) {
                cJSON_AddItemToObject(json, "tags", tags);
            } else {
                cJSON_Delete(tags);
            }
        }
        free(buf);

        payload->data = cJSON_PrintUnformatted(json);
        cJSON_Delete(json);
        if (state->last) {
            state->last->next = payload;
        }
        state->last = payload;
        state->content_length += strlen(payload->data) + 1; /* for comma/closing bracket */
        if (state->root == NULL) {
            state->root = payload;
        }
    }
}

void loop_send(int sock, char *bytes, ssize_t nbytes)
{
    do {
        ssize_t rv = send(sock, bytes, nbytes, 0);
        if (rv < 0) {
            perror("failed to send data to zipkin: ");
            exit(EXIT_FAILURE);
        } else if (rv < nbytes) {
            nbytes -= rv;
            bytes += rv;
            continue;
        }
        break;
    } while (1);
}

void zipkin_flush(lcbtrace_TRACER *tracer)
{
    zipkin_state *state = NULL;
    int sock, rv;

    if (tracer == NULL) {
        return;
    }
    state = tracer->cookie;
    if (state == NULL) {
        return;
    }
    if (state->root == NULL || state->content_length == 0) {
        return;
    }
    {
        struct addrinfo hints, *addr, *a;

        memset(&hints, 0, sizeof(hints));
        hints.ai_family = AF_UNSPEC;
        hints.ai_socktype = SOCK_STREAM;
        rv = getaddrinfo(state->json_api_host, state->json_api_port, &hints, &addr);
        if (rv != 0) {
            fprintf(stderr, "failed to resolve zipkin address getaddrinfo: %s\n", gai_strerror(rv));
            exit(EXIT_FAILURE);
        }
        for (a = addr; a != NULL; a = a->ai_next) {
            sock = socket(a->ai_family, a->ai_socktype, a->ai_protocol);
            if (sock == -1) {
                perror("failed to create socket for zipkin: ");
                continue;
            }
            rv = connect(sock, a->ai_addr, a->ai_addrlen);
            if (rv == -1) {
                perror("failed to connect socket for zipkin: ");
                continue;
            }
            break;
        }
        if (a == NULL) {
            fprintf(stderr, "unable to connect to zipkin. terminating\n");
            exit(EXIT_FAILURE);
        }
        freeaddrinfo(addr);
    }
    {
        char preamble[1000] = "";
        size_t size;

        snprintf(preamble, sizeof(preamble),
                 "POST /api/v2/spans HTTP/1.1\r\n"
                 "Content-Type: application/json\r\n"
                 "Accept: */*\r\n"
                 "Connection: close\r\n"
                 "Host: %s:%s\r\n"
                 "Content-Length: %ld\r\n\r\n",
                 state->json_api_host, state->json_api_port, (long)state->content_length + 1 /* for open bracket */);
        size = strlen(preamble);

        rv = send(sock, preamble, size, 0);
        if (rv == -1) {
            perror("failed to send HTTP headers to zipkin: ");
            exit(EXIT_FAILURE);
        }
    }
    {
        zipkin_payload *ptr = state->root;
        loop_send(sock, "[", 1);
        while (ptr) {
            zipkin_payload *tmp = ptr;
            loop_send(sock, ptr->data, strlen(ptr->data));
            ptr = ptr->next;
            if (ptr) {
                loop_send(sock, ",", 1);
            }
            free(tmp->data);
            free(tmp);
        }
        loop_send(sock, "]", 1);
    }
    close(sock);
    state->root = state->last = NULL;
    state->content_length = 0;
}

lcbtrace_TRACER *zipkin_new()
{
    lcbtrace_TRACER *tracer = calloc(1, sizeof(lcbtrace_TRACER));
    zipkin_state *zipkin = calloc(1, sizeof(zipkin_state));
    tracer->destructor = zipkin_destructor;
    tracer->flags = 0;
    tracer->version = 0;
    tracer->v.v0.report = zipkin_report;
    zipkin->json_api_host = "localhost";
    zipkin->json_api_port = "9411";
    zipkin->sample_rate = 100;
    zipkin->root = NULL;
    zipkin->last = NULL;
    zipkin->content_length = 0;
    tracer->cookie = zipkin;
    return tracer;
}

static void die(lcb_INSTANCE *instance, const char *msg, lcb_STATUS err)
{
    fprintf(stderr, "%s. Received code 0x%X (%s)\n", msg, err, lcb_strerror(instance, err));
    exit(EXIT_FAILURE);
}

static void view_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPVIEW *rv)
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
}

int main(int argc, char *argv[])
{
    lcb_STATUS err;
    lcb_INSTANCE *instance;
    struct lcb_create_st create_options = {0};
    lcbtrace_SPAN *span = NULL;
    lcbtrace_TRACER *tracer = NULL;

    create_options.version = 3;

    if (argc < 2) {
        fprintf(stderr, "Usage: %s couchbase://host/bucket [ password [ username ] ]\n", argv[0]);
        exit(EXIT_FAILURE);
    }

    create_options.v.v3.connstr = argv[1];
    if (argc > 2) {
        create_options.v.v3.passwd = argv[2];
    }
    if (argc > 3) {
        create_options.v.v3.username = argv[3];
    }

    srand(time(NULL));

    err = lcb_create(&instance, &create_options);
    if (err != LCB_SUCCESS) {
        die(NULL, "Couldn't create couchbase handle", err);
    }

    err = lcb_connect(instance);
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't schedule connection", err);
    }

    lcb_wait(instance);

    err = lcb_get_bootstrap_status(instance);
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't bootstrap from cluster", err);
    }

    tracer = zipkin_new();

    lcb_set_tracer(instance, tracer);

    span = lcbtrace_span_start(tracer, "transaction", 0, NULL);
    lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_COMPONENT, COMPONENT_NAME);

    {
        int encoding_time_us = rand() % 1000;
        lcbtrace_SPAN *encoding;
        lcbtrace_REF ref;

        ref.type = LCBTRACE_REF_CHILD_OF;
        ref.span = span;

        encoding = lcbtrace_span_start(tracer, LCBTRACE_OP_REQUEST_ENCODING, 0, &ref);
        lcbtrace_span_add_tag_str(encoding, LCBTRACE_TAG_COMPONENT, COMPONENT_NAME);
        usleep(encoding_time_us);
        lcbtrace_span_finish(encoding, LCBTRACE_NOW);
    }

    lcb_CMDVIEW *vcmd;
    char *doc_name = "beer";
    char *view_name = "by_location";
    char *options = "reduce=false&limit=3";

    lcb_cmdview_create(&vcmd);
    lcb_cmdview_callback(vcmd, view_callback);
    lcb_cmdview_design_document(vcmd, doc_name, strlen(doc_name));
    lcb_cmdview_view_name(vcmd, view_name, strlen(view_name));
    lcb_cmdview_option_string(vcmd, options, strlen(options));
    lcb_cmdview_include_docs(vcmd, 1);
    lcb_cmdview_parent_span(vcmd, span);
    lcb_VIEW_HANDLE *handle;
    lcb_cmdview_handle(vcmd, &handle);

    err = lcb_view(instance, NULL, vcmd);
    lcb_cmdview_destroy(vcmd);
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't schedule view operation", err);
    }

    /* The store_callback is invoked from lcb_wait() */
    fprintf(stderr, "Will wait for view operation to complete..\n");
    lcb_wait(instance);

    lcbtrace_span_finish(span, LCBTRACE_NOW);

    zipkin_flush(tracer);

    /* Now that we're all done, close down the connection handle */
    lcb_destroy(instance);

    return 0;
}
