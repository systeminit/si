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

#include "internal.h"
#include "packetutils.h"
#include "bucketconfig/clconfig.h"
#include "vbucket/aliases.h"
#include "sllist-inl.h"

#define LOGARGS(instance, lvl) (instance)->settings, "newconfig", LCB_LOG_##lvl, __FILE__, __LINE__
#define LOG(instance, lvlbase, msg) lcb_log(instance->settings, "newconfig", LCB_LOG_##lvlbase, __FILE__, __LINE__, msg)

#define SERVER_FMT LCB_LOG_SPEC("%s:%s") " (%p)"
#define SERVER_ARGS(s)                                          \
    (s)->settings->log_redaction ? LCB_LOG_SD_OTAG : "",       \
        (s)->get_host().host, (s)->get_host().port,             \
        (s)->settings->log_redaction ? LCB_LOG_SD_CTAG : "",   \
        (void *)s

typedef struct lcb_GUESSVB_st {
    time_t last_update; /**< Last time this vBucket was heuristically set */
    char newix; /**< New index, heuristically determined */
    char oldix; /**< Original index, according to map */
    char used; /**< Flag indicating whether or not this entry has been used */
} lcb_GUESSVB;

/* Ignore configuration updates for heuristically guessed vBuckets for a
 * maximum amount of [n] seconds */
#define MAX_KEEP_GUESS 20

static int
should_keep_guess(lcb_GUESSVB *guess, lcbvb_VBUCKET *vb)
{
    if (guess->newix == guess->oldix) {
        /* Heuristic position is the same as starting position */
        return 0;
    }
    if (vb->servers[0] != guess->oldix) {
        /* Previous master changed */
        return 0;
    }

    if (time(NULL) - guess->last_update > MAX_KEEP_GUESS) {
        /* Last usage too old */
        return 0;
    }

    return 1;
}

void
lcb_vbguess_newconfig(lcb_INSTANCE *instance, lcbvb_CONFIG *cfg, lcb_GUESSVB *guesses)
{
    unsigned ii;

    if (!guesses) {
        return;
    }

    for (ii = 0; ii < cfg->nvb; ii++) {
        lcb_GUESSVB *guess = guesses + ii;
        lcbvb_VBUCKET *vb = cfg->vbuckets + ii;

        if (!guess->used) {
            continue;
        }

        /* IF: Heuristically learned a new index, _and_ the old index (which is
         * known to be bad) is the same index stated by the new config */
        if (should_keep_guess(guess, vb)) {
            lcb_log(LOGARGS(instance, TRACE), "Keeping heuristically guessed index. VBID=%d. Current=%d. Old=%d.", ii, guess->newix, guess->oldix);
            vb->servers[0] = guess->newix;
        } else {
            /* We don't reassign to the guess structure here. The idea is that
             * we will simply use the new config. If this gives us problems, the
             * config will re-learn again. */
            lcb_log(LOGARGS(instance, TRACE), "Ignoring heuristically guessed index. VBID=%d. Current=%d. Old=%d. New=%d", ii, guess->newix, guess->oldix, vb->servers[0]);
            guess->used = 0;
        }
    }
}

int
lcb_vbguess_remap(lcb_INSTANCE *instance, int vbid, int bad)
{
    if (LCBT_SETTING(instance, vb_noremap)) {
        return -1;
    }

    if (LCBT_SETTING(instance, vb_noguess)) {
        int newix = lcbvb_nmv_remap_ex(LCBT_VBCONFIG(instance), vbid, bad, 0);
        if (newix > -1 && newix != bad) {
            lcb_log(LOGARGS(instance, TRACE), "Got new index from ffmap. VBID=%d. Old=%d. New=%d", vbid, bad, newix);
        }
        return newix;

    } else {
        lcb_GUESSVB *guesses = instance->vbguess;
        if (!guesses) {
            guesses = instance->vbguess =
                reinterpret_cast< lcb_GUESSVB * >(calloc(LCBT_VBCONFIG(instance)->nvb, sizeof(lcb_GUESSVB)));
        }
        lcb_GUESSVB *guess = guesses + vbid;
        int newix = lcbvb_nmv_remap_ex(LCBT_VBCONFIG(instance), vbid, bad, 1);
        if (newix > -1 && newix != bad) {
            guess->newix = newix;
            guess->oldix = bad;
            guess->used = 1;
            guess->last_update = time(NULL);
            lcb_log(LOGARGS(instance, TRACE), "Guessed new heuristic index VBID=%d. Old=%d. New=%d", vbid, bad, newix);
        }
        return newix;
    }
}

