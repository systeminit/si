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

#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include "config.h"
#include "contrib/cJSON/cJSON.h"
#include "json-inl.h"
#include "hash.h"
#include "crc32.h"

#define STRINGIFY_(X) #X
#define STRINGIFY(X) STRINGIFY_(X)
#define MAX_AUTHORITY_SIZE 100
#define SET_ERRSTR(cfg, s)                                                                                             \
    if (!(cfg)->errstr) {                                                                                              \
        (cfg)->errstr = __FILE__ ":" STRINGIFY(__LINE__) " " s;                                                        \
    }

/******************************************************************************
 ******************************************************************************
 ** Core Parsing Routines                                                    **
 ******************************************************************************
 ******************************************************************************/
static lcbvb_VBUCKET *build_vbmap(lcbvb_CONFIG *cfg, cJSON *cj, unsigned *nitems)
{
    lcbvb_VBUCKET *vblist = NULL;
    cJSON *jvb;
    unsigned ii, nalloc;

    /** FIXME: Realloc dynamically when too small */
    if (!(nalloc = cJSON_GetArraySize(cj))) {
        goto GT_ERR;
    }

    if (!(vblist = calloc(nalloc, sizeof(*vblist)))) {
        goto GT_ERR;
    }

    /* Iterate over all the vbuckets */
    jvb = cj->child;
    for (ii = 0; ii < nalloc && jvb; ++ii, jvb = jvb->next) {
        cJSON *jsix;
        lcbvb_VBUCKET *cvb;
        unsigned jj, nservers;

        if (jvb->type != cJSON_Array) {
            goto GT_ERR;
        }

        nservers = cJSON_GetArraySize(jvb);
        jsix = jvb->child;
        cvb = vblist + ii;

        /* Iterate over each index in the vbucket */
        for (jj = 0; jj < nservers && jsix; ++jj, jsix = jsix->next) {
            if (jsix->type != cJSON_Number) {
                goto GT_ERR;
            }
            cvb->servers[jj] = jsix->valueint;
            if (cvb->servers[jj] > (int)cfg->nsrv - 1) {
                SET_ERRSTR(cfg, "Invalid vBucket map received from server. Above-bounds vBucket target found");
                goto GT_ERR;
            }
        }
    }

    *nitems = nalloc;
    return vblist;

GT_ERR:
    free(vblist);
    return NULL;
}

static void copy_address(char *buf, size_t nbuf, const char *host, lcb_U16 port)
{
    if (strchr(host, ':')) {
        // IPv6 and should be bracketed
        snprintf(buf, nbuf, "[%s]:%d", host, port);
    } else {
        snprintf(buf, nbuf, "%s:%d", host, port);
    }
}

static lcbvb_SERVER *find_server_memd(lcbvb_SERVER *servers, unsigned n, const char *s)
{
    unsigned ii;
    for (ii = 0; ii < n; ii++) {
        char buf[4096] = {0};
        lcbvb_SERVER *cur = servers + ii;
        copy_address(buf, sizeof(buf), cur->hostname, cur->svc.data);
        if (!strncmp(s, buf, sizeof(buf))) {
            return cur;
        }
    }
    return NULL;
}

static int assign_dumy_server(lcbvb_CONFIG *cfg, lcbvb_SERVER *dst, const char *s)
{
    int itmp;
    char *colon;
    if (!(dst->authority = strdup(s))) {
        SET_ERRSTR(cfg, "Couldn't allocate authority string");
        goto GT_ERR;
    }

    if (!(colon = strstr(s, ":"))) {
        SET_ERRSTR(cfg, "Badly formatted name string");
        goto GT_ERR;
    }

    if (sscanf(colon + 1, "%d", &itmp) != 1) {
        SET_ERRSTR(cfg, "Badly formatted port");
        goto GT_ERR;
    }

    dst->svc.data = itmp;
    return 1;

GT_ERR:
    free(dst->authority);
    return 0;
}

static void set_vb_count(lcbvb_CONFIG *cfg, lcbvb_VBUCKET *vbs)
{
    unsigned ii, jj;
    if (!vbs) {
        return;
    }

    for (ii = 0; ii < cfg->nvb; ++ii) {
        for (jj = 0; jj < cfg->nrepl + 1; ++jj) {
            int ix = vbs[ii].servers[jj];
            if (ix < 0 || (unsigned)ix > cfg->nsrv) {
                continue;
            }
            cfg->servers[ix].nvbs++;
        }
    }
}

static int pair_server_list(lcbvb_CONFIG *cfg, cJSON *vbconfig)
{
    cJSON *servers;
    lcbvb_SERVER *newlist = NULL;
    unsigned ii, nsrv;

    if (!get_jarray(vbconfig, "serverList", &servers)) {
        SET_ERRSTR(cfg, "Couldn't find serverList");
        goto GT_ERROR;
    }

    nsrv = cJSON_GetArraySize(servers);

    if (nsrv > cfg->nsrv) {
        /* nodes in serverList which are not in nodes/nodesExt */
        void *tmp = realloc(cfg->servers, sizeof(*cfg->servers) * nsrv);
        if (!tmp) {
            SET_ERRSTR(cfg, "Couldn't allocate memory for server list");
            goto GT_ERROR;
        }
        cfg->servers = tmp;
        cfg->nsrv = nsrv;
    }

    /* allocate an array for the reordered server list */
    newlist = calloc(nsrv, sizeof(*cfg->servers));

    for (ii = 0; ii < nsrv; ii++) {
        char *tmp;
        cJSON *jst;
        lcbvb_SERVER *cur;
        jst = cJSON_GetArrayItem(servers, ii);
        tmp = jst->valuestring;
        cur = find_server_memd(cfg->servers, cfg->nsrv, tmp);

        if (cur) {
            newlist[ii] = *cur;
        } else {
            /* found server inside serverList but not in nodes? */
            if (!assign_dumy_server(cfg, &newlist[ii], tmp)) {
                goto GT_ERROR;
            }
        }
    }

    free(cfg->servers);
    cfg->servers = newlist;
    return 1;

GT_ERROR:
    free(newlist);
    return 0;
}

static int parse_vbucket(lcbvb_CONFIG *cfg, cJSON *cj)
{
    cJSON *vbconfig, *vbmap, *ffmap = NULL;

    if (!get_jobj(cj, "vBucketServerMap", &vbconfig)) {
        SET_ERRSTR(cfg, "Expected top-level 'vBucketServerMap'");
        goto GT_ERROR;
    }

    if (!get_juint(vbconfig, "numReplicas", &cfg->nrepl)) {
        SET_ERRSTR(cfg, "'numReplicas' missing");
        goto GT_ERROR;
    }

    if (!get_jarray(vbconfig, "vBucketMap", &vbmap)) {
        SET_ERRSTR(cfg, "Missing 'vBucketMap'");
        goto GT_ERROR;
    }

    get_jarray(vbconfig, "vBucketMapForward", &ffmap);

    if ((cfg->vbuckets = build_vbmap(cfg, vbmap, &cfg->nvb)) == NULL) {
        goto GT_ERROR;
    }

    if (ffmap && (cfg->ffvbuckets = build_vbmap(cfg, ffmap, &cfg->nvb)) == NULL) {
        goto GT_ERROR;
    }

    if (!cfg->is3x) {
        if (!pair_server_list(cfg, vbconfig)) {
            goto GT_ERROR;
        }
    }

    /** Now figure out which server goes where */
    set_vb_count(cfg, cfg->vbuckets);
    set_vb_count(cfg, cfg->ffvbuckets);
    return 1;

GT_ERROR:
    return 0;
}

static int server_cmp(const void *s1, const void *s2)
{
    return strcmp(((const lcbvb_SERVER *)s1)->authority, ((const lcbvb_SERVER *)s2)->authority);
}

