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

#include "internal.h"
#include "clconfig.h"
#include <list>
#include <algorithm>
#include "trace.h"

#define LOGARGS(mon, lvlbase) mon->settings, "confmon", LCB_LOG_##lvlbase, __FILE__, __LINE__
#define LOG(mon, lvlbase, msg) lcb_log(mon->settings, "confmon", LCB_LOG_##lvlbase, __FILE__, __LINE__, msg)

using namespace lcb::clconfig;

Provider *Confmon::next_active(Provider *cur)
{
    ProviderList::iterator ii = std::find(active_providers.begin(), active_providers.end(), cur);

    if (ii == active_providers.end() || (++ii) == active_providers.end()) {
        return NULL;
    } else {
        return *ii;
    }
}

Provider *Confmon::first_active()
{
    if (active_providers.empty()) {
        return NULL;
    } else {
        return active_providers.front();
    }
}

static const char *provider_string(Method type)
{
    if (type == CLCONFIG_HTTP) {
        return "HTTP";
    }
    if (type == CLCONFIG_CCCP) {
        return "CCCP";
    }
    if (type == CLCONFIG_FILE) {
        return "FILE";
    }
    if (type == CLCONFIG_MCRAW) {
        return "MCRAW";
    }
    if (type == CLCONFIG_CLADMIN) {
        return "CLADMIN";
    }
    return "";
}

Confmon::Confmon(lcb_settings *settings_, lcbio_pTABLE iot_, lcb_INSTANCE *instance_)
    : cur_provider(NULL), config(NULL), settings(settings_), last_error(LCB_SUCCESS), iot(iot_), as_start(iot_, this),
      as_stop(iot_, this), state(0), last_stop_us(0), instance(instance_), active_provider_list_id(0)
{

    lcbio_table_ref(iot);
    lcb_settings_ref(settings);

    all_providers[CLCONFIG_FILE] = new_file_provider(this);
    all_providers[CLCONFIG_CCCP] = new_cccp_provider(this);
    all_providers[CLCONFIG_HTTP] = new_http_provider(this);
    all_providers[CLCONFIG_MCRAW] = new_mcraw_provider(this);
    all_providers[CLCONFIG_CLADMIN] = new_cladmin_provider(this);

    for (size_t ii = 0; ii < CLCONFIG_MAX; ii++) {
        all_providers[ii]->parent = this;
    }
}

void Confmon::prepare()
{
    ++this->active_provider_list_id;
    active_providers.clear();
    lcb_log(LOGARGS(this, DEBUG), "Preparing providers (this may be called multiple times)");

    for (size_t ii = 0; ii < CLCONFIG_MAX; ii++) {
        Provider *cur = all_providers[ii];
        if (cur) {
            if (cur->enabled) {
                active_providers.push_back(cur);
                lcb_log(LOGARGS(this, DEBUG), "Provider %s is ENABLED", provider_string(cur->type));
            } else if (cur->pause()) {
                lcb_log(LOGARGS(this, DEBUG), "Provider %s is DISABLED", provider_string(cur->type));
            }
        }
    }

    lcb_assert(!active_providers.empty());
    cur_provider = first_active();
}

Confmon::~Confmon()
{
    as_start.release();
    as_stop.release();

    if (config) {
        config->decref();
        config = NULL;
    }

    for (size_t ii = 0; ii < CLCONFIG_MAX; ii++) {
        Provider *provider = all_providers[ii];
        if (provider == NULL) {
            continue;
        }
        delete provider;
        all_providers[ii] = NULL;
    }

    lcbio_table_unref(iot);
    lcb_settings_unref(settings);
}