/**
 * Finds the index of an older server using the current config.
 *
 * This function is used to help reuse the server structures for memcached
 * packets.
 *
 * @param oldconfig The old configuration. This is the configuration the
 * old server is bound to
 * @param newconfig The new configuration. This will be inspected for new
 * nodes which may have been added, and ones which may have been removed.
 * @param server The server to match
 * @return The new index, or -1 if the current server is not present in the new
 * config.
 */
static int
find_new_data_index(lcbvb_CONFIG *oldconfig, lcbvb_CONFIG* newconfig,
    lcb::Server *server)
{
    lcbvb_SVCMODE mode = LCBT_SETTING_SVCMODE(server->get_instance());
    const char *old_datahost = lcbvb_get_hostport(oldconfig,
        server->get_index(), LCBVB_SVCTYPE_DATA, mode);

    if (!old_datahost) {
        /* Old server had no data service */
        return -1;
    }

    for (size_t ii = 0; ii < LCBVB_NSERVERS(newconfig); ii++) {
        const char *new_datahost = lcbvb_get_hostport(newconfig, ii,
            LCBVB_SVCTYPE_DATA, mode);
        if (new_datahost && strcmp(new_datahost, old_datahost) == 0) {
            return ii;
        }
    }
    return -1;
}

static void
log_vbdiff(lcb_INSTANCE *instance, lcbvb_CONFIGDIFF *diff)
{
    lcb_log(LOGARGS(instance, INFO), "Config Diff: [ vBuckets Modified=%d ], [Sequence Changed=%d]", diff->n_vb_changes, diff->sequence_changed);
    if (diff->servers_added) {
        for (char **curserver = diff->servers_added; *curserver; curserver++) {
            lcb_log(LOGARGS(instance, INFO), "Detected server %s added", *curserver);
        }
    }
    if (diff->servers_removed) {
        for (char **curserver = diff->servers_removed; *curserver; curserver++) {
            lcb_log(LOGARGS(instance, INFO), "Detected server %s removed", *curserver);
        }
    }
}

/**
 * This callback is invoked for packet relocation twice. It tries to relocate
 * commands to their destination server. Some commands may not be relocated
 * either because they have no explicit "Relocation Information" (i.e. no
 * specific vbucket) or because the command is tied to a specific server (i.e.
 * CMD_STAT).
 *
 * Note that `KEEP_PACKET` here doesn't mean to "Save" the packet, but rather
 * to keep the packet in the current queue (so that if the server ends up
 * being removed, the command will fail); rather than being relocated to
 * another server.
 */
static int
iterwipe_cb(mc_CMDQUEUE *cq, mc_PIPELINE *oldpl, mc_PACKET *oldpkt, void *)
{
    protocol_binary_request_header hdr;
    lcb::Server *srv = static_cast<lcb::Server *>(oldpl);
    int newix;
    lcb_INSTANCE *instance = (lcb_INSTANCE *)cq->cqdata;

    mcreq_read_hdr(oldpkt, &hdr);

    if (!lcb_should_retry(srv->get_settings(), oldpkt, LCB_MAX_ERROR)) {
        return MCREQ_KEEP_PACKET;
    }

    if (LCBVB_DISTTYPE(cq->config) == LCBVB_DIST_VBUCKET) {
        newix = lcbvb_vbmaster(cq->config, ntohs(hdr.request.vbucket));

    } else {
        const void *key = NULL;
        lcb_SIZE nkey = 0;
        int tmpid;

        /* XXX: We ignore hashkey. This is going away soon, and is probably
         * better than simply failing the items. */
        mcreq_get_key(instance, oldpkt, &key, &nkey);
        lcbvb_map_key(cq->config, key, nkey, &tmpid, &newix);
    }

    if (newix < 0 || newix > (int)cq->npipelines-1) {
        return MCREQ_KEEP_PACKET;
    }


    mc_PIPELINE *newpl = cq->pipelines[newix];
    if (newpl == oldpl || newpl == NULL) {
        return MCREQ_KEEP_PACKET;
    }

    lcb_log(LOGARGS(instance, DEBUG), "Remapped packet %p (SEQ=%u) from " SERVER_FMT " to " SERVER_FMT,
        (void*)oldpkt, oldpkt->opaque, SERVER_ARGS((lcb::Server*)oldpl), SERVER_ARGS((lcb::Server*)newpl));

    /** Otherwise, copy over the packet and find the new vBucket to map to */
    mc_PACKET *newpkt = mcreq_renew_packet(oldpkt);
    newpkt->flags &= ~MCREQ_STATE_FLAGS;
    mcreq_reenqueue_packet(newpl, newpkt);
    mcreq_packet_handled(oldpl, oldpkt);
    return MCREQ_REMOVE_PACKET;
}