static int continuum_item_cmp(const void *t1, const void *t2)
{
    const lcbvb_CONTINUUM *ct1 = t1, *ct2 = t2;

    if (ct1->point == ct2->point) {
        return 0;
    } else if (ct1->point > ct2->point) {
        return 1;
    } else {
        return -1;
    }
}

static int update_ketama(lcbvb_CONFIG *cfg)
{
    char host[MAX_AUTHORITY_SIZE + 10] = "";
    int nhost;
    unsigned pp, hh, ss, nn;
    unsigned char digest[16];
    lcbvb_CONTINUUM *new_continuum, *old_continuum;

    qsort(cfg->servers, cfg->ndatasrv, sizeof(*cfg->servers), server_cmp);

    new_continuum = calloc(160 * cfg->ndatasrv, sizeof(*new_continuum));
    /* 40 hashes, 4 numbers per hash = 160 points per server */
    for (ss = 0, pp = 0; ss < cfg->ndatasrv; ++ss) {
        /* we can add more points to server which have more memory */
        for (hh = 0; hh < 40; ++hh) {
            lcbvb_SERVER *srv = cfg->servers + ss;
            nhost = snprintf(host, MAX_AUTHORITY_SIZE + 10, "%s-%u", srv->authority, hh);
            vb__hash_md5(host, nhost, digest);
            for (nn = 0; nn < 4; ++nn, ++pp) {
                new_continuum[pp].index = ss;
                new_continuum[pp].point = ((uint32_t)(digest[3 + nn * 4] & 0xFF) << 24) |
                                          ((uint32_t)(digest[2 + nn * 4] & 0xFF) << 16) |
                                          ((uint32_t)(digest[1 + nn * 4] & 0xFF) << 8) | (digest[0 + nn * 4] & 0xFF);
            }
        }
    }

    qsort(new_continuum, pp, sizeof *new_continuum, continuum_item_cmp);
    old_continuum = cfg->continuum;
    cfg->continuum = new_continuum;
    cfg->ncontinuum = pp;
    free(old_continuum);
    return 1;
}

static int extract_services(lcbvb_CONFIG *cfg, cJSON *jsvc, lcbvb_SERVICES *svc, int is_ssl)
{
    int itmp;
    int rv;
    const char *key;

#define EXTRACT_SERVICE(k, fld)                                                                                        \
    key = is_ssl ? k "SSL" : k;                                                                                        \
    rv = get_jint(jsvc, key, &itmp);                                                                                   \
    if (rv) {                                                                                                          \
        svc->fld = itmp;                                                                                               \
    } else {                                                                                                           \
        svc->fld = 0;                                                                                                  \
    }

    EXTRACT_SERVICE("kv", data);
    EXTRACT_SERVICE("mgmt", mgmt);
    EXTRACT_SERVICE("capi", views);
    EXTRACT_SERVICE("n1ql", n1ql);
    EXTRACT_SERVICE("fts", fts);
    EXTRACT_SERVICE("indexAdmin", ixadmin);
    EXTRACT_SERVICE("indexScan", ixquery);
    EXTRACT_SERVICE("cbas", cbas);

#undef EXTRACT_SERVICE

    (void)cfg;
    return 1;
}

static int build_server_strings(lcbvb_CONFIG *cfg, lcbvb_SERVER *server)
{
    /* get the authority */
    char tmpbuf[4096];

    copy_address(tmpbuf, sizeof(tmpbuf), server->hostname, server->svc.data);
    server->authority = strdup(tmpbuf);
    if (!server->authority) {
        SET_ERRSTR(cfg, "Couldn't allocate authority");
        return 0;
    }

    server->svc.hoststrs[LCBVB_SVCTYPE_DATA] = strdup(server->authority);
    if (server->viewpath == NULL && server->svc.views && cfg->bname) {
        server->viewpath = malloc(strlen(cfg->bname) + 2);
        sprintf(server->viewpath, "/%s", cfg->bname);
    }
    if (server->querypath == NULL && server->svc.n1ql) {
        server->querypath = strdup("/query/service");
    }
    if (server->ftspath == NULL && server->svc.fts) {
        server->ftspath = strdup("/");
    }
    if (server->cbaspath == NULL && server->svc.cbas) {
        server->cbaspath = strdup("/query/service");
    }
    return 1;
}

/**
 * Parse a node from the 'nodesExt' array
 * @param cfg
 * @param server
 * @param js
 * @return
 */
static int build_server_3x(lcbvb_CONFIG *cfg, lcbvb_SERVER *server, cJSON *js, char **network)
{
    cJSON *jsvcs;
    char *htmp;

    if (!get_jstr(js, "hostname", &htmp)) {
        htmp = "$HOST";
    }
    if (!(server->hostname = strdup(htmp))) {
        SET_ERRSTR(cfg, "Couldn't allocate memory");
        goto GT_ERR;
    }

    if (!get_jobj(js, "services", &jsvcs)) {
        SET_ERRSTR(cfg, "Couldn't find 'services'");
        goto GT_ERR;
    }

    if (!extract_services(cfg, jsvcs, &server->svc, 0)) {
        goto GT_ERR;
    }
    if (!extract_services(cfg, jsvcs, &server->svc_ssl, 1)) {
        goto GT_ERR;
    }

    if (!build_server_strings(cfg, server)) {
        goto GT_ERR;
    }

    if (network && *network && strcmp(*network, "default") != 0) {
        cJSON *jaltaddr = cJSON_GetObjectItem(js, "alternateAddresses");
        if (jaltaddr && jaltaddr->type == cJSON_Object) {
            cJSON *jnetwork = cJSON_GetObjectItem(jaltaddr, *network);
            if (jnetwork && get_jstr(jnetwork, "hostname", &htmp)) {
                cJSON *jports;
                server->alt_hostname = strdup(htmp);
                jports = cJSON_GetObjectItem(jnetwork, "ports");
                if (jports && jports->type == cJSON_Object) {
                    extract_services(cfg, jports, &server->alt_svc, 0);
                    extract_services(cfg, jports, &server->alt_svc_ssl, 1);
                }

#define COPY_SERVICE(src, dst)                                                                                         \
    if ((dst)->data == 0)                                                                                              \
        (dst)->data = (src)->data;                                                                                     \
    if ((dst)->mgmt == 0)                                                                                              \
        (dst)->mgmt = (src)->mgmt;                                                                                     \
    if ((dst)->views == 0)                                                                                             \
        (dst)->views = (src)->views;                                                                                   \
    if ((dst)->n1ql == 0)                                                                                              \
        (dst)->n1ql = (src)->n1ql;                                                                                     \
    if ((dst)->fts == 0)                                                                                               \
        (dst)->fts = (src)->fts;                                                                                       \
    if ((dst)->ixadmin == 0)                                                                                           \
        (dst)->ixadmin = (src)->ixadmin;                                                                               \
    if ((dst)->ixquery == 0)                                                                                           \
        (dst)->ixquery = (src)->ixquery;                                                                               \
    if ((dst)->cbas == 0)                                                                                              \
        (dst)->cbas = (src)->cbas;

                COPY_SERVICE(&server->svc, &server->alt_svc);
                COPY_SERVICE(&server->svc_ssl, &server->alt_svc_ssl);

#undef COPY_SERVICE
            }
        }
    }

    return 1;

GT_ERR:
    return 0;
}

/**
 * Initialize a server from a JSON Object
 * @param server The server to initialize
 * @param js The object which contains the server information
 * @return nonzero on success, 0 on failure.
 */
