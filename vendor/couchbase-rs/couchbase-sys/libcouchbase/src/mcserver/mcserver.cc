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

#include <cstring>

#include "internal.h"
#include "logging.h"
#include "vbucket/aliases.h"
#include "settings.h"
#include "negotiate.h"
#include "bucketconfig/clconfig.h"
#include "mc/mcreq-flush-inl.h"
#include <lcbio/ssl.h>
#include "ctx-log-inl.h"

#include "sllist.h"
#include "sllist-inl.h"

#define LOGARGS(c, lvl) (c)->settings, "server", LCB_LOG_##lvl, __FILE__, __LINE__
#define LOGARGS_T(lvl) LOGARGS(this, lvl)

#define LOGFMT CTX_LOGFMT_PRE ",SRV=%p,IX=%d) "
#define PKTFMT "OP=0x%x, RC=0x%x, SEQ=%u"
#define PKTARGS(pkt) (pkt).opcode(), (pkt).status(), (pkt).opaque()

#define LOGID(server) CTX_LOGID(server->connctx), (void *)server, server->index
#define LOGID_T() LOGID(this)

#define MCREQ_MAXIOV 32
#define LCBCONN_UNWANT(conn, flags) (conn)->want &= ~(flags)

using namespace lcb;

static void on_error(lcbio_CTX *ctx, lcb_STATUS err);

static void on_flush_ready(lcbio_CTX *ctx)
{
    Server *server = Server::get(ctx);
    nb_IOV iov[MCREQ_MAXIOV] = {};
    int ready;

    do {
        int niov = 0;
        unsigned nb;
        nb = mcreq_flush_iov_fill(server, iov, MCREQ_MAXIOV, &niov);
        if (!nb) {
            return;
        }
#ifdef LCB_DUMP_PACKETS
        {
            char *b64 = NULL;
            int nb64 = 0;
            lcb_base64_encode_iov((lcb_IOV *)iov, niov, nb, &b64, &nb64);
            lcb_log(LOGARGS(server, TRACE), LOGFMT "pkt,snd,fill: size=%d, %.*s", LOGID(server), nb64, nb64, b64);
            free(b64);
        }
#endif
        ready = lcbio_ctx_put_ex(ctx, (lcb_IOV *)iov, niov, nb);
    } while (ready);
    lcbio_ctx_wwant(ctx);
}

static void on_flush_done(lcbio_CTX *ctx, unsigned expected, unsigned actual)
{
    Server *server = Server::get(ctx);
    lcb_U64 now = 0;
    if (server->settings->readj_ts_wait) {
        now = gethrtime();
    }

#ifdef LCB_DUMP_PACKETS
    lcb_log(LOGARGS(server, TRACE), LOGFMT "pkt,snd,flush: expected=%u, actual=%u", LOGID(server), expected, actual);
#endif
    mcreq_flush_done_ex(server, actual, expected, now);
    server->check_closed();
}

void Server::flush()
{
    /** Call into the wwant stuff.. */
    if (!connctx->rdwant) {
        lcbio_ctx_rwant(connctx, 24);
    }

    lcbio_ctx_wwant(connctx);
    lcbio_ctx_schedule(connctx);

    if (!lcbio_timer_armed(io_timer)) {
        /**
         * XXX: Maybe use get_next_timeout(), although here we can assume
         * that a command was just scheduled
         */
        lcbio_timer_rearm(io_timer, default_timeout());
    }
}

LIBCOUCHBASE_API
void lcb_sched_flush(lcb_INSTANCE *instance)
{
    for (size_t ii = 0; ii < LCBT_NSERVERS(instance); ii++) {
        Server *server = instance->get_server(ii);

        if (!server->has_pending()) {
            continue;
        }
        server->flush_start(server);
    }
}

/**
 * Invoked when get a NOT_MY_VBUCKET response. If the response contains a JSON
 * payload then we refresh the configuration with it.
 *
 * This function returns 1 if the operation was successfully rescheduled;
 * otherwise it returns 0. If it returns 0 then we give the error back to the
 * user.
 */
bool Server::handle_nmv(MemcachedResponse &resinfo, mc_PACKET *oldpkt)
{
    protocol_binary_request_header hdr;
    lcb_STATUS err = LCB_ERROR;
    lcb_U16 vbid;
    lcb::clconfig::Provider *cccp = instance->confmon->get_provider(lcb::clconfig::CLCONFIG_CCCP);

    MC_INCR_METRIC(this, packets_nmv, 1);

    mcreq_read_hdr(oldpkt, &hdr);
    vbid = ntohs(hdr.request.vbucket);
    lcb_log(LOGARGS_T(WARN), LOGFMT "NOT_MY_VBUCKET. Packet=%p (S=%u). VBID=%u", LOGID_T(), (void *)oldpkt,
            oldpkt->opaque, vbid);

    /* Notify of new map */
    lcb_vbguess_remap(instance, vbid, index);

    if (resinfo.vallen() && cccp->enabled) {
        std::string s(resinfo.value(), resinfo.vallen());
        err = lcb::clconfig::cccp_update(cccp, curhost->host, s.c_str());
    }

    if (err != LCB_SUCCESS) {
        int bs_options;
        if (instance->cur_configinfo->get_origin() == lcb::clconfig::CLCONFIG_CCCP) {
            /**
             * XXX: Not enough to see if cccp was enabled, since cccp might
             * be requested by a user, but would still not actually be active
             * for clusters < 2.5 If our current config is from CCCP
             * then we can be fairly certain that CCCP is indeed working.
             *
             * For this reason, we don't use if (cccp->enabled) {...}
             */
            bs_options = BS_REFRESH_THROTTLE;
        } else {
            bs_options = BS_REFRESH_ALWAYS;
        }
        instance->bootstrap(bs_options);
    }

    if (!lcb_should_retry(settings, oldpkt, LCB_NOT_MY_VBUCKET)) {
        return false;
    }

    /** Reschedule the packet again .. */
    mc_PACKET *newpkt = mcreq_renew_packet(oldpkt);
    newpkt->flags &= ~MCREQ_STATE_FLAGS;
    instance->retryq->nmvadd((mc_EXPACKET *)newpkt);
    return true;
}

