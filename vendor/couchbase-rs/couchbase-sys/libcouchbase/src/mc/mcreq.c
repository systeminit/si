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

#include "mcreq.h"
#include "compress.h"
#include "sllist-inl.h"
#include "internal.h"

#define PKT_HDRSIZE(pkt) (MCREQ_PKT_BASESIZE + (pkt)->extlen)

lcb_STATUS mcreq_reserve_header(mc_PIPELINE *pipeline, mc_PACKET *packet, uint8_t hdrsize)
{
    int rv;
    packet->extlen = hdrsize - MCREQ_PKT_BASESIZE;
    packet->kh_span.size = hdrsize;
    rv = netbuf_mblock_reserve(&pipeline->nbmgr, &packet->kh_span);
    if (rv != 0) {
        return LCB_CLIENT_ENOMEM;
    }
    return LCB_SUCCESS;
}

int leb128_encode(uint32_t value, uint8_t *buf)
{
    int idx = 0;
    if (value == 0) {
        buf[0] = 0;
        return 1;
    }
    while (value > 0) {
        uint8_t byte = (uint8_t)(value & 0x7f);
        value >>= 7;
        if (value > 0) {
            byte |= 0x80;
        }
        buf[idx++] = byte;
    }
    return idx;
}

int leb128_decode(uint8_t *buf, size_t nbuf, uint32_t *result)
{
    uint32_t value = buf[0] & 0x7f;
    size_t idx = 1;
    if (buf[0] & 0x80) {
        int shift = 7;
        for (; idx < nbuf; idx++) {
            value |= (buf[idx] & 0x7f) << shift;
            if ((buf[idx] & 0x80) == 0) {
                break;
            }
            shift += 7;
        }
        if (idx == nbuf) {
            *result = 0;
            return 0;
        }
    }
    *result = value;
    return (int)idx;
}

lcb_STATUS mcreq_reserve_key(mc_PIPELINE *pipeline, mc_PACKET *packet, uint8_t hdrsize, const lcb_KEYBUF *kreq,
                             uint32_t collection_id)
{
    const struct lcb_CONTIGBUF *contig = &kreq->contig;
    lcb_KVBUFTYPE buftype = kreq->type;
    lcb_INSTANCE *instance = (lcb_INSTANCE *)pipeline->parent->cqdata;
    int rv;
    uint8_t ncid = 0;
    uint8_t cid[5] = {0};

    /** encode collection ID */
    if ((packet->flags & MCREQ_F_NOCID) == 0 && instance && LCBT_SETTING(instance, use_collections)) {
        ncid = leb128_encode(collection_id, cid);
    }
    /** Set the key offset which is the start of the key from the buffer */
    packet->extlen = hdrsize - MCREQ_PKT_BASESIZE;
    packet->kh_span.size = kreq->contig.nbytes;

    if (buftype == LCB_KV_COPY) {
        /**
         * If the key is to be copied then just allocate the span size
         * for the key+24+extras
         */
        packet->kh_span.size += hdrsize + ncid;
        rv = netbuf_mblock_reserve(&pipeline->nbmgr, &packet->kh_span);
        if (rv != 0) {
            return LCB_CLIENT_ENOMEM;
        }
        /* copy collection ID prefix */
        if (ncid) {
            memcpy(SPAN_BUFFER(&packet->kh_span) + hdrsize, cid, ncid);
        }
        /**
         * Copy the key into the packet starting at the extras end
         */
        memcpy(SPAN_BUFFER(&packet->kh_span) + hdrsize + ncid, contig->bytes, contig->nbytes);

    } else if (buftype == LCB_KV_CONTIG) {
        /**
         * Don't do any copying.
         * Assume the key buffer has enough space for the packet as well.
         */
        CREATE_STANDALONE_SPAN(&packet->kh_span, contig->bytes, contig->nbytes);
        packet->flags |= MCREQ_F_KEY_NOCOPY;

    } else {
        /** IOVs not supported for keys */
        return LCB_EINVAL;
    }

    return LCB_SUCCESS;
}

lcb_STATUS mcreq_reserve_value2(mc_PIPELINE *pl, mc_PACKET *pkt, lcb_size_t n)
{
    int rv;
    pkt->u_value.single.size = n;
    if (!n) {
        return LCB_SUCCESS;
    }

    pkt->flags |= MCREQ_F_HASVALUE;
    rv = netbuf_mblock_reserve(&pl->nbmgr, &pkt->u_value.single);
    if (rv) {
        return LCB_CLIENT_ENOMEM;
    }
    return LCB_SUCCESS;
}

