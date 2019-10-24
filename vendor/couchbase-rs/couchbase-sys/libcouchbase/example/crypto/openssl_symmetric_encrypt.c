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
        fprintf(stderr, "CAS:    0x%" PRIx64 "\n", rb->cas);
    } else {
        die(instance, lcb_strcbtype(cbtype), rb->rc);
    }
}

static void store_encrypted(lcb_INSTANCE *instance, const char *key, const char *val)
{
    lcb_STATUS err;
    lcb_CMDSTORE cmd = {};
    lcbcrypto_CMDENCRYPT ecmd = {};
    lcbcrypto_FIELDSPEC field = {};

    printf("KEY:    %s\n", key);
    printf("PLAIN:  %s\n", val);

    ecmd.version = 0;
    ecmd.prefix = NULL;
    ecmd.doc = val;
    ecmd.ndoc = strlen(val);
    ecmd.out = NULL;
    ecmd.nout = 0;
    ecmd.nfields = 1;
    ecmd.fields = &field;
    field.name = "message";
    field.alg = "AES-256-HMAC-SHA256";

    err = lcbcrypto_encrypt_fields(instance, &ecmd);
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't encrypt field 'message'", err);
    }
    /* chop trailing LF for nicer look */
    if (ecmd.out[ecmd.nout - 1] == '\n') {
        ecmd.out[ecmd.nout - 1] = ' ';
    }
    printf("CIPHER: %s\n", ecmd.out);

    LCB_CMD_SET_KEY(&cmd, key, strlen(key));
    LCB_CMD_SET_VALUE(&cmd, ecmd.out, ecmd.nout);
    cmd.operation = LCB_STORE_SET cmd.datatype = LCB_DATATYPE_JSON;

    err = lcb_store3(instance, NULL, &cmd);
    free(ecmd.out); // NOTE: it should be compatible with what providers use to allocate memory
    if (err != LCB_SUCCESS) {
        die(instance, "Couldn't schedule storage operation", err);
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

        lcb_install_callback3(instance, LCB_CALLBACK_STORE, op_callback);
    }

    lcbcrypto_register(instance, "AES-256-HMAC-SHA256", osp_create());

    store_encrypted(instance, "secret-1", "{\"message\":\"The old grey goose jumped over the wrickety gate.\"}");
    printf("\n");
    store_encrypted(instance, "secret-2", "{\"message\":10}");
    printf("\n");
    store_encrypted(instance, "secret-3", "{\"message\":\"10\"}");
    printf("\n");
    store_encrypted(
        instance, "secret-4",
        "{\"message\":[\"The\",\"Old\",\"Grey\",\"Goose\",\"Jumped\",\"over\",\"the\",\"wrickety\",\"gate\"]}");
    printf("\n");
    store_encrypted(instance, "secret-5",
                    "{\"message\":{\"myValue\":\"The old grey goose jumped over the wrickety gate.\",\"myInt\":10}}");

    lcb_destroy(instance);
    return 0;
}