static lcb_STATUS reschedule_clone(const void *src, void **dst)
{
    *dst = (void *)src;
    return LCB_SUCCESS;
}

static lcb_STATUS reschedule_destroy(void *)
{
    return LCB_SUCCESS;
}

static lcb_STATUS reschedule_with_collection(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    /** Reschedule the packet again .. */
    mc_PACKET *newpkt = (mc_PACKET *)arg;
    newpkt->flags &= ~MCREQ_STATE_FLAGS;
    mcreq_set_cid(newpkt, cid);
    instance->retryq->ucadd((mc_EXPACKET *)newpkt);
    return LCB_SUCCESS;
}

bool Server::handle_unknown_collection(MemcachedResponse &, mc_PACKET *oldpkt)
{
    uint32_t cid = mcreq_get_cid(instance, oldpkt);
    lcb_log(LOGARGS_T(WARN), LOGFMT "UNKNOWN_COLLECTION. Packet=%p (S=%u), CID=%u", LOGID_T(), (void *)oldpkt,
            oldpkt->opaque, (unsigned)cid);
    std::string name = instance->collcache->id_to_name(cid);
    if (name.empty()) {
        return false;
    }
    instance->collcache->erase(cid);

    mc_PACKET *newpkt = mcreq_renew_packet(oldpkt);
    lcb_STATUS rc = collcache_exec_str(name, instance, NULL, reschedule_with_collection, reschedule_clone,
                                       reschedule_destroy, newpkt);
    return rc == LCB_SUCCESS;
}

/**
 * Determine if this is an error code that we can pass to the user, or can
 * otherwise handle "innately"
 */
static bool is_fastpath_error(uint16_t rc)
{
    switch (rc) {
        case PROTOCOL_BINARY_RESPONSE_SUCCESS:
        case PROTOCOL_BINARY_RESPONSE_KEY_ENOENT:
        case PROTOCOL_BINARY_RESPONSE_KEY_EEXISTS:
        case PROTOCOL_BINARY_RESPONSE_E2BIG:
        case PROTOCOL_BINARY_RESPONSE_NOT_STORED:
        case PROTOCOL_BINARY_RESPONSE_DELTA_BADVAL:
        case PROTOCOL_BINARY_RESPONSE_ERANGE:
        case PROTOCOL_BINARY_RESPONSE_NOT_SUPPORTED:
        case PROTOCOL_BINARY_RESPONSE_UNKNOWN_COMMAND:
        case PROTOCOL_BINARY_RESPONSE_ETMPFAIL:
        case PROTOCOL_BINARY_RESPONSE_ENOMEM:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_ENOENT:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_EEXISTS:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_MISMATCH:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_EINVAL:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_E2BIG:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_VALUE_CANTINSERT:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_VALUE_ETOODEEP:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_DOC_NOTJSON:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_NUM_ERANGE:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_DELTA_ERANGE:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_INVALID_COMBO:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_MULTI_PATH_FAILURE:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_SUCCESS_DELETED:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_INVALID_FLAG_COMBO:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_INVALID_KEY_COMBO:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_UNKNOWN_MACRO:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_UNKNOWN_VATTR:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_CANT_MODIFY_VATTR:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_MULTI_PATH_FAILURE_DELETED:
        case PROTOCOL_BINARY_RESPONSE_SUBDOC_INVALID_XATTR_ORDER:
        case PROTOCOL_BINARY_RESPONSE_EACCESS:
        case PROTOCOL_BINARY_RESPONSE_DURABILITY_INVALID_LEVEL:
        case PROTOCOL_BINARY_RESPONSE_DURABILITY_IMPOSSIBLE:
        case PROTOCOL_BINARY_RESPONSE_SYNC_WRITE_IN_PROGRESS:
        case PROTOCOL_BINARY_RESPONSE_SYNC_WRITE_AMBIGUOUS:
            return true;
        default:
            if (rc >= 0xc0 && rc <= 0xcc) {
                // other subdoc?
                return true;
            } else {
                return false;
            }
            break;
    }
}

#define ERRMAP_HANDLE_CONTINUE 0
#define ERRMAP_HANDLE_DISCONN 1
#define ERRMAP_HANDLE_RETRY 2