static int build_server_2x(lcbvb_CONFIG *cfg, lcbvb_SERVER *server, cJSON *js)
{
    char *tmp = NULL, *colon;
    int itmp;
    cJSON *jsports;

    if (!get_jstr(js, "hostname", &tmp)) {
        SET_ERRSTR(cfg, "Couldn't find hostname");
        goto GT_ERR;
    }

    /** Hostname is the _rest_ API host, e.g. '8091' */
    if ((server->hostname = strdup(tmp)) == NULL) {
        SET_ERRSTR(cfg, "Couldn't allocate hostname");
        goto GT_ERR;
    }

    colon = strchr(server->hostname, ':');
    if (!colon) {
        SET_ERRSTR(cfg, "Expected ':' in 'hostname'");
        goto GT_ERR;
    }
    if (sscanf(colon + 1, "%d", &itmp) != 1) {
        SET_ERRSTR(cfg, "Expected port after ':'");
        goto GT_ERR;
    }

    /* plain mgmt port is extracted from hostname */
    server->svc.mgmt = itmp;
    *colon = '\0';

    /** Handle the views name */
    if (get_jstr(js, "couchApiBase", &tmp)) {
        /** Have views */
        char *path_begin;
        colon = strrchr(tmp, ':');

        if (!colon) {
            /* no port */
            goto GT_ERR;
        }
        if (sscanf(colon + 1, "%d", &itmp) != 1) {
            goto GT_ERR;
        }

        /* Assign the port */
        server->svc.views = itmp;
        path_begin = strstr(colon, "/");
        if (!path_begin) {
            SET_ERRSTR(cfg, "Expected path in couchApiBase");
            goto GT_ERR;
        }
        server->viewpath = strdup(path_begin);
    } else {
        server->svc.views = 0;
    }

    /* get the 'ports' dictionary */
    if (!get_jobj(js, "ports", &jsports)) {
        SET_ERRSTR(cfg, "Expected 'ports' dictionary");
        goto GT_ERR;
    }

    /* memcached port */
    if (get_jint(jsports, "direct", &itmp)) {
        server->svc.data = itmp;
    } else {
        SET_ERRSTR(cfg, "Expected 'direct' field in 'ports'");
        goto GT_ERR;
    }

    /* set the authority */
    if (!build_server_strings(cfg, server)) {
        goto GT_ERR;
    }
    return 1;

GT_ERR:
    return 0;
}