lcb_STATUS mcreq_reserve_value(mc_PIPELINE *pipeline, mc_PACKET *packet, const lcb_VALBUF *vreq)
{
    const lcb_CONTIGBUF *contig = &vreq->u_buf.contig;
    nb_SPAN *vspan = &packet->u_value.single;
    int rv;

    if (vreq->vtype == LCB_KV_COPY) {
        /** Copy the value into a single SPAN */
        if (!(vspan->size = vreq->u_buf.contig.nbytes)) {
            return LCB_SUCCESS;
        }
        rv = netbuf_mblock_reserve(&pipeline->nbmgr, vspan);

        if (rv != 0) {
            return LCB_CLIENT_ENOMEM;
        }

        memcpy(SPAN_BUFFER(vspan), contig->bytes, contig->nbytes);

    } else if (vreq->vtype == LCB_KV_CONTIG) {
        /** It's still contiguous so make it a 'standalone' span */
        CREATE_STANDALONE_SPAN(vspan, contig->bytes, contig->nbytes);
        packet->flags |= MCREQ_F_VALUE_NOCOPY;

    } else if (vreq->vtype == LCB_KV_IOV) {
        /** Multiple spans, no copy */
        unsigned int ii;
        const lcb_FRAGBUF *msrc = &vreq->u_buf.multi;
        lcb_FRAGBUF *mdst = &packet->u_value.multi;

        packet->flags |= MCREQ_F_VALUE_IOV | MCREQ_F_VALUE_NOCOPY;
        mdst->niov = msrc->niov;
        mdst->iov = malloc(mdst->niov * sizeof(*mdst->iov));
        mdst->total_length = 0;

        for (ii = 0; ii < mdst->niov; ii++) {
            mdst->iov[ii] = msrc->iov[ii];
            mdst->total_length += mdst->iov[ii].iov_len;
        }
    } else if (vreq->vtype == LCB_KV_IOVCOPY) {
        /** Multiple input buffers, normal copying output buffer */
        unsigned int ii, cur_offset;
        const lcb_FRAGBUF *msrc = &vreq->u_buf.multi;

        if (msrc->total_length) {
            vspan->size = msrc->total_length;
        } else {
            vspan->size = 0;
            for (ii = 0; ii < msrc->niov; ii++) {
                vspan->size += msrc->iov[ii].iov_len;
            }
        }

        rv = netbuf_mblock_reserve(&pipeline->nbmgr, vspan);
        if (rv != 0) {
            return LCB_CLIENT_ENOMEM;
        }

        for (ii = 0, cur_offset = 0; ii < msrc->niov; ii++) {
            char *buf = SPAN_BUFFER(vspan) + cur_offset;
            memcpy(buf, msrc->iov[ii].iov_base, msrc->iov[ii].iov_len);
            cur_offset += msrc->iov[ii].iov_len;
        }
    }

    packet->flags |= MCREQ_F_HASVALUE;
    return LCB_SUCCESS;
}

static int pkt_tmo_compar(sllist_node *a, sllist_node *b)
{
    mc_PACKET *pa, *pb;
    hrtime_t tmo_a, tmo_b;

    pa = SLLIST_ITEM(a, mc_PACKET, slnode);
    pb = SLLIST_ITEM(b, mc_PACKET, slnode);

    tmo_a = MCREQ_PKT_RDATA(pa)->start;
    tmo_b = MCREQ_PKT_RDATA(pb)->start;

    if (tmo_a == tmo_b) {
        return 0;
    } else if (tmo_a < tmo_b) {
        return -1;
    } else {
        return 1;
    }
}

void mcreq_reenqueue_packet(mc_PIPELINE *pipeline, mc_PACKET *packet)
{
    sllist_root *reqs = &pipeline->requests;
    mcreq_enqueue_packet(pipeline, packet);
    sllist_remove(reqs, &packet->slnode);
    sllist_insert_sorted(reqs, &packet->slnode, pkt_tmo_compar);
}

void mcreq_enqueue_packet(mc_PIPELINE *pipeline, mc_PACKET *packet)
{
    nb_SPAN *vspan = &packet->u_value.single;
    sllist_append(&pipeline->requests, &packet->slnode);
    netbuf_enqueue_span(&pipeline->nbmgr, &packet->kh_span, packet);
    MC_INCR_METRIC(pipeline, bytes_queued, packet->kh_span.size);

    if (!(packet->flags & MCREQ_F_HASVALUE)) {
        goto GT_ENQUEUE_PDU;
    }

    if (packet->flags & MCREQ_F_VALUE_IOV) {
        unsigned int ii;
        lcb_FRAGBUF *multi = &packet->u_value.multi;
        for (ii = 0; ii < multi->niov; ii++) {
            netbuf_enqueue(&pipeline->nbmgr, (nb_IOV *)multi->iov + ii, packet);
            MC_INCR_METRIC(pipeline, bytes_queued, multi->iov[ii].iov_len);
        }

    } else if (vspan->size) {
        MC_INCR_METRIC(pipeline, bytes_queued, vspan->size);
        netbuf_enqueue_span(&pipeline->nbmgr, vspan, packet);
    }

GT_ENQUEUE_PDU:
    netbuf_pdu_enqueue(&pipeline->nbmgr, packet, offsetof(mc_PACKET, sl_flushq));
    MC_INCR_METRIC(pipeline, packets_queued, 1);
}