/**
 * Handle an unknown memcached error
 *
 * @param mcresp Response which contains the unknown error
 * @param[out] newerr more user-friendly based on error map attributes
 *
 * @return true if this function handled the error specially (by disconnecting)
 * or false if normal handling should continue.
 */
int Server::handle_unknown_error(const mc_PACKET *request, const MemcachedResponse &mcresp, lcb_STATUS &newerr)
{

    if (!settings->errmap->isLoaded() || !settings->use_errmap) {
        // If there's no error map, just return false
        return ERRMAP_HANDLE_CONTINUE;
    }

    // Look up the error map definition for this error
    const errmap::Error &err = settings->errmap->getError(mcresp.status());

    if (!err.isValid() || err.hasAttribute(errmap::SPECIAL_HANDLING)) {
        lcb_log(LOGARGS_T(ERR), LOGFMT "Received error not in error map or requires special handling! " PKTFMT,
                LOGID_T(), PKTARGS(mcresp));
        lcbio_ctx_senderr(connctx, LCB_PROTOCOL_ERROR);
        return ERRMAP_HANDLE_DISCONN;
    } else {
        lcb_log(LOGARGS_T(WARN), LOGFMT "Received server error %s (0x%x) on packet: " PKTFMT, LOGID_T(),
                err.shortname.c_str(), err.code, PKTARGS(mcresp));
    }

    if (err.hasAttribute(errmap::FETCH_CONFIG)) {
        instance->bootstrap(BS_REFRESH_THROTTLE);
    }

    if (err.hasAttribute(errmap::TEMPORARY)) {
        newerr = LCB_GENERIC_TMPERR;
    }

    if (err.hasAttribute(errmap::CONSTRAINT_FAILURE)) {
        newerr = LCB_GENERIC_CONSTRAINT_ERR;
    }

    if (err.hasAttribute(errmap::AUTH)) {
        newerr = LCB_AUTH_ERROR;
    }

    if (err.hasAttribute(errmap::SUBDOC) && newerr == LCB_SUCCESS) {
        newerr = LCB_GENERIC_SUBDOCERR;
    }

    /* TODO: remove masking LOCKED in 3.0 release */
    if (err.hasAttribute(errmap::ITEM_LOCKED)) {
        switch (mcresp.opcode()) {
            case PROTOCOL_BINARY_CMD_SET:
            case PROTOCOL_BINARY_CMD_REPLACE:
            case PROTOCOL_BINARY_CMD_DELETE:
                newerr = LCB_KEY_EEXISTS;
                break;
            default:
                newerr = LCB_ETMPFAIL;
        }
    }

    int rv = 0;

    if (err.hasAttribute(errmap::AUTO_RETRY)) {
        errmap::RetrySpec *spec = err.getRetrySpec();

        mc_PACKET *newpkt = mcreq_renew_packet(request);
        newpkt->flags &= ~MCREQ_STATE_FLAGS;
        instance->retryq->add((mc_EXPACKET *)newpkt, newerr ? newerr : LCB_ERROR, spec);
        rv |= ERRMAP_HANDLE_RETRY;
    }

    if (err.hasAttribute(errmap::CONN_STATE_INVALIDATED)) {
        if (newerr != LCB_SUCCESS) {
            newerr = LCB_ERROR;
        }
        lcbio_ctx_senderr(connctx, newerr);
        rv |= ERRMAP_HANDLE_DISCONN;
    }

    return rv;
}

/* This function is called within a loop to process a single packet.
 *
 * If a full packet is available, it will process the packet and return
 * PKT_READ_COMPLETE, resulting in the `on_read()` function calling this
 * function in a loop.
 *
 * When a complete packet is not available, PKT_READ_PARTIAL will be returned
 * and the `on_read()` loop will exit, scheduling any required pending I/O.
 */