static void
replace_config(lcb_INSTANCE *instance, lcbvb_CONFIG *oldconfig, lcbvb_CONFIG *newconfig)
{
    mc_CMDQUEUE *cq = &instance->cmdq;
    mc_PIPELINE **ppold, **ppnew;
    unsigned ii, nold, nnew;

    lcb_assert(LCBT_VBCONFIG(instance) == newconfig);

    nnew = LCBVB_NSERVERS(newconfig);
    ppnew = reinterpret_cast<mc_PIPELINE**>(calloc(nnew, sizeof(*ppnew)));
    ppold = mcreq_queue_take_pipelines(cq, &nold);

    /**
     * Determine which existing servers are still part of the new cluster config
     * and place it inside the new list.
     */
    for (ii = 0; ii < nold; ii++) {
        lcb::Server *cur = static_cast<lcb::Server *>(ppold[ii]);
        int newix = find_new_data_index(oldconfig, newconfig, cur);
        if (newix > -1) {
            cur->set_new_index(newix);
            ppnew[newix] = cur;
            ppold[ii] = NULL;
            lcb_log(LOGARGS(instance, INFO), "Reusing server " SERVER_FMT ". OldIndex=%d. NewIndex=%d", SERVER_ARGS(cur), ii, newix);
        }
    }

    /**
     * Once we've moved the kept servers to the new list, allocate new lcb::Server
     * structures for slots that don't have an existing lcb::Server. We must do
     * this before add_pipelines() is called, so that there are no holes inside
     * ppnew
     */
    for (ii = 0; ii < nnew; ii++) {
        if (!ppnew[ii]) {
            ppnew[ii] = new lcb::Server(instance, ii);
        }
    }

    /**
     * Once we have all the server structures in place for the new config,
     * transfer the new config along with the new list over to the CQ structure.
     */
    mcreq_queue_add_pipelines(cq, ppnew, nnew, newconfig);
    for (ii = 0; ii < nnew; ii++) {
        mcreq_iterwipe(cq, ppnew[ii], iterwipe_cb, NULL);
    }

    /**
     * Go through all the servers that are to be removed and relocate commands
     * from their queues into the new queues
     */
    for (ii = 0; ii < nold; ii++) {
        if (!ppold[ii]) {
            continue;
        }

        mcreq_iterwipe(cq, ppold[ii], iterwipe_cb, NULL);
        static_cast<lcb::Server*>(ppold[ii])->purge(LCB_MAP_CHANGED);
        static_cast<lcb::Server*>(ppold[ii])->close();
    }

    for (ii = 0; ii < nnew; ii++) {
        if (static_cast<lcb::Server*>(ppnew[ii])->has_pending()) {
            ppnew[ii]->flush_start(ppnew[ii]);
        }
    }

    free(ppnew);
    free(ppold);
}

void lcb_update_vbconfig(lcb_INSTANCE *instance, lcb_pCONFIGINFO config)
{
    lcb::clconfig::ConfigInfo *old_config = instance->cur_configinfo;
    mc_CMDQUEUE *q = &instance->cmdq;

    instance->cur_configinfo = config;
    config->incref();
    q->config = instance->cur_configinfo->vbc;
    q->cqdata = instance;

    if (old_config) {
        lcbvb_CONFIGDIFF *diff = lcbvb_compare(old_config->vbc, config->vbc);

        if (diff) {
            log_vbdiff(instance, diff);
            lcbvb_free_diff(diff);
        }

        /* Apply the vb guesses */
        lcb_vbguess_newconfig(instance, config->vbc, instance->vbguess);

        replace_config(instance, old_config->vbc, config->vbc);
        old_config->decref();
    } else {
        size_t nservers = VB_NSERVERS(config->vbc);
        std::vector<mc_PIPELINE*> servers;

        for (size_t ii = 0; ii < nservers; ii++) {
            servers.push_back(new lcb::Server(instance, ii));
        }

        mcreq_queue_add_pipelines(q, &servers[0], nservers, config->vbc);
    }

    /* Update the list of nodes here for server list */
    instance->ht_nodes->clear();
    for (size_t ii = 0; ii < LCBVB_NSERVERS(config->vbc); ++ii) {
        const char *hp = lcbvb_get_hostport(config->vbc, ii,
            LCBVB_SVCTYPE_MGMT, LCBT_SETTING_SVCMODE(instance));
        if (hp) {
            instance->ht_nodes->add(hp, LCB_CONFIG_HTTP_PORT);
        }
    }

    lcb_maybe_breakout(instance);
}