void mcreq_wipe_packet(mc_PIPELINE *pipeline, mc_PACKET *packet)
{
    if (!(packet->flags & MCREQ_F_KEY_NOCOPY)) {
        if (packet->flags & MCREQ_F_DETACHED) {
            free(SPAN_BUFFER(&packet->kh_span));
        } else {
            netbuf_mblock_release(&pipeline->nbmgr, &packet->kh_span);
        }
    }

    if (!(packet->flags & MCREQ_F_HASVALUE)) {
        return;
    }

    if (packet->flags & MCREQ_F_VALUE_NOCOPY) {
        if (packet->flags & MCREQ_F_VALUE_IOV) {
            free(packet->u_value.multi.iov);
        }

        return;
    }

    if (packet->flags & MCREQ_F_DETACHED) {
        free(SPAN_BUFFER(&packet->u_value.single));
    } else {
        netbuf_mblock_release(&pipeline->nbmgr, &packet->u_value.single);
    }
}

mc_PACKET *mcreq_allocate_packet(mc_PIPELINE *pipeline)
{
    nb_SPAN span;
    int rv;
    mc_PACKET *ret;
    span.size = sizeof(*ret);

    rv = netbuf_mblock_reserve(&pipeline->reqpool, &span);
    if (rv != 0) {
        return NULL;
    }

    ret = (void *)SPAN_MBUFFER_NC(&span);
    ret->alloc_parent = span.parent;
    ret->flags = 0;
    ret->retries = 0;
    ret->opaque = pipeline->parent->seq++;
    ret->u_rdata.reqdata.span = NULL;
    return ret;
}

void mcreq_release_packet(mc_PIPELINE *pipeline, mc_PACKET *packet)
{
    nb_SPAN span;
    if (packet->flags & MCREQ_F_DETACHED) {
        sllist_iterator iter;
        mc_EXPACKET *epkt = (mc_EXPACKET *)packet;

        SLLIST_ITERFOR(&epkt->data, &iter)
        {
            mc_EPKTDATUM *d = SLLIST_ITEM(iter.cur, mc_EPKTDATUM, slnode);
            sllist_iter_remove(&epkt->data, &iter);
            d->dtorfn(d);
        }
        free(epkt);
        return;
    }

    span.size = sizeof(*packet);
    span.parent = packet->alloc_parent;
    span.offset = (char *)packet - packet->alloc_parent->root;

    netbuf_mblock_release(&pipeline->reqpool, &span);
}

#define MCREQ_DETACH_WIPESRC 1

mc_PACKET *mcreq_renew_packet(const mc_PACKET *src)
{
    char *kdata, *vdata;
    unsigned nvdata;
    mc_PACKET *dst;
    mc_EXPACKET *edst = calloc(1, sizeof(*edst));

    dst = &edst->base;
    *dst = *src;

    kdata = malloc(src->kh_span.size);
    memcpy(kdata, SPAN_BUFFER(&src->kh_span), src->kh_span.size);
    CREATE_STANDALONE_SPAN(&dst->kh_span, kdata, src->kh_span.size);

    dst->flags &= ~(MCREQ_F_KEY_NOCOPY | MCREQ_F_VALUE_NOCOPY | MCREQ_F_VALUE_IOV);
    dst->flags |= MCREQ_F_DETACHED;
    dst->alloc_parent = NULL;
    dst->sl_flushq.next = NULL;
    dst->slnode.next = NULL;
    dst->retries = src->retries;

    if (src->flags & MCREQ_F_HASVALUE) {
        /** Get the length */
        if (src->flags & MCREQ_F_VALUE_IOV) {
            unsigned ii;
            unsigned offset = 0;

            nvdata = src->u_value.multi.total_length;
            vdata = malloc(nvdata);
            for (ii = 0; ii < src->u_value.multi.niov; ii++) {
                const lcb_IOV *iov = src->u_value.multi.iov + ii;

                memcpy(vdata + offset, iov->iov_base, iov->iov_len);
                offset += iov->iov_len;
            }
        } else {
            protocol_binary_request_header hdr;
            const nb_SPAN *origspan = &src->u_value.single;
            mcreq_read_hdr(dst, &hdr);

            if (hdr.request.datatype & PROTOCOL_BINARY_DATATYPE_COMPRESSED) {
                /* For compressed payloads we need to uncompress it first
                 * because it may be forwarded to a server without compression.
                 * TODO: might be more clever to check a setting flag somewhere
                 * and see if we should do this. */

                lcb_SIZE n_inflated;
                const void *inflated;
                int rv;

                vdata = NULL;
                rv =
                    mcreq_inflate_value(SPAN_BUFFER(origspan), origspan->size, &inflated, &n_inflated, (void **)&vdata);

                lcb_assert(vdata == inflated);

                if (rv != 0) {
                    /* TODO: log error details when snappy will be enabled */
                    free(edst);
                    return NULL;
                }
                nvdata = n_inflated;
                hdr.request.datatype &= ~PROTOCOL_BINARY_DATATYPE_COMPRESSED;
                hdr.request.bodylen = htonl(ntohs(hdr.request.keylen) + hdr.request.extlen + n_inflated);
                mcreq_write_hdr(dst, &hdr);

            } else {
                nvdata = origspan->size;
                vdata = malloc(nvdata);
                memcpy(vdata, SPAN_BUFFER(origspan), nvdata);
            }
        }

        /* Declare the value as a standalone malloc'd span */
        CREATE_STANDALONE_SPAN(&dst->u_value.single, vdata, nvdata);
    }

    if (src->flags & MCREQ_F_DETACHED) {
        mc_EXPACKET *esrc = (mc_EXPACKET *)src;
        sllist_iterator iter;
        SLLIST_ITERFOR(&esrc->data, &iter)
        {
            sllist_node *cur = iter.cur;
            sllist_iter_remove(&esrc->data, &iter);
            sllist_append(&edst->data, cur);
        }
    }
    return dst;
}