Server::ReadState Server::try_read(lcbio_CTX *ctx, rdb_IOROPE *ior)
{
    MemcachedResponse mcresp;
    mc_PACKET *request;
    unsigned pktsize = 24, is_last = 1;

#define RETURN_NEED_MORE(n)                                                                                            \
    if (has_pending()) {                                                                                               \
        lcbio_ctx_rwant(ctx, n);                                                                                       \
    }                                                                                                                  \
    return PKT_READ_PARTIAL;

#define DO_ASSIGN_PAYLOAD()                                                                                            \
    rdb_consumed(ior, mcresp.hdrsize());                                                                               \
    if (mcresp.bodylen()) {                                                                                            \
        mcresp.payload = rdb_get_consolidated(ior, mcresp.bodylen());                                                  \
    }                                                                                                                  \
    {

#define DO_SWALLOW_PAYLOAD()                                                                                           \
    }                                                                                                                  \
    if (mcresp.bodylen()) {                                                                                            \
        rdb_consumed(ior, mcresp.bodylen());                                                                           \
    }

    if (rdb_get_nused(ior) < pktsize) {
        RETURN_NEED_MORE(pktsize)
    }

    MC_INCR_METRIC(this, packets_read, 1);

    /* copy bytes into the info structure */
    rdb_copyread(ior, mcresp.hdrbytes(), mcresp.hdrsize());

    pktsize += mcresp.bodylen();
    if (rdb_get_nused(ior) < pktsize) {
        RETURN_NEED_MORE(pktsize);
    }

    /* Find the packet */
    if (mcresp.opcode() == PROTOCOL_BINARY_CMD_STAT && mcresp.keylen() != 0) {
        is_last = 0;
        request = mcreq_pipeline_find(this, mcresp.opaque());
    } else {
        is_last = 1;
        request = mcreq_pipeline_remove(this, mcresp.opaque());
    }

    if (!request) {
        MC_INCR_METRIC(this, packets_ownerless, 1);
        lcb_log(LOGARGS_T(DEBUG), LOGFMT "Server sent us reply for a timed-out command. (OP=0x%x, RC=0x%x, SEQ=%u)",
                LOGID_T(), mcresp.opcode(), mcresp.status(), mcresp.opaque());
        rdb_consumed(ior, pktsize);
        return PKT_READ_COMPLETE;
    }

    lcb_STATUS err_override = LCB_SUCCESS;
    ReadState rdstate = PKT_READ_COMPLETE;
    int unknown_err_rv;

    /* Check if the status code is one which must be handled carefully by the
     * client */
    if (is_fastpath_error(mcresp.status())) {
        // Nothing here!
    } else if (mcresp.status() == PROTOCOL_BINARY_RESPONSE_NOT_MY_VBUCKET) {
        /* consume the header */
        DO_ASSIGN_PAYLOAD()
        if (!handle_nmv(mcresp, request)) {
            mcreq_dispatch_response(this, request, &mcresp, LCB_NOT_MY_VBUCKET);
        }
        DO_SWALLOW_PAYLOAD()
        goto GT_DONE;
    } else if (mcresp.status() == PROTOCOL_BINARY_RESPONSE_UNKNOWN_COLLECTION) {
        /* consume the header */
        DO_ASSIGN_PAYLOAD()
        handle_unknown_collection(mcresp, request);
        DO_SWALLOW_PAYLOAD()
        goto GT_DONE;
    } else if ((unknown_err_rv = handle_unknown_error(request, mcresp, err_override)) != ERRMAP_HANDLE_CONTINUE) {
        DO_ASSIGN_PAYLOAD()
        if (!(unknown_err_rv & ERRMAP_HANDLE_RETRY)) {
            mcreq_dispatch_response(this, request, &mcresp, err_override);
        }
        DO_SWALLOW_PAYLOAD()
        if (unknown_err_rv & ERRMAP_HANDLE_DISCONN) {
            rdstate = PKT_READ_ABORT;
        }
        goto GT_DONE;
    }

    /* Figure out if the request is 'ufwd' or not */
    if (!(request->flags & MCREQ_F_UFWD)) {
        DO_ASSIGN_PAYLOAD();
        mcresp.bufh = rdb_get_first_segment(ior);
        mcreq_dispatch_response(this, request, &mcresp, err_override);
        DO_SWALLOW_PAYLOAD()

    } else {
        /* figure out how many buffers we want to use as an upper limit for the
         * IOV arrays. Currently we'll keep it simple and ensure the entire
         * response is contiguous. */
        lcb_PKTFWDRESP resp = {0}; /* TODO: next ABI version should include is_last flag */
        rdb_ROPESEG *segs;
        nb_IOV iov;

        rdb_consolidate(ior, pktsize);
        rdb_refread_ex(ior, &iov, &segs, 1, pktsize);

        resp.bufs = &segs;
        resp.iovs = (lcb_IOV *)&iov;
        resp.nitems = 1;
        resp.header = mcresp.hdrbytes();
        instance->callbacks.pktfwd(instance, MCREQ_PKT_COOKIE(request), LCB_SUCCESS, &resp);
        rdb_consumed(ior, pktsize);
    }

GT_DONE:
    if (is_last) {
        mcreq_packet_handled(this, request);
    }
    return rdstate;
}

static void on_read(lcbio_CTX *ctx, unsigned)
{
    Server *server = Server::get(ctx);
    rdb_IOROPE *ior = &ctx->ior;

    if (server->check_closed()) {
        return;
    }

    Server::ReadState rv;
    while ((rv = server->try_read(ctx, ior)) == Server::PKT_READ_COMPLETE)
        ;
    lcbio_ctx_schedule(ctx);
    lcb_maybe_breakout(server->instance);
}

static void flush_noop(mc_PIPELINE *pipeline)
{
    (void)pipeline;
}

static void server_connect(Server *server)
{
    server->connect();
}

bool Server::maybe_retry_packet(mc_PACKET *pkt, lcb_STATUS err)
{
    lcbvb_DISTMODE dist_t = lcbvb_get_distmode(parent->config);

    if (dist_t != LCBVB_DIST_VBUCKET) {
        /** memcached bucket */
        return false;
    }
    if (!lcb_should_retry(settings, pkt, err)) {
        return false;
    }

    mc_PACKET *newpkt = mcreq_renew_packet(pkt);
    newpkt->flags &= ~MCREQ_STATE_FLAGS;
    // TODO: Load the 4th argument from the error map
    instance->retryq->add((mc_EXPACKET *)newpkt, err, NULL);
    return true;
}