int Confmon::do_set_next(ConfigInfo *new_config, bool notify_miss)
{
    unsigned ii;

    if (config && new_config == config) {
        return 0;
    }
    if (config) {
        lcbvb_CHANGETYPE chstatus = LCBVB_NO_CHANGES;
        lcbvb_CONFIGDIFF *diff = lcbvb_compare(config->vbc, new_config->vbc);

        if (!diff) {
            lcb_log(LOGARGS(this, DEBUG), "Couldn't create vbucket diff");
            return 0;
        }

        chstatus = lcbvb_get_changetype(diff);
        lcbvb_free_diff(diff);

        if (chstatus == 0 || config->compare(*new_config) >= 0) {
            const lcbvb_CONFIG *ca, *cb;

            ca = config->vbc;
            cb = new_config->vbc;

            lcb_log(LOGARGS(this, TRACE),
                    "Not applying configuration received via %s. No changes detected. A.rev=%d, B.rev=%d",
                    provider_string(new_config->get_origin()), ca->revid, cb->revid);
            if (notify_miss) {
                invoke_listeners(CLCONFIG_EVENT_GOT_ANY_CONFIG, new_config);
            }
            return 0;
        }
    }

    lcb_log(LOGARGS(this, INFO), "Setting new configuration. Received via %s",
            provider_string(new_config->get_origin()));
    TRACE_NEW_CONFIG(instance, new_config);

    if (config) {
        /** DECREF the old one */
        config->decref();
    }

    for (ii = 0; ii < CLCONFIG_MAX; ii++) {
        Provider *cur = all_providers[ii];
        if (cur && cur->enabled) {
            cur->config_updated(new_config->vbc);
        }
    }

    new_config->incref();
    config = new_config;
    stop();

    invoke_listeners(CLCONFIG_EVENT_GOT_NEW_CONFIG, new_config);

    return 1;
}

void Confmon::provider_failed(Provider *provider, lcb_STATUS reason)
{
    lcb_log(LOGARGS(this, INFO), "Provider '%s' failed", provider_string(provider->type));

    if (provider != cur_provider) {
        lcb_log(LOGARGS(this, TRACE), "Ignoring failure. Current=%p (%s)", (void *)cur_provider,
                provider_string(cur_provider->type));
        return;
    }
    if (!is_refreshing()) {
        lcb_log(LOGARGS(this, DEBUG), "Ignoring failure. Refresh not active");
    }

    if (reason != LCB_SUCCESS) {
        if (settings->detailed_neterr && last_error != LCB_SUCCESS) {
            /* Filter out any artificial 'connect error' or 'network error' codes */
            if (reason != LCB_CONNECT_ERROR && reason != LCB_NETWORK_ERROR) {
                last_error = reason;
            }
        } else {
            last_error = reason;
        }
        if (reason == LCB_AUTH_ERROR) {
            goto GT_ERROR;
        }
    }

    if (settings->conntype == LCB_TYPE_CLUSTER && provider->type == CLCONFIG_HTTP
                                                  && LCBT_SETTING(instance, allow_static_config)) {
        Provider *cladmin = get_provider(CLCONFIG_CLADMIN);
        if (!cladmin->enabled) {
            cladmin->enable();
            cladmin->configure_nodes(*provider->get_nodes());
            active_providers.push_back(cladmin);
            lcb_log(LOGARGS(this, DEBUG), "Static configuration provider has been enabled");
        }
    }

    cur_provider = next_active(cur_provider);
    if (cur_provider) {
        uint32_t interval = 0;
        if (config) {
            /* Not first */
            interval = settings->grace_next_provider;
        }
        lcb_log(LOGARGS(this, DEBUG), "Will try next provider in %uus", interval);
        state |= CONFMON_S_ITERGRACE;
        as_start.rearm(interval);
        return;
    } else {
        LOG(this, TRACE, "Maximum provider reached. Resetting index");
    }

GT_ERROR:
    invoke_listeners(CLCONFIG_EVENT_PROVIDERS_CYCLED, NULL);
    cur_provider = first_active();
    stop();
}

void Confmon::provider_got_config(Provider *, ConfigInfo *config_)
{
    do_set_next(config_, true);
    stop();
}

void Confmon::do_next_provider()
{
    state &= ~CONFMON_S_ITERGRACE;
    size_t previous_active_provider_list_id = this->active_provider_list_id;
    ProviderList::const_iterator ii = active_providers.begin();
    while (ii != active_providers.end()) {
        if (previous_active_provider_list_id != this->active_provider_list_id) {
            ii = active_providers.begin();
            previous_active_provider_list_id = this->active_provider_list_id;
        }

        Provider *cached_provider = *ii;
        ++ii;
        if (!cached_provider) {
            continue;
        }
        ConfigInfo *info = cached_provider->get_cached();
        if (!info) {
            continue;
        }

        if (do_set_next(info, false)) {
            LOG(this, DEBUG, "Using cached configuration");
        }
    }

    lcb_log(LOGARGS(this, TRACE), "Attempting to retrieve cluster map via %s", provider_string(cur_provider->type));

    cur_provider->refresh();
}