int mcreq_epkt_insert(mc_EXPACKET *ep, mc_EPKTDATUM *datum)
{
    if (!(ep->base.flags & MCREQ_F_DETACHED)) {
        return -1;
    }
    lcb_assert(!sllist_contains(&ep->data, &datum->slnode));
    sllist_append(&ep->data, &datum->slnode);
    return 0;
}

mc_EPKTDATUM *mcreq_epkt_find(mc_EXPACKET *ep, const char *key)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(&ep->data, &iter)
    {
        mc_EPKTDATUM *d = SLLIST_ITEM(iter.cur, mc_EPKTDATUM, slnode);
        if (!strcmp(key, d->key)) {
            return d;
        }
    }
    return NULL;
}

void mcreq_map_key(mc_CMDQUEUE *queue, const lcb_KEYBUF *key, unsigned nhdr, int *vbid, int *srvix)
{
    const void *hk;
    size_t nhk = 0;
    switch (key->type) {
        case LCB_KV_VBID:
            *vbid = key->vbid;
            *srvix = lcbvb_vbmaster(queue->config, *vbid);
            return;
        case LCB_KV_COPY:
            hk = key->contig.bytes;
            nhk = key->contig.nbytes;
            break;
        case LCB_KV_HEADER_AND_KEY:
        default:
            hk = ((const char *)key->contig.bytes) + nhdr;
            nhk = key->contig.nbytes - nhdr;
            break;
    }
    lcbvb_map_key(queue->config, hk, nhk, vbid, srvix);
}

lcb_STATUS mcreq_basic_packet(mc_CMDQUEUE *queue, const lcb_CMDBASE *cmd, protocol_binary_request_header *req,
                              lcb_uint8_t extlen, lcb_uint8_t ffextlen, mc_PACKET **packet, mc_PIPELINE **pipeline,
                              int options)
{
    int vb, srvix;
    uint16_t nkey;

    if (!queue->config) {
        return LCB_CLIENT_ETMPFAIL;
    }
    if (!cmd) {
        return LCB_EINVAL;
    }

    mcreq_map_key(queue, &cmd->key, sizeof(*req) + extlen + ffextlen, &vb, &srvix);
    if (srvix > -1 && srvix < (int)queue->npipelines) {
        *pipeline = queue->pipelines[srvix];

    } else {
        if ((options & MCREQ_BASICPACKET_F_FALLBACKOK) && queue->fallback) {
            *pipeline = queue->fallback;
        } else {
            return LCB_NO_MATCHING_SERVER;
        }
    }

    *packet = mcreq_allocate_packet(*pipeline);
    if (*packet == NULL) {
        return LCB_CLIENT_ENOMEM;
    }

    mcreq_reserve_key(*pipeline, *packet, sizeof(*req) + extlen + ffextlen, &cmd->key, cmd->cid);

    nkey = (*packet)->kh_span.size - PKT_HDRSIZE(*packet);

    if (ffextlen) {
        req->request.magic = PROTOCOL_BINARY_AREQ;
        req->request.keylen = ((0xff & nkey) << 8) | ffextlen;
    } else {
        req->request.magic = PROTOCOL_BINARY_REQ;
        req->request.keylen = htons(nkey);
    }
    req->request.vbucket = htons(vb);
    req->request.extlen = extlen;
    return LCB_SUCCESS;
}