static void fail_callback(mc_PIPELINE *pipeline, mc_PACKET *pkt, lcb_STATUS err, void *)
{
    static_cast< Server * >(pipeline)->purge_single(pkt, err);
}

static const char *opcode_name(uint8_t code)
{
    switch (code) {
        case PROTOCOL_BINARY_CMD_GET:
            return "get";
        case PROTOCOL_BINARY_CMD_SET:
            return "set";
        case PROTOCOL_BINARY_CMD_ADD:
            return "add";
        case PROTOCOL_BINARY_CMD_REPLACE:
            return "replace";
        case PROTOCOL_BINARY_CMD_DELETE:
            return "delete";
        case PROTOCOL_BINARY_CMD_INCREMENT:
            return "incr";
        case PROTOCOL_BINARY_CMD_DECREMENT:
            return "decr";
        case PROTOCOL_BINARY_CMD_FLUSH:
            return "flush";
        case PROTOCOL_BINARY_CMD_GETQ:
            return "getq";
        case PROTOCOL_BINARY_CMD_NOOP:
            return "noop";
        case PROTOCOL_BINARY_CMD_VERSION:
            return "version";
        case PROTOCOL_BINARY_CMD_APPEND:
            return "append";
        case PROTOCOL_BINARY_CMD_PREPEND:
            return "prepend";
        case PROTOCOL_BINARY_CMD_STAT:
            return "stat";
        case PROTOCOL_BINARY_CMD_VERBOSITY:
            return "verbosity";
        case PROTOCOL_BINARY_CMD_TOUCH:
            return "touch";
        case PROTOCOL_BINARY_CMD_GAT:
            return "gat";
        case PROTOCOL_BINARY_CMD_HELLO:
            return "hello";
        case PROTOCOL_BINARY_CMD_SASL_LIST_MECHS:
            return "sasl_list_mechs";
        case PROTOCOL_BINARY_CMD_SASL_AUTH:
            return "sasl_auth";
        case PROTOCOL_BINARY_CMD_SASL_STEP:
            return "sasl_step";
        case PROTOCOL_BINARY_CMD_GET_REPLICA:
            return "get_replica";
        case PROTOCOL_BINARY_CMD_SELECT_BUCKET:
            return "select_bucket";
        case PROTOCOL_BINARY_CMD_OBSERVE_SEQNO:
            return "observe_seqno";
        case PROTOCOL_BINARY_CMD_OBSERVE:
            return "observe";
        case PROTOCOL_BINARY_CMD_GET_LOCKED:
            return "get_locked";
        case PROTOCOL_BINARY_CMD_UNLOCK_KEY:
            return "unlock_key";
        case PROTOCOL_BINARY_CMD_GET_CLUSTER_CONFIG:
            return "get_cluster_config";
        case PROTOCOL_BINARY_CMD_SUBDOC_GET:
            return "subdoc_get";
        case PROTOCOL_BINARY_CMD_SUBDOC_EXISTS:
            return "subdoc_exists";
        case PROTOCOL_BINARY_CMD_SUBDOC_DICT_ADD:
            return "subdoc_dict_add";
        case PROTOCOL_BINARY_CMD_SUBDOC_DICT_UPSERT:
            return "subdoc_dict_upsert";
        case PROTOCOL_BINARY_CMD_SUBDOC_DELETE:
            return "subdoc_delete";
        case PROTOCOL_BINARY_CMD_SUBDOC_REPLACE:
            return "subdoc_replace";
        case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_LAST:
            return "subdoc_array_push_last";
        case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_FIRST:
            return "subdoc_array_push_first";
        case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_INSERT:
            return "subdoc_array_insert";
        case PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_ADD_UNIQUE:
            return "subdoc_array_add_unique";
        case PROTOCOL_BINARY_CMD_SUBDOC_COUNTER:
            return "subdoc_counter";
        case PROTOCOL_BINARY_CMD_SUBDOC_MULTI_LOOKUP:
            return "subdoc_multi_lookup";
        case PROTOCOL_BINARY_CMD_SUBDOC_MULTI_MUTATION:
            return "subdoc_multi_mutation";
        case PROTOCOL_BINARY_CMD_SUBDOC_GET_COUNT:
            return "subdoc_get_count";
        case PROTOCOL_BINARY_CMD_GET_ERROR_MAP:
            return "get_error_map";
        default:
            return "unknown";
    }
}

