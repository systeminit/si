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
#ifndef LIBCOUCHBASE_INTERNAL_H
#define LIBCOUCHBASE_INTERNAL_H 1

/* System/Standard includes */
#include "config.h"
#include <errno.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <ctype.h>

/* Global Project Dependencies/Includes */
#include <memcached/protocol_binary.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include <libcouchbase/pktfwd.h>
#include <libcouchbase/crypto.h>

/* Internal dependencies */
#include <lcbio/lcbio.h>
#include <strcodecs/strcodecs.h>
#include "mcserver/mcserver.h"
#include "mc/mcreq.h"
#include "settings.h"
#include "contrib/genhash/genhash.h"

#include "internalstructs.h"
#include "collections.h"

/* lcb_INSTANCE-specific includes */
#include "retryq.h"
#include "aspend.h"
#include "bootstrap.h"

/* n1ql cache */
#include "n1ql/n1ql-internal.h"

#include "tracing/tracing-internal.h"

#include "hostlist.h"

#ifdef __cplusplus
namespace lcb
{
class Connspec;
struct Spechost;
class RetryQueue;
class Bootstrap;
namespace clconfig
{
struct Confmon;
class ConfigInfo;
} // namespace clconfig
} // namespace lcb
extern "C" {
#endif

struct lcb_callback_st {
    lcb_RESPCALLBACK v3callbacks[LCB_CALLBACK__MAX];
    lcb_errmap_callback errmap;
    lcb_bootstrap_callback bootstrap;
    lcb_pktfwd_callback pktfwd;
    lcb_pktflushed_callback pktflushed;
    lcb_open_callback open;
};

struct lcb_GUESSVB_st;

#ifdef __cplusplus
#include <string>
typedef std::string *lcb_pSCRATCHBUF;
typedef lcb::RetryQueue lcb_RETRYQ;
typedef lcb::clconfig::Confmon *lcb_pCONFMON;
typedef lcb::clconfig::ConfigInfo *lcb_pCONFIGINFO;
typedef lcb::Bootstrap lcb_BOOTSTRAP;
#else
typedef struct lcb_SCRATCHBUF *lcb_pSCRATCHBUF;
typedef struct lcb_RETRYQ_st lcb_RETRYQ;
typedef struct lcb_CONFMON_st *lcb_pCONFMON;
typedef struct lcb_CONFIGINFO_st *lcb_pCONFIGINFO;
typedef struct lcb_BOOTSTRAP_st lcb_BOOTSTRAP;
#endif

struct lcb_st {
    mc_CMDQUEUE cmdq;                 /**< Base command queue object */
    const void *cookie;               /**< User defined pointer */
    lcb_pCONFMON confmon;             /**< Cluster config manager */
    hostlist_t mc_nodes;              /**< List of current memcached endpoints */
    hostlist_t ht_nodes;              /**< List of current management endpoints */
    lcb_pCONFIGINFO cur_configinfo;   /**< Pointer to current config */
    lcb_BOOTSTRAP *bs_state;          /**< Bootstrapping state */
    struct lcb_callback_st callbacks; /**< Callback table */
    lcb_HISTOGRAM *kv_timings;        /**< Histogram object (for timing) */
    lcb_ASPEND pendops;               /**< Pending asynchronous requests */
    int wait;                         /**< Are we in lcb_wait() ?*/
    lcbio_MGR *memd_sockpool;         /**< Connection pool for memcached connections */
    lcbio_MGR *http_sockpool;         /**< Connection pool for capi connections */
    lcb_STATUS last_error;            /**< Seldom used. Mainly for bootstrap */
    lcb_settings *settings;           /**< User settings */
    lcbio_pTABLE iotable;             /**< IO Routine table */
    lcb_RETRYQ *retryq;               /**< Retry queue for failed operations */
    lcb_pSCRATCHBUF scratch;          /**< Generic buffer space */
    struct lcb_GUESSVB_st *vbguess;   /**< Heuristic masters for vbuckets */
    lcb_N1QLCACHE *n1ql_cache;
    lcb_MUTATION_TOKEN *dcpinfo; /**< Mapping of known vbucket to {uuid,seqno} info */
    lcbio_pTIMER dtor_timer;     /**< Asynchronous destruction timer */
    lcb_BTYPE btype;             /**< Type of the bucket */
    lcb_COLLCACHE *collcache;    /**< Collection cache */

#ifdef __cplusplus
    typedef std::map< std::string, lcbcrypto_PROVIDER * > lcb_ProviderMap;
    lcb_ProviderMap *crypto;
    lcb_settings *getSettings()
    {
        return settings;
    }
    lcbio_pTABLE getIOT()
    {
        return iotable;
    }
    inline void add_bs_host(const char *host, int port, unsigned bstype);
    inline void add_bs_host(const lcb::Spechost &host, int defl_http, int defl_cccp);
    inline lcb_STATUS process_dns_srv(lcb::Connspec &spec);
    inline void populate_nodes(const lcb::Connspec &);
    lcb::Server *get_server(size_t index) const
    {
        return static_cast< lcb::Server * >(cmdq.pipelines[index]);
    }
    lcb::Server *find_server(const lcb_host_t &host) const;
    lcb_STATUS request_config(const void *cookie, lcb::Server *server);
    lcb_STATUS select_bucket(const void *cookie, lcb::Server *server);

    /**
     * @brief Request that the handle update its configuration.
     *
     * This function acts as a gateway to the more abstract confmon interface.
     *
     * @param instance The instance
     * @param options A set of options specified as flags, indicating under what
     * conditions a new configuration should be refetched.
     *
     * This should be a combination of one or more @ref lcb::BootstrapOptions
     *
     * Note, the definition for this function (and the flags)
     * are found in bootstrap.cc
     */
    inline lcb_STATUS bootstrap(unsigned options)
    {
        if (!bs_state) {
            bs_state = new lcb::Bootstrap(this);
        }
        return bs_state->bootstrap(options);
    }

    lcbvb_CONFIG *getConfig() const
    {
        return cur_configinfo->vbc;
    }

    int map_key(const std::string &key)
    {
        int srvix, tmpvb;
        lcbvb_map_key(getConfig(), key.c_str(), key.size(), &tmpvb, &srvix);
        return srvix;
    }

    const char *get_bucketname() const
    {
        return settings->bucket;
    }

#endif
};

#define LCBT_VBCONFIG(instance) (instance)->cmdq.config
#define LCBT_NSERVERS(instance) (instance)->cmdq.npipelines
#define LCBT_NDATASERVERS(instance) LCBVB_NDATASERVERS(LCBT_VBCONFIG(instance))
#define LCBT_NREPLICAS(instance) LCBVB_NREPLICAS(LCBT_VBCONFIG(instance))
#define LCBT_GET_SERVER(instance, ix) (instance)->cmdq.pipelines[ix]
#define LCBT_SETTING(instance, name) (instance)->settings->name
#define LCBT_SETTING_SVCMODE(instance)                                                                                 \
    (((instance)->settings->sslopts & LCB_SSL_ENABLED) ? LCBVB_SVCMODE_SSL : LCBVB_SVCMODE_PLAIN)
#define LCBT_SUPPORT_SYNCREPLICATION(instance) LCBT_SETTING(instance, enable_durable_write)

void lcb_initialize_packet_handlers(lcb_INSTANCE *instance);

LCB_INTERNAL_API
void lcb_maybe_breakout(lcb_INSTANCE *instance);

void lcb_update_vbconfig(lcb_INSTANCE *instance, lcb_pCONFIGINFO config);
/**
 * Hashtable wrappers
 */
genhash_t *lcb_hashtable_nc_new(lcb_size_t est);
genhash_t *lcb_hashtable_szt_new(lcb_size_t est);

lcb_STATUS lcb_iops_cntl_handler(int mode, lcb_INSTANCE *instance, int cmd, void *arg);

/**
 * These two routines define portable ways to get environment variables
 * on various platforms.
 *
 * They are mainly useful for Windows compatibility.
 */
LCB_INTERNAL_API
int lcb_getenv_nonempty(const char *key, char *buf, lcb_size_t len);
LCB_INTERNAL_API
int lcb_getenv_boolean(const char *key);
LCB_INTERNAL_API
int lcb_getenv_nonempty_multi(char *buf, lcb_size_t nbuf, ...);
int lcb_getenv_boolean_multi(const char *key, ...);
LCB_INTERNAL_API
const char *lcb_get_tmpdir(void);

/**
 * Initialize the socket subsystem. For windows, this initializes Winsock.
 * On Unix, this does nothing
 */
LCB_INTERNAL_API
lcb_STATUS lcb_initialize_socket_subsystem(void);

lcb_STATUS lcb_init_providers2(lcb_INSTANCE *obj, const struct lcb_create_st2 *e_options);
lcb_STATUS lcb_reinit3(lcb_INSTANCE *obj, const char *connstr);

int lcb_should_retry(const lcb_settings *settings, const mc_PACKET *pkt, lcb_STATUS err);

lcb_RESPCALLBACK lcb_find_callback(lcb_INSTANCE *instance, lcb_CALLBACK_TYPE cbtype);

/* These two functions exist to allow the tests to keep the loop alive while
 * scheduling other operations asynchronously */

LCB_INTERNAL_API void lcb_loop_ref(lcb_INSTANCE *instance);
LCB_INTERNAL_API void lcb_loop_unref(lcb_INSTANCE *instance);

#define MAYBE_SCHEDLEAVE(o)                                                                                            \
    if (!o->cmdq.ctxenter) {                                                                                           \
        lcb_sched_leave(o);                                                                                            \
    }

#define LCB_SCHED_ADD(instance, pl, pkt)                                                                               \
    mcreq_sched_add(pl, pkt);                                                                                          \
    MAYBE_SCHEDLEAVE(instance)

void lcb_vbguess_newconfig(lcb_INSTANCE *instance, lcbvb_CONFIG *cfg, struct lcb_GUESSVB_st *guesses);
int lcb_vbguess_remap(lcb_INSTANCE *instance, int vbid, int bad);
#define lcb_vbguess_destroy(p) free(p)

LCB_INTERNAL_API uint32_t lcb_durability_timeout(lcb_INSTANCE *instance);

#ifdef __cplusplus
}
#endif

#endif