void mcreq_set_cid(mc_PACKET *packet, uint32_t cid)
{
    uint8_t ffext = 0;
    uint16_t nk = 0;
    uint8_t nbuf = 0;
    uint8_t buf[5] = {0};
    uint32_t old;
    int nold;
    protocol_binary_request_header req;
    char *kh = SPAN_BUFFER(&packet->kh_span);
    char *k = NULL;

    memcpy(&req, kh, sizeof(req));
    if (req.request.magic == PROTOCOL_BINARY_AREQ) {
        ffext = req.request.keylen & 0xff;
        nk = req.request.keylen >> 8;
    } else {
        nk = ntohs(req.request.keylen);
    }
    size_t nhdr = sizeof(req) + req.request.extlen + ffext;
    k = kh + nhdr;
    nold = leb128_decode((uint8_t *)k, nk, &old);
    nbuf = leb128_encode(cid, buf);

    int diff = (int)nbuf - (int)nold;
    size_t new_size = packet->kh_span.size + diff;
    req.request.bodylen = htonl(ntohl(req.request.bodylen) + diff);
    size_t new_klen = nk + diff;
    if (req.request.magic == PROTOCOL_BINARY_AREQ) {
        req.request.keylen = (new_klen << 8) | (ffext & 0xff);
    } else {
        req.request.keylen = htons(new_klen);
    }
    char *kdata = (char *)calloc(new_size, sizeof(char));
    char *ptr = kh;
    memcpy(kdata, ptr, nhdr);
    memcpy(kdata, req.bytes, sizeof(req.bytes));
    ptr += nhdr + nold;
    memcpy(kdata + nhdr, buf, nbuf);
    memcpy(kdata + nhdr + nbuf, ptr, new_size - nbuf - nhdr);
    CREATE_STANDALONE_SPAN(&packet->kh_span, kdata, new_size);
    free(kh);
}

uint32_t mcreq_get_cid(lcb_INSTANCE *instance, const mc_PACKET *packet)
{
    uint8_t ffext = 0;
    uint16_t nk = 0;
    uint8_t ncid = 0;
    uint32_t cid = 0;
    protocol_binary_request_header req;
    char *kh = SPAN_BUFFER(&packet->kh_span);
    char *k = NULL;

    memcpy(&req, kh, sizeof(req));
    if (req.request.magic == PROTOCOL_BINARY_AREQ) {
        ffext = req.request.keylen & 0xff;
        nk = req.request.keylen >> 8;
    } else {
        nk = ntohs(req.request.keylen);
    }
    k = kh + sizeof(req) + req.request.extlen + ffext;
    if ((packet->flags & MCREQ_F_NOCID) == 0 && instance && LCBT_SETTING(instance, use_collections)) {
        ncid = leb128_decode((uint8_t *)k, nk, &cid);
        if (ncid) {
            return cid;
        }
    }
    return 0;
}

void mcreq_get_key(lcb_INSTANCE *instance, const mc_PACKET *packet, const void **key, lcb_size_t *nkey)
{
    uint8_t ffext = 0;
    uint16_t nk = 0;
    uint8_t ncid = 0;
    uint32_t cid = 0;
    protocol_binary_request_header req;
    char *kh = SPAN_BUFFER(&packet->kh_span);
    uint8_t *k = NULL;

    memcpy(&req, kh, sizeof(req));
    if (req.request.magic == PROTOCOL_BINARY_AREQ) {
        ffext = req.request.keylen & 0xff;
        nk = req.request.keylen >> 8;
    } else {
        nk = ntohs(req.request.keylen);
    }
    k = (uint8_t *)kh + sizeof(req) + req.request.extlen + ffext;
    if ((packet->flags & MCREQ_F_NOCID) == 0 && instance && LCBT_SETTING(instance, use_collections)) {
        ncid = leb128_decode(k, nk, &cid);
        (void)cid;
    }
    *key = k + ncid;
    *nkey = nk - ncid;
}

lcb_uint32_t mcreq_get_bodysize(const mc_PACKET *packet)
{
    lcb_uint32_t ret;
    char *retptr = SPAN_BUFFER(&packet->kh_span) + 8;
    if ((uintptr_t)retptr % sizeof(ret) == 0) {
        return ntohl(*(lcb_uint32_t *)(void *)retptr);
    } else {
        memcpy(&ret, retptr, sizeof(ret));
        return ntohl(ret);
    }
}

uint16_t mcreq_get_vbucket(const mc_PACKET *packet)
{
    uint16_t ret;
    char *retptr = SPAN_BUFFER(&packet->kh_span) + 6;
    if ((uintptr_t)retptr % sizeof(ret) == 0) {
        return ntohs(*(uint16_t *)(void *)retptr);
    } else {
        memcpy(&ret, retptr, sizeof ret);
        return ntohs(ret);
    }
}

uint32_t mcreq_get_size(const mc_PACKET *packet)
{
    uint32_t sz = packet->kh_span.size;
    if (packet->flags & MCREQ_F_HASVALUE) {
        if (packet->flags & MCREQ_F_VALUE_IOV) {
            sz += packet->u_value.multi.total_length;
        } else {
            sz += packet->u_value.single.size;
        }
    }
    return sz;
}

void mcreq_pipeline_cleanup(mc_PIPELINE *pipeline)
{
    netbuf_cleanup(&pipeline->nbmgr);
    netbuf_cleanup(&pipeline->reqpool);
}