void Server::purge_single(mc_PACKET *pkt, lcb_STATUS err)
{
    if (maybe_retry_packet(pkt, err)) {
        return;
    }

    if (err == LCB_AUTH_ERROR) {
        /* In-situ auth errors are actually dead servers. Let's provide this
         * as the actual error code. */
        err = LCB_MAP_CHANGED;
    }

    if (err == LCB_ETIMEDOUT) {
        lcb_STATUS tmperr = lcb::RetryQueue::error_for(pkt);
        if (tmperr != LCB_SUCCESS) {
            err = tmperr;
        }
    }

    protocol_binary_request_header hdr;
    memcpy(hdr.bytes, SPAN_BUFFER(&pkt->kh_span), sizeof(hdr.bytes));
    MemcachedResponse resp(protocol_binary_command(hdr.request.opcode), hdr.request.opaque,
                           PROTOCOL_BINARY_RESPONSE_EINVAL);

    lcbtrace_span_set_orphaned(MCREQ_PKT_RDATA(pkt)->span, true);
    if (err == LCB_ETIMEDOUT && settings->use_tracing) {
        Json::Value info;

        char opid[30] = {};
        snprintf(opid, sizeof(opid), "kv:%s", opcode_name(hdr.request.opcode));
        info["s"] = opid;
        info["b"] = settings->bucket;
        info["t"] = (Json::UInt64)LCB_NS2US(MCREQ_PKT_RDATA(pkt)->deadline - MCREQ_PKT_RDATA(pkt)->start);

        const lcb_host_t &remote = get_host();
        std::string rhost;
        if (remote.ipv6) {
            rhost.append("[").append(remote.host).append("]:").append(remote.port);
        } else {
            rhost.append(remote.host).append(":").append(remote.port);
        }
        info["r"] = rhost.c_str();

        if (connctx) {
            char local_id[54] = {};
            snprintf(local_id, sizeof(local_id), "%016" PRIx64 "/%016" PRIx64 "/%x", settings->iid, connctx->sock->id,
                     (int)pkt->opaque);
            info["i"] = local_id;
            info["l"] = lcbio__inet_ntop(&connctx->sock->info->sa_local).c_str();
        }
        std::string msg(Json::FastWriter().write(info));
        if (msg.size() > 1) {
            lcb_log(LOGARGS(instance, WARN), "Failing command with error %s: %.*s", lcb_strerror_short(err),
                    (int)(msg.size() - 1), msg.c_str());
        }
    } else {
        lcb_log(LOGARGS_T(WARN), LOGFMT "Failing command (pkt=%p, opaque=%lu, opcode=0x%x) with error %s", LOGID_T(),
                (void *)pkt, (unsigned long)pkt->opaque, hdr.request.opcode, lcb_strerror_short(err));
    }
    int rv = mcreq_dispatch_response(this, pkt, &resp, err);
    lcb_assert(rv == 0);
}

int Server::purge(lcb_STATUS error, hrtime_t now, RefreshPolicy policy)
{
    unsigned affected;

    if (now) {
        affected = mcreq_pipeline_timeout(this, error, fail_callback, NULL, now);

    } else {
        mcreq_pipeline_fail(this, error, fail_callback, NULL);
        affected = -1;
    }

    MC_INCR_METRIC(this, packets_errored, affected);
    if (policy == REFRESH_NEVER) {
        return affected;
    }

    if (affected || policy == REFRESH_ALWAYS) {
        instance->bootstrap(BS_REFRESH_THROTTLE | BS_REFRESH_INCRERR);
    }
    return affected;
}

static void flush_errdrain(mc_PIPELINE *pipeline)
{
    /* Called when we are draining errors. */
    Server *server = (Server *)pipeline;
    if (!lcbio_timer_armed(server->io_timer)) {
        lcbio_timer_rearm(server->io_timer, server->default_timeout());
    }
}

uint32_t Server::next_timeout() const
{
    hrtime_t now, expiry, diff, min = 0;
    mc_PACKET *pkt = NULL;

    sllist_iterator iter;
    SLLIST_ITERFOR(const_cast<sllist_root *>(&requests), &iter)
    {
        mc_PACKET *p = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
        hrtime_t deadline = MCREQ_PKT_RDATA(p)->deadline;
        if (pkt == NULL) {
            min = deadline;
            pkt = p;
        } else if (deadline < min) {
            min = deadline;
            pkt = p;
        }
    }

    if (!pkt) {
        return default_timeout();
    }

    now = gethrtime();
    expiry = MCREQ_PKT_RDATA(pkt)->deadline;
    if (expiry <= now) {
        diff = 0;
    } else {
        diff = expiry - now;
    }

    return LCB_NS2US(diff);
}

void mcreq_rearm_timeout(mc_PIPELINE *pipeline)
{
    if ((unsigned)pipeline->index == pipeline->parent->npipelines) {
        return; /* this is fallback pipeline, skip it */
    }
    Server *server = reinterpret_cast<Server *>(pipeline);
    if (server->io_timer) {
        lcbio_timer_rearm(server->io_timer, server->next_timeout());
    }
}

static void timeout_server(void *arg)
{
    reinterpret_cast< Server * >(arg)->io_timeout();
}

void Server::io_timeout()
{
    hrtime_t now = gethrtime();

    int npurged = purge(LCB_ETIMEDOUT, now, Server::REFRESH_ONFAILED);
    if (npurged) {
        MC_INCR_METRIC(this, packets_timeout, npurged);
        lcb_log(LOGARGS_T(DEBUG), LOGFMT "Server timed out. Some commands have failed", LOGID_T());
    }

    uint32_t next_us = next_timeout();
    lcb_log(LOGARGS_T(TRACE), LOGFMT "Scheduling next timeout for %u ms. This is not an error", LOGID_T(),
            next_us / 1000);
    lcbio_timer_rearm(io_timer, next_us);
    lcb_maybe_breakout(instance);
}