void Confmon::start(bool refresh)
{
    lcb_U32 tmonext = 0;
    as_stop.cancel();
    if (is_refreshing()) {
        LOG(this, DEBUG, "Cluster map refresh already in progress");
        return;
    }

    LOG(this, TRACE, "Refreshing current cluster map");
    lcb_assert(cur_provider);
    state = CONFMON_S_ACTIVE | CONFMON_S_ITERGRACE;

    if (last_stop_us > 0) {
        lcb_U32 diff = LCB_NS2US(gethrtime()) - last_stop_us;
        if (diff <= settings->grace_next_cycle) {
            tmonext = settings->grace_next_cycle - diff;
        }
    }

    if (refresh) {
        cur_provider->refresh();
    }
    as_start.rearm(tmonext);
}

void Confmon::stop_real()
{
    ProviderList::const_iterator ii;
    for (ii = active_providers.begin(); ii != active_providers.end(); ++ii) {
        (*ii)->pause();
    }

    last_stop_us = LCB_NS2US(gethrtime());
    invoke_listeners(CLCONFIG_EVENT_MONITOR_STOPPED, NULL);
}

void Confmon::stop()
{
    if (!is_refreshing()) {
        return;
    }
    as_start.cancel();
    as_stop.cancel();
    state = CONFMON_S_INACTIVE;
}

Provider::Provider(Confmon *parent_, Method type_) : type(type_), enabled(false), parent(parent_) {}

Provider::~Provider()
{
    parent = NULL;
}

ConfigInfo::~ConfigInfo()
{
    if (vbc) {
        lcbvb_destroy(vbc);
    }
}

int ConfigInfo::compare(const ConfigInfo &other)
{
    /** First check if new config has bucket name */
    if (vbc->bname == NULL && other.vbc->bname != NULL) {
        return -1; /* we want to upgrade config after opening bucket */
    }
    /** Then check if both have revisions */
    int rev_a, rev_b;
    rev_a = lcbvb_get_revision(this->vbc);
    rev_b = lcbvb_get_revision(other.vbc);
    if (rev_a >= 0 && rev_b >= 0) {
        return rev_a - rev_b;
    }

    if (this->cmpclock == other.cmpclock) {
        return 0;

    } else if (this->cmpclock < other.cmpclock) {
        return -1;
    }

    return 1;
}

ConfigInfo::ConfigInfo(lcbvb_CONFIG *config_, Method origin_)
    : vbc(config_), cmpclock(gethrtime()), refcount(1), origin(origin_)
{
}

void Confmon::add_listener(Listener *lsn)
{
    listeners.push_back(lsn);
}

void Confmon::remove_listener(Listener *lsn)
{
    listeners.remove(lsn);
}

void Confmon::invoke_listeners(EventType event, ConfigInfo *info)
{
    ListenerList::iterator ii = listeners.begin();
    while (ii != listeners.end()) {
        ListenerList::iterator cur = ii++;
        (*cur)->clconfig_lsn(event, info);
    }
}

void Confmon::set_active(Method type, bool enabled)
{
    Provider *provider = all_providers[type];
    if (provider->enabled == enabled) {
        return;
    } else {
        provider->enabled = enabled;
    }
    prepare();
}

void Confmon::dump(FILE *fp)
{
    fprintf(fp, "CONFMON=%p\n", (void *)this);
    fprintf(fp, "STATE= (0x%x)", state);
    if (state & CONFMON_S_ACTIVE) {
        fprintf(fp, "ACTIVE|");
    }
    if (state == CONFMON_S_INACTIVE) {
        fprintf(fp, "INACTIVE/IDLE");
    }
    if (state & CONFMON_S_ITERGRACE) {
        fprintf(fp, "ITERGRACE");
    }
    fprintf(fp, "\n");
    fprintf(fp, "LAST ERROR: 0x%x\n", last_error);

    for (size_t ii = 0; ii < CLCONFIG_MAX; ii++) {
        Provider *cur = all_providers[ii];
        if (!cur) {
            continue;
        }

        fprintf(fp, "** PROVIDER: 0x%x (%s) %p\n", cur->type, provider_string(cur->type), (void *)cur);
        fprintf(fp, "** ENABLED: %s\n", cur->enabled ? "YES" : "NO");
        fprintf(fp, "** CURRENT: %s\n", cur == cur_provider ? "YES" : "NO");
        cur->dump(fp);
        fprintf(fp, "\n");
    }
}