int mcreq_pipeline_init(mc_PIPELINE *pipeline)
{
    nb_SETTINGS settings;

    /* Initialize all members to 0 */
    memset(&pipeline->requests, 0, sizeof pipeline->requests);
    pipeline->parent = NULL;
    pipeline->flush_start = NULL;
    pipeline->index = 0;
    memset(&pipeline->ctxqueued, 0, sizeof pipeline->ctxqueued);
    pipeline->buf_done_callback = NULL;

    netbuf_default_settings(&settings);

    /** Initialize datapool */
    netbuf_init(&pipeline->nbmgr, &settings);

    /** Initialize request pool */
    settings.data_basealloc = sizeof(mc_PACKET) * 32;
    netbuf_init(&pipeline->reqpool, &settings);
    ;
    pipeline->metrics = NULL;
    return 0;
}

void mcreq_queue_add_pipelines(mc_CMDQUEUE *queue, mc_PIPELINE *const *pipelines, unsigned npipelines,
                               lcbvb_CONFIG *config)
{
    unsigned ii;

    lcb_assert(queue->pipelines == NULL);
    queue->npipelines = npipelines;
    queue->_npipelines_ex = queue->npipelines;
    queue->pipelines = malloc(sizeof(*pipelines) * (npipelines + 1));
    queue->config = config;

    memcpy(queue->pipelines, pipelines, sizeof(*pipelines) * npipelines);

    free(queue->scheds);
    queue->scheds = calloc(npipelines + 1, 1);

    for (ii = 0; ii < npipelines; ii++) {
        pipelines[ii]->parent = queue;
        pipelines[ii]->index = ii;
    }

    if (queue->fallback) {
        queue->fallback->index = npipelines;
        queue->pipelines[queue->npipelines] = queue->fallback;
        queue->_npipelines_ex++;
    }
}

mc_PIPELINE **mcreq_queue_take_pipelines(mc_CMDQUEUE *queue, unsigned *count)
{
    mc_PIPELINE **ret = queue->pipelines;
    *count = queue->npipelines;
    queue->pipelines = NULL;
    queue->npipelines = 0;
    return ret;
}

int mcreq_queue_init(mc_CMDQUEUE *queue)
{
    queue->seq = 0;
    queue->pipelines = NULL;
    queue->scheds = NULL;
    queue->fallback = NULL;
    queue->npipelines = 0;
    return 0;
}

void mcreq_queue_cleanup(mc_CMDQUEUE *queue)
{
    if (queue->fallback) {
        mcreq_pipeline_cleanup(queue->fallback);
        free(queue->fallback);
        queue->fallback = NULL;
    }
    free(queue->scheds);
    free(queue->pipelines);
    queue->pipelines = NULL;
    queue->npipelines = 0;
    queue->scheds = NULL;
}

void mcreq_sched_enter(mc_CMDQUEUE *queue)
{
    queue->ctxenter = 1;
}

static void queuectx_leave(mc_CMDQUEUE *queue, int success, int flush)
{
    unsigned ii;

    if (queue->ctxenter) {
        queue->ctxenter = 0;
    }

    for (ii = 0; ii < queue->_npipelines_ex; ii++) {
        mc_PIPELINE *pipeline;
        sllist_node *ll_next, *ll;

        if (!queue->scheds[ii]) {
            continue;
        }

        pipeline = queue->pipelines[ii];
        ll = SLLIST_FIRST(&pipeline->ctxqueued);

        while (ll) {
            mc_PACKET *pkt = SLLIST_ITEM(ll, mc_PACKET, slnode);
            ll_next = ll->next;

            if (success) {
                mcreq_enqueue_packet(pipeline, pkt);
            } else {
                if (pkt->flags & MCREQ_F_REQEXT) {
                    mc_REQDATAEX *rd = pkt->u_rdata.exdata;
                    if (rd->procs->fail_dtor) {
                        rd->procs->fail_dtor(pkt);
                    }
                }
                mcreq_wipe_packet(pipeline, pkt);
                mcreq_release_packet(pipeline, pkt);
            }

            ll = ll_next;
        }
        SLLIST_FIRST(&pipeline->ctxqueued) = pipeline->ctxqueued.last = NULL;
        if (flush) {
            pipeline->flush_start(pipeline);
        }
        queue->scheds[ii] = 0;
    }
}

void mcreq_sched_leave(mc_CMDQUEUE *queue, int do_flush)
{
    queuectx_leave(queue, 1, do_flush);
}

void mcreq_sched_fail(mc_CMDQUEUE *queue)
{
    queuectx_leave(queue, 0, 0);
}

void mcreq_sched_add(mc_PIPELINE *pipeline, mc_PACKET *pkt)
{
    mc_CMDQUEUE *cq = pipeline->parent;
    if (MCREQ_PKT_RDATA(pkt)->deadline == 0) {
        lcb_INSTANCE *instance = (lcb_INSTANCE *)pipeline->parent->cqdata;
        MCREQ_PKT_RDATA(pkt)->deadline = instance ? LCBT_SETTING(instance, operation_timeout) : LCB_DEFAULT_TIMEOUT;
    }
    if (!cq->scheds[pipeline->index]) {
        cq->scheds[pipeline->index] = 1;
    }
    sllist_append(&pipeline->ctxqueued, &pkt->slnode);
    mcreq_rearm_timeout(pipeline);
}