static void guess_network(cJSON *jnodes, int nsrv, const char *source, char **network)
{
    int ii;
    for (ii = 0; ii < nsrv; ii++) {
        cJSON *jsrv = cJSON_GetArrayItem(jnodes, ii);
        {
            cJSON *jhostname = cJSON_GetObjectItem(jsrv, "hostname");
            if (jhostname && jhostname->type == cJSON_String) {
                if (strcmp(jhostname->valuestring, source) == 0) {
                    *network = strdup("default");
                    return;
                }
            }
        }
        {
            cJSON *jaltaddr = cJSON_GetObjectItem(jsrv, "alternateAddresses");
            if (jaltaddr && jaltaddr->type == cJSON_Object) {
                cJSON *cur;
                for (cur = jaltaddr->child; cur != NULL; cur = cur->next) {
                    if (cur->type == cJSON_Object) {
                        cJSON *jhostname = cJSON_GetObjectItem(cur, "hostname");
                        if (jhostname && jhostname->type == cJSON_String) {
                            if (strcmp(jhostname->valuestring, source) == 0) {
                                *network = strdup(cur->string);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
    *network = strdup("default");
}

int lcbvb_load_json_ex(lcbvb_CONFIG *cfg, const char *data, const char *source, char **network)
{
    cJSON *cj = NULL, *jnodes_ext = NULL, *jnodes = NULL;
    char *tmp = NULL;
    unsigned ii, jnodes_size = 0;
    int jnodes_defined = 0;

    if ((cj = cJSON_Parse(data)) == NULL) {
        SET_ERRSTR(cfg, "Couldn't parse JSON");
        goto GT_ERROR;
    }

    if (get_jstr(cj, "name", &tmp)) {
        cfg->bname = strdup(tmp);
    }

    cfg->dtype = LCBVB_DIST_UNKNOWN;
    if (get_jstr(cj, "nodeLocator", &tmp)) {
        if (!strcmp(tmp, "ketama")) {
            cfg->dtype = LCBVB_DIST_KETAMA;
        } else {
            cfg->dtype = LCBVB_DIST_VBUCKET;
        }
    }

    if (get_jstr(cj, "uuid", &tmp)) {
        cfg->buuid = strdup(tmp);
    }

    if (!get_jint(cj, "rev", &cfg->revid)) {
        cfg->revid = -1;
    }

    get_jarray(cj, "nodes", &jnodes);
    if (jnodes) {
        jnodes_defined = 1;
        jnodes_size = cJSON_GetArraySize(jnodes);
    }
    if (get_jarray(cj, "nodesExt", &jnodes_ext)) {
        cfg->is3x = 1;
        cfg->nsrv = cJSON_GetArraySize(jnodes_ext);
        jnodes = jnodes_ext;
    } else if (jnodes == NULL) {
        SET_ERRSTR(cfg, "expected 'nodesExt' or 'nodes' array");
        goto GT_ERROR;
    }

    cfg->caps = 0;
    {
        cJSON *jcaps = NULL;
        if (get_jarray(cj, "bucketCapabilities", &jcaps)) {
            unsigned ncaps = cJSON_GetArraySize(jcaps);
            for (ii = 0; ii < ncaps; ii++) {
                cJSON *jcap = cJSON_GetArrayItem(jcaps, ii);
                if (jcap || jcap->type == cJSON_String) {
                    if (strcmp(jcap->valuestring, "xattr") == 0) {
                        cfg->caps |= LCBVB_CAP_XATTR;
                    } else if (strcmp(jcap->valuestring, "dcp") == 0) {
                        cfg->caps |= LCBVB_CAP_DCP;
                    } else if (strcmp(jcap->valuestring, "cbhello") == 0) {
                        cfg->caps |= LCBVB_CAP_CBHELLO;
                    } else if (strcmp(jcap->valuestring, "touch") == 0) {
                        cfg->caps |= LCBVB_CAP_TOUCH;
                    } else if (strcmp(jcap->valuestring, "couchapi") == 0) {
                        cfg->caps |= LCBVB_CAP_COUCHAPI;
                    } else if (strcmp(jcap->valuestring, "cccp") == 0) {
                        cfg->caps |= LCBVB_CAP_CCCP;
                    } else if (strcmp(jcap->valuestring, "xdcrCheckpointing") == 0) {
                        cfg->caps |= LCBVB_CAP_XDCR_CHECKPOINTING;
                    } else if (strcmp(jcap->valuestring, "nodesExt") == 0) {
                        cfg->caps |= LCBVB_CAP_NODES_EXT;
                    } else if (strcmp(jcap->valuestring, "collections") == 0) {
                        cfg->caps |= LCBVB_CAP_COLLECTIONS;
                    } else if (strcmp(jcap->valuestring, "durableWrite") == 0) {
                        cfg->caps |= LCBVB_CAP_DURABLE_WRITE;
                    }
                }
            }
        }
    }
    cfg->ccaps = 0;
    {
        cJSON *jcaps = NULL;
        if (get_jobj(cj, "clusterCapabilities", &jcaps)) {
            cJSON *jn1ql = NULL;
            if (get_jarray(jcaps, "n1ql", &jn1ql)) {
                unsigned ncaps = cJSON_GetArraySize(jn1ql);
                for (ii = 0; ii < ncaps; ii++) {
                    cJSON *jcap = cJSON_GetArrayItem(jn1ql, ii);
                    if (jcap || jcap->type == cJSON_String) {
                        if (strcmp(jcap->valuestring, "enhancedPreparedStatements") == 0) {
                            cfg->ccaps |= LCBVB_CCAP_N1QL_ENHANCED_PREPARED_STATEMENTS;
                        }
                    }
                }
            }
        }
    }

    /** Get the number of nodes. This traverses the list. Yuck */
    cfg->nsrv = cJSON_GetArraySize(jnodes);

    if (network && *network == NULL) {
        guess_network(jnodes, cfg->nsrv, source, network);
    }

    /** Allocate a temporary one on the heap */
    cfg->servers = calloc(cfg->nsrv, sizeof(*cfg->servers));
    for (ii = 0; ii < cfg->nsrv; ii++) {
        int rv;
        cJSON *jsrv = cJSON_GetArrayItem(jnodes, ii);

        if (cfg->is3x) {
            rv = build_server_3x(cfg, cfg->servers + ii, jsrv, network);
            if (jnodes_defined && rv && ii >= jnodes_size) {
                cfg->servers[ii].svc.data = 0;
                cfg->servers[ii].svc_ssl.data = 0;
                cfg->servers[ii].alt_svc.data = 0;
                cfg->servers[ii].alt_svc_ssl.data = 0;
            }
        } else {
            rv = build_server_2x(cfg, cfg->servers + ii, jsrv);
        }

        if (!rv) {
            SET_ERRSTR(cfg, "Failed to build server");
            goto GT_ERROR;
        }
    }

    /* Count the number of _data_ servers in the cluster. Per the spec,
     * these will always appear in order (so that we won't ever have "holes") */
    for (ii = 0; ii < cfg->nsrv; ii++) {
        if (!cfg->servers[ii].svc.data) {
            break;
        }
    }
    cfg->ndatasrv = ii;

    if (cfg->dtype == LCBVB_DIST_VBUCKET) {
        if (!parse_vbucket(cfg, cj)) {
            SET_ERRSTR(cfg, "Failed to parse vBucket map");
            goto GT_ERROR;
        }
    } else {
        /* If there is no $HOST then we can update the ketama config, otherwise
         * we must wait for the hostname to be replaced! */
        if (strstr(data, "$HOST") == NULL) {
            if (!update_ketama(cfg)) {
                SET_ERRSTR(cfg, "Failed to establish ketama continuums");
            }
        }
    }
    cfg->servers = realloc(cfg->servers, sizeof(*cfg->servers) * cfg->nsrv);
    cfg->randbuf = malloc(cfg->nsrv * sizeof(*cfg->randbuf));
    cJSON_Delete(cj);
    return 0;

GT_ERROR:
    if (cj) {
        cJSON_Delete(cj);
    }
    return -1;
}

int lcbvb_load_json(lcbvb_CONFIG *cfg, const char *data)
{
    return lcbvb_load_json_ex(cfg, data, NULL, NULL);
}

static void replace_hoststr(char **orig, const char *replacement)
{
    char *match;
    char *newbuf;

    if (!*orig) {
        return;
    }

    match = strstr(*orig, "$HOST");
    if (match == NULL || *match == '\0') {
        return;
    }

    newbuf = malloc(strlen(*orig) + strlen(replacement));
    *match = '\0';

    /* copy until the placeholder */
    strcpy(newbuf, *orig);
    /* copy the host */
    strcat(newbuf, replacement);
    /* copy after the placeholder */
    match += sizeof("$HOST") - 1;
    strcat(newbuf, match);
    free(*orig);
    *orig = newbuf;
}

LIBCOUCHBASE_API
void lcbvb_replace_host(lcbvb_CONFIG *cfg, const char *hoststr)
{
    unsigned ii, copy = 0;
    char *replacement = (char *)hoststr;
    if (strchr(replacement, ':')) {
        size_t len = strlen(hoststr);
        replacement = calloc(len + 2, sizeof(char));
        replacement[0] = '[';
        memcpy(replacement + 1, hoststr, len);
        replacement[len + 1] = ']';
        copy = 1;
    }
    for (ii = 0; ii < cfg->nsrv; ++ii) {
        unsigned jj;
        lcbvb_SERVER *srv = cfg->servers + ii;
        lcbvb_SERVICES *svcs[] = {&srv->svc, &srv->svc_ssl};

        replace_hoststr(&srv->hostname, hoststr);
        for (jj = 0; jj < 2; ++jj) {
            unsigned kk;
            lcbvb_SERVICES *cursvc = svcs[jj];
            replace_hoststr(&cursvc->views_base_, replacement);
            for (kk = 0; kk < LCBVB_SVCTYPE__MAX; ++kk) {
                replace_hoststr(&cursvc->hoststrs[kk], replacement);
            }
        }
        /* reassign authority */
        free(srv->authority);
        srv->authority = strdup(srv->svc.hoststrs[LCBVB_SVCTYPE_DATA]);
    }
    if (copy) {
        free(replacement);
    }
    if (cfg->dtype == LCBVB_DIST_KETAMA) {
        update_ketama(cfg);
    }
}

lcbvb_CONFIG *lcbvb_parse_json(const char *js)
{
    int rv;
    lcbvb_CONFIG *cfg = calloc(1, sizeof(*cfg));
    rv = lcbvb_load_json(cfg, js);
    if (rv) {
        lcbvb_destroy(cfg);
        return NULL;
    }
    return cfg;
}

LIBCOUCHBASE_API
lcbvb_CONFIG *lcbvb_create(void)
{
    return calloc(1, sizeof(lcbvb_CONFIG));
}

static void free_service_strs(lcbvb_SERVICES *svc)
{
    unsigned ii;
    for (ii = 0; ii < LCBVB_SVCTYPE__MAX; ii++) {
        free(svc->hoststrs[ii]);
    }
    free(svc->views_base_);
    free(svc->query_base_);
    free(svc->fts_base_);
    free(svc->cbas_base_);
}

void lcbvb_destroy(lcbvb_CONFIG *conf)
{
    unsigned ii;
    for (ii = 0; ii < conf->nsrv; ii++) {
        lcbvb_SERVER *srv = conf->servers + ii;
        free(srv->hostname);
        free(srv->viewpath);
        free(srv->querypath);
        free(srv->ftspath);
        free(srv->cbaspath);
        free_service_strs(&srv->svc);
        free_service_strs(&srv->svc_ssl);
        free(srv->authority);
        free(srv->alt_hostname);
        free_service_strs(&srv->alt_svc);
        free_service_strs(&srv->alt_svc_ssl);
    }
    free(conf->servers);
    free(conf->continuum);
    free(conf->buuid);
    free(conf->bname);
    free(conf->vbuckets);
    free(conf->ffvbuckets);
    free(conf->randbuf);
    free(conf);
}

static void svcs_to_json(lcbvb_SERVICES *svc, cJSON *jsvc, int is_ssl)
{
    cJSON *tmp;
    const char *key;
#define EXTRACT_SERVICE(name, fld)                                                                                     \
    if (svc->fld) {                                                                                                    \
        key = is_ssl ? name "SSL" : name;                                                                              \
        tmp = cJSON_CreateNumber(svc->fld);                                                                            \
        cJSON_AddItemToObject(jsvc, key, tmp);                                                                         \
    }

    EXTRACT_SERVICE("mgmt", mgmt);
    EXTRACT_SERVICE("capi", views);
    EXTRACT_SERVICE("kv", data);
    EXTRACT_SERVICE("n1ql", n1ql);
    EXTRACT_SERVICE("indexScan", ixquery);
    EXTRACT_SERVICE("indexAdmin", ixadmin);
    EXTRACT_SERVICE("fts", fts);
    EXTRACT_SERVICE("cbas", cbas);

#undef EXTRACT_SERVICE
}

LIBCOUCHBASE_API
char *lcbvb_save_json(lcbvb_CONFIG *cfg)
{
    unsigned ii;
    char *ret;
    cJSON *tmp = NULL, *nodes = NULL;
    cJSON *root = cJSON_CreateObject();

    if (cfg->dtype == LCBVB_DIST_VBUCKET) {
        tmp = cJSON_CreateString("vbucket");
    } else {
        tmp = cJSON_CreateString("ketama");
    }
    cJSON_AddItemToObject(root, "nodeLocator", tmp);

    if (cfg->buuid) {
        tmp = cJSON_CreateString(cfg->buuid);
        cJSON_AddItemToObject(root, "uuid", tmp);
    }
    if (cfg->revid > -1) {
        tmp = cJSON_CreateNumber(cfg->revid);
        cJSON_AddItemToObject(root, "rev", tmp);
    }
    tmp = cJSON_CreateString(cfg->bname);
    cJSON_AddItemToObject(root, "name", tmp);

    nodes = cJSON_CreateArray();
    cJSON_AddItemToObject(root, "nodesExt", nodes);

    for (ii = 0; ii < cfg->nsrv; ii++) {
        cJSON *sj = cJSON_CreateObject(), *jsvc = cJSON_CreateObject();
        lcbvb_SERVER *srv = cfg->servers + ii;

        tmp = cJSON_CreateString(srv->hostname);
        cJSON_AddItemToObject(sj, "hostname", tmp);
        svcs_to_json(&srv->svc, jsvc, 0);
        svcs_to_json(&srv->svc_ssl, jsvc, 1);

        /* add the services to the server */
        cJSON_AddItemToObject(sj, "services", jsvc);
        cJSON_AddItemToArray(nodes, sj);
    }

    /* Now either add the vbucket or ketama stuff */
    if (cfg->dtype == LCBVB_DIST_VBUCKET) {
        cJSON *vbroot = cJSON_CreateObject();
        cJSON *vbmap = cJSON_CreateArray();

        tmp = cJSON_CreateNumber(cfg->nrepl);
        cJSON_AddItemToObject(vbroot, "numReplicas", tmp);

        for (ii = 0; ii < cfg->nvb; ii++) {
            cJSON *curvb = cJSON_CreateIntArray(cfg->vbuckets[ii].servers, cfg->nrepl + 1);
            cJSON_AddItemToArray(vbmap, curvb);
        }

        cJSON_AddItemToObject(vbroot, "vBucketMap", vbmap);
        cJSON_AddItemToObject(root, "vBucketServerMap", vbroot);
    }
    if (cfg->caps != 0) {
        cJSON *jcaps = cJSON_CreateArray();
        if (cfg->caps & LCBVB_CAP_XATTR) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("xattr"));
        }
        if (cfg->caps & LCBVB_CAP_DCP) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("dcp"));
        }
        if (cfg->caps & LCBVB_CAP_CBHELLO) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("cbhello"));
        }
        if (cfg->caps & LCBVB_CAP_TOUCH) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("touch"));
        }
        if (cfg->caps & LCBVB_CAP_COUCHAPI) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("couchapi"));
        }
        if (cfg->caps & LCBVB_CAP_CCCP) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("cccp"));
        }
        if (cfg->caps & LCBVB_CAP_XDCR_CHECKPOINTING) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("xdcrCheckpointing"));
        }
        if (cfg->caps & LCBVB_CAP_NODES_EXT) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("nodesExt"));
        }
        if (cfg->caps & LCBVB_CAP_COLLECTIONS) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("collections"));
        }
        if (cfg->caps & LCBVB_CAP_DURABLE_WRITE) {
            cJSON_AddItemToArray(jcaps, cJSON_CreateString("durableWrite"));
        }
        cJSON_AddItemToObject(root, "bucketCapabilities", jcaps);
    }
    if (cfg->ccaps != 0) {
        cJSON *jcaps = cJSON_CreateObject();
        cJSON *jn1ql = cJSON_CreateArray();
        if (cfg->ccaps & LCBVB_CCAP_N1QL_ENHANCED_PREPARED_STATEMENTS) {
            cJSON_AddItemToArray(jn1ql, cJSON_CreateString("enhancedPreparedStatements"));
        }
        cJSON_AddItemToObject(jcaps, "n1ql", jn1ql);
        cJSON_AddItemToObject(root, "clusterCapabilities", jcaps);
    }

    ret = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return ret;
}

