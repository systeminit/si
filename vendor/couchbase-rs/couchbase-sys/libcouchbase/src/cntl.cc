/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#include "internal.h"
#include "bucketconfig/clconfig.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <lcbio/iotable.h>
#include <mcserver/negotiate.h>
#include <lcbio/ssl.h>

#define LOGARGS(instance, lvl) instance->settings, "cntl", LCB_LOG_##lvl, __FILE__, __LINE__

#define CNTL__MODE_SETSTRING 0x1000

/* Basic definition/declaration for handlers */
#define HANDLER(name) static lcb_STATUS name(int mode, lcb_INSTANCE *instance, int cmd, void *arg)

/* For handlers which only retrieve values */
#define RETURN_GET_ONLY(T, acc) \
    if (mode != LCB_CNTL_GET) { return LCB_ECTL_UNSUPPMODE; } \
    *reinterpret_cast<T*>(arg) = (T)acc; \
    return LCB_SUCCESS; \
    (void)cmd;

#define RETURN_SET_ONLY(T, acc) \
    if (mode != LCB_CNTL_SET) { return LCB_ECTL_UNSUPPMODE; } \
    acc = *reinterpret_cast<T*>(arg); \
    return LCB_SUCCESS;

#define RETURN_GET_SET(T, acc) \
        if (mode == LCB_CNTL_GET) { \
            RETURN_GET_ONLY(T, acc); \
        } \
        else if (mode == LCB_CNTL_SET) { \
            RETURN_SET_ONLY(T, acc); \
        } \
        else { \
            return LCB_ECTL_UNSUPPMODE; \
        }

typedef lcb_STATUS (*ctl_handler)(int, lcb_INSTANCE *, int, void *);
typedef struct { const char *s; lcb_U32 u32; } STR_u32MAP;
static const STR_u32MAP* u32_from_map(const char *s, const STR_u32MAP *lookup) {
    const STR_u32MAP *ret;
    for (ret = lookup; ret->s; ret++) {
        lcb_SIZE maxlen = strlen(ret->s);
        if (!strncmp(ret->s, s, maxlen)) { return ret; }
    }
    return NULL;
}
#define DO_CONVERT_STR2NUM(s, lookup, v) { \
    const STR_u32MAP *str__rv = u32_from_map(s, lookup); \
    if (str__rv) { v = str__rv->u32; } else { return LCB_ECTL_BADARG; } }

static lcb_uint32_t *get_timeout_field(lcb_INSTANCE *instance, int cmd)
{
    lcb_settings *settings = instance->settings;
    switch (cmd) {
    case LCB_CNTL_OP_TIMEOUT: return &settings->operation_timeout;
    case LCB_CNTL_VIEW_TIMEOUT: return &settings->views_timeout;
    case LCB_CNTL_N1QL_TIMEOUT: return &settings->n1ql_timeout;
    case LCB_CNTL_DURABILITY_INTERVAL: return &settings->durability_interval;
    case LCB_CNTL_DURABILITY_TIMEOUT: return &settings->durability_timeout;
    case LCB_CNTL_HTTP_TIMEOUT: return &settings->http_timeout;
    case LCB_CNTL_CONFIGURATION_TIMEOUT: return &settings->config_timeout;
    case LCB_CNTL_CONFDELAY_THRESH: return &settings->weird_things_delay;
    case LCB_CNTL_CONFIG_NODE_TIMEOUT: return &settings->config_node_timeout;
    case LCB_CNTL_HTCONFIG_IDLE_TIMEOUT: return &settings->bc_http_stream_time;
    case LCB_CNTL_RETRY_INTERVAL: return &settings->retry_interval;
    case LCB_CNTL_RETRY_NMV_INTERVAL: return &settings->retry_nmv_interval;
    case LCB_CNTL_CONFIG_POLL_INTERVAL: return &settings->config_poll_interval;
    case LCB_CNTL_TRACING_ORPHANED_QUEUE_FLUSH_INTERVAL: return &settings->tracer_orphaned_queue_flush_interval;
    case LCB_CNTL_TRACING_THRESHOLD_QUEUE_FLUSH_INTERVAL: return &settings->tracer_threshold_queue_flush_interval;
    case LCB_CNTL_TRACING_THRESHOLD_KV: return &settings->tracer_threshold[LCBTRACE_THRESHOLD_KV];
    case LCB_CNTL_TRACING_THRESHOLD_N1QL: return &settings->tracer_threshold[LCBTRACE_THRESHOLD_N1QL];
    case LCB_CNTL_TRACING_THRESHOLD_VIEW: return &settings->tracer_threshold[LCBTRACE_THRESHOLD_VIEW];
    case LCB_CNTL_TRACING_THRESHOLD_FTS: return &settings->tracer_threshold[LCBTRACE_THRESHOLD_FTS];
    case LCB_CNTL_TRACING_THRESHOLD_ANALYTICS: return &settings->tracer_threshold[LCBTRACE_THRESHOLD_ANALYTICS];
    case LCB_CNTL_PERSISTENCE_TIMEOUT_FLOOR: return &settings->persistence_timeout_floor;
    default: return NULL;
    }
}

HANDLER(timeout_common) {
    lcb_U32 *ptr;
    lcb_U32 *user = reinterpret_cast<lcb_U32*>(arg);

    ptr = get_timeout_field(instance, cmd);
    if (!ptr) {
        return LCB_ECTL_BADARG;
    }
    if (mode == LCB_CNTL_GET) {
        *user = *ptr;
    } else {
        if (cmd == LCB_CNTL_PERSISTENCE_TIMEOUT_FLOOR && *user < LCB_DEFAULT_PERSISTENCE_TIMEOUT_FLOOR) {
            return LCB_ECTL_BADARG;
        }
        *ptr = *user;
    }
    return LCB_SUCCESS;
}