bool Server::maybe_reconnect_on_fake_timeout(lcb_STATUS err)
{
    if (err != LCB_ETIMEDOUT) {
        return false; /* not a timeout */
    }
    if (!settings->readj_ts_wait) {
        return false; /* normal timeout behavior */
    }
    if (!has_pending()) {
        return false; /* nothing pending */
    }

    uint32_t next_tmo = next_timeout();
    if (next_tmo < default_timeout() / 2) {
        /* Ideally we'd have a fuzz interval to shave off the actual timeout,
         * since there will inevitably be some time taken off the next timeout */
        return false;
    }

    lcb_log(LOGARGS_T(INFO), LOGFMT "Retrying connection. Assuming timeout because of stalled event loop", LOGID_T());
    connect();
    return true;
}

static void on_connected(lcbio_SOCKET *sock, void *data, lcb_STATUS err, lcbio_OSERR syserr)
{
    Server *server = reinterpret_cast< Server * >(data);
    server->handle_connected(sock, err, syserr);
}

static void mcserver_flush(Server *s)
{
    s->flush();
}

void Server::handle_connected(lcbio_SOCKET *sock, lcb_STATUS err, lcbio_OSERR syserr)
{
    connreq = NULL;

    if (err != LCB_SUCCESS) {
        lcb_log(LOGARGS_T(ERR),
                LOGFMT "Connection attempt failed. Received %s from libcouchbase, received %d from operating system",
                LOGID_T(), lcb_strerror_short(err), syserr);
        MC_INCR_METRIC(this, iometrics.io_error, 1);
        if (!maybe_reconnect_on_fake_timeout(err)) {
            socket_failed(err);
        }
        return;
    }

    lcb_assert(sock);
    if (metrics) {
        lcbio_set_metrics(sock, &metrics->iometrics);
    }

    /** Do we need sasl? */
    SessionInfo *sessinfo = SessionInfo::get(sock);
    if (sessinfo == NULL) {
        lcb_log(LOGARGS_T(TRACE), "<%s:%s> (SRV=%p) Session not yet negotiated. Negotiating", curhost->host,
                curhost->port, (void *)this);
        connreq = SessionRequest::start(sock, settings, settings->config_node_timeout, on_connected, this);
        return;
    } else {
        jsonsupport = sessinfo->has_feature(PROTOCOL_BINARY_FEATURE_JSON);
        compsupport = sessinfo->has_feature(PROTOCOL_BINARY_FEATURE_SNAPPY);
        mutation_tokens = sessinfo->has_feature(PROTOCOL_BINARY_FEATURE_MUTATION_SEQNO);
        new_durability = sessinfo->has_feature(PROTOCOL_BINARY_FEATURE_SYNC_REPLICATION) &&
                         sessinfo->has_feature(PROTOCOL_BINARY_FEATURE_ALT_REQUEST_SUPPORT);
    }

    lcbio_CTXPROCS procs;
    procs.cb_err = on_error;
    procs.cb_read = on_read;
    procs.cb_flush_done = on_flush_done;
    procs.cb_flush_ready = on_flush_ready;
    connctx = lcbio_ctx_new(sock, this, &procs);
    connctx->subsys = "memcached";
    sock->service = LCBIO_SERVICE_KV;
    flush_start = (mcreq_flushstart_fn)mcserver_flush;

    uint32_t tmo = next_timeout();
    lcbio_timer_rearm(io_timer, tmo);
    flush();
}

void Server::connect()
{
    connreq = instance->memd_sockpool->get(*curhost, default_timeout(), on_connected, this);
    flush_start = flush_noop;
    state = Server::S_CLEAN;
}

static void buf_done_cb(mc_PIPELINE *pl, const void *cookie, void *, void *)
{
    Server *server = static_cast< Server * >(pl);
    server->instance->callbacks.pktflushed(server->instance, cookie);
}

Server::Server(lcb_INSTANCE *instance_, int ix)
    : mc_PIPELINE(), state(S_CLEAN), io_timer(lcbio_timer_new(instance_->iotable, this, timeout_server)),
      instance(instance_), settings(lcb_settings_ref2(instance_->settings)), compsupport(0), jsonsupport(0),
      mutation_tokens(0), new_durability(-1), connctx(NULL), curhost(new lcb_host_t())
{
    mcreq_pipeline_init(this);
    flush_start = (mcreq_flushstart_fn)server_connect;
    buf_done_callback = buf_done_cb;
    index = ix;

    std::memset(&connreq, 0, sizeof connreq);
    std::memset(curhost, 0, sizeof *curhost);

    const char *datahost =
        lcbvb_get_hostport(LCBT_VBCONFIG(instance), ix, LCBVB_SVCTYPE_DATA, LCBT_SETTING_SVCMODE(instance));
    if (datahost) {
        lcb_host_parsez(curhost, datahost, LCB_CONFIG_MCD_PORT);
    }

    if (settings->metrics) {
        /** Allocate / reinitialize the metrics here */
        metrics = lcb_metrics_getserver(settings->metrics, curhost->host, curhost->port, 1);
        lcb_metrics_reset_pipeline_gauges(metrics);
    }
}

Server::Server()
    : state(S_TEMPORARY), io_timer(NULL), instance(NULL), settings(NULL), compsupport(0), jsonsupport(0),
      mutation_tokens(0), connctx(NULL), connreq(NULL), curhost(NULL)
{
}

