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

#include <stdio.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/crypto.h>
#include <stdlib.h>
#include <string.h> /* strlen */
#ifdef _WIN32
#define PRIx64 "I64x"
#else
#include <inttypes.h>
#endif

#include "openssl_symmetric_provider.h"

static void die(lcb_INSTANCE *instance, const char *msg, lcb_STATUS err)
{
    fprintf(stderr, "%s. Received code 0x%X (%s)\n", msg, err, lcb_strerror(instance, err));
    exit(EXIT_FAILURE);
}

static void op_callback(lcb_INSTANCE *instance, int cbtype, const lcb_RESPBASE *rb)
{
    if (rb->rc == LCB_SUCCESS) {
        const lcb_RESPGET *rg = (const lcb_RESPGET *)rb;
        lcbcrypto_CMDDECRYPT dcmd = {};
        lcbcrypto_FIELDSPEC field = {};
        lcb_STATUS err;

        printf("VALUE:  %.*s\n", (int)rg->nvalue, rg->value);
        dcmd.version = 0;
        dcmd.prefix = NULL;
        dcmd.doc = rg->value;
        dcmd.ndoc = rg->nvalue;
        dcmd.out = NULL;
        dcmd.nout = 0;
        dcmd.nfields = 1;
        dcmd.fields = &field;
        field.name = "message";
        field.alg = "AES-256-HMAC-SHA256";
        err = lcbcrypto_decrypt_fields(instance, &dcmd);
        if (err != LCB_SUCCESS) {
            die(instance, "Couldn't decrypt field 'message'", err);
        }
        if (dcmd.out == NULL) {
            die(instance, "Crypto provider returned success, but document is NULL", LCB_EINVAL);
        }
        /* chop trailing LF for nicer look */
        if (dcmd.out[dcmd.nout - 1] == '\n') {
            dcmd.out[dcmd.nout - 1] = ' ';
        }
        printf("PLAIN:  %.*s\n", (int)dcmd.nout, dcmd.out);
        free(dcmd.out); // NOTE: it should be compatible with what providers use to allocate memory
        printf("CAS:    0x%" PRIx64 "\n", rb->cas);
    } else {
        die(instance, lcb_strcbtype(cbtype), rb->rc);
    }
}

static void get_encrypted(lcb_INSTANCE *instance, const char *key)
{
    lcb_CMDGET cmd = {};
    lcb_STATUS err;
    LCB_CMD_SET_KEY(&cmd, key, strlen(key));
    printf("KEY:    %s\n", key);
    err = lcb_get3(instance, NULL, &cmd);
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't schedule get operation", err);
    }
    lcb_wait(instance);
}

int main(int argc, char *argv[])
{
    lcb_STATUS err;
    lcb_INSTANCE *instance;

    {
        struct lcb_create_st create_options = {};
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

        lcb_install_callback3(instance, LCB_CALLBACK_GET, op_callback);
    }

    lcbcrypto_register(instance, "AES-256-HMAC-SHA256", osp_create());

    get_encrypted(instance, "secret-1");
    printf("\n");
    get_encrypted(instance, "secret-2");
    printf("\n");
    get_encrypted(instance, "secret-3");
    printf("\n");
    get_encrypted(instance, "secret-4");
    printf("\n");
    get_encrypted(instance, "secret-5");

    lcb_destroy(instance);
    return 0;
}