static mc_PACKET *pipeline_find(mc_PIPELINE *pipeline, lcb_uint32_t opaque, int do_remove)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(&pipeline->requests, &iter)
    {
        mc_PACKET *pkt = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
        if (pkt->opaque == opaque) {
            if (do_remove) {
                sllist_iter_remove(&pipeline->requests, &iter);
            }
            return pkt;
        }
    }
    return NULL;
}

mc_PACKET *mcreq_pipeline_find(mc_PIPELINE *pipeline, lcb_uint32_t opaque)
{
    return pipeline_find(pipeline, opaque, 0);
}

mc_PACKET *mcreq_pipeline_remove(mc_PIPELINE *pipeline, lcb_uint32_t opaque)
{
    return pipeline_find(pipeline, opaque, 1);
}

void mcreq_packet_done(mc_PIPELINE *pipeline, mc_PACKET *pkt)
{
    lcb_assert(pkt->flags & MCREQ_F_FLUSHED);
    lcb_assert(pkt->flags & MCREQ_F_INVOKED);
    if (pkt->flags & MCREQ_UBUF_FLAGS) {
        void *kbuf, *vbuf;
        const void *cookie;

        cookie = MCREQ_PKT_COOKIE(pkt);
        if (pkt->flags & MCREQ_F_KEY_NOCOPY) {
            kbuf = SPAN_BUFFER(&pkt->kh_span);
        } else {
            kbuf = NULL;
        }
        if (pkt->flags & MCREQ_F_VALUE_NOCOPY) {
            if (pkt->flags & MCREQ_F_VALUE_IOV) {
                vbuf = pkt->u_value.multi.iov->iov_base;
            } else {
                vbuf = SPAN_SABUFFER_NC(&pkt->u_value.single);
            }
        } else {
            vbuf = NULL;
        }

        pipeline->buf_done_callback(pipeline, cookie, kbuf, vbuf);
    }
    mcreq_wipe_packet(pipeline, pkt);
    mcreq_release_packet(pipeline, pkt);
}

void mcreq_reset_timeouts(mc_PIPELINE *pl, lcb_U64 nstime)
{
    sllist_node *nn;
    SLLIST_ITERBASIC(&pl->requests, nn)
    {
        mc_PACKET *pkt = SLLIST_ITEM(nn, mc_PACKET, slnode);
        hrtime_t old_timeout = (MCREQ_PKT_RDATA(pkt)->deadline - MCREQ_PKT_RDATA(pkt)->start);
        MCREQ_PKT_RDATA(pkt)->start = nstime;
        MCREQ_PKT_RDATA(pkt)->deadline = nstime + old_timeout;
    }
}

unsigned mcreq_pipeline_timeout(mc_PIPELINE *pl, lcb_STATUS err, mcreq_pktfail_fn failcb, void *cbarg, hrtime_t now)
{
    sllist_iterator iter;
    unsigned count = 0;

    SLLIST_ITERFOR(&pl->requests, &iter)
    {
        mc_PACKET *pkt = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
        mc_REQDATA *rd = MCREQ_PKT_RDATA(pkt);
        if (now == 0 || rd->deadline <= now) {
            sllist_iter_remove(&pl->requests, &iter);
            failcb(pl, pkt, err, cbarg);
            mcreq_packet_handled(pl, pkt);
            count++;
        }
    }
    return count;
}

unsigned mcreq_pipeline_fail(mc_PIPELINE *pl, lcb_STATUS err, mcreq_pktfail_fn failcb, void *arg)
{
    return mcreq_pipeline_timeout(pl, err, failcb, arg, 0);
}

void mcreq_iterwipe(mc_CMDQUEUE *queue, mc_PIPELINE *src, mcreq_iterwipe_fn callback, void *arg)
{
    sllist_iterator iter;

    SLLIST_ITERFOR(&src->requests, &iter)
    {
        int rv;
        mc_PACKET *orig = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
        rv = callback(queue, src, orig, arg);
        if (rv == MCREQ_REMOVE_PACKET) {
            sllist_iter_remove(&src->requests, &iter);
        }
    }
}

#include "mcreq-flush-inl.h"
typedef struct {
    mc_PIPELINE base;
    mcreq_fallback_cb handler;
} mc_FALLBACKPL;

static void do_fallback_flush(mc_PIPELINE *pipeline)
{
    nb_IOV iov;
    unsigned nb;
    int nused;
    sllist_iterator iter;
    mc_FALLBACKPL *fpl = (mc_FALLBACKPL *)pipeline;

    while ((nb = mcreq_flush_iov_fill(pipeline, &iov, 1, &nused))) {
        mcreq_flush_done(pipeline, nb, nb);
    }
    /* Now handle all the packets, for real */
    SLLIST_ITERFOR(&pipeline->requests, &iter)
    {
        mc_PACKET *pkt = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
        fpl->handler(pipeline->parent, pkt);
        sllist_iter_remove(&pipeline->requests, &iter);
        mcreq_packet_handled(pipeline, pkt);
    }
}

