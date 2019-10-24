/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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

#define NOMINMAX
#include <map>
#include <sstream>
#include <iostream>
#include <iomanip>
#include <fstream>
#include <algorithm>
#include <limits>
#include <stddef.h>
#include <errno.h>
#include "common/options.h"
#include "common/histogram.h"
#include "cbc-handlers.h"
#include "connspec.h"
#include "rnd.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <libcouchbase/vbucket.h>
#include <libcouchbase/utils.h>

#ifndef LCB_NO_SSL
#include <openssl/crypto.h>
#endif
#include <snappy-stubs-public.h>

using namespace cbc;

using std::map;
using std::string;
using std::stringstream;
using std::vector;

static void printEnhancedError(int cbtype, const lcb_RESPBASE *resp, const char *additional = NULL)
{
    const char *ctx = lcb_resp_get_error_context(cbtype, resp);
    if (ctx != NULL) {
        fprintf(stderr, "%-20s %s\n", "", ctx);
    }
    const char *ref = lcb_resp_get_error_ref(cbtype, resp);
    if (ref != NULL) {
        fprintf(stderr, "%-20s Ref: %s\n", "", ref);
    }
    if (additional) {
        fprintf(stderr, "%-20s %s\n", "", additional);
    }
}

static void printKeyError(string &key, lcb_STATUS rc, int cbtype, const lcb_RESPBASE *resp,
                          const char *additional = NULL)
{
    fprintf(stderr, "%-20s %s\n", key.c_str(), lcb_strerror_short(rc));
    printEnhancedError(cbtype, resp, additional);
}

extern "C" {
static void get_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE cbtype, const lcb_RESPGET *resp)
{
    const char *p;
    size_t n;
    lcb_respget_key(resp, &p, &n);
    string key(p, n);
    lcb_STATUS rc = lcb_respget_status(resp);
    if (rc == LCB_SUCCESS) {
        const char *value;
        size_t nvalue;
        uint32_t flags;
        uint64_t cas;
        uint8_t datatype;

        lcb_respget_value(resp, &value, &nvalue);
        lcb_respget_flags(resp, &flags);
        lcb_respget_cas(resp, &cas);
        lcb_respget_datatype(resp, &datatype);
        fprintf(stderr, "%-20s CAS=0x%" PRIx64 ", Flags=0x%x, Size=%lu, Datatype=0x%02x", key.c_str(), cas, flags,
                (unsigned long)nvalue, (int)datatype);
        if (datatype) {
            int nflags = 0;
            fprintf(stderr, "(");
            if (datatype & LCB_VALUE_F_JSON) {
                fprintf(stderr, "JSON");
                nflags++;
            }
            if (datatype & LCB_VALUE_F_SNAPPYCOMP) {
                fprintf(stderr, "%sSNAPPY", nflags > 0 ? "," : "");
                nflags++;
            }
            fprintf(stderr, ")");
        }
        fprintf(stderr, "\n");
        fflush(stderr);
        fwrite(value, 1, nvalue, stdout);
        fflush(stdout);
        fprintf(stderr, "\n");
    } else {
        printKeyError(key, rc, cbtype, (const lcb_RESPBASE *)resp);
    }
}

static void getreplica_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE cbtype, const lcb_RESPGETREPLICA *resp)
{
    const char *p;
    size_t n;
    lcb_respgetreplica_key(resp, &p, &n);
    string key(p, n);
    lcb_STATUS rc = lcb_respgetreplica_status(resp);
    if (rc == LCB_SUCCESS) {
        const char *value;
        size_t nvalue;
        uint32_t flags;
        uint64_t cas;
        uint8_t datatype;

        lcb_respgetreplica_value(resp, &value, &nvalue);
        lcb_respgetreplica_flags(resp, &flags);
        lcb_respgetreplica_cas(resp, &cas);
        lcb_respgetreplica_datatype(resp, &datatype);
        fprintf(stderr, "%-20s CAS=0x%" PRIx64 ", Flags=0x%x, Size=%lu, Datatype=0x%02x", key.c_str(), cas, flags,
                (unsigned long)nvalue, (int)datatype);
        if (datatype) {
            int nflags = 0;
            fprintf(stderr, "(");
            if (datatype & LCB_VALUE_F_JSON) {
                fprintf(stderr, "JSON");
                nflags++;
            }
            if (datatype & LCB_VALUE_F_SNAPPYCOMP) {
                fprintf(stderr, "%sSNAPPY", nflags > 0 ? "," : "");
                nflags++;
            }
            fprintf(stderr, ")");
        }
        fprintf(stderr, "\n");
        fflush(stderr);
        fwrite(value, 1, nvalue, stdout);
        fflush(stdout);
        fprintf(stderr, "\n");
    } else {
        printKeyError(key, rc, cbtype, (const lcb_RESPBASE *)resp);
    }
}

static void storePrintSuccess(const lcb_RESPSTORE *resp, const char *message = NULL)
{
    const char *key;
    size_t nkey;

    lcb_respstore_key(resp, &key, &nkey);
    fprintf(stderr, "%-20.*s ", (int)nkey, key);
    if (message != NULL) {
        fprintf(stderr, "%s ", message);
    }

    uint64_t cas;
    lcb_respstore_cas(resp, &cas);
    fprintf(stderr, "CAS=0x%" PRIx64 "\n", cas);

    lcb_MUTATION_TOKEN token = {0};
    lcb_respstore_mutation_token(resp, &token);
    if (lcb_mutation_token_is_valid(&token)) {
        fprintf(stderr, "%-20s SYNCTOKEN=%u,%" PRIu64 ",%" PRIu64 "\n", "", token.vbid_, token.uuid_, token.seqno_);
    }
}

static void storePrintError(const lcb_RESPSTORE *resp, const char *message = NULL)
{
    size_t sz = 0;
    const char *key;

    lcb_respstore_key(resp, &key, &sz);
    fprintf(stderr, "%-20.*s %s\n", (int)sz, key, lcb_strerror_short(lcb_respstore_status(resp)));

    const char *ctx = NULL;
    lcb_respstore_error_context(resp, &ctx, &sz);
    if (ctx != NULL) {
        fprintf(stderr, "%-20s %.*s\n", "", (int)sz, ctx);
    }

    const char *ref = NULL;
    lcb_respstore_error_ref(resp, &ref, &sz);
    if (ref != NULL) {
        fprintf(stderr, "%-20s Ref: %.*s\n", "", (int)sz, ref);
    }

    if (message) {
        fprintf(stderr, "%-20s %s\n", "", message);
    }
}

static void store_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTORE *resp)
{
    lcb_STATUS rc = lcb_respstore_status(resp);

    if (lcb_respstore_observe_attached(resp)) {
        // Storage with durability
        char buf[4096];
        uint16_t npersisted, nreplicated;
        lcb_respstore_observe_num_persisted(resp, &npersisted);
        lcb_respstore_observe_num_replicated(resp, &nreplicated);
        if (rc == LCB_SUCCESS) {
            sprintf(buf, "Stored. Persisted(%u). Replicated(%u)", npersisted, nreplicated);
            storePrintSuccess(resp, buf);
        } else {
            int store_ok;
            lcb_respstore_observe_stored(resp, &store_ok);
            if (store_ok) {
                sprintf(buf, "Store OK, but durability failed. Persisted(%u). Replicated(%u)", npersisted, nreplicated);
            } else {
                sprintf(buf, "%s", "Store failed");
            }
            storePrintError(resp, buf);
        }
    } else {
        if (rc == LCB_SUCCESS) {
            storePrintSuccess(resp, "Stored.");
        } else {
            storePrintError(resp);
        }
    }
}

static void exists_callback(lcb_INSTANCE *, int type, const lcb_RESPEXISTS *resp)
{
    const char *p;
    size_t n;

    lcb_respexists_key(resp, &p, &n);
    string key(p, n);

    lcb_STATUS rc = lcb_respexists_status(resp);
    if (rc != LCB_SUCCESS) {
        printKeyError(key, rc, type, (const lcb_RESPBASE *)resp);
        return;
    }
    if (lcb_respexists_is_found(resp)) {
        uint64_t cas;
        lcb_respexists_cas(resp, &cas);
        fprintf(stderr, "%-20s FOUND, CAS=0x%" PRIx64 "\n", key.c_str(), cas);
    } else {
        fprintf(stderr, "%-20s NOT FOUND\n", key.c_str());
    }
}

static void unlock_callback(lcb_INSTANCE *, int type, const lcb_RESPUNLOCK *resp)
{
    const char *p;
    size_t n;

    lcb_respunlock_key(resp, &p, &n);
    string key(p, n);

    lcb_STATUS rc = lcb_respunlock_status(resp);
    if (rc != LCB_SUCCESS) {
        printKeyError(key, rc, type, (const lcb_RESPBASE *)resp);
        return;
    }
    fprintf(stderr, "%-20s Unlocked\n", key.c_str());
}

static void remove_callback(lcb_INSTANCE *, int type, const lcb_RESPREMOVE *resp)
{
    const char *p;
    size_t n;

    lcb_respremove_key(resp, &p, &n);
    string key(p, n);

    lcb_STATUS rc = lcb_respremove_status(resp);
    if (rc != LCB_SUCCESS) {
        printKeyError(key, rc, type, (const lcb_RESPBASE *)resp);
        return;
    }
    fprintf(stderr, "%-20s Deleted\n", key.c_str());
}

