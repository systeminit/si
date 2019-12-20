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

#include "internal.h"
#include <lcbio/lcbio.h>
#include "bucketconfig/clconfig.h"
#include "hostlist.h"

/**This file contains routines to assist users in retrieving valid nodes */

/* We're gonna try to be extra careful in this function since many SDKs use
 * this to display node and/or host-port information.*/

static std::string&
ensure_scratch(lcb_INSTANCE *instance)
{
    if (!instance->scratch) {
        instance->scratch = new std::string;
    }
    instance->scratch->clear();
    return *instance->scratch;
}

static const char *
mk_scratch_host(lcb_INSTANCE *instance, const lcb_host_t *host)
{
    std::string& s = ensure_scratch(instance);
    s.append(host->host);
    s.append(":");
    s.append(host->port);
    return s.c_str();
}

static const char *
return_badhost(lcb_GETNODETYPE type)
{
    if (type & LCB_NODE_NEVERNULL) {
        return LCB_GETNODE_UNAVAILABLE;
    } else {
        return NULL;
    }
}

LIBCOUCHBASE_API
const char *
lcb_get_node(lcb_INSTANCE *instance, lcb_GETNODETYPE type, unsigned ix)
{
    lcbvb_SVCMODE mode = LCBT_SETTING_SVCMODE(instance);
    lcbvb_CONFIG *vbc = LCBT_VBCONFIG(instance);

    if (type & LCB_NODE_HTCONFIG) {
        if (type & LCB_NODE_CONNECTED) {
            const lcb_host_t *host = lcb::clconfig::http_get_host(instance->confmon);
            if (host) {
                return mk_scratch_host(instance, host);
            } else {
                return return_badhost(type);
            }

        } else {
            /* Retrieve one from the vbucket configuration */
            const char *hp = NULL;

            if (instance->settings->conntype == LCB_TYPE_BUCKET) {
                if (vbc) {
                    ix %= LCBVB_NSERVERS(vbc);
                    hp = lcbvb_get_hostport(vbc, ix, LCBVB_SVCTYPE_MGMT, mode);

                } else if ((type & LCB_NODE_NEVERNULL) == 0) {
                    return NULL;
                }
            }
            if (hp == NULL && instance->ht_nodes && !instance->ht_nodes->empty()) {
                ix %= instance->ht_nodes->size();
                instance->ht_nodes->ensure_strlist();
                hp = instance->ht_nodes->hoststrs[ix];
            }
            if (!hp) {
                return return_badhost(type);
            }
            return ensure_scratch(instance).append(hp).c_str();
        }
    } else if (type & (LCB_NODE_DATA|LCB_NODE_VIEWS)) {
        ix %= LCBT_NSERVERS(instance);
        const lcb::Server *server = instance->get_server(ix);

        if ((type & LCB_NODE_CONNECTED) && !server->is_connected()) {
            return return_badhost(type);
        }

        /* otherwise, return the actual host:port of the server */
        if (type & LCB_NODE_DATA) {
            return mk_scratch_host(instance, &server->get_host());
        } else {
            return lcbvb_get_hostport(vbc, ix, LCBVB_SVCTYPE_VIEWS, mode);
        }
    } else {
        return NULL; /* don't know the type */
    }
}

LIBCOUCHBASE_API
lcb_int32_t lcb_get_num_replicas(lcb_INSTANCE *instance)
{
    if (LCBT_VBCONFIG(instance)) {
        return LCBT_NREPLICAS(instance);
    } else {
        return -1;
    }
}

LIBCOUCHBASE_API
lcb_int32_t lcb_get_num_nodes(lcb_INSTANCE *instance)
{
    if (LCBT_VBCONFIG(instance)) {
        return LCBT_NSERVERS(instance);
    } else {
        return -1;
    }
}

LIBCOUCHBASE_API
const char *const *lcb_get_server_list(lcb_INSTANCE *instance)
{
    return instance->ht_nodes->get_strlist();
}

LIBCOUCHBASE_API
const char *
lcb_get_keynode(lcb_INSTANCE *instance, const void *key, size_t nkey)
{
    lcbvb_CONFIG *vbc = LCBT_VBCONFIG(instance);
    int srvix, vbid;

    if (!vbc) {
        return NULL;
    }

    lcbvb_map_key(vbc, key, nkey, &vbid, &srvix);
    if (srvix < 0) {
        return NULL;
    }

    return lcbvb_get_hostname(vbc, srvix);
}
