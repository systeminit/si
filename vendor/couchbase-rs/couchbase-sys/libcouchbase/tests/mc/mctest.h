/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#include "mc/mcreq.h"
#include "mcserver/mcserver.h"
#include "sllist-inl.h"
#include <gtest/gtest.h>
#include "internalstructs.h"

#define NUM_PIPELINES 4

struct CQWrap : mc_CMDQUEUE {
    lcbvb_CONFIG *config;
    CQWrap()
    {
        mc_PIPELINE **pll;
        pll = (mc_PIPELINE **)malloc(sizeof(*pll) * NUM_PIPELINES);
        config = lcbvb_create();
        for (unsigned ii = 0; ii < NUM_PIPELINES; ii++) {
            mc_PIPELINE *pipeline = new lcb::Server();
            mcreq_pipeline_init(pipeline);
            pll[ii] = pipeline;
        }
        lcbvb_genconfig(config, NUM_PIPELINES, 3, 1024);
        this->cqdata = NULL; /* instance pointer */
        mcreq_queue_init(this);
        this->seq = 100;
        mcreq_queue_add_pipelines(this, pll, NUM_PIPELINES, config);
        free(pll);
    }

    ~CQWrap()
    {
        for (int ii = 0; ii < NUM_PIPELINES; ii++) {
            mc_PIPELINE *pipeline = pipelines[ii];
            EXPECT_NE(0, netbuf_is_clean(&pipeline->nbmgr));
            EXPECT_NE(0, netbuf_is_clean(&pipeline->reqpool));
            mcreq_pipeline_cleanup(pipeline);
            free(pipeline);
        }
        mcreq_queue_cleanup(this);
        lcbvb_destroy(config);
    }

    void clearPipelines()
    {
        for (unsigned ii = 0; ii < npipelines; ii++) {
            mc_PIPELINE *pipeline = pipelines[ii];
            sllist_iterator iter;
            SLLIST_ITERFOR(&pipeline->requests, &iter)
            {
                mc_PACKET *pkt = SLLIST_ITEM(iter.cur, mc_PACKET, slnode);
                sllist_iter_remove(&pipeline->requests, &iter);
                mcreq_wipe_packet(pipeline, pkt);
                mcreq_release_packet(pipeline, pkt);
            }
        }
    }

    void setBufFreeCallback(mcreq_bufdone_fn cb)
    {
        for (unsigned ii = 0; ii < npipelines; ii++) {
            pipelines[ii]->buf_done_callback = cb;
        }
    }

    CQWrap(CQWrap &);
};

struct PacketWrap {
    mc_PACKET *pkt;
    mc_PIPELINE *pipeline;
    protocol_binary_request_header hdr;
    lcb_CMDBASE cmd;
    char *pktbuf;
    char *kbuf;

    PacketWrap()
    {
        pkt = NULL;
        pipeline = NULL;
        pktbuf = NULL;
        kbuf = NULL;
        memset(&hdr, 0, sizeof(hdr));
        memset(&cmd, 0, sizeof(cmd));
    }

    void setKey(const char *key)
    {
        size_t nkey = strlen(key);
        pktbuf = new char[24 + nkey + 1];
        kbuf = pktbuf + 24;
        memcpy(kbuf, key, nkey);
        kbuf[nkey] = '\0';
    }

    void setContigKey(const char *key)
    {
        setKey(key);
        cmd.key.type = LCB_KV_HEADER_AND_KEY;
        cmd.key.contig.bytes = pktbuf;
        cmd.key.contig.nbytes = strlen(key) + 24;
    }

    void setCopyKey(const char *key)
    {
        setKey(key);
        LCB_KREQ_SIMPLE(&cmd.key, kbuf, strlen(key));
    }

    void setHeaderSize()
    {
        hdr.request.bodylen = htonl((lcb_uint32_t)strlen(kbuf));
    }

    void copyHeader()
    {
        memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
    }

    void setCookie(void *ptr)
    {
        pkt->u_rdata.reqdata.cookie = ptr;
    }

    bool reservePacket(mc_CMDQUEUE *cq)
    {
        lcb_STATUS err;
        err = mcreq_basic_packet(cq, &cmd, &hdr, 0, 0, &pkt, &pipeline, 0);
        return err == LCB_SUCCESS;
    }

    ~PacketWrap()
    {
        if (pktbuf != NULL) {
            delete[] pktbuf;
        }
    }
};