HANDLER(noop_handler) {
    (void)mode;(void)instance;(void)cmd;(void)arg; return LCB_SUCCESS;
}
HANDLER(get_vbconfig) {
    RETURN_GET_ONLY(lcbvb_CONFIG*, LCBT_VBCONFIG(instance))
}
HANDLER(get_htype) {
    RETURN_GET_ONLY(lcb_type_t, static_cast<lcb_type_t>(instance->settings->conntype))
}
HANDLER(get_iops) {
    RETURN_GET_ONLY(lcb_io_opt_t, instance->iotable->p)
}
HANDLER(ippolicy) {
    RETURN_GET_SET(lcb_ipv6_t, instance->settings->ipv6)
}
HANDLER(confthresh) {
    RETURN_GET_SET(lcb_SIZE, instance->settings->weird_things_threshold)
}
HANDLER(randomize_bootstrap_hosts_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, randomize_bootstrap_nodes))
}
HANDLER(get_changeset) {
    (void)instance; RETURN_GET_ONLY(char*, LCB_VERSION_CHANGESET)
}
HANDLER(ssl_mode_handler) {
    RETURN_GET_ONLY(int, LCBT_SETTING(instance, sslopts))
}
HANDLER(ssl_truststorepath_handler) {
    RETURN_GET_ONLY(char*, LCBT_SETTING(instance, truststorepath))
}
HANDLER(ssl_certpath_handler) {
    RETURN_GET_ONLY(char*, LCBT_SETTING(instance, certpath))
}
HANDLER(ssl_keypath_handler) {
    RETURN_GET_ONLY(char*, LCBT_SETTING(instance, keypath))
}
HANDLER(htconfig_urltype_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, bc_http_urltype));
}
HANDLER(syncdtor_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, syncdtor));
}
HANDLER(detailed_errcode_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, detailed_neterr))
}
HANDLER(http_poolsz_handler) {
    RETURN_GET_SET(lcb_SIZE, instance->http_sockpool->get_options().maxidle)
}
HANDLER(http_pooltmo_handler) {
    RETURN_GET_SET(uint32_t, instance->http_sockpool->get_options().tmoidle)
}
HANDLER(http_refresh_config_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, refresh_on_hterr))
}
HANDLER(compmode_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, compressopts))
}
HANDLER(bucketname_handler) {
    RETURN_GET_ONLY(const char*, LCBT_SETTING(instance, bucket))
}
HANDLER(buckettype_handler) {
    RETURN_GET_ONLY(lcb_BTYPE, static_cast<lcb_BTYPE>(instance->btype))
}
HANDLER(schedflush_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, sched_implicit_flush))
}
HANDLER(vbguess_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, keep_guess_vbs))
}
HANDLER(vb_noremap_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, vb_noremap))
}
HANDLER(wait_for_config_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, wait_for_config))
}
HANDLER(fetch_mutation_tokens_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, fetch_mutation_tokens))
}
HANDLER(dur_mutation_tokens_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, dur_mutation_tokens))
}
HANDLER(nmv_imm_retry_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, nmv_retry_imm));
}
HANDLER(tcp_nodelay_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, tcp_nodelay));
}
HANDLER(tcp_keepalive_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, tcp_keepalive));
}
HANDLER(readj_ts_wait_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, readj_ts_wait));
}
HANDLER(kv_hg_handler) {
    RETURN_GET_ONLY(lcb_HISTOGRAM*, instance->kv_timings);
}
HANDLER(read_chunk_size_handler) {
    RETURN_GET_SET(lcb_U32, LCBT_SETTING(instance, read_chunk_size));
}
HANDLER(enable_errmap_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, use_errmap));
}
HANDLER(select_bucket_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, select_bucket));
}
HANDLER(send_hello_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, send_hello));
}
HANDLER(log_redaction_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, log_redaction));
}
HANDLER(enable_tracing_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, use_tracing));
}
HANDLER(tracing_orphaned_queue_size_handler) {
    RETURN_GET_SET(lcb_U32, LCBT_SETTING(instance, tracer_orphaned_queue_size));
}
HANDLER(tracing_threshold_queue_size_handler) {
    RETURN_GET_SET(lcb_U32, LCBT_SETTING(instance, tracer_threshold_queue_size));
}

HANDLER(config_poll_interval_handler) {
    lcb_U32 *user = reinterpret_cast<lcb_U32*>(arg);
    if (mode == LCB_CNTL_SET && *user > 0 && *user < LCB_CONFIG_POLL_INTERVAL_FLOOR) {
        lcb_log(LOGARGS(instance, ERROR), "Interval for background poll is too low: %dus (min: %dus)", *user, LCB_CONFIG_POLL_INTERVAL_FLOOR);
        return LCB_ECTL_BADARG;
    }
    lcb_STATUS rv = timeout_common(mode, instance, cmd, arg);
    if (rv == LCB_SUCCESS &&
            (mode == LCB_CNTL_SET || CNTL__MODE_SETSTRING) &&
            // Note: This might be NULL during creation!
            instance->bs_state) {
        instance->bs_state->check_bgpoll();
    }
    return rv;
}

HANDLER(get_kvb) {
    lcb_cntl_vbinfo_st *vbi = reinterpret_cast<lcb_cntl_vbinfo_st*>(arg);

    if (mode != LCB_CNTL_GET) { return LCB_ECTL_UNSUPPMODE; }
    if (!LCBT_VBCONFIG(instance)) { return LCB_CLIENT_ETMPFAIL; }
    if (vbi->version != 0) { return LCB_ECTL_BADARG; }

    lcbvb_map_key(LCBT_VBCONFIG(instance), vbi->v.v0.key, vbi->v.v0.nkey,
        &vbi->v.v0.vbucket, &vbi->v.v0.server_index);
    (void)cmd; return LCB_SUCCESS;
}


