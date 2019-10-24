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

/*
 * BUILD: `cc -o observe observe.c -lcouchbase`
 * RUN: `./observe key`
 */
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/utils.h>

#define fail(msg)                                                                                                      \
    fprintf(stderr, "%s\n", msg);                                                                                      \
    exit(EXIT_FAILURE)

#define fail2(msg, err)                                                                                                \
    fprintf(stderr, "%s\n", msg);                                                                                      \
    fprintf(stderr, "Error was 0x%x (%s)\n", err, lcb_strerror(NULL, err));                                            \
    exit(EXIT_FAILURE)

typedef struct {
    int master;
    lcb_U8 status;
    lcb_U64 cas;
} node_info;

typedef struct {
    unsigned nresp;
    node_info *nodeinfo;
} observe_info;

static void observe_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
{
    const lcb_RESPOBSERVE *resp = (const lcb_RESPOBSERVE *)rb;
    observe_info *obs_info = (observe_info *)resp->cookie;
    node_info *ni = &obs_info->nodeinfo[obs_info->nresp];

    if (resp->nkey == 0) {
        fprintf(stderr, "All nodes have replied\n");
        return;
    }

    if (resp->rc != LCB_SUCCESS) {
        fprintf(stderr, "Failed to observe key from node. 0x%x (%s)\n", resp->rc, lcb_strerror(instance, resp->rc));
        obs_info->nresp++;
        return;
    }

    /* Copy over the fields we care about */
    ni->cas = resp->cas;
    ni->status = resp->status;
    ni->master = resp->ismaster;

    /* Increase the response counter */
    obs_info->nresp++;
}

int main(int argc, char *argv[])
{
    lcb_INSTANCE *instance;
    lcb_STATUS err;
    lcb_CMDOBSERVE cmd = {0};
    lcb_MULTICMD_CTX *mctx = NULL;
    observe_info obs_info;
    unsigned nservers, ii;
    struct lcb_create_st create_options = {0};

    if (argc < 2) {
        fail("Requires key as argument\n"
             "Usage: observe KEY [CONNSTRING [ PASSWORD [ USERNAME ] ] ]\n");
    }
    create_options.version = 3;
    if (argc > 2) {
        create_options.v.v3.connstr = argv[2];
    }
    if (argc > 3) {
        create_options.v.v3.passwd = argv[3];
    }
    if (argc > 4) {
        create_options.v.v3.username = argv[4];
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
    lcb_install_callback3(instance, LCB_CALLBACK_OBSERVE, observe_callback);

    nservers = lcb_get_num_nodes(instance);
    obs_info.nodeinfo = calloc(nservers, sizeof(*obs_info.nodeinfo));
    obs_info.nresp = 0;

    mctx = lcb_observe3_ctxnew(instance);
    LCB_CMD_SET_KEY(&cmd, argv[1], strlen(argv[1]));
    mctx->addcmd(mctx, (const lcb_CMDBASE *)&cmd);

    printf("observing the state of '%s':\n", argv[1]);
    if ((err = mctx->done(mctx, &obs_info)) != LCB_SUCCESS) {
        fail2("Couldn't schedule observe request", err);
    }

    lcb_wait(instance);
    for (ii = 0; ii < obs_info.nresp; ii++) {
        node_info *ni = &obs_info.nodeinfo[ii];
        fprintf(stderr, "Got status from %s node:\n", ni->master ? "master" : "replica");
        fprintf(stderr, "\tCAS: 0x0%llx\n", (unsigned long long)ni->cas);
        fprintf(stderr, "\tStatus (RAW): 0x%02x\n", ni->status);
        fprintf(stderr, "\tExists [CACHE]: %s\n", ni->status & LCB_OBSERVE_NOT_FOUND ? "No" : "Yes");
        fprintf(stderr, "\tExists [DISK]: %s\n", ni->status & LCB_OBSERVE_PERSISTED ? "Yes" : "No");
        fprintf(stderr, "\n");
    }

    /* The next example shows how to use lcb_observe() to only request the
     * CAS from the master node */
    obs_info.nresp = 0;
    memset(obs_info.nodeinfo, 0, sizeof(obs_info.nodeinfo[0]) * nservers);

    fprintf(stderr, "Will request CAS from master...\n");
    cmd.cmdflags |= LCB_CMDOBSERVE_F_MASTER_ONLY;
    mctx = lcb_observe3_ctxnew(instance);
    mctx->addcmd(mctx, (const lcb_CMDBASE *)&cmd);
    if ((err = mctx->done(mctx, &obs_info)) != LCB_SUCCESS) {
        fail2("Couldn't schedule observe request!\n", err);
    }

    lcb_wait(instance);

    assert(obs_info.nresp == 1 && obs_info.nodeinfo[0].master);
    fprintf(stderr, "CAS on master is 0x%llx\n", (unsigned long long)obs_info.nodeinfo[0].cas);

    lcb_destroy(instance);
    free(obs_info.nodeinfo);
    return EXIT_SUCCESS;
}