static void touch_callback(lcb_INSTANCE *, int type, const lcb_RESPTOUCH *resp)
{
    const char *p;
    size_t n;

    lcb_resptouch_key(resp, &p, &n);
    string key(p, n);

    lcb_STATUS rc = lcb_resptouch_status(resp);
    if (rc != LCB_SUCCESS) {
        printKeyError(key, rc, type, (const lcb_RESPBASE *)resp);
        return;
    }
    fprintf(stderr, "%-20s Touch\n", key.c_str());
}

static void observe_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE cbtype, const lcb_RESPOBSERVE *resp)
{
    if (resp->nkey == 0) {
        return;
    }

    string key((const char *)resp->key, resp->nkey);
    if (resp->rc == LCB_SUCCESS) {
        fprintf(stderr, "%-20s [%s] Status=0x%x, CAS=0x%" PRIx64 "\n", key.c_str(),
                resp->ismaster ? "Master" : "Replica", resp->status, resp->cas);
    } else {
        printKeyError(key, resp->rc, cbtype, (const lcb_RESPBASE *)resp);
    }
}

static void obseqno_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPOBSEQNO *resp)
{
    int ix = resp->server_index;
    if (resp->rc != LCB_SUCCESS) {
        fprintf(stderr, "[%d] ERROR %s\n", ix, lcb_strerror_long(resp->rc));
        return;
    }
    lcb_U64 uuid, seq_disk, seq_mem;
    if (resp->old_uuid) {
        seq_disk = seq_mem = resp->old_seqno;
        uuid = resp->old_uuid;
    } else {
        uuid = resp->cur_uuid;
        seq_disk = resp->persisted_seqno;
        seq_mem = resp->mem_seqno;
    }
    fprintf(stderr, "[%d] UUID=0x%" PRIx64 ", Cache=%" PRIu64 ", Disk=%" PRIu64, ix, uuid, seq_mem, seq_disk);
    if (resp->old_uuid) {
        fprintf(stderr, "\n");
        fprintf(stderr, "    FAILOVER. New: UUID=%" PRIx64 ", Cache=%" PRIu64 ", Disk=%" PRIu64, resp->cur_uuid,
                resp->mem_seqno, resp->persisted_seqno);
    }
    fprintf(stderr, "\n");
}

static void stats_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTATS *resp)
{
    if (resp->rc != LCB_SUCCESS) {
        fprintf(stderr, "ERROR %s\n", lcb_strerror_long(resp->rc));
        return;
    }
    if (resp->server == NULL || resp->key == NULL) {
        return;
    }

    string server = resp->server;
    string key((const char *)resp->key, resp->nkey);
    string value;
    if (resp->nvalue > 0) {
        value.assign((const char *)resp->value, resp->nvalue);
    }
    fprintf(stdout, "%s\t%s", server.c_str(), key.c_str());
    if (!value.empty()) {
        if (*static_cast< bool * >(resp->cookie) && key == "key_flags") {
            // Is keystats
            // Flip the bits so the display formats correctly
            unsigned flags_u = 0;
            sscanf(value.c_str(), "%u", &flags_u);
            flags_u = htonl(flags_u);
            fprintf(stdout, "\t%u (cbc: converted via htonl)", flags_u);
        } else {
            fprintf(stdout, "\t%s", value.c_str());
        }
    }
    fprintf(stdout, "\n");
}

static void watch_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE, const lcb_RESPSTATS *resp)
{
    if (resp->rc != LCB_SUCCESS) {
        fprintf(stderr, "ERROR %s\n", lcb_strerror_long(resp->rc));
        return;
    }
    if (resp->server == NULL || resp->key == NULL) {
        return;
    }

    string key((const char *)resp->key, resp->nkey);
    if (resp->nvalue > 0) {
        char *nptr = NULL;
        uint64_t val =
#ifdef _WIN32
            _strtoi64
#else
            strtoll
#endif
            ((const char *)resp->value, &nptr, 10);
        if (nptr != (const char *)resp->value) {
            map< string, int64_t > *entry = reinterpret_cast< map< string, int64_t > * >(resp->cookie);
            (*entry)[key] += val;
        }
    }
}

static void common_server_callback(lcb_INSTANCE *, int cbtype, const lcb_RESPSERVERBASE *sbase)
{
    string msg;
    if (cbtype == LCB_CALLBACK_VERBOSITY) {
        msg = "Set verbosity";
    } else if (cbtype == LCB_CALLBACK_VERSIONS) {
        const lcb_RESPMCVERSION *resp = (const lcb_RESPMCVERSION *)sbase;
        msg = string(resp->mcversion, resp->nversion);
    } else {
        msg = "";
    }
    if (!sbase->server) {
        return;
    }
    if (sbase->rc != LCB_SUCCESS) {
        fprintf(stderr, "%s failed for server %s: %s\n", msg.c_str(), sbase->server, lcb_strerror_short(sbase->rc));
    } else {
        fprintf(stderr, "%s: %s\n", msg.c_str(), sbase->server);
    }
}

static void ping_callback(lcb_INSTANCE *, int, const lcb_RESPPING *resp)
{
    lcb_STATUS rc = lcb_respping_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "failed: %s\n", lcb_strerror_short(rc));
    } else {
        const char *json;
        size_t njson;
        lcb_respping_value(resp, &json, &njson);
        if (njson) {
            printf("%.*s", (int)njson, json);
        }
    }
}

static void arithmetic_callback(lcb_INSTANCE *, lcb_CALLBACK_TYPE type, const lcb_RESPCOUNTER *resp)
{
    const char *p;
    size_t n;
    lcb_respcounter_key(resp, &p, &n);
    string key(p, n);
    lcb_STATUS rc = lcb_respcounter_status(resp);
    if (rc != LCB_SUCCESS) {
        printKeyError(key, rc, type, (lcb_RESPBASE *)resp);
    } else {
        uint64_t value;
        lcb_respcounter_value(resp, &value);
        fprintf(stderr, "%-20s Current value is %" PRIu64 ".", key.c_str(), value);
        uint64_t cas;
        lcb_respcounter_cas(resp, &cas);
        fprintf(stderr, "CAS=0x%" PRIx64 "\n", cas);
        lcb_MUTATION_TOKEN token = {0};
        lcb_respcounter_mutation_token(resp, &token);
        if (lcb_mutation_token_is_valid(&token)) {
            fprintf(stderr, "%-20sSYNCTOKEN=%u,%" PRIu64 ",%" PRIu64 "\n", "", token.vbid_, token.uuid_, token.seqno_);
        }
    }
}

static void http_callback(lcb_INSTANCE *, int, const lcb_RESPHTTP *resp)
{
    HttpReceiver *ctx;
    lcb_resphttp_cookie(resp, (void **)&ctx);
    ctx->maybeInvokeStatus(resp);

    const char *body;
    size_t nbody;
    lcb_resphttp_body(resp, &body, &nbody);
    if (nbody) {
        ctx->onChunk(body, nbody);
    }
    if (lcb_resphttp_is_final(resp)) {
        ctx->onDone();
    }
}

static void view_callback(lcb_INSTANCE *, int, const lcb_RESPVIEW *resp)
{
    if (lcb_respview_is_final(resp)) {
        fprintf(stderr, "View query complete!\n");
    }

    lcb_STATUS rc = lcb_respview_status(resp);
    if (rc) {
        fprintf(stderr, "View query failed: %s\n", lcb_strerror_short(rc));

        if (rc == LCB_HTTP_ERROR) {
            const lcb_RESPHTTP *http;
            lcb_respview_http_response(resp, &http);
            if (http != NULL) {
                HttpReceiver ctx;
                ctx.maybeInvokeStatus(http);
                const char *body;
                size_t nbody;
                lcb_resphttp_body(http, &body, &nbody);
                if (nbody) {
                    fprintf(stderr, "%.*s", (int)nbody, body);
                }
            }
        }
    }

    if (lcb_respview_is_final(resp)) {
        const char *value;
        size_t nvalue;
        lcb_respview_row(resp, &value, &nvalue);
        if (value) {
            fprintf(stderr, "Non-row data: %.*s\n", (int)nvalue, value);
        }
        return;
    }

    const char *p;
    size_t n;

    lcb_respview_key(resp, &p, &n);
    printf("KEY: %.*s\n", (int)n, p);
    lcb_respview_row(resp, &p, &n);
    printf("     VALUE: %.*s\n", (int)n, p);
    lcb_respview_doc_id(resp, &p, &n);
    printf("     DOCID: %.*s\n", (int)n, p);
    const lcb_RESPGET *doc = NULL;
    lcb_respview_document(resp, &doc);
    if (doc) {
        get_callback(NULL, LCB_CALLBACK_GET, doc);
    }
}
}

Handler::Handler(const char *name) : parser(name), instance(NULL)
{
    if (name != NULL) {
        cmdname = name;
    }
}

