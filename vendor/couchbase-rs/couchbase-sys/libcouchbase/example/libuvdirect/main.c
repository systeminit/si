/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2019 Couchbase, Inc.
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
 * cmake -DLCB_BUILD_EXAMPLES=ON .
 * make
 *
 * # perform STORE and 20 iterations of GET commands with interval 3 seconds
 * ./build/bin/examples/libevent-direct couchbase://localhost password Administrator 20 3
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libcouchbase/couchbase.h>
#include <uv.h>

const char key[] = "foo";
lcb_SIZE nkey = sizeof(key);

const char val[] = "{\"answer\":42}";
lcb_SIZE nval = sizeof(val);

int nreq = 1;
int nresp = 1;
int interval = 0;

uv_timer_t timer;

static void timer_close_cb(uv_handle_t *handle)
{
    (void)handle;
}

static void delete_timer()
{
    uv_timer_stop(&timer);
    uv_close((uv_handle_t *)&timer, timer_close_cb);
}

static void bootstrap_callback(lcb_INSTANCE *instance, lcb_STATUS err)
{
    lcb_CMDSTORE *cmd;
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "bootstrap error: %s\n", lcb_strerror(instance, err));
        lcb_destroy_async(instance, NULL);
        return;
    }
    printf("successfully bootstrapped\n");
    fflush(stdout);
    /* Since we've got our configuration, let's go ahead and store a value */
    lcb_cmdstore_create(&cmd, LCB_STORE_SET);
    lcb_cmdstore_key(cmd, key, nkey);
    lcb_cmdstore_value(cmd, val, nval);
    err = lcb_store(instance, NULL, cmd);
    lcb_cmdstore_destroy(cmd);
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "failed to set up store request: %s\n", lcb_strerror(instance, err));
        lcb_destroy_async(instance, NULL);
        return;
    }
}

static void get_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPGET *rg)
{
    const char *value;
    size_t nvalue;
    lcb_STATUS rc = lcb_respget_status(rg);

    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "failed to get key: %s\n", lcb_strerror(instance, rc));
        lcb_destroy_async(instance, NULL);
        return;
    }

    lcb_respget_value(rg, &value, &nvalue);
    printf("%d. retrieved the key 'foo', value(%d): %.*s\n", nresp, (int)nvalue, (int)nvalue, value);
    fflush(stdout);
    nresp--;
    if (nresp == 0) {
        printf("done with libcouchbase. Destroying it\n");
        delete_timer();
        lcb_destroy_async(instance, NULL);
    }
    (void)cbtype;
}

static void schedule_timer(lcb_INSTANCE *instance);

static void timer_callback(uv_timer_t *event)
{
    lcb_INSTANCE *instance = event->data;
    lcb_STATUS rc;
    lcb_CMDGET *gcmd;

    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key, nkey);
    rc = lcb_get(instance, NULL, gcmd);
    lcb_cmdget_destroy(gcmd);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "failed to schedule get request: %s\n", lcb_strerror(NULL, rc));
        delete_timer();
        lcb_destroy_async(instance, NULL);
        return;
    }
    schedule_timer(instance);
}

static void schedule_timer(lcb_INSTANCE *instance)
{
    if (!nreq) {
        return;
    }
    timer.data = instance;
    uv_timer_start(&timer, timer_callback, interval, 0);
    nreq--;
}

static void store_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPSTORE *resp)
{
    lcb_STATUS rc = lcb_respstore_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "failed to store key: %s\n", lcb_strerror(instance, rc));
        lcb_destroy_async(instance, NULL);
        return;
    }
    printf("stored key 'foo'\n");
    fflush(stdout);
    {
        uv_loop_t *evbase = (uv_loop_t *)lcb_get_cookie(instance);

        printf("try to get value %d times with %dsec interval\n", nreq, interval);
        uv_timer_init(evbase, &timer);
        schedule_timer(instance);
    }

    (void)cbtype;
}

static lcb_io_opt_t create_libuv_io_ops(uv_loop_t *evbase)
{
    struct lcb_create_io_ops_st ciops;
    lcb_io_opt_t ioops;
    lcb_STATUS error;
    struct {
        int version;
        union {
            struct {
                uv_loop_t *loop;
                int startsop_noop;
            } v0;
        } v;
    } cookie = {};

    cookie.version = 0;
    cookie.v.v0.loop = evbase;
    cookie.v.v0.startsop_noop = 1;

    memset(&ciops, 0, sizeof(ciops));
    ciops.v.v0.type = LCB_IO_OPS_LIBUV;
    ciops.v.v0.cookie = &cookie;

    error = lcb_create_io_ops(&ioops, &ciops);
    if (error != LCB_SUCCESS) {
        fprintf(stderr, "Failed to create an IOOPS structure for libuv: %s\n", lcb_strerror(NULL, error));
        return NULL;
    }

    return ioops;
}

static lcb_INSTANCE *create_libcouchbase_handle(lcb_io_opt_t ioops, int argc, char **argv)
{
    lcb_INSTANCE *instance;
    lcb_STATUS error;
    struct lcb_create_st copts;

    memset(&copts, 0, sizeof(copts));

    /* If NULL, will default to localhost */
    copts.version = 3;
    copts.v.v3.connstr = "couchbase://localhost";

    if (argc > 1) {
        copts.v.v3.connstr = argv[1];
    }
    if (argc > 2) {
        copts.v.v3.passwd = argv[2];
    }
    if (argc > 3) {
        copts.v.v3.username = argv[3];
    }
    copts.v.v3.io = ioops;
    error = lcb_create(&instance, &copts);

    if (error != LCB_SUCCESS) {
        fprintf(stderr, "Failed to create a libcouchbase instance: %s\n", lcb_strerror(NULL, error));
        return NULL;
    }

    /* Set up the callbacks */
    lcb_set_bootstrap_callback(instance, bootstrap_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)get_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);

    if ((error = lcb_connect(instance)) != LCB_SUCCESS) {
        fprintf(stderr, "Failed to connect libcouchbase instance: %s\n", lcb_strerror(NULL, error));
        lcb_destroy(instance);
        return NULL;
    }

    return instance;
}

int main(int argc, char **argv)
{
    uv_loop_t evbase;
    uv_loop_init(&evbase);
    lcb_io_opt_t ioops;
    lcb_INSTANCE *instance;

    ioops = create_libuv_io_ops(&evbase);
    if (ioops == NULL) {
        exit(EXIT_FAILURE);
    }
    instance = create_libcouchbase_handle(ioops, argc, argv);
    if (instance == NULL) {
        exit(EXIT_FAILURE);
    }

    if (argc > 4) {
        nreq = nresp = atoi(argv[4]);
    }
    if (argc > 5) {
        interval = atoi(argv[4]);
    }
    /* Store the event base as the user cookie in our instance so that
     * we may terminate the program when we're done */
    lcb_set_cookie(instance, &evbase);

    /* Run the event loop */
    uv_run(&evbase, UV_RUN_DEFAULT);

    /* dump some libuv stats */
    fprintf(stderr, "uv_loop_alive(): %d\n", uv_loop_alive(&evbase));
    fprintf(stderr, "evbase.active_handles: %d\n", evbase.active_handles);
    fprintf(stderr, "evbase.active_reqs.count: %d\n", evbase.active_reqs.count);
    fprintf(stderr, "evbase.closing_handles: %p\n", (void *)evbase.closing_handles);

    uv_loop_close(&evbase);
    lcb_destroy_io_ops(ioops);

    return EXIT_SUCCESS;
}
