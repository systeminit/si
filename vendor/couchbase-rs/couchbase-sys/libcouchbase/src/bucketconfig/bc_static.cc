/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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

#include <lcbio/lcbio.h>
#include <lcbio/timer-ng.h>
#include <lcbio/timer-cxx.h>
#include <libcouchbase/vbucket.h>
#include "clconfig.h"

#define LOGARGS(mcr, lvlbase) mcr->parent->settings, "bc_static", LCB_LOG_##lvlbase, __FILE__, __LINE__
#define LOGFMT "(STATIC=%p)> "
#define LOGID(mcr) (void *)mcr

using namespace lcb::clconfig;
using lcb::Hostlist;

// Base class for providers which only generate a config once, statically.
class StaticProvider : public Provider
{
  public:
    StaticProvider(Confmon *parent_, Method m) : Provider(parent_, m), async(parent_->iot, this), config(NULL) {}

    virtual ~StaticProvider()
    {
        if (config) {
            config->decref();
        }
        async.release();
    }

    ConfigInfo *get_cached()
    {
        return config;
    }

    lcb_STATUS refresh()
    {
        async.signal();
        return LCB_SUCCESS;
    }

    void configure_nodes(const Hostlist &hl)
    {
        if (hl.empty()) {
            lcb_log(LOGARGS(this, FATAL), "No nodes provided");
            return;
        }

        lcbvb_CONFIG *vbc = gen_config(hl);
        if (vbc != NULL) {
            if (config != NULL) {
                config->decref();
                config = NULL;
            }
            config = ConfigInfo::create(vbc, type);
        }
    }

    virtual lcbvb_CONFIG *gen_config(const Hostlist &hl) = 0;

  private:
    void async_update()
    {
        if (config != NULL) {
            parent->provider_got_config(this, config);
        }
    }

    lcb::io::Timer< StaticProvider, &StaticProvider::async_update > async;
    ConfigInfo *config;
};

/* Raw memcached provider */

struct McRawProvider : public StaticProvider {
    McRawProvider(Confmon *parent_) : StaticProvider(parent_, CLCONFIG_MCRAW) {}
    lcbvb_CONFIG *gen_config(const lcb::Hostlist &l);
};

lcbvb_CONFIG *McRawProvider::gen_config(const lcb::Hostlist &hl)
{
    std::vector< lcbvb_SERVER > servers;
    servers.reserve(hl.size());

    for (size_t ii = 0; ii < hl.size(); ii++) {
        const lcb_host_t &curhost = hl[ii];
        servers.resize(servers.size() + 1);

        lcbvb_SERVER &srv = servers.back();
        memset(&srv, 0, sizeof srv);

        /* just set the memcached port and hostname */
        srv.hostname = (char *)curhost.host;
        srv.svc.data = std::atoi(curhost.port);
        if (parent->settings->sslopts) {
            srv.svc_ssl.data = srv.svc.data;
        }
    }

    lcbvb_CONFIG *newconfig = lcbvb_create();
    lcbvb_genconfig_ex(newconfig, "NOBUCKET", "deadbeef", &servers[0], servers.size(), 0, 2);
    lcbvb_make_ketama(newconfig);
    newconfig->revid = -1;
    return newconfig;
}

Provider *lcb::clconfig::new_mcraw_provider(Confmon *parent)
{
    return new McRawProvider(parent);
}

struct ClusterAdminProvider : public StaticProvider {
    ClusterAdminProvider(Confmon *parent_) : StaticProvider(parent_, CLCONFIG_CLADMIN) {}

    lcbvb_CONFIG *gen_config(const lcb::Hostlist &hl)
    {
        std::vector< lcbvb_SERVER > servers;
        servers.reserve(hl.size());
        for (size_t ii = 0; ii < hl.size(); ++ii) {
            servers.resize(servers.size() + 1);
            lcbvb_SERVER &srv = servers[ii];
            const lcb_host_t &curhost = hl[ii];
            srv.hostname = const_cast< char * >(curhost.host);
            if (parent->settings->sslopts) {
                srv.svc_ssl.mgmt = std::atoi(curhost.port);
            } else {
                srv.svc.mgmt = std::atoi(curhost.port);
            }
        }
        lcbvb_CONFIG *vbc = lcbvb_create();
        lcbvb_genconfig_ex(vbc, "NOBUCKET", "deadbeef", &servers[0], servers.size(), 0, 0);
        return vbc;
    }
};

Provider *lcb::clconfig::new_cladmin_provider(Confmon *parent)
{
    return new ClusterAdminProvider(parent);
}