/******************************************************************************
 ******************************************************************************
 ** Mapping Routines                                                         **
 ******************************************************************************
 ******************************************************************************/

static int map_ketama(lcbvb_CONFIG *cfg, const void *key, size_t nkey)
{
    uint32_t digest, mid, prev;
    lcbvb_CONTINUUM *beginp, *endp, *midp, *highp, *lowp;
    lcb_assert(cfg->continuum);
    digest = vb__hash_ketama(key, nkey);
    beginp = lowp = cfg->continuum;
    endp = highp = cfg->continuum + cfg->ncontinuum;

    /* divide and conquer array search to find server with next biggest
     * point after what this key hashes to */
    while (1) {
        /* pick the middle point */
        midp = lowp + (highp - lowp) / 2;

        if (midp == endp) {
            /* if at the end, roll back to zeroth */
            return beginp->index;
            break;
        }

        mid = midp->point;
        prev = (midp == beginp) ? 0 : (midp - 1)->point;

        if (digest <= mid && digest > prev) {
            /* we found nearest server */
            return midp->index;
            break;
        }

        /* adjust the limits */
        if (mid < digest) {
            lowp = midp + 1;
        } else {
            highp = midp - 1;
        }

        if (lowp > highp) {
            return beginp->index;
            break;
        }
    }
    return -1;
}

int lcbvb_k2vb(lcbvb_CONFIG *cfg, const void *k, lcb_SIZE n)
{
    uint32_t digest = hash_crc32(k, n);
    return digest % cfg->nvb;
}

int lcbvb_vbmaster(lcbvb_CONFIG *cfg, int vbid)
{
    return cfg->vbuckets[vbid].servers[0];
}

int lcbvb_vbreplica(lcbvb_CONFIG *cfg, int vbid, unsigned ix)
{
    if (ix < cfg->nrepl) {
        return cfg->vbuckets[vbid].servers[ix + 1];
    } else {
        return -1;
    }
}

/*
 * (https://www.couchbase.com/issues/browse/MB-12268?focusedCommentId=101894&page=com.atlassian.jira.plugin.system.issuetabpanels:comment-tabpanel#comment-101894)
 *
 * So (from my possibly partially ignorant view of that matter) I'd do the following:
 *
 * 1) Send first request according to lated vbucket map you have.
 *    If it works, we're good. Exit.
 *
 * 2) if that fails, look if you've newer vbucket map. If there's newer vbucket map
 *    and it points to _different_ node, send request to that node and proceed
 *    to step 3. Otherwise go to step 4
 *
 * 3) if newer node still gives you not-my-vbucket, go to step 4
 *
 * 4) if there's fast forward map in latest bucket info and fast forward map
 *    points to different node, send request to that node. And go to step 5.
 *    Otherwise (not ff map or it points to one of nodes you've tried already),
 *    go to step 6
 *
 * 5) if ff map node request succeeds. Exit. Otherwise go to step 6.
 *
 * 6) Try first replica unless it's one of nodes you've already tried.
 *    If it succeeds. Exit. Otherwise go to step 7.
 *
 * 7) Try all nodes in turn, prioritizing other replicas to beginning of list
 *    and nodes you have already tried to end. If one of nodes agrees to perform
 *    your request. Exit. Otherwise propagate error to back to app
 */