Handler::~Handler()
{
    if (params.shouldDump()) {
        lcb_dump(instance, stderr, LCB_DUMP_ALL);
    }
    if (instance) {
        lcb_destroy(instance);
    }
}

void Handler::execute(int argc, char **argv)
{
    addOptions();
    parser.default_settings.argstring = usagestr();
    parser.default_settings.shortdesc = description();
    parser.parse(argc, argv, true);
    run();
    if (instance != NULL && params.useTimings()) {
        fprintf(stderr, "Output command timings as requested (--timings)\n");
        hg.write();
    }
}

void Handler::addOptions()
{
    params.addToParser(parser);
}

void Handler::run()
{
    lcb_create_st cropts;
    params.fillCropts(cropts);
    lcb_STATUS err;
    err = lcb_create(&instance, &cropts);
    if (err != LCB_SUCCESS) {
        throw LcbError(err, "Failed to create instance");
    }
    params.doCtls(instance);
    err = lcb_connect(instance);
    if (err != LCB_SUCCESS) {
        throw LcbError(err, "Failed to connect instance");
    }
    lcb_wait(instance);
    err = lcb_get_bootstrap_status(instance);
    if (err != LCB_SUCCESS) {
        throw LcbError(err, "Failed to bootstrap instance");
    }

    if (params.useTimings()) {
        hg.install(instance, stdout);
    }
}

const string &Handler::getLoneArg(bool required)
{
    static string empty("");

    const vector< string > &args = parser.getRestArgs();
    if (args.empty() || args.size() != 1) {
        if (required) {
            throw std::runtime_error("Command requires single argument");
        }
        return empty;
    }
    return args[0];
}

void GetHandler::addOptions()
{
    Handler::addOptions();
    o_exptime.abbrev('e');
    if (isLock()) {
        o_exptime.description("Time the lock should be held for");
    } else {
        o_exptime.description("Update the expiration time for the item");
        o_replica.abbrev('r');
        o_replica.description("Read from replica. Possible values are 'first': read from first available replica. "
                              "'all': read from all replicas, and <N>, where 0 < N < nreplicas");
        parser.addOption(o_replica);
    }
    parser.addOption(o_exptime);
    parser.addOption(o_scope);
    parser.addOption(o_collection);
    parser.addOption(o_durability);
}

void GetHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_GET, (lcb_RESPCALLBACK)get_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_GETREPLICA, (lcb_RESPCALLBACK)getreplica_callback);
    const vector< string > &keys = parser.getRestArgs();
    std::string replica_mode = o_replica.result();

    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < keys.size(); ++ii) {
        lcb_STATUS err;
        if (o_replica.passed()) {
            lcb_REPLICA_MODE mode;
            if (replica_mode == "first" || replica_mode == "first") {
                mode = LCB_REPLICA_MODE_ANY;
            } else if (replica_mode == "all") {
                mode = LCB_REPLICA_MODE_ALL;
            } else {
                switch (std::atoi(replica_mode.c_str())) {
                    case 0:
                        mode = LCB_REPLICA_MODE_IDX0;
                        break;
                    case 1:
                        mode = LCB_REPLICA_MODE_IDX1;
                        break;
                    case 2:
                        mode = LCB_REPLICA_MODE_IDX2;
                        break;
                    default:
                        throw LcbError(err, "invalid replica mode");
                }
            }
            lcb_CMDGETREPLICA *cmd;
            lcb_cmdgetreplica_create(&cmd, mode);
            const string &key = keys[ii];
            lcb_cmdgetreplica_key(cmd, key.c_str(), key.size());
            if (o_collection.passed()) {
                std::string s = o_scope.result();
                std::string c = o_collection.result();
                lcb_cmdgetreplica_collection(cmd, s.c_str(), s.size(), c.c_str(), c.size());
            }
            err = lcb_getreplica(instance, this, cmd);
        } else {
            lcb_CMDGET *cmd;

            lcb_cmdget_create(&cmd);
            const string &key = keys[ii];
            lcb_cmdget_key(cmd, key.c_str(), key.size());
            if (o_collection.passed()) {
                std::string s = o_scope.result();
                std::string c = o_collection.result();
                lcb_cmdget_collection(cmd, s.c_str(), s.size(), c.c_str(), c.size());
            }
            if (o_exptime.passed()) {
                if (isLock()) {
                    lcb_cmdget_locktime(cmd, o_exptime.result());
                } else {
                    lcb_cmdget_expiration(cmd, o_exptime.result());
                }
            }
            err = lcb_get(instance, this, cmd);
            lcb_cmdget_destroy(cmd);
        }
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void TouchHandler::addOptions()
{
    Handler::addOptions();
    parser.addOption(o_exptime);
    parser.addOption(o_durability);
}

void TouchHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_TOUCH, (lcb_RESPCALLBACK)touch_callback);
    const vector< string > &keys = parser.getRestArgs();
    lcb_STATUS err;
    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < keys.size(); ++ii) {
        const string &key = keys[ii];
        lcb_CMDTOUCH *cmd;
        lcb_cmdtouch_create(&cmd);
        lcb_cmdtouch_key(cmd, key.c_str(), key.size());
        lcb_cmdtouch_expiration(cmd, o_exptime.result());
        err = lcb_touch(instance, this, cmd);
        lcb_cmdtouch_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void SetHandler::addOptions()
{
    Handler::addOptions();
    parser.addOption(o_mode);
    parser.addOption(o_flags);
    parser.addOption(o_exp);
    parser.addOption(o_add);
    parser.addOption(o_persist);
    parser.addOption(o_replicate);
    if (!hasFileList()) {
        parser.addOption(o_value);
    }
    parser.addOption(o_json);
    parser.addOption(o_scope);
    parser.addOption(o_collection);
    parser.addOption(o_durability);
}

lcb_STORE_OPERATION SetHandler::mode()
{
    if (o_add.passed()) {
        return LCB_STORE_ADD;
    }

    string s = o_mode.const_result();
    std::transform(s.begin(), s.end(), s.begin(), ::tolower);
    if (s == "upsert") {
        return LCB_STORE_SET;
    } else if (s == "replace") {
        return LCB_STORE_REPLACE;
    } else if (s == "insert") {
        return LCB_STORE_ADD;
    } else if (s == "append") {
        return LCB_STORE_APPEND;
    } else if (s == "prepend") {
        return LCB_STORE_PREPEND;
    } else {
        throw BadArg(string("Mode must be one of upsert, insert, replace. Got ") + s);
        return LCB_STORE_SET;
    }
}

void SetHandler::storeItem(const string &key, const char *value, size_t nvalue)
{
    lcb_STATUS err;
    lcb_CMDSTORE *cmd;

    lcb_cmdstore_create(&cmd, mode());
    lcb_cmdstore_key(cmd, key.c_str(), key.size());
    if (o_collection.passed()) {
        std::string s = o_scope.result();
        std::string c = o_collection.result();
        lcb_cmdstore_collection(cmd, s.c_str(), s.size(), c.c_str(), c.size());
    }
    lcb_cmdstore_value(cmd, value, nvalue);

    if (o_json.result()) {
        lcb_cmdstore_datatype(cmd, LCB_VALUE_F_JSON);
    }
    if (o_exp.passed()) {
        lcb_cmdstore_expiration(cmd, o_exp.result());
    }
    if (o_flags.passed()) {
        lcb_cmdstore_flags(cmd, o_flags.result());
    }
    if (o_persist.passed() || o_replicate.passed()) {
        lcb_cmdstore_durability_observe(cmd, o_persist.result(), o_replicate.result());
    } else if (o_durability.passed()) {
        lcb_cmdstore_durability(cmd, durability());
    }
    err = lcb_store(instance, NULL, cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
    lcb_cmdstore_destroy(cmd);
}

void SetHandler::storeItem(const string &key, FILE *input)
{
    char tmpbuf[4096];
    vector< char > vbuf;
    size_t nr;
    while ((nr = fread(tmpbuf, 1, sizeof tmpbuf, input))) {
        vbuf.insert(vbuf.end(), tmpbuf, &tmpbuf[nr]);
    }
    storeItem(key, &vbuf[0], vbuf.size());
}

void SetHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_STORE, (lcb_RESPCALLBACK)store_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_STOREDUR, (lcb_RESPCALLBACK)store_callback);
    const vector< string > &keys = parser.getRestArgs();

    lcb_sched_enter(instance);

    if (hasFileList()) {
        for (size_t ii = 0; ii < keys.size(); ii++) {
            const string &key = keys[ii];
            FILE *fp = fopen(key.c_str(), "rb");
            if (fp == NULL) {
                perror(key.c_str());
                continue;
            }
            storeItem(key, fp);
            fclose(fp);
        }
    } else if (keys.size() > 1 || keys.empty()) {
        throw BadArg("create must be passed a single key");
    } else {
        const string &key = keys[0];
        if (o_value.passed()) {
            const string &value = o_value.const_result();
            storeItem(key, value.c_str(), value.size());
        } else {
            storeItem(key, stdin);
        }
    }

    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void HashHandler::run()
{
    Handler::run();

    lcbvb_CONFIG *vbc;
    lcb_STATUS err;
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }

    const vector< string > &args = parser.getRestArgs();
    for (size_t ii = 0; ii < args.size(); ii++) {
        const string &key = args[ii];
        const void *vkey = (const void *)key.c_str();
        int vbid, srvix;
        lcbvb_map_key(vbc, vkey, key.size(), &vbid, &srvix);
        fprintf(stderr, "%s: [vBucket=%d, Index=%d]", key.c_str(), vbid, srvix);
        if (srvix != -1) {
            fprintf(stderr, " Server: %s", lcbvb_get_hostport(vbc, srvix, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN));
            const char *vapi = lcbvb_get_capibase(vbc, srvix, LCBVB_SVCMODE_PLAIN);
            if (vapi) {
                fprintf(stderr, ", CouchAPI: %s", vapi);
            }
        }
        fprintf(stderr, "\n");

        for (size_t jj = 0; jj < lcbvb_get_nreplicas(vbc); jj++) {
            int rix = lcbvb_vbreplica(vbc, vbid, jj);
            const char *rname = NULL;
            if (rix >= 0) {
                rname = lcbvb_get_hostport(vbc, rix, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN);
            }
            if (rname == NULL) {
                rname = "N/A";
            }
            fprintf(stderr, "Replica #%d: Index=%d, Host=%s\n", (int)jj, rix, rname);
        }
    }
}

void ObserveHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_OBSERVE, (lcb_RESPCALLBACK)observe_callback);
    const vector< string > &keys = parser.getRestArgs();
    lcb_MULTICMD_CTX *mctx = lcb_observe3_ctxnew(instance);
    if (mctx == NULL) {
        throw std::bad_alloc();
    }

    lcb_STATUS err;
    for (size_t ii = 0; ii < keys.size(); ii++) {
        lcb_CMDOBSERVE cmd = {0};
        LCB_KREQ_SIMPLE(&cmd.key, keys[ii].c_str(), keys[ii].size());
        err = mctx->addcmd(mctx, (lcb_CMDBASE *)&cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }

    lcb_sched_enter(instance);
    err = mctx->done(mctx, NULL);
    if (err == LCB_SUCCESS) {
        lcb_sched_leave(instance);
        lcb_wait(instance);
    } else {
        lcb_sched_fail(instance);
        throw LcbError(err);
    }
}

void ObserveSeqnoHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_OBSEQNO, (lcb_RESPCALLBACK)obseqno_callback);
    const vector< string > &infos = parser.getRestArgs();
    lcb_CMDOBSEQNO cmd = {0};
    lcbvb_CONFIG *vbc;
    lcb_STATUS rc;

    rc = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
    if (rc != LCB_SUCCESS) {
        throw LcbError(rc);
    }

    lcb_sched_enter(instance);

    for (size_t ii = 0; ii < infos.size(); ++ii) {
        const string &cur = infos[ii];
        unsigned vbid;
        unsigned long long uuid;
        int rv = sscanf(cur.c_str(), "%u,%llu", &vbid, &uuid);
        if (rv != 2) {
            throw BadArg("Must pass sequences of base10 vbid and base16 uuids");
        }
        cmd.uuid = uuid;
        cmd.vbid = vbid;
        for (size_t jj = 0; jj < lcbvb_get_nreplicas(vbc) + 1; ++jj) {
            int ix = lcbvb_vbserver(vbc, vbid, jj);
            if (ix < 0) {
                fprintf(stderr, "Server %d unavailable (skipping)\n", ix);
            }
            cmd.server_index = ix;
            rc = lcb_observe_seqno3(instance, NULL, &cmd);
            if (rc != LCB_SUCCESS) {
                throw LcbError(rc);
            }
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void ExistsHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_EXISTS, (lcb_RESPCALLBACK)exists_callback);
    const vector< string > &args = parser.getRestArgs();

    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < args.size(); ii++) {
        const string &key = args[ii];
        lcb_CMDEXISTS *cmd;
        lcb_cmdexists_create(&cmd);
        lcb_cmdexists_key(cmd, key.c_str(), key.size());
        if (o_collection.passed()) {
            std::string s = o_scope.result();
            std::string c = o_collection.result();
            lcb_cmdexists_collection(cmd, s.c_str(), s.size(), c.c_str(), c.size());
        }
        lcb_STATUS err = lcb_exists(instance, NULL, cmd);
        lcb_cmdexists_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void UnlockHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_UNLOCK, (lcb_RESPCALLBACK)unlock_callback);
    const vector< string > &args = parser.getRestArgs();

    if (args.size() % 2) {
        throw BadArg("Expect key-cas pairs. Argument list must be even");
    }

    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < args.size(); ii += 2) {
        const string &key = args[ii];
        lcb_CAS cas;
        int rv;
        rv = sscanf(args[ii + 1].c_str(), "0x%" PRIx64, &cas);
        if (rv != 1) {
            throw BadArg("CAS must be formatted as a hex string beginning with '0x'");
        }

        lcb_CMDUNLOCK *cmd;
        lcb_cmdunlock_create(&cmd);
        lcb_cmdunlock_key(cmd, key.c_str(), key.size());
        lcb_cmdunlock_cas(cmd, cas);
        lcb_STATUS err = lcb_unlock(instance, NULL, cmd);
        lcb_cmdunlock_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

static const char *iops_to_string(lcb_io_ops_type_t type)
{
    switch (type) {
        case LCB_IO_OPS_LIBEV:
            return "libev";
        case LCB_IO_OPS_LIBEVENT:
            return "libevent";
        case LCB_IO_OPS_LIBUV:
            return "libuv";
        case LCB_IO_OPS_SELECT:
            return "select";
        case LCB_IO_OPS_WINIOCP:
            return "iocp";
        case LCB_IO_OPS_INVALID:
            return "user-defined";
        default:
            return "invalid";
    }
}

void VersionHandler::run()
{
    const char *changeset;
    lcb_STATUS err;
    err = lcb_cntl(NULL, LCB_CNTL_GET, LCB_CNTL_CHANGESET, (void *)&changeset);
    if (err != LCB_SUCCESS) {
        changeset = "UNKNOWN";
    }
    fprintf(stderr, "cbc:\n");
    fprintf(stderr, "  Runtime: Version=%s, Changeset=%s\n", lcb_get_version(NULL), changeset);
    fprintf(stderr, "  Headers: Version=%s, Changeset=%s\n", LCB_VERSION_STRING, LCB_VERSION_CHANGESET);
    fprintf(stderr, "  Build Timestamp: %s\n", LCB_BUILD_TIMESTAMP);

    struct lcb_cntl_iops_info_st info;
    memset(&info, 0, sizeof info);
    err = lcb_cntl(NULL, LCB_CNTL_GET, LCB_CNTL_IOPS_DEFAULT_TYPES, &info);
    if (err == LCB_SUCCESS) {
        fprintf(stderr, "  IO: Default=%s, Current=%s, Accessible=", iops_to_string(info.v.v0.os_default),
                iops_to_string(info.v.v0.effective));
    }
    {
        size_t ii;
        char buf[256] = {0}, *p = buf;
        lcb_io_ops_type_t known_io[] = {LCB_IO_OPS_WINIOCP, LCB_IO_OPS_LIBEVENT, LCB_IO_OPS_LIBUV, LCB_IO_OPS_LIBEV,
                                        LCB_IO_OPS_SELECT};

        for (ii = 0; ii < sizeof(known_io) / sizeof(known_io[0]); ii++) {
            struct lcb_create_io_ops_st cio = {0};
            lcb_io_opt_t io = NULL;

            cio.v.v0.type = known_io[ii];
            if (lcb_create_io_ops(&io, &cio) == LCB_SUCCESS) {
                p += sprintf(p, "%s,", iops_to_string(known_io[ii]));
                lcb_destroy_io_ops(io);
            }
        }
        *(--p) = '\n';
        fprintf(stderr, "%s", buf);
    }

    if (lcb_supports_feature(LCB_SUPPORTS_SSL)) {
#ifdef LCB_NO_SSL
        printf("  SSL: SUPPORTED\n");
#else
#if defined(OPENSSL_VERSION)
        printf("  SSL Runtime: %s\n", OpenSSL_version(OPENSSL_VERSION));
#elif defined(SSLEAY_VERSION)
        printf("  SSL Runtime: %s\n", SSLeay_version(SSLEAY_VERSION));
#endif
        printf("  SSL Headers: %s\n", OPENSSL_VERSION_TEXT);
#endif
    } else {
        printf("  SSL: NOT SUPPORTED\n");
    }
    if (lcb_supports_feature(LCB_SUPPORTS_SNAPPY)) {
#define EXPAND(VAR) VAR##1
#define IS_EMPTY(VAR) EXPAND(VAR)

#if defined(SNAPPY_MAJOR) && (IS_EMPTY(SNAPPY_MAJOR) != 1)
        printf("  Snappy: %d.%d.%d\n", SNAPPY_MAJOR, SNAPPY_MINOR, SNAPPY_PATCHLEVEL);
#else
        printf("  Snappy: unknown\n");
#endif
    } else {
        printf("  Snappy: NOT SUPPORTED\n");
    }
    printf("  Tracing: %sSUPPORTED\n", lcb_supports_feature(LCB_SUPPORTS_TRACING) ? "" : "NOT ");
    printf("  System: %s; %s\n", LCB_SYSTEM, LCB_SYSTEM_PROCESSOR);
    printf("  CC: %s; %s\n", LCB_C_COMPILER, LCB_C_FLAGS);
    printf("  CXX: %s; %s\n", LCB_CXX_COMPILER, LCB_CXX_FLAGS);
}

void RemoveHandler::run()
{
    Handler::run();
    const vector< string > &keys = parser.getRestArgs();
    lcb_sched_enter(instance);
    lcb_install_callback3(instance, LCB_CALLBACK_REMOVE, (lcb_RESPCALLBACK)remove_callback);
    for (size_t ii = 0; ii < keys.size(); ++ii) {
        const string &key = keys[ii];
        lcb_CMDREMOVE *cmd;
        lcb_cmdremove_create(&cmd);
        lcb_cmdremove_key(cmd, key.c_str(), key.size());
        lcb_STATUS err = lcb_remove(instance, NULL, cmd);
        lcb_cmdremove_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void StatsHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_STATS, (lcb_RESPCALLBACK)stats_callback);
    vector< string > keys = parser.getRestArgs();
    if (keys.empty()) {
        keys.push_back("");
    }
    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < keys.size(); ii++) {
        lcb_CMDSTATS cmd = {0};
        const string &key = keys[ii];
        if (!key.empty()) {
            LCB_KREQ_SIMPLE(&cmd.key, key.c_str(), key.size());
            if (o_keystats.result()) {
                cmd.cmdflags = LCB_CMDSTATS_F_KV;
            }
        }
        bool is_keystats = o_keystats.result();
        lcb_STATUS err = lcb_stats3(instance, &is_keystats, &cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void WatchHandler::run()
{
    Handler::run();
    lcb_install_callback3(instance, LCB_CALLBACK_STATS, (lcb_RESPCALLBACK)watch_callback);
    vector< string > keys = parser.getRestArgs();
    if (keys.empty()) {
        keys.push_back("cmd_total_ops");
        keys.push_back("cmd_total_gets");
        keys.push_back("cmd_total_sets");
    }
    int interval = o_interval.result();

    map< string, int64_t > prev;

    bool first = true;
    while (true) {
        map< string, int64_t > entry;
        lcb_sched_enter(instance);
        lcb_CMDSTATS cmd = {0};
        lcb_STATUS err = lcb_stats3(instance, (void *)&entry, &cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
        lcb_sched_leave(instance);
        lcb_wait(instance);
        if (first) {
            for (vector< string >::iterator it = keys.begin(); it != keys.end(); ++it) {
                fprintf(stderr, "%s: %" PRId64 "\n", it->c_str(), entry[*it]);
            }
            first = false;
        } else {
#ifndef _WIN32
            if (isatty(STDERR_FILENO)) {
                fprintf(stderr, "\033[%dA", (int)keys.size());
            }
#endif
            for (vector< string >::iterator it = keys.begin(); it != keys.end(); ++it) {
                fprintf(stderr, "%s: %" PRId64 "%20s\n", it->c_str(), (entry[*it] - prev[*it]) / interval, "");
            }
        }
        prev = entry;
#ifdef _WIN32
        Sleep(interval * 1000);
#else
        sleep(interval);
#endif
    }
}

void VerbosityHandler::run()
{
    Handler::run();

    const string &slevel = getRequiredArg();
    lcb_verbosity_level_t level;
    if (slevel == "detail") {
        level = LCB_VERBOSITY_DETAIL;
    } else if (slevel == "debug") {
        level = LCB_VERBOSITY_DEBUG;
    } else if (slevel == "info") {
        level = LCB_VERBOSITY_INFO;
    } else if (slevel == "warning") {
        level = LCB_VERBOSITY_WARNING;
    } else {
        throw BadArg("Verbosity level must be {detail,debug,info,warning}");
    }

    lcb_install_callback3(instance, LCB_CALLBACK_VERBOSITY, (lcb_RESPCALLBACK)common_server_callback);
    lcb_CMDVERBOSITY cmd = {0};
    cmd.level = level;
    lcb_STATUS err;
    lcb_sched_enter(instance);
    err = lcb_server_verbosity3(instance, NULL, &cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void McVersionHandler::run()
{
    Handler::run();

    lcb_install_callback3(instance, LCB_CALLBACK_VERSIONS, (lcb_RESPCALLBACK)common_server_callback);
    lcb_CMDVERSIONS cmd = {0};
    lcb_STATUS err;
    lcb_sched_enter(instance);
    err = lcb_server_versions3(instance, NULL, &cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

static void collection_dump_manifest_callback(lcb_INSTANCE *, int, const lcb_RESPGETMANIFEST *resp)
{
    lcb_STATUS rc = lcb_respgetmanifest_status(resp);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "Failed to get collection manifest: %s\n", lcb_strerror_short(rc));
    } else {
        const char *value;
        size_t nvalue;
        lcb_respgetmanifest_value(resp, &value, &nvalue);
        fwrite(value, 1, nvalue, stdout);
        fflush(stdout);
        fprintf(stderr, "\n");
    }
}

void CollectionGetManifestHandler::run()
{
    Handler::run();

    lcb_install_callback3(instance, LCB_CALLBACK_COLLECTIONS_GET_MANIFEST,
                          (lcb_RESPCALLBACK)collection_dump_manifest_callback);

    lcb_STATUS err;
    lcb_CMDGETMANIFEST *cmd;
    lcb_cmdgetmanifest_create(&cmd);
    lcb_sched_enter(instance);
    err = lcb_getmanifest(instance, NULL, cmd);
    lcb_cmdgetmanifest_destroy(cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

static void getcid_callback(lcb_INSTANCE *, int, const lcb_RESPGETCID *resp)
{
    lcb_STATUS rc = lcb_respgetcid_status(resp);
    const char *key;
    size_t nkey;
    lcb_respgetcid_scoped_collection(resp, &key, &nkey);
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "%-20.*s Failed to get collection ID: %s\n", (int)nkey, (char *)key, lcb_strerror_short(rc));
    } else {
        uint64_t manifest_id;
        uint32_t collection_id;
        lcb_respgetcid_manifest_id(resp, &manifest_id);
        lcb_respgetcid_collection_id(resp, &collection_id);
        printf("%-20.*s ManifestId=0x%02" PRIx64 ", CollectionId=0x%02x\n", (int)nkey, (char *)key, manifest_id,
               collection_id);
    }
}

void CollectionGetCIDHandler::run()
{
    Handler::run();

    lcb_install_callback3(instance, LCB_CALLBACK_GETCID, (lcb_RESPCALLBACK)getcid_callback);

    std::string scope = o_scope.result();

    const vector< string > &collections = parser.getRestArgs();
    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < collections.size(); ++ii) {
        lcb_STATUS err;
        lcb_CMDGETCID *cmd;
        lcb_cmdgetcid_create(&cmd);
        const string &collection = collections[ii];
        lcb_cmdgetcid_scope(cmd, scope.c_str(), scope.size());
        lcb_cmdgetcid_collection(cmd, collection.c_str(), collection.size());
        err = lcb_getcid(instance, NULL, cmd);
        lcb_cmdgetcid_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void KeygenHandler::run()
{
    Handler::run();

    lcbvb_CONFIG *vbc;
    lcb_STATUS err;
    err = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }

    unsigned num_vbuckets = lcbvb_get_nvbuckets(vbc);
    if (num_vbuckets == 0) {
        throw LcbError(LCB_EINVAL, "the configuration does not contain any vBuckets");
    }
    unsigned num_keys_per_vbucket = o_keys_per_vbucket.result();
    vector< vector< string > > keys(num_vbuckets);
#define MAX_KEY_SIZE 16
    char buf[MAX_KEY_SIZE] = {0};
    unsigned i = 0;
    int left = num_keys_per_vbucket * num_vbuckets;
    while (left > 0 && i < UINT_MAX) {
        int nbuf = snprintf(buf, MAX_KEY_SIZE, "key_%010u", i++);
        if (nbuf <= 0) {
            throw LcbError(LCB_ERROR, "unable to render new key into buffer");
        }
        int vbid, srvix;
        lcbvb_map_key(vbc, buf, nbuf, &vbid, &srvix);
        if (keys[vbid].size() < num_keys_per_vbucket) {
            keys[vbid].push_back(buf);
            left--;
        }
    }
    for (i = 0; i < num_vbuckets; i++) {
        for (vector< string >::iterator it = keys[i].begin(); it != keys[i].end(); ++it) {
            printf("%s %u\n", it->c_str(), i);
        }
    }
    if (left > 0) {
        fprintf(stderr, "some vBuckets don't have enough keys\n");
    }
}

void PingHandler::run()
{
    Handler::run();

    lcb_install_callback3(instance, LCB_CALLBACK_PING, (lcb_RESPCALLBACK)ping_callback);
    lcb_STATUS err;
    lcb_CMDPING *cmd;
    lcb_cmdping_create(&cmd);
    lcb_cmdping_all(cmd);
    lcb_cmdping_encode_json(cmd, true, true, o_details.passed());
    lcb_sched_enter(instance);
    err = lcb_ping(instance, NULL, cmd);
    lcb_cmdping_destroy(cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

extern "C" {
static void cbFlushCb(lcb_INSTANCE *, int, const lcb_RESPCBFLUSH *resp)
{
    if (resp->rc == LCB_SUCCESS) {
        fprintf(stderr, "Flush OK\n");
    } else {
        fprintf(stderr, "Flush failed: %s\n", lcb_strerror_short(resp->rc));
    }
}
}
void BucketFlushHandler::run()
{
    Handler::run();
    lcb_CMDCBFLUSH cmd = {0};
    lcb_STATUS err;
    lcb_install_callback3(instance, LCB_CALLBACK_CBFLUSH, (lcb_RESPCALLBACK)cbFlushCb);
    err = lcb_cbflush3(instance, NULL, &cmd);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    } else {
        lcb_wait(instance);
    }
}

void ArithmeticHandler::run()
{
    Handler::run();

    const vector< string > &keys = parser.getRestArgs();
    lcb_install_callback3(instance, LCB_CALLBACK_COUNTER, (lcb_RESPCALLBACK)arithmetic_callback);
    lcb_sched_enter(instance);
    for (size_t ii = 0; ii < keys.size(); ++ii) {
        const string &key = keys[ii];
        lcb_CMDCOUNTER *cmd;
        lcb_cmdcounter_create(&cmd);
        lcb_cmdcounter_key(cmd, key.c_str(), key.size());
        if (o_initial.passed()) {
            lcb_cmdcounter_initial(cmd, o_initial.result());
        }
        if (o_delta.result() > static_cast< uint64_t >(std::numeric_limits< int64_t >::max())) {
            throw BadArg("Delta too big");
        }
        int64_t delta = static_cast< int64_t >(o_delta.result());
        if (shouldInvert()) {
            delta *= -1;
        }
        lcb_cmdcounter_delta(cmd, delta);
        lcb_cmdcounter_expiration(cmd, o_expiry.result());
        lcb_STATUS err = lcb_counter(instance, NULL, cmd);
        lcb_cmdcounter_destroy(cmd);
        if (err != LCB_SUCCESS) {
            throw LcbError(err);
        }
    }
    lcb_sched_leave(instance);
    lcb_wait(instance);
}

void ViewsHandler::run()
{
    Handler::run();

    const string &s = getRequiredArg();
    size_t pos = s.find('/');
    if (pos == string::npos) {
        throw BadArg("View must be in the format of design/view");
    }

    string ddoc = s.substr(0, pos);
    string view = s.substr(pos + 1);
    string opts = o_params.result();

    lcb_CMDVIEW *cmd;
    lcb_cmdview_create(&cmd);
    lcb_cmdview_design_document(cmd, ddoc.c_str(), ddoc.size());
    lcb_cmdview_view_name(cmd, view.c_str(), view.size());
    lcb_cmdview_option_string(cmd, opts.c_str(), opts.size());
    lcb_cmdview_callback(cmd, view_callback);
    if (o_incdocs) {
        lcb_cmdview_include_docs(cmd, true);
    }

    lcb_STATUS rc;
    rc = lcb_view(instance, NULL, cmd);
    lcb_cmdview_destroy(cmd);
    if (rc != LCB_SUCCESS) {
        throw LcbError(rc);
    }
    lcb_wait(instance);
}

static void splitKvParam(const string &src, string &key, string &value)
{
    size_t pp = src.find('=');
    if (pp == string::npos) {
        throw BadArg("Param must be in the form of key=value");
    }

    key = src.substr(0, pp);
    value = src.substr(pp + 1);
}

extern "C" {
static void n1qlCallback(lcb_INSTANCE *, int, const lcb_RESPN1QL *resp)
{
    const char *row;
    size_t nrow;
    lcb_respn1ql_row(resp, &row, &nrow);

    if (lcb_respn1ql_is_final(resp)) {
        lcb_STATUS rc = lcb_respn1ql_status(resp);
        fprintf(stderr, "---> Query response finished\n");
        if (rc != LCB_SUCCESS) {
            fprintf(stderr, "---> Query failed with library code %s\n", lcb_strerror_short(rc));
            const lcb_RESPHTTP *http;
            lcb_respn1ql_http_response(resp, &http);
            if (http) {
                uint16_t status;
                lcb_resphttp_http_status(http, &status);
                fprintf(stderr, "---> Inner HTTP request failed with library code %s and HTTP status %d\n",
                        lcb_strerror_short(lcb_resphttp_status(http)), status);
            }
        }
        if (row) {
            printf("%.*s\n", (int)nrow, row);
        }
    } else {
        printf("%.*s,\n", (int)nrow, row);
    }
}
}

void N1qlHandler::run()
{
    Handler::run();
    const string &qstr = getRequiredArg();
    lcb_STATUS rc;

    lcb_CMDN1QL *cmd;
    lcb_cmdn1ql_create(&cmd);

    rc = lcb_cmdn1ql_statement(cmd, qstr.c_str(), qstr.size());
    if (rc != LCB_SUCCESS) {
        throw LcbError(rc);
    }

    const vector< string > &vv_args = o_args.const_result();
    for (size_t ii = 0; ii < vv_args.size(); ii++) {
        string key, value;
        splitKvParam(vv_args[ii], key, value);
        rc = lcb_cmdn1ql_named_param(cmd, key.c_str(), key.size(), value.c_str(), value.size());
        if (rc != LCB_SUCCESS) {
            throw LcbError(rc);
        }
    }

    const vector< string > &vv_opts = o_opts.const_result();
    for (size_t ii = 0; ii < vv_opts.size(); ii++) {
        string key, value;
        splitKvParam(vv_opts[ii], key, value);
        rc = lcb_cmdn1ql_option(cmd, key.c_str(), key.size(), value.c_str(), value.size());
        if (rc != LCB_SUCCESS) {
            throw LcbError(rc);
        }
    }
    lcb_cmdn1ql_adhoc(cmd, !o_prepare.passed());
    lcb_cmdn1ql_callback(cmd, n1qlCallback);

    const char *payload;
    size_t npayload;
    lcb_cmdn1ql_payload(cmd, &payload, &npayload);
    fprintf(stderr, "---> Encoded query: %.*s\n", (int)npayload, payload);

    // TODO: deprecate and expose in analytics
    // if (o_analytics.passed()) {
    //     cmd.cmdflags |= LCB_CMDN1QL_F_ANALYTICSQUERY;
    // }
    rc = lcb_n1ql(instance, NULL, cmd);
    lcb_cmdn1ql_destroy(cmd);
    if (rc != LCB_SUCCESS) {
        throw LcbError(rc);
    }
    lcb_wait(instance);
}

void HttpReceiver::install(lcb_INSTANCE *instance)
{
    lcb_install_callback3(instance, LCB_CALLBACK_HTTP, (lcb_RESPCALLBACK)http_callback);
}

void HttpReceiver::maybeInvokeStatus(const lcb_RESPHTTP *resp)
{
    if (statusInvoked) {
        return;
    }

    statusInvoked = true;
    const char *const *hdr;
    lcb_resphttp_headers(resp, &hdr);
    if (hdr) {
        for (const char *const *cur = hdr; *cur; cur += 2) {
            string key = cur[0];
            string value = cur[1];
            headers[key] = value;
        }
    }
    uint16_t status;
    lcb_resphttp_http_status(resp, &status);
    handleStatus(lcb_resphttp_status(resp), status);
}

void HttpBaseHandler::run()
{
    Handler::run();
    install(instance);
    lcb_CMDHTTP *cmd;
    string uri = getURI();
    const string &body = getBody();

    lcb_cmdhttp_create(&cmd, isAdmin() ? LCB_HTTP_TYPE_MANAGEMENT : LCB_HTTP_TYPE_VIEW);
    lcb_cmdhttp_method(cmd, getMethod());
    lcb_cmdhttp_path(cmd, uri.c_str(), uri.size());
    if (!body.empty()) {
        lcb_cmdhttp_body(cmd, body.c_str(), body.size());
    }
    string ctype = getContentType();
    if (!ctype.empty()) {
        lcb_cmdhttp_content_type(cmd, ctype.c_str(), ctype.size());
    }
    lcb_cmdhttp_streaming(cmd, true);

    lcb_STATUS err;
    err = lcb_http(instance, this, cmd);
    lcb_cmdhttp_destroy(cmd);

    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }

    lcb_wait(instance);
}

lcb_HTTP_METHOD HttpBaseHandler::getMethod()
{
    string smeth = o_method.result();
    if (smeth == "GET") {
        return LCB_HTTP_METHOD_GET;
    } else if (smeth == "POST") {
        return LCB_HTTP_METHOD_POST;
    } else if (smeth == "DELETE") {
        return LCB_HTTP_METHOD_DELETE;
    } else if (smeth == "PUT") {
        return LCB_HTTP_METHOD_PUT;
    } else {
        throw BadArg("Unrecognized method string");
    }
}

const string &HttpBaseHandler::getBody()
{
    if (!body_cached.empty()) {
        return body_cached;
    }
    lcb_HTTP_METHOD meth = getMethod();
    if (meth == LCB_HTTP_METHOD_GET || meth == LCB_HTTP_METHOD_DELETE) {
        return body_cached; // empty
    }

    char buf[4096];
    size_t nr;
    while ((nr = fread(buf, 1, sizeof buf, stdin)) != 0) {
        body_cached.append(buf, nr);
    }
    return body_cached;
}

void HttpBaseHandler::handleStatus(lcb_STATUS err, int code)
{
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "ERROR: %s ", lcb_strerror_short(err));
    }
    fprintf(stderr, "%d\n", code);
    map< string, string >::const_iterator ii = headers.begin();
    for (; ii != headers.end(); ii++) {
        fprintf(stderr, "  %s: %s\n", ii->first.c_str(), ii->second.c_str());
    }
}

string AdminHandler::getURI()
{
    return getRequiredArg();
}

void AdminHandler::run()
{
    fprintf(stderr, "Requesting %s\n", getURI().c_str());
    HttpBaseHandler::run();
    printf("%s\n", resbuf.c_str());
}

void BucketCreateHandler::run()
{
    const string &name = getRequiredArg();
    const string &btype = o_btype.const_result();
    stringstream ss;

    if (btype == "couchbase" || btype == "membase") {
        isMemcached = false;
    } else if (btype == "memcached") {
        isMemcached = true;
    } else {
        throw BadArg("Unrecognized bucket type");
    }
    if (o_proxyport.passed() && o_bpass.passed()) {
        throw BadArg("Custom ASCII port is only available for auth-less buckets");
    }

    ss << "name=" << name;
    ss << "&bucketType=" << btype;
    ss << "&ramQuotaMB=" << o_ramquota.result();
    if (o_proxyport.passed()) {
        ss << "&authType=none&proxyPort=" << o_proxyport.result();
    } else {
        ss << "&authType=sasl&saslPassword=" << o_bpass.result();
    }

    ss << "&replicaNumber=" << o_replicas.result();
    body_s = ss.str();

    AdminHandler::run();
}

void RbacHandler::run()
{
    fprintf(stderr, "Requesting %s\n", getURI().c_str());
    HttpBaseHandler::run();
    if (o_raw.result()) {
        printf("%s\n", resbuf.c_str());
    } else {
        format();
    }
}

void RoleListHandler::format()
{
    Json::Value json;
    if (!Json::Reader().parse(resbuf, json)) {
        fprintf(stderr, "Failed to parse response as JSON, falling back to raw mode\n");
        printf("%s\n", resbuf.c_str());
    }

    std::map< string, string > roles;
    size_t max_width = 0;
    for (Json::Value::iterator i = json.begin(); i != json.end(); i++) {
        Json::Value role = *i;
        string role_id = role["role"].asString() + ": ";
        roles[role_id] = role["desc"].asString();
        if (max_width < role_id.size()) {
            max_width = role_id.size();
        }
    }
    for (map< string, string >::iterator i = roles.begin(); i != roles.end(); i++) {
        std::cout << std::left << std::setw(max_width) << i->first << i->second << std::endl;
    }
}

void UserListHandler::format()
{
    Json::Value json;
    if (!Json::Reader().parse(resbuf, json)) {
        fprintf(stderr, "Failed to parse response as JSON, falling back to raw mode\n");
        printf("%s\n", resbuf.c_str());
    }

    map< string, map< string, string > > users;
    size_t max_width = 0;
    for (Json::Value::iterator i = json.begin(); i != json.end(); i++) {
        Json::Value user = *i;
        string domain = user["domain"].asString();
        string user_id = user["id"].asString();
        string user_name = user["name"].asString();
        if (!user_name.empty()) {
            user_id += " (" + user_name + "): ";
        }
        stringstream roles;
        Json::Value roles_ary = user["roles"];
        for (Json::Value::iterator j = roles_ary.begin(); j != roles_ary.end(); j++) {
            Json::Value role = *j;
            roles << "\n   - " << role["role"].asString();
            if (!role["bucket_name"].empty()) {
                roles << "[" << role["bucket_name"].asString() << "]";
            }
        }
        if (max_width < user_id.size()) {
            max_width = user_id.size();
        }
        users[domain][user_id] = roles.str();
    }
    if (!users["local"].empty()) {
        std::cout << "Local users:" << std::endl;
        int j = 1;
        for (map< string, string >::iterator i = users["local"].begin(); i != users["local"].end(); i++, j++) {
            std::cout << j << ". " << std::left << std::setw(max_width) << i->first << i->second << std::endl;
        }
    }
    if (!users["external"].empty()) {
        std::cout << "External users:" << std::endl;
        int j = 1;
        for (map< string, string >::iterator i = users["external"].begin(); i != users["external"].end(); i++, j++) {
            std::cout << j << ". " << std::left << std::setw(max_width) << i->first << i->second << std::endl;
        }
    }
}

void UserUpsertHandler::run()
{
    stringstream ss;

    name = getRequiredArg();
    domain = o_domain.result();
    if (domain != "local" && domain != "external") {
        throw BadArg("Unrecognized domain type");
    }
    if (!o_roles.passed()) {
        throw BadArg("At least one role has to be specified");
    }
    std::vector< std::string > roles = o_roles.result();
    std::string roles_param;
    for (size_t ii = 0; ii < roles.size(); ii++) {
        if (roles_param.empty()) {
            roles_param += roles[ii];
        } else {
            roles_param += std::string(",") + roles[ii];
        }
    }
    ss << "roles=" << roles_param;
    if (o_full_name.passed()) {
        ss << "&name=" << o_full_name.result();
    }
    if (o_password.passed()) {
        ss << "&password=" << o_password.result();
    }
    body = ss.str();

    AdminHandler::run();
}

struct HostEnt {
    string protostr;
    string hostname;
    HostEnt(const std::string &host, const char *proto)
    {
        protostr = proto;
        hostname = host;
    }
    HostEnt(const std::string &host, const char *proto, int port)
    {
        protostr = proto;
        hostname = host;
        stringstream ss;
        ss << std::dec << port;
        hostname += ":";
        hostname += ss.str();
    }
};

void ConnstrHandler::run()
{
    const string &connstr_s = getRequiredArg();
    lcb_STATUS err;
    const char *errmsg;
    lcb::Connspec spec;
    err = spec.parse(connstr_s.c_str(), &errmsg);
    if (err != LCB_SUCCESS) {
        throw BadArg(errmsg);
    }

    printf("Bucket: %s\n", spec.bucket().c_str());
    printf("Implicit port: %d\n", spec.default_port());
    string sslOpts;
    if (spec.sslopts() & LCB_SSL_ENABLED) {
        sslOpts = "ENABLED";
        if (spec.sslopts() & LCB_SSL_NOVERIFY) {
            sslOpts += "|NOVERIFY";
        }
    } else {
        sslOpts = "DISABLED";
    }
    printf("SSL: %s\n", sslOpts.c_str());

    printf("Boostrap Protocols: ");
    string bsStr;
    if (spec.is_bs_cccp()) {
        bsStr += "CCCP, ";
    }
    if (spec.is_bs_http()) {
        bsStr += "HTTP, ";
    }
    if (bsStr.empty()) {
        bsStr = "CCCP,HTTP";
    } else {
        bsStr.erase(bsStr.size() - 1, 1);
    }
    printf("%s\n", bsStr.c_str());
    printf("Hosts:\n");
    vector< HostEnt > hosts;

    for (size_t ii = 0; ii < spec.hosts().size(); ++ii) {
        const lcb::Spechost *dh = &spec.hosts()[ii];
        lcb_U16 port = dh->port;
        if (!port) {
            port = spec.default_port();
        }

        if (dh->type == LCB_CONFIG_MCD_PORT) {
            hosts.push_back(HostEnt(dh->hostname, "memcached", port));
        } else if (dh->type == LCB_CONFIG_MCD_SSL_PORT) {
            hosts.push_back(HostEnt(dh->hostname, "memcached+ssl", port));
        } else if (dh->type == LCB_CONFIG_HTTP_PORT) {
            hosts.push_back(HostEnt(dh->hostname, "restapi", port));
        } else if (dh->type == LCB_CONFIG_HTTP_SSL_PORT) {
            hosts.push_back(HostEnt(dh->hostname, "restapi+ssl", port));
        } else {
            if (spec.sslopts()) {
                hosts.push_back(HostEnt(dh->hostname, "memcached+ssl", LCB_CONFIG_MCD_SSL_PORT));
                hosts.push_back(HostEnt(dh->hostname, "restapi+ssl", LCB_CONFIG_HTTP_SSL_PORT));
            } else {
                hosts.push_back(HostEnt(dh->hostname, "memcached", LCB_CONFIG_MCD_PORT));
                hosts.push_back(HostEnt(dh->hostname, "restapi", LCB_CONFIG_HTTP_PORT));
            }
        }
    }
    for (size_t ii = 0; ii < hosts.size(); ii++) {
        HostEnt &ent = hosts[ii];
        string protostr = "[" + ent.protostr + "]";
        printf("  %-20s%s\n", protostr.c_str(), ent.hostname.c_str());
    }

    printf("Options: \n");
    lcb::Connspec::Options::const_iterator it = spec.options().begin();
    for (; it != spec.options().end(); ++it) {
        printf("  %s=%s\n", it->first.c_str(), it->second.c_str());
    }
}

void WriteConfigHandler::run()
{
    lcb_create_st cropts;
    params.fillCropts(cropts);
    string outname = getLoneArg();
    if (outname.empty()) {
        outname = ConnParams::getConfigfileName();
    }
    // Generate the config
    params.writeConfig(outname);
}

static map< string, Handler * > handlers;
static map< string, Handler * > handlers_s;
static const char *optionsOrder[] = {"help",
                                     "cat",
                                     "create",
                                     "touch",
                                     "observe",
                                     "observe-seqno",
                                     "incr",
                                     "decr",
                                     "hash",
                                     "lock",
                                     "unlock",
                                     "cp",
                                     "rm",
                                     "stats",
                                     "version",
                                     "verbosity",
                                     "view",
                                     "query",
                                     "admin",
                                     "bucket-create",
                                     "bucket-delete",
                                     "bucket-flush",
                                     "role-list",
                                     "user-list",
                                     "user-upsert",
                                     "user-delete",
                                     "connstr",
                                     "write-config",
                                     "strerror",
                                     "ping",
                                     "watch",
                                     "keygen",
                                     "collection-manifest",
                                     "collection-id",
                                     NULL};

class HelpHandler : public Handler
{
  public:
    HelpHandler() : Handler("help") {}
    HANDLER_DESCRIPTION("Show help")
  protected:
    void run()
    {
        fprintf(stderr, "Usage: cbc <command> [options]\n");
        fprintf(stderr, "command may be:\n");
        for (const char **cur = optionsOrder; *cur; cur++) {
            const Handler *handler = handlers[*cur];
            fprintf(stderr, "   %-20s", *cur);
            fprintf(stderr, " %s\n", handler->description());
        }
    }
};

class StrErrorHandler : public Handler
{
  public:
    StrErrorHandler() : Handler("strerror") {}
    HANDLER_DESCRIPTION("Decode library error code")
    HANDLER_USAGE("HEX OR DECIMAL CODE")
  protected:
    void handleOptions() {}
    void run()
    {
        std::string nn = getRequiredArg();
        // Try to parse it as a hexadecimal number
        unsigned errcode;
        int rv = sscanf(nn.c_str(), "0x%x", &errcode);
        if (rv != 1) {
            rv = sscanf(nn.c_str(), "%u", &errcode);
            if (rv != 1) {
                throw BadArg("Need decimal or hex code!");
            }
        }

#define X(cname, code, cat, desc)                                                                                      \
    if (code == errcode) {                                                                                             \
        fprintf(stderr, "%s\n", #cname);                                                                               \
        fprintf(stderr, "  Type: 0x%x\n", cat);                                                                        \
        fprintf(stderr, "  Description: %s\n", desc);                                                                  \
        return;                                                                                                        \
    }

        LCB_XERR(X)
#undef X

        fprintf(stderr, "-- Error code not found in header. Trying runtime..\n");
        fprintf(stderr, "%s\n", lcb_strerror_long((lcb_STATUS)errcode));
    }
};

static void setupHandlers()
{
    handlers_s["get"] = new GetHandler();
    handlers_s["create"] = new SetHandler();
    handlers_s["hash"] = new HashHandler();
    handlers_s["help"] = new HelpHandler();
    handlers_s["lock"] = new GetHandler("lock");
    handlers_s["observe"] = new ObserveHandler();
    handlers_s["unlock"] = new UnlockHandler();
    handlers_s["version"] = new VersionHandler();
    handlers_s["rm"] = new RemoveHandler();
    handlers_s["cp"] = new SetHandler("cp");
    handlers_s["stats"] = new StatsHandler();
    handlers_s["watch"] = new WatchHandler();
    handlers_s["verbosity"] = new VerbosityHandler();
    handlers_s["ping"] = new PingHandler();
    handlers_s["incr"] = new IncrHandler();
    handlers_s["decr"] = new DecrHandler();
    handlers_s["admin"] = new AdminHandler();
    handlers_s["bucket-create"] = new BucketCreateHandler();
    handlers_s["bucket-delete"] = new BucketDeleteHandler();
    handlers_s["bucket-flush"] = new BucketFlushHandler();
    handlers_s["view"] = new ViewsHandler();
    handlers_s["query"] = new N1qlHandler();
    handlers_s["connstr"] = new ConnstrHandler();
    handlers_s["write-config"] = new WriteConfigHandler();
    handlers_s["strerror"] = new StrErrorHandler();
    handlers_s["observe-seqno"] = new ObserveSeqnoHandler();
    handlers_s["touch"] = new TouchHandler();
    handlers_s["role-list"] = new RoleListHandler();
    handlers_s["user-list"] = new UserListHandler();
    handlers_s["user-upsert"] = new UserUpsertHandler();
    handlers_s["user-delete"] = new UserDeleteHandler();
    handlers_s["mcversion"] = new McVersionHandler();
    handlers_s["keygen"] = new KeygenHandler();
    handlers_s["collection-manifest"] = new CollectionGetManifestHandler();
    handlers_s["collection-id"] = new CollectionGetCIDHandler();
    handlers_s["exists"] = new ExistsHandler();

    map< string, Handler * >::iterator ii;
    for (ii = handlers_s.begin(); ii != handlers_s.end(); ++ii) {
        handlers[ii->first] = ii->second;
    }

    handlers["cat"] = handlers["get"];
    handlers["n1ql"] = handlers["query"];
}

#if _POSIX_VERSION >= 200112L
#include <libgen.h>
#define HAVE_BASENAME
#endif

static void parseCommandname(string &cmdname, int &, char **&argv)
{
#ifdef HAVE_BASENAME
    cmdname = basename(argv[0]);
    size_t dashpos;

    if (cmdname.find("cbc") != 0) {
        cmdname.clear();
        // Doesn't start with cbc
        return;
    }

    if ((dashpos = cmdname.find('-')) != string::npos && cmdname.find("cbc") != string::npos &&
        dashpos + 1 < cmdname.size()) {

        // Get the actual command name
        cmdname = cmdname.substr(dashpos + 1);
        return;
    }
#else
    (void)argv;
#endif
    cmdname.clear();
}

static void wrapExternalBinary(int argc, char **argv, const std::string &name)
{
#ifdef _POSIX_VERSION
    vector< char * > args;
    string exePath(argv[0]);
    size_t cbc_pos = exePath.find("cbc");

    if (cbc_pos == string::npos) {
        fprintf(stderr, "Failed to invoke %s (%s)\n", name.c_str(), exePath.c_str());
        exit(EXIT_FAILURE);
    }

    exePath.replace(cbc_pos, 3, name);
    args.push_back((char *)exePath.c_str());

    // { "cbc", "name" }
    argv += 2;
    argc -= 2;
    for (int ii = 0; ii < argc; ii++) {
        args.push_back(argv[ii]);
    }
    args.push_back((char *)NULL);
    execvp(exePath.c_str(), &args[0]);
    fprintf(stderr, "Failed to execute execute %s (%s): %s\n", name.c_str(), exePath.c_str(), strerror(errno));
#else
    fprintf(stderr, "Can't wrap around %s on non-POSIX environments", name.c_str());
#endif
    exit(EXIT_FAILURE);
}

static void cleanupHandlers()
{
    map< string, Handler * >::iterator iter = handlers_s.begin();
    for (; iter != handlers_s.end(); iter++) {
        delete iter->second;
    }
}

int main(int argc, char *argv[])
{

    // Wrap external binaries immediately
    if (argc >= 2) {
        if (strcmp(argv[1], "pillowfight") == 0) {
            wrapExternalBinary(argc, argv, "cbc-pillowfight");
        } else if (strcmp(argv[1], "n1qlback") == 0) {
            wrapExternalBinary(argc, argv, "cbc-n1qlback");
        } else if (strcmp(argv[1], "subdoc") == 0) {
            wrapExternalBinary(argc, argv, "cbc-subdoc");
        } else if (strcmp(argv[1], "proxy") == 0) {
            wrapExternalBinary(argc, argv, "cbc-proxy");
        }
    }

    setupHandlers();
    std::atexit(cleanupHandlers);

    string cmdname;
    parseCommandname(cmdname, argc, argv);

    if (cmdname.empty()) {
        if (argc < 2) {
            fprintf(stderr, "Must provide an option name\n");
            try {
                HelpHandler().execute(argc, argv);
            } catch (std::exception &exc) {
                std::cerr << exc.what() << std::endl;
            }
            exit(EXIT_FAILURE);
        } else {
            cmdname = argv[1];
            argv++;
            argc--;
        }
    }

    Handler *handler = handlers[cmdname];
    if (handler == NULL) {
        fprintf(stderr, "Unknown command %s\n", cmdname.c_str());
        HelpHandler().execute(argc, argv);
        exit(EXIT_FAILURE);
    }

    try {
        handler->execute(argc, argv);

    } catch (std::exception &err) {
        fprintf(stderr, "%s\n", err.what());
        exit(EXIT_FAILURE);
    }
}