Server::~Server()
{
    if (state == S_TEMPORARY) {
        return;
    }

    if (this->instance) {
        unsigned ii;
        mc_CMDQUEUE *cmdq = &this->instance->cmdq;
        for (ii = 0; ii < cmdq->npipelines; ii++) {
            lcb::Server *server = static_cast< lcb::Server * >(cmdq->pipelines[ii]);
            if (server == this) {
                cmdq->pipelines[ii] = NULL;
                break;
            }
        }
    }
    this->instance = NULL;
    mcreq_pipeline_cleanup(this);

    if (io_timer) {
        lcbio_timer_destroy(io_timer);
    }

    delete curhost;
    lcb_settings_unref(settings);
}

static void close_cb(lcbio_SOCKET *sock, int, void *)
{
    lcbio_ref(sock);
    lcb::io::Pool::discard(sock);
}

static void on_error(lcbio_CTX *ctx, lcb_STATUS err)
{
    Server *server = Server::get(ctx);
    lcb_log(LOGARGS(server, WARN), LOGFMT "Got socket error %s", LOGID(server), lcb_strerror_short(err));
    if (server->check_closed()) {
        return;
    }
    server->socket_failed(err);
}

/**Handle a socket error. This function will close the current connection
 * and trigger a failout of any pending commands.
 * This function triggers a configuration refresh */
void Server::socket_failed(lcb_STATUS err)
{
    if (check_closed()) {
        return;
    }

    purge(err, 0, REFRESH_ALWAYS);
    lcb_maybe_breakout(instance);
    start_errored_ctx(S_ERRDRAIN);
}

void Server::close()
{
    /* Should never be called twice */
    lcb_assert(state != Server::S_CLOSED);
    start_errored_ctx(S_CLOSED);
}

/**
 * Call to signal an error or similar on the current socket.
 * @param server The server
 * @param next_state The next state (S_CLOSED or S_ERRDRAIN)
 */
void Server::start_errored_ctx(State next_state)
{
    lcbio_CTX *ctx = connctx;

    state = next_state;
    /* Cancel any pending connection attempt? */
    lcb::io::ConnectionRequest::cancel(&connreq);

    /* If the server is being destroyed, silence the timer */
    if (next_state == Server::S_CLOSED && io_timer != NULL) {
        lcbio_timer_destroy(io_timer);
        io_timer = NULL;
    }

    if (ctx == NULL) {
        if (next_state == Server::S_CLOSED) {
            delete this;
            return;
        } else {
            /* Not closed but don't have a current context */
            if (has_pending()) {
                if (!lcbio_timer_armed(io_timer)) {
                    /* TODO: Maybe throttle reconnection attempts? */
                    lcbio_timer_rearm(io_timer, default_timeout());
                }
                connect();
            } else {
                // Connect once someone actually wants a connection.
                flush_start = (mcreq_flushstart_fn)server_connect;
            }
        }

    } else {
        if (ctx->npending) {
            /* Have pending items? */

            /* Flush any remaining events */
            lcbio_ctx_schedule(ctx);

            /* Close the socket not to leak resources */
            lcbio_shutdown(lcbio_ctx_sock(ctx));
            if (next_state == Server::S_ERRDRAIN) {
                flush_start = (mcreq_flushstart_fn)flush_errdrain;
            }
        } else {
            finalize_errored_ctx();
        }
    }
}

/**
 * This function actually finalizes a ctx which has an error on it. If the
 * ctx has pending operations remaining then this function returns immediately.
 * Otherwise this will either reinitialize the connection or free the server
 * object depending on the actual object state (i.e. if it was closed or
 * simply errored).
 */
void Server::finalize_errored_ctx()
{
    if (connctx->npending) {
        return;
    }

    lcb_log(LOGARGS_T(DEBUG), LOGFMT "Finalizing context", LOGID_T());

    /* Always close the existing context. */
    lcbio_ctx_close(connctx, close_cb, NULL);
    connctx = NULL;

    /**Marks any unflushed data inside this server as being already flushed. This
     * should be done within error handling. If subsequent data is flushed on this
     * pipeline to the same connection, the results are undefined. */

    unsigned toflush;
    nb_IOV iov;
    while ((toflush = mcreq_flush_iov_fill(this, &iov, 1, NULL))) {
        mcreq_flush_done(this, toflush, toflush);
    }

    if (state == Server::S_CLOSED) {
        /* If the server is closed, time to free it */
        delete this;
    } else {
        /* Otherwise, cycle the state back to CLEAN and reinit
         * the connection */
        state = Server::S_CLEAN;
        connect();
    }
}

/**
 * This little function checks to see if the server struct is still valid, or
 * whether it should just be cleaned once no pending I/O remainds.
 *
 * If this function returns false then the server is still valid; otherwise it
 * is invalid and must not be used further.
 */
bool Server::check_closed()
{
    if (state == Server::S_CLEAN) {
        return false;
    }
    lcb_log(LOGARGS_T(INFO), LOGFMT "Got handler after close. Checking pending calls (pending=%u)", LOGID_T(),
            connctx->npending);
    finalize_errored_ctx();
    return 1;
}