HANDLER(conninfo) {
    const lcbio_SOCKET *sock;
    lcb_cntl_server_st *si = reinterpret_cast<lcb_cntl_server_st*>(arg);
    const lcb_host_t *host;

    if (mode != LCB_CNTL_GET) { return LCB_ECTL_UNSUPPMODE; }
    if (si->version < 0 || si->version > 1) { return LCB_ECTL_BADARG; }

    if (cmd == LCB_CNTL_MEMDNODE_INFO) {
        lcb::Server *server;
        int ix = si->v.v0.index;

        if (ix < 0 || ix > (int)LCBT_NSERVERS(instance)) {
            return LCB_ECTL_BADARG;
        }
        server = instance->get_server(ix);
        if (!server) {
            return LCB_NETWORK_ERROR;
        }
        sock = server->connctx->sock;
        if (si->version == 1 && sock) {
            lcb::SessionInfo *info = lcb::SessionInfo::get(server->connctx->sock);
            if (info) {
                si->v.v1.sasl_mech = info->get_mech().c_str();
            }
        }
    } else if (cmd == LCB_CNTL_CONFIGNODE_INFO) {
        sock = lcb::clconfig::http_get_conn(instance->confmon);
    } else {
        return LCB_ECTL_BADARG;
    }

    if (!sock) {
        return LCB_SUCCESS;
    }
    host = lcbio_get_host(sock);
    si->v.v0.connected = 1;
    si->v.v0.host = host->host;
    si->v.v0.port = host->port;
    if (instance->iotable->model == LCB_IOMODEL_EVENT) {
        si->v.v0.sock.sockfd = sock->u.fd;
    } else {
        si->v.v0.sock.sockptr = sock->u.sd;
    }
    return LCB_SUCCESS;
}

