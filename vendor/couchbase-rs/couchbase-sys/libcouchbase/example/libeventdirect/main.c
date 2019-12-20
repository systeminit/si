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

/**
 * gcc -levent -lcouchbase main.c
 *
 * # perform STORE and 20 iterations of GET commands with interval 3 seconds
 * ./a.out couchbase://localhost password Administrator 20 3
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libcouchbase/couchbase.h>
#include <event2/event.h>

const char key[] = "foo";
lcb_SIZE nkey = sizeof(key);

const char val[] = "{\"answer\":42}";
lcb_SIZE nval = sizeof(val);

int nreq = 1;
int nresp = 1;
int interval = 0;
struct event *timer = NULL;

static void bootstrap_callback(lcb_INSTANCE *instance, lcb_STATUS err)
{
    lcb_CMDSTORE *cmd;
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "ERROR: %s\n", lcb_strerror(instance, err));
        exit(EXIT_FAILURE);
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
        fprintf(stderr, "Failed to set up store request: %s\n", lcb_strerror(instance, err));
        exit(EXIT_FAILURE);
    }
}

static void get_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPGET *rg)
{
    const char *value;
    size_t nvalue;
    lcb_STATUS rc = lcb_respget_status(rg);

    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Failed to get key: %s\n", lcb_strerror(instance, rc));
        exit(EXIT_FAILURE);
    }

    lcb_respget_value(rg, &value, &nvalue);
    printf("%d. retrieved the key 'foo', value: %.*s\n", nresp, (int)nvalue, value);
    fflush(stdout);
    nresp--;
    if (nresp == 0) {
        printf("stopping the loop\n");
        event_base_loopbreak((void *)lcb_get_cookie(instance));
    }
    (void)cbtype;
}

static void schedule_timer();

static void timer_callback(int fd, short event, void *arg)
{
    lcb_INSTANCE *instance = arg;
    lcb_STATUS rc;
    lcb_CMDGET *gcmd;

    lcb_cmdget_create(&gcmd);
    lcb_cmdget_key(gcmd, key, nkey);
    rc = lcb_get(instance, NULL, gcmd);
    lcb_cmdget_destroy(gcmd);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Failed to schedule get request: %s\n", lcb_strerror(NULL, rc));
        exit(EXIT_FAILURE);
    }
    (void)fd;
    (void)event;
    schedule_timer();
}

static void schedule_timer()
{
    struct timeval tv;

    if (!nreq) {
        return;
    }
    tv.tv_sec = interval;
    tv.tv_usec = 0;
    evtimer_add(timer, &tv);
    nreq--;
}

static void store_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPSTORE *resp)
{
    lcb_STATUS rc = lcb_respstore_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Failed to store key: %s\n", lcb_strerror(instance, rc));
        exit(EXIT_FAILURE);
    }
    printf("stored key 'foo'\n");
    fflush(stdout);
    {
        struct event_base *evbase = (struct event_base *)lcb_get_cookie(instance);

        printf("try to get value %d times with %dsec interval\n", nreq, interval);
        timer = evtimer_new(evbase, timer_callback, instance);
        schedule_timer();
    }

    (void)cbtype;
}

static lcb_io_opt_t create_libevent_io_ops(struct event_base *evbase)
{
    struct lcb_create_io_ops_st ciops;
    lcb_io_opt_t ioops;
    lcb_STATUS error;

    memset(&ciops, 0, sizeof(ciops));
    ciops.v.v0.type = LCB_IO_OPS_LIBEVENT;
    ciops.v.v0.cookie = evbase;

    error = lcb_create_io_ops(&ioops, &ciops);
    if (error != LCB_SUCCESS) {
        fprintf(stderr, "Failed to create an IOOPS structure for libevent: %s\n", lcb_strerror(NULL, error));
        exit(EXIT_FAILURE);
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
        exit(EXIT_FAILURE);
    }

    /* Set up the callbacks */
    lcb_set_bootstrap_callback(instance, bootstrap_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)get_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);

    if ((error = lcb_connect(instance)) != LCB_SUCCESS) {
        fprintf(stderr, "Failed to connect libcouchbase instance: %s\n", lcb_strerror(NULL, error));
        lcb_destroy(instance);
        exit(EXIT_FAILURE);
    }

    return instance;
}

/* This example shows how we can hook ourself into an external event loop.
 * You may find more information in the blogpost: http://goo.gl/fCTrX */
int main(int argc, char **argv)
{
    struct event_base *evbase = event_base_new();
    lcb_io_opt_t ioops = create_libevent_io_ops(evbase);
    lcb_INSTANCE *instance = create_libcouchbase_handle(ioops, argc, argv);

    if (argc > 4) {
        nreq = nresp = atoi(argv[4]);
    }
    if (argc > 5) {
        interval = atoi(argv[5]);
    }
    /*Store the event base as the user cookie in our instance so that
     * we may terminate the program when we're done */
    lcb_set_cookie(instance, evbase);

    /* Run the event loop */
    event_base_loop(evbase, 0);

    /* Cleanup */
    lcb_destroy(instance);
    if (timer) {
        evtimer_del(timer);
    }
    lcb_destroy_io_ops(ioops);
    event_base_free(evbase);

    return EXIT_SUCCESS;
}