int lcbvb_nmv_remap_ex(lcbvb_CONFIG *cfg, int vbid, int bad, int heuristic)
{
    int cur = cfg->vbuckets[vbid].servers[0];
    int rv = cur;
    unsigned ii;

    if (bad != cur) {
        return cur;
    }

    /* if a forward table exists, then return the vbucket id from the forward table
     * and update that information in the current table. We also need to Update the
     * replica information for that vbucket */

    if (cfg->ffvbuckets && (rv = cfg->ffvbuckets[vbid].servers[0]) != bad && rv > -1) {
        memcpy(&cfg->vbuckets[vbid], &cfg->ffvbuckets[vbid], sizeof(lcbvb_VBUCKET));
    }

    /* this path is usually only followed if fvbuckets is not present */
    if (heuristic && cur == bad) {
        int validrv = -1;
        for (ii = 0; ii < cfg->ndatasrv; ii++) {
            rv = (rv + 1) % cfg->ndatasrv;
            /* check that the new index has assigned vbuckets (master or replica) */
            if (cfg->servers[rv].nvbs) {
                validrv = rv;
                cfg->vbuckets[vbid].servers[0] = rv;
                break;
            }
        }

        if (validrv == -1) {
            /* this should happen when there is only one valid node remaining
             * in the cluster, and we've removed serveral other nodes that are
             * still present in the map due to the grace period window.*/
            return -1;
        }
    }

    if (rv == bad) {
        return -1;
    }

    return rv;
}

int lcbvb_map_key(lcbvb_CONFIG *cfg, const void *key, lcb_SIZE nkey, int *vbid, int *srvix)
{
    if (cfg->dtype == LCBVB_DIST_KETAMA) {
        *srvix = map_ketama(cfg, key, nkey);
        if (vbid) {
            *vbid = 0;
        }
        return 0;
    } else {
        int vb = lcbvb_k2vb(cfg, key, nkey);
        *srvix = lcbvb_vbmaster(cfg, vb);
        if (vbid) {
            *vbid = vb;
        }
    }
    return 0;
}

int lcbvb_has_vbucket(lcbvb_CONFIG *vbc, int vbid, int ix)
{
    unsigned ii;
    lcbvb_VBUCKET *vb = &vbc->vbuckets[vbid];
    for (ii = 0; ii < vbc->nrepl + 1; ii++) {
        if (vb->servers[ii] == ix) {
            return 1;
        }
    }
    return 0;
}

/******************************************************************************
 ******************************************************************************
 ** Configuration Comparisons/Diffs                                          **
 ******************************************************************************
 ******************************************************************************/
static void compute_vb_list_diff(lcbvb_CONFIG *from, lcbvb_CONFIG *to, char **out)
{
    int offset = 0;
    unsigned ii, jj;

    for (ii = 0; ii < to->nsrv; ii++) {
        int found = 0;
        lcbvb_SERVER *newsrv = to->servers + ii;
        for (jj = 0; !found && jj < from->nsrv; jj++) {
            lcbvb_SERVER *oldsrv = from->servers + jj;
            found |= (strcmp(newsrv->authority, oldsrv->authority) == 0);
        }
        if (!found) {
            char *infostr = malloc(strlen(newsrv->authority) + 128);
            lcb_assert(infostr);
            sprintf(infostr, "%s(Data=%d, Index=%d, Query=%d)", newsrv->authority, newsrv->svc.data, newsrv->svc.n1ql,
                    newsrv->svc.ixquery);
            out[offset] = infostr;
            ++offset;
        }
    }
}

lcbvb_CONFIGDIFF *lcbvb_compare(lcbvb_CONFIG *from, lcbvb_CONFIG *to)
{
    int nservers;
    lcbvb_CONFIGDIFF *ret;
    unsigned ii;

    ret = calloc(1, sizeof(*ret));
    nservers = (from->nsrv > to->nsrv ? from->nsrv : to->nsrv) + 1;
    ret->servers_added = calloc(nservers, sizeof(*ret->servers_added));
    ret->servers_removed = calloc(nservers, sizeof(*ret->servers_removed));
    compute_vb_list_diff(from, to, ret->servers_added);
    compute_vb_list_diff(to, from, ret->servers_removed);

    if (to->nsrv == from->nsrv) {
        for (ii = 0; ii < from->nsrv; ii++) {
            const char *sa, *sb;
            sa = from->servers[ii].authority;
            sb = to->servers[ii].authority;
            ret->sequence_changed |= (0 != strcmp(sa, sb));
        }
    } else {
        ret->sequence_changed = 1;
    }

    if (from->nvb == to->nvb) {
        for (ii = 0; ii < from->nvb; ii++) {
            lcbvb_VBUCKET *vba = from->vbuckets + ii, *vbb = to->vbuckets + ii;
            if (vba->servers[0] != vbb->servers[0]) {
                ret->n_vb_changes++;
            }
        }
    } else {
        ret->n_vb_changes = -1;
    }
    return ret;
}

static void free_array_helper(char **l)
{
    int ii;
    for (ii = 0; l[ii]; ii++) {
        free(l[ii]);
    }
    free(l);
}

void lcbvb_free_diff(lcbvb_CONFIGDIFF *diff)
{
    lcb_assert(diff);
    free_array_helper(diff->servers_added);
    free_array_helper(diff->servers_removed);
    free(diff);
}

lcbvb_CHANGETYPE lcbvb_get_changetype(lcbvb_CONFIGDIFF *diff)
{
    lcbvb_CHANGETYPE ret = 0;
    if (diff->n_vb_changes) {
        ret |= LCBVB_MAP_MODIFIED;
    }
    if (*diff->servers_added || *diff->servers_removed || diff->sequence_changed) {
        ret |= LCBVB_SERVERS_MODIFIED;
    }
    return ret;
}

/******************************************************************************
 ******************************************************************************
 ** String/Port Getters                                                      **
 ******************************************************************************
 ******************************************************************************/

static const lcbvb_SERVICES *get_svc(const lcbvb_SERVER *srv, lcbvb_SVCMODE mode)
{
    if (srv->alt_hostname) {
        if (mode == LCBVB_SVCMODE_PLAIN) {
            return &srv->alt_svc;
        } else {
            return &srv->alt_svc_ssl;
        }
    } else {
        if (mode == LCBVB_SVCMODE_PLAIN) {
            return &srv->svc;
        } else {
            return &srv->svc_ssl;
        }
    }
}

static const char *get_hostname(const lcbvb_SERVER *srv)
{
    if (srv->alt_hostname) {
        return srv->alt_hostname;
    } else {
        return srv->hostname;
    }
}

LIBCOUCHBASE_API
unsigned lcbvb_get_port(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode)
{
    const lcbvb_SERVICES *svc;
    lcbvb_SERVER *srv;
    if (type >= LCBVB_SVCTYPE__MAX || mode >= LCBVB_SVCMODE__MAX) {
        return 0;
    }
    if (ix >= cfg->nsrv) {
        return 0;
    }

    srv = cfg->servers + ix;
    svc = get_svc(srv, mode);

    if (type == LCBVB_SVCTYPE_DATA) {
        return svc->data;
    } else if (type == LCBVB_SVCTYPE_MGMT) {
        return svc->mgmt;
    } else if (type == LCBVB_SVCTYPE_VIEWS) {
        return svc->views;
    } else if (type == LCBVB_SVCTYPE_IXADMIN) {
        return svc->ixadmin;
    } else if (type == LCBVB_SVCTYPE_IXQUERY) {
        return svc->ixquery;
    } else if (type == LCBVB_SVCTYPE_N1QL) {
        return svc->n1ql;
    } else if (type == LCBVB_SVCTYPE_FTS) {
        return svc->fts;
    } else if (type == LCBVB_SVCTYPE_CBAS) {
        return svc->cbas;
    } else {
        return 0;
    }
}

LIBCOUCHBASE_API
const char *lcbvb_get_hostport(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode)
{
    char **strp;
    lcbvb_SERVER *srv;
    lcbvb_SERVICES *svc;
    unsigned port = lcbvb_get_port(cfg, ix, type, mode);

    if (!port) {
        return NULL;
    }

    srv = cfg->servers + ix;
    svc = (lcbvb_SERVICES *)get_svc(srv, mode);

    strp = &svc->hoststrs[type];
    if (*strp == NULL) {
        const char *hostname = get_hostname(srv);
        size_t strn = strlen(hostname) + 20;
        *strp = calloc(strn, sizeof(char));
        copy_address(*strp, strn, hostname, port);
    }
    return *strp;
}