HANDLER(config_cache_loaded_handler) {
    if (mode != LCB_CNTL_GET) { return LCB_ECTL_UNSUPPMODE; }
    *(int *)arg = instance->cur_configinfo &&
            instance->cur_configinfo->get_origin() == lcb::clconfig::CLCONFIG_FILE;
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(force_sasl_mech_handler) {
    if (mode == LCB_CNTL_SET) {
        free(instance->settings->sasl_mech_force);
        if (arg) {
            const char *s = reinterpret_cast<const char*>(arg);
            instance->settings->sasl_mech_force = strdup(s);
        }
    } else {
        *(char**)arg = instance->settings->sasl_mech_force;
    }
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(max_redirects) {
    if (mode == LCB_CNTL_SET && *(int*)arg < -1) { return LCB_ECTL_BADARG; }
    RETURN_GET_SET(int, LCBT_SETTING(instance, max_redir))
}

HANDLER(logprocs_handler) {
    if (mode == LCB_CNTL_GET) {
        *(lcb_logprocs**)arg = LCBT_SETTING(instance, logger);
    } else if (mode == LCB_CNTL_SET) {
        LCBT_SETTING(instance, logger) = (lcb_logprocs *)arg;
    }
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(config_transport) {
    lcb_config_transport_t *val = reinterpret_cast<lcb_config_transport_t*>(arg);
    if (mode == LCB_CNTL_SET) { return LCB_ECTL_UNSUPPMODE; }
    if (!instance->cur_configinfo) { return LCB_CLIENT_ETMPFAIL; }

    switch (instance->cur_configinfo->get_origin()) {
        case lcb::clconfig::CLCONFIG_HTTP: *val = LCB_CONFIG_TRANSPORT_HTTP; break;
        case lcb::clconfig::CLCONFIG_CCCP: *val = LCB_CONFIG_TRANSPORT_CCCP; break;
        default: return LCB_CLIENT_ETMPFAIL;
    }
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(config_nodes) {
    const char *node_strs = reinterpret_cast<const char*>(arg);
    lcb::clconfig::Provider *target;
    lcb::Hostlist hostlist;
    lcb_STATUS err;

    if (mode != LCB_CNTL_SET) {
        return LCB_ECTL_UNSUPPMODE;
    }

    err = hostlist.add(node_strs, -1,
        cmd == LCB_CNTL_CONFIG_HTTP_NODES
        ? LCB_CONFIG_HTTP_PORT : LCB_CONFIG_MCD_PORT);

    if (err != LCB_SUCCESS) {
        return err;
    }

    if (cmd == LCB_CNTL_CONFIG_HTTP_NODES) {
        target = instance->confmon->get_provider(lcb::clconfig::CLCONFIG_HTTP);
    } else {
        target = instance->confmon->get_provider(lcb::clconfig::CLCONFIG_CCCP);
    }

    target->configure_nodes(hostlist);

    return LCB_SUCCESS;
}


HANDLER(init_providers) {
    lcb_create_st2 *opts = reinterpret_cast<lcb_create_st2*>(arg);
    if (mode != LCB_CNTL_SET) { return LCB_ECTL_UNSUPPMODE; }
    (void)cmd; return lcb_init_providers2(instance, opts);
}

HANDLER(config_cache_handler) {
    using namespace lcb::clconfig;
    Provider *provider;

    provider = instance->confmon->get_provider(lcb::clconfig::CLCONFIG_FILE);
    if (mode == LCB_CNTL_SET) {
        bool rv = file_set_filename(provider,
            reinterpret_cast<const char*>(arg),
            cmd == LCB_CNTL_CONFIGCACHE_RO);

        if (rv) {
            instance->settings->bc_http_stream_time = LCB_MS2US(10000);
            return LCB_SUCCESS;
        }
        return LCB_ERROR;
    } else {
        *(const char **)arg = file_get_filename(provider);
        return LCB_SUCCESS;
    }
}

HANDLER(retrymode_handler) {
    lcb_U32 *val = reinterpret_cast<lcb_U32*>(arg);
    lcb_U32 rmode = LCB_RETRYOPT_GETMODE(*val);
    uint8_t *p = NULL;

    if (rmode >= LCB_RETRY_ON_MAX) { return LCB_ECTL_BADARG; }
    p = &(LCBT_SETTING(instance, retry)[rmode]);
    if (mode == LCB_CNTL_SET) {
        *p = LCB_RETRYOPT_GETPOLICY(*val);
    } else {
        *val = LCB_RETRYOPT_CREATE(rmode, *p);
    }
    (void)cmd;
    return LCB_SUCCESS;
}

HANDLER(allocfactory_handler) {
    lcb_cntl_rdballocfactory *cbw = reinterpret_cast<lcb_cntl_rdballocfactory*>(arg);
    if (mode == LCB_CNTL_SET) {
        LCBT_SETTING(instance, allocator_factory) = cbw->factory;
    } else {
        cbw->factory = LCBT_SETTING(instance, allocator_factory);
    }
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(console_log_handler) {
    lcb_U32 level;
    struct lcb_CONSOLELOGGER *logger;
    lcb_logprocs *procs;

    level = *(lcb_U32*)arg;
    if (mode != LCB_CNTL_SET) {
        return LCB_ECTL_UNSUPPMODE;
    }

    procs = LCBT_SETTING(instance, logger);
    if (!procs) {
        procs = lcb_init_console_logger();
    }
    if (procs) {
        /* don't override previous config */
        return LCB_SUCCESS;
    }

    logger = (struct lcb_CONSOLELOGGER* ) lcb_console_logprocs;
    level = LCB_LOG_ERROR - level;
    logger->minlevel = level;
    LCBT_SETTING(instance, logger) = &logger->base;
    (void)cmd; return LCB_SUCCESS;
}

HANDLER(console_fp_handler) {
    struct lcb_CONSOLELOGGER *logger =
            (struct lcb_CONSOLELOGGER*)lcb_console_logprocs;
    if (mode == LCB_CNTL_GET) {
        *(FILE **)arg = logger->fp;
    } else if (mode == LCB_CNTL_SET) {
        logger->fp = *(FILE**)arg;
    } else if (mode == CNTL__MODE_SETSTRING) {
        FILE *fp = fopen(reinterpret_cast<const char*>(arg), "w");
        if (!fp) {
            return LCB_ERROR;
        } else {
            logger->fp = fp;
        }
    }
    (void)cmd; (void)instance;
    return LCB_SUCCESS;
}

HANDLER(reinit_spec_handler) {
    if (mode == LCB_CNTL_GET) { return LCB_ECTL_UNSUPPMODE; }
    (void)cmd; return lcb_reinit3(instance, reinterpret_cast<const char*>(arg));
}

HANDLER(client_string_handler) {
    if (mode == LCB_CNTL_SET) {
        const char *val = reinterpret_cast<const char*>(arg);
        free(LCBT_SETTING(instance, client_string));
        LCBT_SETTING(instance, client_string) = NULL;
        if (val) {
            char *p, *buf = strdup(val);
            for (p = buf; *p != '\0'; p++) {
                switch (*p) {
                    case '\n':
                    case '\r':
                        *p = ' ';
                        break;
                    default:
                        break;
                }
            }
            LCBT_SETTING(instance, client_string) = buf;
        }
    } else {
        *(const char **)arg = LCBT_SETTING(instance, client_string);
    }
    (void)cmd;
    return LCB_SUCCESS;
}

HANDLER(unsafe_optimize) {
    lcb_STATUS rc;
    int val = *(int *)arg;
    if (mode != LCB_CNTL_SET) {
        return LCB_ECTL_UNSUPPMODE;
    }
    if (!val) {
        return LCB_ECTL_BADARG;
    }

    /* Simpler to just input strings here. */
    #define APPLY_UNSAFE(k, v) \
        if ((rc = lcb_cntl_string(instance, k, v)) != LCB_SUCCESS) { return rc; }

    APPLY_UNSAFE("vbguess_persist", "1");
    APPLY_UNSAFE("retry_policy", "topochange:none")
    APPLY_UNSAFE("retry_policy", "sockerr:none");
    APPLY_UNSAFE("retry_policy", "maperr:none");
    APPLY_UNSAFE("retry_policy", "missingnode:none");
    (void)cmd;
    return LCB_SUCCESS;
}

HANDLER(mutation_tokens_supported_handler) {
    size_t ii;
    if (mode != LCB_CNTL_GET) {
        return LCB_ECTL_UNSUPPMODE;
    }

    *(int *)arg = 0;

    for (ii = 0; ii < LCBT_NSERVERS(instance); ii++) {
        if (instance->get_server(ii)->supports_mutation_tokens()) {
            *(int *)arg = 1;
            break;
        }
    }
    (void)cmd;
    return LCB_SUCCESS;
}

HANDLER(n1ql_cache_clear_handler) {
    if (mode != LCB_CNTL_SET) {
        return LCB_ECTL_UNSUPPMODE;
    }

    lcb_n1qlcache_clear(instance->n1ql_cache);

    (void)cmd;
    (void)arg;
    return LCB_SUCCESS;
}

HANDLER(bucket_auth_handler) {
    const lcb_BUCKETCRED *cred;
    if (mode == LCB_CNTL_SET) {
        if (LCBT_SETTING(instance, keypath)) {
            return LCB_ECTL_UNSUPPMODE;
        }
        /* Parse the bucket string... */
        cred = (const lcb_BUCKETCRED *)arg;
        return lcbauth_add_pass(instance->settings->auth, (*cred)[0], (*cred)[1], LCBAUTH_F_BUCKET);
        (void)cmd; (void)arg;
    } else if (mode == CNTL__MODE_SETSTRING) {
        const char *ss = reinterpret_cast<const char *>(arg);
        size_t sslen = strlen(ss);
        Json::Value root;
        if (!Json::Reader().parse(ss, ss + sslen, root)) {
            return LCB_ECTL_BADARG;
        }
        if (!root.isArray() || root.size() != 2) {
            return LCB_ECTL_BADARG;
        }
        return lcbauth_add_pass(instance->settings->auth,
            root[0].asString().c_str(),
            root[1].asString().c_str(), LCBAUTH_F_BUCKET);
    } else {
        return LCB_ECTL_UNSUPPMODE;
    }
    return LCB_SUCCESS;
}

HANDLER(metrics_handler) {
    if (mode == LCB_CNTL_SET) {
        int val = *(int *)arg;
        if (!val) {
            return LCB_ECTL_BADARG;
        }
        if (!instance->settings->metrics) {
            instance->settings->metrics = lcb_metrics_new();
        }
        return LCB_SUCCESS;
    } else if (mode == LCB_CNTL_GET) {
        *(lcb_METRICS**)arg = instance->settings->metrics;
        return LCB_SUCCESS;
    } else {
        return LCB_ECTL_UNSUPPMODE;
    }
    (void)cmd;
}

HANDLER(collections_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, use_collections));
}

HANDLER(allow_static_config_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, allow_static_config));
}

HANDLER(comp_min_size_handler) {
    if (mode == LCB_CNTL_SET && *reinterpret_cast<lcb_U32*>(arg) < LCB_DEFAULT_COMPRESS_MIN_SIZE) {
        return LCB_ECTL_BADARG;
    }
    RETURN_GET_SET(lcb_U32, LCBT_SETTING(instance, compress_min_size))
}

HANDLER(comp_min_ratio_handler) {
    if (mode == LCB_CNTL_SET) {
        float val = *reinterpret_cast<float*>(arg);
        if (val > 1 || val < 0) {
            return LCB_ECTL_BADARG;
        }
    }
    RETURN_GET_SET(float, LCBT_SETTING(instance, compress_min_ratio))
}

HANDLER(network_handler) {
    if (mode == LCB_CNTL_SET) {
        const char *val = reinterpret_cast<const char*>(arg);
        free(LCBT_SETTING(instance, network));
        LCBT_SETTING(instance, network) = NULL;
        if (val) {
            LCBT_SETTING(instance, network) = strdup(val);
        }
    } else {
        *(const char **)arg = LCBT_SETTING(instance, network);
    }
    (void)cmd;
    return LCB_SUCCESS;
}

HANDLER(durable_write_handler) {
    RETURN_GET_SET(int, LCBT_SETTING(instance, enable_durable_write));
}

static ctl_handler handlers[] = {
    timeout_common,                       /* LCB_CNTL_OP_TIMEOUT */
    timeout_common,                       /* LCB_CNTL_VIEW_TIMEOUT */
    noop_handler,                         /* LCB_CNTL_RBUFSIZE */
    noop_handler,                         /* LCB_CNTL_WBUFSIZE */
    get_htype,                            /* LCB_CNTL_HANDLETYPE */
    get_vbconfig,                         /* LCB_CNTL_VBCONFIG */
    get_iops,                             /* LCB_CNTL_IOPS */
    get_kvb,                              /* LCB_CNTL_VBMAP */
    conninfo,                             /* LCB_CNTL_MEMDNODE_INFO */
    conninfo,                             /* LCB_CNTL_CONFIGNODE_INFO */
    NULL,                                 /* deprecated LCB_CNTL_SYNCMODE (0x0a) */
    ippolicy,                             /* LCB_CNTL_IP6POLICY */
    confthresh,                           /* LCB_CNTL_CONFERRTHRESH */
    timeout_common,                       /* LCB_CNTL_DURABILITY_INTERVAL */
    timeout_common,                       /* LCB_CNTL_DURABILITY_TIMEOUT */
    timeout_common,                       /* LCB_CNTL_HTTP_TIMEOUT */
    lcb_iops_cntl_handler,                /* LCB_CNTL_IOPS_DEFAULT_TYPES */
    lcb_iops_cntl_handler,                /* LCB_CNTL_IOPS_DLOPEN_DEBUG */
    timeout_common,                       /* LCB_CNTL_CONFIGURATION_TIMEOUT */
    noop_handler,                         /* LCB_CNTL_SKIP_CONFIGURATION_ERRORS_ON_CONNECT */
    randomize_bootstrap_hosts_handler,    /* LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS */
    config_cache_loaded_handler,          /* LCB_CNTL_CONFIG_CACHE_LOADED */
    force_sasl_mech_handler,              /* LCB_CNTL_FORCE_SASL_MECH */
    max_redirects,                        /* LCB_CNTL_MAX_REDIRECTS */
    logprocs_handler,                     /* LCB_CNTL_LOGGER */
    timeout_common,                       /* LCB_CNTL_CONFDELAY_THRESH */
    config_transport,                     /* LCB_CNTL_CONFIG_TRANSPORT */
    timeout_common,                       /* LCB_CNTL_CONFIG_NODE_TIMEOUT */
    timeout_common,                       /* LCB_CNTL_HTCONFIG_IDLE_TIMEOUT */
    config_nodes,                         /* LCB_CNTL_CONFIG_HTTP_NODES */
    config_nodes,                         /* LCB_CNTL_CONFIG_CCCP_NODES */
    get_changeset,                        /* LCB_CNTL_CHANGESET */
    init_providers,                       /* LCB_CNTL_CONFIG_ALL_NODES */
    config_cache_handler,                 /* LCB_CNTL_CONFIGCACHE */
    ssl_mode_handler,                     /* LCB_CNTL_SSL_MODE */
    ssl_certpath_handler,                 /* LCB_CNTL_SSL_CERT */
    retrymode_handler,                    /* LCB_CNTL_RETRYMODE */
    htconfig_urltype_handler,             /* LCB_CNTL_HTCONFIG_URLTYPE */
    compmode_handler,                     /* LCB_CNTL_COMPRESSION_OPTS */
    allocfactory_handler,                 /* LCB_CNTL_RDBALLOCFACTORY */
    syncdtor_handler,                     /* LCB_CNTL_SYNCDESTROY */
    console_log_handler,                  /* LCB_CNTL_CONLOGGER_LEVEL */
    detailed_errcode_handler,             /* LCB_CNTL_DETAILED_ERRCODES */
    reinit_spec_handler,                  /* LCB_CNTL_REINIT_CONNSTR */
    timeout_common,                       /* LCB_CNTL_RETRY_INTERVAL */
    NULL,                                 /* deprecated LCB_CNTL_RETRY_BACKOFF (0x2D) */
    http_poolsz_handler,                  /* LCB_CNTL_HTTP_POOLSIZE */
    http_refresh_config_handler,          /* LCB_CNTL_HTTP_REFRESH_CONFIG_ON_ERROR */
    bucketname_handler,                   /* LCB_CNTL_BUCKETNAME */
    schedflush_handler,                   /* LCB_CNTL_SCHED_IMPLICIT_FLUSH */
    vbguess_handler,                      /* LCB_CNTL_VBGUESS_PERSIST */
    unsafe_optimize,                      /* LCB_CNTL_UNSAFE_OPTIMIZE */
    fetch_mutation_tokens_handler,        /* LCB_CNTL_FETCH_MUTATION_TOKENS */
    dur_mutation_tokens_handler,          /* LCB_CNTL_DURABILITY_MUTATION_TOKENS */
    config_cache_handler,                 /* LCB_CNTL_CONFIGCACHE_READONLY */
    nmv_imm_retry_handler,                /* LCB_CNTL_RETRY_NMV_IMM */
    mutation_tokens_supported_handler,    /* LCB_CNTL_MUTATION_TOKENS_SUPPORTED */
    tcp_nodelay_handler,                  /* LCB_CNTL_TCP_NODELAY */
    readj_ts_wait_handler,                /* LCB_CNTL_RESET_TIMEOUT_ON_WAIT */
    console_fp_handler,                   /* LCB_CNTL_CONLOGGER_FP */
    kv_hg_handler,                        /* LCB_CNTL_KVTIMINGS */
    timeout_common,                       /* LCB_CNTL_N1QL_TIMEOUT */
    n1ql_cache_clear_handler,             /* LCB_CNTL_N1QL_CLEARCACHE */
    client_string_handler,                /* LCB_CNTL_CLIENT_STRING */
    bucket_auth_handler,                  /* LCB_CNTL_BUCKET_CRED */
    timeout_common,                       /* LCB_CNTL_RETRY_NMV_DELAY */
    read_chunk_size_handler,              /* LCB_CNTL_READ_CHUNKSIZE */
    enable_errmap_handler,                /* LCB_CNTL_ENABLE_ERRMAP */
    select_bucket_handler,                /* LCB_CNTL_SELECT_BUCKET */
    tcp_keepalive_handler,                /* LCB_CNTL_TCP_KEEPALIVE */
    config_poll_interval_handler,         /* LCB_CNTL_CONFIG_POLL_INTERVAL */
    send_hello_handler,                   /* LCB_CNTL_SEND_HELLO */
    buckettype_handler,                   /* LCB_CNTL_BUCKETTYPE */
    metrics_handler,                      /* LCB_CNTL_METRICS */
    collections_handler,                  /* LCB_CNTL_ENABLE_COLLECTIONS */
    ssl_keypath_handler,                  /* LCB_CNTL_SSL_KEY */
    log_redaction_handler,                /* LCB_CNTL_LOG_REDACTION */
    ssl_truststorepath_handler,           /* LCB_CNTL_SSL_TRUSTSTORE */
    enable_tracing_handler,               /* LCB_CNTL_ENABLE_TRACING */
    timeout_common,                       /* LCB_CNTL_TRACING_ORPHANED_QUEUE_FLUSH_INTERVAL */
    tracing_orphaned_queue_size_handler,  /* LCB_CNTL_TRACING_ORPHANED_QUEUE_SIZE */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_QUEUE_FLUSH_INTERVAL */
    tracing_threshold_queue_size_handler, /* LCB_CNTL_TRACING_THRESHOLD_QUEUE_SIZE */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_KV */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_N1QL */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_VIEW */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_FTS */
    timeout_common,                       /* LCB_CNTL_TRACING_THRESHOLD_ANALYTICS */
    comp_min_size_handler,                /* LCB_CNTL_COMPRESSION_MIN_SIZE */
    comp_min_ratio_handler,               /* LCB_CNTL_COMPRESSION_MIN_RATIO */
    vb_noremap_handler,                   /* LCB_CNTL_VB_NOREMAP */
    network_handler,                      /* LCB_CNTL_NETWORK */
    wait_for_config_handler,              /* LCB_CNTL_WAIT_FOR_CONFIG */
    http_pooltmo_handler,                 /* LCB_CNTL_HTTP_POOL_TIMEOUT */
    durable_write_handler,                /* LCB_CNTL_ENABLE_DURABLE_WRITE */
    timeout_common,                       /* LCB_CNTL_PERSISTENCE_TIMEOUT_FLOOR */
    allow_static_config_handler,          /* LCB_CNTL_ALLOW_STATIC_CONFIG */
    NULL
};

/* Union used for conversion to/from string functions */
typedef union {
    lcb_U32 u32;
    lcb_SIZE sz;
    int i;
    float f;
    void *p;
} u_STRCONVERT;

/* This handler should convert the string argument to the appropriate
 * type needed for the actual control handler. It should return an error if the
 * argument is invalid.
 */
typedef lcb_STATUS (*ctl_str_cb)(const char *value, u_STRCONVERT *u);

typedef struct {
    const char *key;
    int opcode;
    ctl_str_cb converter;
} cntl_OPCODESTRS;

static lcb_STATUS convert_timevalue(const char *arg, u_STRCONVERT *u) {
    int rv;
    unsigned long tmp;

    /* Parse as a float */
    double dtmp;
    rv = sscanf(arg, "%lf", &dtmp);
    if (rv != 1) { return LCB_ECTL_BADARG; }
    tmp = dtmp * (double) 1000000;
    u->u32 = tmp;
    return LCB_SUCCESS;
}

static lcb_STATUS convert_intbool(const char *arg, u_STRCONVERT *u) {
    if (!strcmp(arg, "true") || !strcmp(arg, "on")) {
        u->i = 1;
    } else if (!strcmp(arg, "false") || !strcmp(arg, "off")) {
        u->i = 0;
    } else {
        u->i = atoi(arg);
    }
    return LCB_SUCCESS;
}

static lcb_STATUS convert_passthru(const char *arg, u_STRCONVERT *u) {
    u->p = (void*)arg;
    return LCB_SUCCESS;
}

static lcb_STATUS convert_int(const char *arg, u_STRCONVERT *u) {
    int rv = sscanf(arg, "%d", &u->i);
    return rv == 1 ? LCB_SUCCESS : LCB_ECTL_BADARG;
}

static lcb_STATUS convert_u32(const char *arg, u_STRCONVERT *u) {
    unsigned int tmp;
    int rv = sscanf(arg, "%u", &tmp);
    u->u32 = tmp;
    return rv == 1 ? LCB_SUCCESS : LCB_ECTL_BADARG;
}

static lcb_STATUS convert_float(const char *arg, u_STRCONVERT *u) {
    double d;
    int rv = sscanf(arg, "%lf", &d);
    if (rv != 1) { return LCB_ECTL_BADARG; }
    u->f = d;
    return LCB_SUCCESS;
}

static lcb_STATUS convert_SIZE(const char *arg, u_STRCONVERT *u) {
    unsigned long lu;
    int rv;
    rv = sscanf(arg, "%lu", &lu);
    if (rv != 1) { return LCB_ECTL_BADARG; }
    u->sz = lu;
    return LCB_SUCCESS;
}

static lcb_STATUS convert_compression(const char *arg, u_STRCONVERT *u) {
    static const STR_u32MAP optmap[] = {
        { "on", LCB_COMPRESS_INOUT },
        { "off", LCB_COMPRESS_NONE },
        { "inflate_only", LCB_COMPRESS_IN },
        { "deflate_only", LCB_COMPRESS_OUT },
        { "force", LCB_COMPRESS_INOUT|LCB_COMPRESS_FORCE },
        { NULL }
    };
    DO_CONVERT_STR2NUM(arg, optmap, u->i);
    return LCB_SUCCESS;
}

static lcb_STATUS convert_retrymode(const char *arg, u_STRCONVERT *u) {
    static const STR_u32MAP modemap[] = {
        { "topochange", LCB_RETRY_ON_TOPOCHANGE },
        { "sockerr", LCB_RETRY_ON_SOCKERR },
        { "maperr", LCB_RETRY_ON_VBMAPERR },
        { "missingnode", LCB_RETRY_ON_MISSINGNODE }, { NULL }
    };
    static const STR_u32MAP polmap[] = {
        { "all", LCB_RETRY_CMDS_ALL },
        { "get", LCB_RETRY_CMDS_GET },
        { "safe", LCB_RETRY_CMDS_SAFE },
        { "none", LCB_RETRY_CMDS_NONE }, { NULL }
    };

    lcb_U32 polval, modeval;
    const char *polstr = strchr(arg, ':');
    if (!polstr) { return LCB_ECTL_BADARG; }
    polstr++;
    DO_CONVERT_STR2NUM(arg, modemap, modeval);
    DO_CONVERT_STR2NUM(polstr, polmap, polval);
    u->u32 = LCB_RETRYOPT_CREATE(modeval, polval);
    return LCB_SUCCESS;
}

static lcb_STATUS convert_ipv6(const char *arg, u_STRCONVERT *u)
{
    static const STR_u32MAP optmap[] = {
        {"disabled", LCB_IPV6_DISABLED},
        {"only", LCB_IPV6_ONLY},
        {"allow", LCB_IPV6_ALLOW},
        {NULL}
    };
    DO_CONVERT_STR2NUM(arg, optmap, u->i);
    return LCB_SUCCESS;
}

static cntl_OPCODESTRS stropcode_map[] = {
    {"operation_timeout", LCB_CNTL_OP_TIMEOUT, convert_timevalue},
    {"timeout", LCB_CNTL_OP_TIMEOUT, convert_timevalue},
    {"views_timeout", LCB_CNTL_VIEW_TIMEOUT, convert_timevalue},
    {"n1ql_timeout", LCB_CNTL_N1QL_TIMEOUT, convert_timevalue},
    {"durability_timeout", LCB_CNTL_DURABILITY_TIMEOUT, convert_timevalue},
    {"durability_interval", LCB_CNTL_DURABILITY_INTERVAL, convert_timevalue},
    {"http_timeout", LCB_CNTL_HTTP_TIMEOUT, convert_timevalue},
    {"randomize_nodes", LCB_CNTL_RANDOMIZE_BOOTSTRAP_HOSTS, convert_intbool},
    {"sasl_mech_force", LCB_CNTL_FORCE_SASL_MECH, convert_passthru},
    {"error_thresh_count", LCB_CNTL_CONFERRTHRESH, convert_SIZE},
    {"error_thresh_delay", LCB_CNTL_CONFDELAY_THRESH, convert_timevalue},
    {"config_total_timeout", LCB_CNTL_CONFIGURATION_TIMEOUT, convert_timevalue},
    {"config_node_timeout", LCB_CNTL_CONFIG_NODE_TIMEOUT, convert_timevalue},
    {"compression", LCB_CNTL_COMPRESSION_OPTS, convert_compression},
    {"console_log_level", LCB_CNTL_CONLOGGER_LEVEL, convert_u32},
    {"config_cache", LCB_CNTL_CONFIGCACHE, convert_passthru},
    {"config_cache_ro", LCB_CNTL_CONFIGCACHE_RO, convert_passthru},
    {"detailed_errcodes", LCB_CNTL_DETAILED_ERRCODES, convert_intbool},
    {"retry_policy", LCB_CNTL_RETRYMODE, convert_retrymode},
    {"http_urlmode", LCB_CNTL_HTCONFIG_URLTYPE, convert_int},
    {"sync_dtor", LCB_CNTL_SYNCDESTROY, convert_intbool},
    {"_reinit_connstr", LCB_CNTL_REINIT_CONNSTR},
    {"", -1}, /* deprecated "retry_backoff" */
    {"retry_interval", LCB_CNTL_RETRY_INTERVAL, convert_timevalue},
    {"http_poolsize", LCB_CNTL_HTTP_POOLSIZE, convert_SIZE},
    {"vbguess_persist", LCB_CNTL_VBGUESS_PERSIST, convert_intbool},
    {"unsafe_optimize", LCB_CNTL_UNSAFE_OPTIMIZE, convert_intbool},
    {"fetch_mutation_tokens", LCB_CNTL_FETCH_MUTATION_TOKENS, convert_intbool},
    {"dur_mutation_tokens", LCB_CNTL_DURABILITY_MUTATION_TOKENS, convert_intbool},
    {"retry_nmv_imm", LCB_CNTL_RETRY_NMV_IMM, convert_intbool},
    {"tcp_nodelay", LCB_CNTL_TCP_NODELAY, convert_intbool},
    {"readj_ts_wait", LCB_CNTL_RESET_TIMEOUT_ON_WAIT, convert_intbool},
    {"console_log_file", LCB_CNTL_CONLOGGER_FP, NULL},
    {"client_string", LCB_CNTL_CLIENT_STRING, convert_passthru},
    {"retry_nmv_delay", LCB_CNTL_RETRY_NMV_INTERVAL, convert_timevalue},
    {"bucket_cred", LCB_CNTL_BUCKET_CRED, NULL},
    {"read_chunk_size", LCB_CNTL_READ_CHUNKSIZE, convert_u32},
    {"enable_errmap", LCB_CNTL_ENABLE_ERRMAP, convert_intbool},
    {"select_bucket", LCB_CNTL_SELECT_BUCKET, convert_intbool},
    {"tcp_keepalive", LCB_CNTL_TCP_KEEPALIVE, convert_intbool},
    {"config_poll_interval", LCB_CNTL_CONFIG_POLL_INTERVAL, convert_timevalue},
    {"send_hello", LCB_CNTL_SEND_HELLO, convert_intbool},
    {"ipv6", LCB_CNTL_IP6POLICY, convert_ipv6},
    {"metrics", LCB_CNTL_METRICS, convert_intbool},
    {"log_redaction", LCB_CNTL_LOG_REDACTION, convert_intbool},
    {"enable_tracing", LCB_CNTL_ENABLE_TRACING, convert_intbool},
    {"tracing_orphaned_queue_flush_interval", LCB_CNTL_TRACING_ORPHANED_QUEUE_FLUSH_INTERVAL, convert_timevalue},
    {"tracing_orphaned_queue_size", LCB_CNTL_TRACING_ORPHANED_QUEUE_SIZE, convert_u32},
    {"tracing_threshold_queue_flush_interval", LCB_CNTL_TRACING_THRESHOLD_QUEUE_FLUSH_INTERVAL, convert_timevalue},
    {"tracing_threshold_queue_size", LCB_CNTL_TRACING_THRESHOLD_QUEUE_SIZE, convert_u32},
    {"tracing_threshold_kv", LCB_CNTL_TRACING_THRESHOLD_KV, convert_timevalue},
    {"tracing_threshold_n1ql", LCB_CNTL_TRACING_THRESHOLD_N1QL, convert_timevalue},
    {"tracing_threshold_view", LCB_CNTL_TRACING_THRESHOLD_VIEW, convert_timevalue},
    {"tracing_threshold_fts", LCB_CNTL_TRACING_THRESHOLD_FTS, convert_timevalue},
    {"tracing_threshold_analytics", LCB_CNTL_TRACING_THRESHOLD_ANALYTICS, convert_timevalue},
    {"compression_min_size", LCB_CNTL_COMPRESSION_MIN_SIZE, convert_u32},
    {"compression_min_ratio", LCB_CNTL_COMPRESSION_MIN_RATIO, convert_float},
    {"vb_noremap", LCB_CNTL_VB_NOREMAP, convert_intbool},
    {"network", LCB_CNTL_NETWORK, convert_passthru},
    {"wait_for_config", LCB_CNTL_WAIT_FOR_CONFIG, convert_intbool},
    {"http_pool_timeout", LCB_CNTL_HTTP_POOL_TIMEOUT, convert_timevalue},
    {"enable_collections", LCB_CNTL_ENABLE_COLLECTIONS, convert_intbool},
    {"enable_durable_write", LCB_CNTL_ENABLE_DURABLE_WRITE, convert_intbool},
    {"persistence_timeout_floor", LCB_CNTL_PERSISTENCE_TIMEOUT_FLOOR, convert_timevalue},
    {"allow_static_config", LCB_CNTL_ALLOW_STATIC_CONFIG, convert_intbool},
    {NULL, -1}};

#define CNTL_NUM_HANDLERS (sizeof(handlers) / sizeof(handlers[0]))

static lcb_STATUS
wrap_return(lcb_INSTANCE *instance, lcb_STATUS retval)
{
    if (retval == LCB_SUCCESS) {
        return retval;
    }
    if (instance && LCBT_SETTING(instance, detailed_neterr) == 0) {
        switch (retval) {
        case LCB_ECTL_UNKNOWN:
            return LCB_NOT_SUPPORTED;
        case LCB_ECTL_UNSUPPMODE:
            return LCB_NOT_SUPPORTED;
        case LCB_ECTL_BADARG:
            return LCB_EINVAL;
        default:
            return retval;
        }
    } else {
        return retval;
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_cntl(lcb_INSTANCE *instance, int mode, int cmd, void *arg)
{
    ctl_handler handler;
    if (cmd >= (int)CNTL_NUM_HANDLERS || cmd < 0) {
        return wrap_return(instance, LCB_ECTL_UNKNOWN);
    }

    handler = handlers[cmd];

    if (!handler) {
        return wrap_return(instance, LCB_ECTL_UNKNOWN);
    }

    return wrap_return(instance, handler(mode, instance, cmd, arg));
}

LIBCOUCHBASE_API
lcb_STATUS
lcb_cntl_string(lcb_INSTANCE *instance, const char *key, const char *value)
{
    cntl_OPCODESTRS *cur;
    u_STRCONVERT u;
    lcb_STATUS err;

    for (cur = stropcode_map; cur->key; cur++) {
        if (!strcmp(cur->key, key)) {
            if (cur->opcode < 0) {
                return LCB_ECTL_UNKNOWN;
            }
            if (cur->converter) {
                err = cur->converter(value, &u);
                if (err != LCB_SUCCESS) {
                    return err;
                }
                if (cur->converter == convert_passthru) {
                    return lcb_cntl(instance, LCB_CNTL_SET, cur->opcode, u.p);
                } else {
                    return lcb_cntl(instance, LCB_CNTL_SET, cur->opcode, &u);
                }
            }

            return lcb_cntl(instance, CNTL__MODE_SETSTRING, cur->opcode,
                (void *)value);
        }
    }
    return wrap_return(instance, LCB_NOT_SUPPORTED);
}

LIBCOUCHBASE_API
int
lcb_cntl_exists(int ctl)
{
    if (ctl >= (int)CNTL_NUM_HANDLERS || ctl < 0) {
        return 0;
    }
    return handlers[ctl] != NULL;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_cntl_setu32(lcb_INSTANCE *instance, int cmd, lcb_uint32_t arg)
{
    return lcb_cntl(instance, LCB_CNTL_SET, cmd, &arg);
}

LIBCOUCHBASE_API
lcb_uint32_t lcb_cntl_getu32(lcb_INSTANCE *instance, int cmd)
{
    lcb_uint32_t ret = 0;
    lcb_cntl(instance, LCB_CNTL_GET, cmd, &ret);
    return ret;
}