void mcreq_set_fallback_handler(mc_CMDQUEUE *cq, mcreq_fallback_cb handler)
{
    mc_FALLBACKPL *fallback;
    lcb_assert(!cq->fallback);
    fallback = calloc(1, sizeof(mc_FALLBACKPL));
    cq->fallback = (mc_PIPELINE *)fallback;
    mcreq_pipeline_init(cq->fallback);
    cq->fallback->parent = cq;
    cq->fallback->index = cq->npipelines;
    ((mc_FALLBACKPL *)cq->fallback)->handler = handler;
    cq->fallback->flush_start = do_fallback_flush;
}

static void noop_dumpfn(const void *d, unsigned n, FILE *fp)
{
    (void)d;
    (void)n;
    (void)fp;
}

#define MCREQ_XFLAGS(X)                                                                                                \
    X(KEY_NOCOPY)                                                                                                      \
    X(VALUE_NOCOPY)                                                                                                    \
    X(VALUE_IOV)                                                                                                       \
    X(HASVALUE)                                                                                                        \
    X(REQEXT)                                                                                                          \
    X(UFWD)                                                                                                            \
    X(FLUSHED)                                                                                                         \
    X(INVOKED)                                                                                                         \
    X(DETACHED)

void mcreq_dump_packet(const mc_PACKET *packet, FILE *fp, mcreq_payload_dump_fn dumpfn)
{
    const char *indent = "  ";
    const mc_REQDATA *rdata = MCREQ_PKT_RDATA(packet);

    if (!dumpfn) {
        dumpfn = noop_dumpfn;
    }
    if (!fp) {
        fp = stderr;
    }

    fprintf(fp, "Packet @%p\n", (void *)packet);
    fprintf(fp, "%sOPAQUE: %u\n", indent, (unsigned int)packet->opaque);

    fprintf(fp, "%sPKTFLAGS: 0x%x ", indent, packet->flags);
#define X(base)                                                                                                        \
    if (packet->flags & MCREQ_F_##base) {                                                                              \
        fprintf(fp, "%s, ", #base);                                                                                    \
    }
    MCREQ_XFLAGS(X)
#undef X
    fprintf(fp, "\n");

    fprintf(fp, "%sKey+Header Size: %u\n", indent, (unsigned int)packet->kh_span.size);
    fprintf(fp, "%sKey Offset: %u\n", indent, MCREQ_PKT_BASESIZE + packet->extlen);

    if (packet->flags & MCREQ_F_HASVALUE) {
        if (packet->flags & MCREQ_F_VALUE_IOV) {
            fprintf(fp, "%sValue Length: %u\n", indent, packet->u_value.multi.total_length);

            fprintf(fp, "%sValue IOV: [start=%p, n=%d]\n", indent, (void *)packet->u_value.multi.iov,
                    packet->u_value.multi.niov);
        } else {
            if (packet->flags & MCREQ_F_VALUE_NOCOPY) {
                fprintf(fp, "%sValue is user allocated\n", indent);
            }
            fprintf(fp, "%sValue: %p, %u bytes\n", indent, (void *)SPAN_BUFFER(&packet->u_value.single),
                    packet->u_value.single.size);
        }
    }

    fprintf(fp, "%sRDATA(%s): %p\n", indent, (packet->flags & MCREQ_F_REQEXT) ? "ALLOC" : "EMBEDDED", (void *)rdata);

    indent = "    ";
    fprintf(fp, "%sStart: %lu\n", indent, (unsigned long)rdata->start);
    fprintf(fp, "%sCookie: %p\n", indent, rdata->cookie);

    indent = "  ";
    fprintf(fp, "%sNEXT: %p\n", indent, (void *)packet->slnode.next);
    if (dumpfn != noop_dumpfn) {
        fprintf(fp, "PACKET CONTENTS:\n");
    }

    fwrite(SPAN_BUFFER(&packet->kh_span), 1, packet->kh_span.size, fp);
    if (packet->flags & MCREQ_F_HASVALUE) {
        if (packet->flags & MCREQ_F_VALUE_IOV) {
            const lcb_IOV *iovs = packet->u_value.multi.iov;
            unsigned ii, ixmax = packet->u_value.multi.niov;
            for (ii = 0; ii < ixmax; ii++) {
                dumpfn(iovs[ii].iov_base, iovs[ii].iov_len, fp);
            }
        } else {
            const nb_SPAN *vspan = &packet->u_value.single;
            dumpfn(SPAN_BUFFER(vspan), vspan->size, fp);
        }
    }
}

void mcreq_dump_chain(const mc_PIPELINE *pipeline, FILE *fp, mcreq_payload_dump_fn dumpfn)
{
    sllist_node *ll;
    SLLIST_FOREACH(&pipeline->requests, ll)
    {
        const mc_PACKET *pkt = SLLIST_ITEM(ll, mc_PACKET, slnode);
        mcreq_dump_packet(pkt, fp, dumpfn);
    }
}