LIBCOUCHBASE_API
const char *lcbvb_get_hostname(const lcbvb_CONFIG *cfg, unsigned ix)
{
    if (cfg->nsrv > ix) {
        return get_hostname(cfg->servers + ix);
    } else {
        return NULL;
    }
}

LIBCOUCHBASE_API
int lcbvb_get_randhost_ex(const lcbvb_CONFIG *cfg, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode, int *used)
{
    size_t nn, oix = 0;

    if (cfg == NULL) {
        return -1;
    }

    /*
     * Since not all nodes support all service types, we need to make it a
     * fair selection by pre-filtering the nodes which actually support the
     * service, and then proceed to actually select a suitable node.
     */
    for (nn = 0; nn < cfg->nsrv; nn++) {
        const lcbvb_SERVER *server = cfg->servers + nn;
        const lcbvb_SERVICES *svcs = get_svc(server, mode);
        int has_svc = 0;

        // Check if this node is in the exclude list
        if (used && used[nn]) {
            continue;
        }

        has_svc = (type == LCBVB_SVCTYPE_DATA && svcs->data) || (type == LCBVB_SVCTYPE_IXADMIN && svcs->ixadmin) ||
                  (type == LCBVB_SVCTYPE_IXQUERY && svcs->ixquery) || (type == LCBVB_SVCTYPE_MGMT && svcs->mgmt) ||
                  (type == LCBVB_SVCTYPE_N1QL && svcs->n1ql) || (type == LCBVB_SVCTYPE_FTS && svcs->fts) ||
                  (type == LCBVB_SVCTYPE_VIEWS && svcs->views) || (type == LCBVB_SVCTYPE_CBAS && svcs->cbas);

        if (has_svc) {
            cfg->randbuf[oix++] = (int)nn;
        }
    }

    if (!oix) {
        /* nothing supports it! */
        return -1;
    }

    nn = rand();
    nn %= oix;
    return cfg->randbuf[nn];
}

LIBCOUCHBASE_API
int lcbvb_get_randhost(const lcbvb_CONFIG *cfg, lcbvb_SVCTYPE type, lcbvb_SVCMODE mode)
{
    return lcbvb_get_randhost_ex(cfg, type, mode, NULL);
}

LIBCOUCHBASE_API
const char *lcbvb_get_resturl(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCTYPE svc, lcbvb_SVCMODE mode)
{
    char **strp;
    const char *prefix;
    const char *path;

    lcbvb_SERVER *srv;
    lcbvb_SERVICES *svcs;
    unsigned port;
    port = lcbvb_get_port(cfg, ix, svc, mode);
    if (!port) {
        return NULL;
    }

    srv = cfg->servers + ix;
    if (mode == LCBVB_SVCMODE_PLAIN) {
        prefix = "http";
    } else {
        prefix = "https";
    }
    svcs = (lcbvb_SERVICES *)get_svc(srv, mode);

    if (svc == LCBVB_SVCTYPE_VIEWS) {
        path = srv->viewpath;
        strp = &svcs->views_base_;
    } else if (svc == LCBVB_SVCTYPE_N1QL) {
        path = srv->querypath;
        strp = &svcs->query_base_;
    } else if (svc == LCBVB_SVCTYPE_FTS) {
        path = srv->ftspath;
        strp = &svcs->fts_base_;
    } else if (svc == LCBVB_SVCTYPE_CBAS) {
        path = srv->cbaspath;
        strp = &svcs->cbas_base_;
    } else {
        /* Unknown service! */
        return NULL;
    }

    if (path == NULL) {
        return NULL;
    } else if (!*strp) {
        char buf[4096];
        const char *hostname = get_hostname(srv);
        if (strchr(hostname, ':')) {
            // IPv6 and should be bracketed
            snprintf(buf, sizeof(buf), "%s://[%s]:%d%s", prefix, hostname, port, path);
        } else {
            snprintf(buf, sizeof(buf), "%s://%s:%d%s", prefix, hostname, port, path);
        }
        *strp = strdup(buf);
    }

    return *strp;
}

LIBCOUCHBASE_API
const char *lcbvb_get_capibase(lcbvb_CONFIG *cfg, unsigned ix, lcbvb_SVCMODE mode)
{
    return lcbvb_get_resturl(cfg, ix, LCBVB_SVCTYPE_VIEWS, mode);
}

LIBCOUCHBASE_API int lcbvb_get_revision(const lcbvb_CONFIG *cfg)
{
    return cfg->revid;
}
LIBCOUCHBASE_API unsigned lcbvb_get_nservers(const lcbvb_CONFIG *cfg)
{
    return cfg->nsrv;
}
LIBCOUCHBASE_API unsigned lcbvb_get_nreplicas(const lcbvb_CONFIG *cfg)
{
    return cfg->nrepl;
}
LIBCOUCHBASE_API unsigned lcbvb_get_nvbuckets(const lcbvb_CONFIG *cfg)
{
    return cfg->nvb;
}
LIBCOUCHBASE_API lcbvb_DISTMODE lcbvb_get_distmode(const lcbvb_CONFIG *cfg)
{
    return cfg->dtype;
}
LIBCOUCHBASE_API const char *lcbvb_get_error(const lcbvb_CONFIG *cfg)
{
    return cfg->errstr;
}
/******************************************************************************
 ******************************************************************************
 ** Generation Functions                                                     **
 ******************************************************************************
 ******************************************************************************/

static void copy_service(const char *hostname, const lcbvb_SERVICES *src, lcbvb_SERVICES *dst)
{
    *dst = *src;
    memset(&dst->hoststrs, 0, sizeof dst->hoststrs);
    if (src->views_base_) {
        dst->views_base_ = strdup(src->views_base_);
    }
    if (src->query_base_) {
        dst->query_base_ = strdup(src->query_base_);
    }
    if (src->fts_base_) {
        dst->fts_base_ = strdup(src->fts_base_);
    }
    if (src->cbas_base_) {
        dst->cbas_base_ = strdup(src->cbas_base_);
    }
    if (dst->data) {
        char buf[4096];
        copy_address(buf, sizeof(buf), hostname, dst->data);
        dst->hoststrs[LCBVB_SVCTYPE_DATA] = strdup(buf);
    }
}

LIBCOUCHBASE_API
int lcbvb_genconfig_ex(lcbvb_CONFIG *vb, const char *name, const char *uuid, const lcbvb_SERVER *servers,
                       unsigned nservers, unsigned nreplica, unsigned nvbuckets)
{
    unsigned ii, jj;
    int srvix = 0, in_nondata = 0;

    lcb_assert(nservers);

    if (!name) {
        name = "default";
    }

    memset(vb, 0, sizeof(*vb));
    vb->dtype = LCBVB_DIST_VBUCKET;
    vb->nvb = nvbuckets;
    vb->nrepl = nreplica;
    vb->nsrv = nservers;
    vb->bname = strdup(name);
    if (uuid) {
        vb->buuid = strdup(uuid);
    }

    if (nreplica >= nservers) {
        vb->errstr = "nservers must be > nreplicas";
        return -1;
    }

    if (nreplica > 4) {
        vb->errstr = "Replicas must be <= 4";
        return -1;
    }

    /* Count the number of data servers.. */
    for (ii = 0; ii < nservers; ii++) {
        const lcbvb_SERVER *server = servers + ii;
        if (server->svc.data) {
            if (in_nondata) {
                vb->errstr = "All data servers must be specified before non-data servers";
                return -1;
            }
            vb->ndatasrv++;
        } else {
            in_nondata = 1;
        }
    }

    if (vb->nvb) {
        vb->vbuckets = malloc(vb->nvb * sizeof(*vb->vbuckets));
        if (!vb->vbuckets) {
            vb->errstr = "Couldn't allocate vbucket array";
            return -1;
        }
    }

    for (ii = 0; ii < vb->nvb; ii++) {
        lcbvb_VBUCKET *cur = vb->vbuckets + ii;
        cur->servers[0] = srvix;
        for (jj = 1; jj < vb->nrepl + 1; jj++) {
            cur->servers[jj] = (srvix + jj) % vb->ndatasrv;
        }
        srvix = (srvix + 1) % vb->ndatasrv;
    }

    vb->servers = calloc(vb->nsrv, sizeof(*vb->servers));
    vb->randbuf = calloc(vb->nsrv, sizeof(*vb->randbuf));

    for (ii = 0; ii < vb->nsrv; ii++) {
        lcbvb_SERVER *dst = vb->servers + ii;
        const lcbvb_SERVER *src = servers + ii;

        *dst = *src;
        dst->hostname = strdup(src->hostname);
        if (src->viewpath) {
            dst->viewpath = strdup(src->viewpath);
        }
        if (src->querypath) {
            dst->querypath = strdup(src->querypath);
        }
        if (src->ftspath) {
            dst->ftspath = strdup(src->ftspath);
        }
        if (src->cbaspath) {
            dst->cbaspath = strdup(src->cbaspath);
        }

        copy_service(src->hostname, &src->svc, &dst->svc);
        copy_service(src->hostname, &src->svc_ssl, &dst->svc_ssl);
        {
            char tmpbuf[MAX_AUTHORITY_SIZE] = {0};
            copy_address(tmpbuf, sizeof(tmpbuf), dst->hostname, dst->svc.data);
            dst->authority = strdup(tmpbuf);
        }
    }

    for (ii = 0; ii < vb->nvb; ii++) {
        for (jj = 0; jj < vb->nrepl + 1; jj++) {
            int ix = vb->vbuckets[ii].servers[jj];
            if (ix >= 0) {
                vb->servers[ix].nvbs++;
            }
        }
    }
    return 0;
}

int lcbvb_genconfig(lcbvb_CONFIG *vb, unsigned nservers, unsigned nreplica, unsigned nvbuckets)
{
    unsigned ii;
    int rv;
    lcbvb_SERVER *srvarry;

    srvarry = calloc(nservers, sizeof(*srvarry));
    for (ii = 0; ii < nservers; ii++) {
        srvarry[ii].svc.data = 1000 + ii;
        srvarry[ii].svc.views = 2000 + ii;
        srvarry[ii].svc.mgmt = 3000 + ii;
        srvarry[ii].hostname = "localhost";
        srvarry[ii].svc.views_base_ = "/default";
    }
    rv = lcbvb_genconfig_ex(vb, "default", NULL, srvarry, nservers, nreplica, nvbuckets);
    free(srvarry);
    return rv;
}

void lcbvb_genffmap(lcbvb_CONFIG *cfg)
{
    size_t ii;
    lcb_assert(cfg->nrepl);
    if (cfg->ffvbuckets) {
        free(cfg->ffvbuckets);
    }
    cfg->ffvbuckets = calloc(cfg->nvb, sizeof *cfg->ffvbuckets);
    for (ii = 0; ii < cfg->nvb; ++ii) {
        size_t jj;
        lcbvb_VBUCKET *vb = cfg->ffvbuckets + ii;
        memcpy(vb, cfg->vbuckets + ii, sizeof *vb);
        for (jj = 0; jj < cfg->ndatasrv; ++jj) {
            vb->servers[jj] = (vb->servers[jj] + 1) % cfg->ndatasrv;
        }
    }
}

void lcbvb_make_ketama(lcbvb_CONFIG *vb)
{
    if (vb->dtype == LCBVB_DIST_KETAMA) {
        return;
    }
    vb->dtype = LCBVB_DIST_KETAMA;
    vb->nrepl = 0;
    vb->nvb = 0;
    update_ketama(vb);
}

/******************************************************************************
 ******************************************************************************
 ** Compatibility APIs                                                       **
 ******************************************************************************
 ******************************************************************************/
LIBCOUCHBASE_API lcbvb_CONFIG *vbucket_config_create(void)
{
    return lcbvb_create();
}
LIBCOUCHBASE_API void vbucket_config_destroy(lcbvb_CONFIG *h)
{
    lcbvb_destroy(h);
}
LIBCOUCHBASE_API int vbucket_config_parse(lcbvb_CONFIG *h, vbucket_source_t src, const char *s)
{
    (void)src;
    return lcbvb_load_json(h, s);
}
LIBCOUCHBASE_API const char *vbucket_get_error_message(lcbvb_CONFIG *h)
{
    return h->errstr;
}
LIBCOUCHBASE_API int vbucket_config_get_num_servers(lcbvb_CONFIG *cfg)
{
    return cfg->nsrv;
}
LIBCOUCHBASE_API int vbucket_config_get_num_replicas(lcbvb_CONFIG *cfg)
{
    return cfg->nrepl;
}
LIBCOUCHBASE_API int vbucket_config_get_num_vbuckets(lcbvb_CONFIG *cfg)
{
    return cfg->nvb;
}
LIBCOUCHBASE_API const char *vbucket_config_get_server(lcbvb_CONFIG *cfg, int ix)
{
    return lcbvb_get_hostport(cfg, ix, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN);
}
LIBCOUCHBASE_API const char *vbucket_config_get_rest_api_server(lcbvb_CONFIG *cfg, int ix)
{
    return lcbvb_get_hostport(cfg, ix, LCBVB_SVCTYPE_MGMT, LCBVB_SVCMODE_PLAIN);
}
LIBCOUCHBASE_API const char *vbucket_config_get_couch_api_base(lcbvb_CONFIG *cfg, int ix)
{
    return lcbvb_get_capibase(cfg, ix, LCBVB_SVCMODE_PLAIN);
}
LIBCOUCHBASE_API lcbvb_DISTMODE vbucket_config_get_distribution_type(lcbvb_CONFIG *cfg)
{
    return cfg->dtype;
}
LIBCOUCHBASE_API int vbucket_map(lcbvb_CONFIG *cfg, const void *k, lcb_SIZE nk, int *pvb, int *pix)
{
    return lcbvb_map_key(cfg, k, nk, pvb, pix);
}
LIBCOUCHBASE_API int vbucket_get_vbucket_by_key(lcbvb_CONFIG *cfg, const void *k, lcb_SIZE nk)
{
    return lcbvb_k2vb(cfg, k, nk);
}
LIBCOUCHBASE_API int vbucket_get_master(lcbvb_CONFIG *cfg, int vb)
{
    return lcbvb_vbmaster(cfg, vb);
}
LIBCOUCHBASE_API int vbucket_get_replica(lcbvb_CONFIG *cfg, int vb, int repl)
{
    return lcbvb_vbreplica(cfg, vb, repl);
}
LIBCOUCHBASE_API lcbvb_CONFIGDIFF *vbucket_compare(lcbvb_CONFIG *a, lcbvb_CONFIG *b)
{
    return lcbvb_compare(a, b);
}
LIBCOUCHBASE_API void vbucket_free_diff(lcbvb_CONFIGDIFF *p)
{
    lcbvb_free_diff(p);
}
LIBCOUCHBASE_API int vbucket_config_get_revision(lcbvb_CONFIG *p)
{
    return lcbvb_get_revision(p);
}
LIBCOUCHBASE_API lcbvb_CHANGETYPE vbucket_what_changed(lcbvb_CONFIGDIFF *diff)
{
    return lcbvb_get_changetype(diff);
}
LIBCOUCHBASE_API int vbucket_config_generate(lcbvb_CONFIG *cfg, unsigned nsrv, unsigned nrepl, unsigned nvb)
{
    return lcbvb_genconfig(cfg, nsrv, nrepl, nvb);
}
