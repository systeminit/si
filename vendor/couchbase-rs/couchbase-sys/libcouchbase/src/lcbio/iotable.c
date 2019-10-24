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

#define LCB_IOPS_V12_NO_DEPRECATE
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "iotable.h"
#include "connect.h" /* prototypes for iotable functions */

#define GET_23_FIELD(iops, fld) ((iops)->version == 2 ? (iops)->v.v2.fld : (iops)->v.v3.fld)

struct W_1to3_st {
    lcb_ioC_write2_callback callback;
    void *udata;
    unsigned int refcount;
    unsigned int last_error;
};

static void W_1to3_callback(lcb_sockdata_t *sd, lcb_io_writebuf_t *wb, int status)
{
    struct W_1to3_st *ott = (struct W_1to3_st *)(void *)wb->buffer.root;
    lcb_ioC_wbfree_fn wbfree;

    wb->buffer.root = NULL;
    wb->buffer.ringbuffer = NULL;

    if (sd->parent->version >= 2) {
        wbfree = GET_23_FIELD(sd->parent, iot->u_io.completion.wbfree);
    } else {
        wbfree = sd->parent->v.v1.release_writebuf;
    }

    wbfree(sd->parent, sd, wb);

    if (status != 0 && ott->last_error == 0) {
        ott->last_error = sd->parent->v.v0.error;
    }

    if (--ott->refcount == 0) {
        ott->callback(sd, ott->last_error, ott->udata);
        free(ott);
    }
}

static int W_1to3_write(lcb_io_opt_t iops, lcb_sockdata_t *sd, struct lcb_iovec_st *iov, lcb_size_t niov, void *uarg,
                        lcb_ioC_write2_callback cb)
{
    unsigned int ii = 0;
    struct W_1to3_st *ott;
    lcb_ioC_write_fn start_write;
    lcb_ioC_wballoc_fn wballoc;

    /** Schedule IOV writes, two at a time... */
    ott = malloc(sizeof(*ott));
    ott->callback = cb;
    ott->udata = uarg;
    ott->refcount = 0;
    ott->last_error = 0;

    if (iops->version >= 2) {
        start_write = GET_23_FIELD(iops, iot->u_io.completion.write);
        wballoc = GET_23_FIELD(iops, iot->u_io.completion.wballoc);
    } else {
        start_write = iops->v.v1.start_write;
        wballoc = iops->v.v1.create_writebuf;
    }

    while (ii < niov) {
        int jj = 0;
        lcb_io_writebuf_t *wb;

        wb = wballoc(iops, sd);
        wb->buffer.root = (char *)ott;
        wb->buffer.ringbuffer = NULL;

        for (jj = 0; jj < 2 && ii < niov; ii++, jj++) {
            wb->buffer.iov[jj] = iov[ii];
        }
        ott->refcount++;
        start_write(iops, sd, wb, W_1to3_callback);
    }
    return 0;
}

struct R_1to3_st {
    lcb_ioC_read2_callback callback;
    void *uarg;
};

static void R_1to3_callback(lcb_sockdata_t *sd, lcb_ssize_t nread)
{
    struct lcb_buf_info *bi = &sd->read_buffer;
    struct R_1to3_st *st = (void *)bi->root;
    bi->root = NULL;
    st->callback(sd, nread, st->uarg);
    free(st);
}

static int R_1to3_read(lcb_io_opt_t io, lcb_sockdata_t *sd, lcb_IOV *iov, lcb_size_t niov, void *uarg,
                       lcb_ioC_read2_callback callback)
{
    unsigned ii;
    int rv;
    struct R_1to3_st *st;
    struct lcb_buf_info *bi = &sd->read_buffer;
    lcb_ioC_read_fn rdstart;

    st = calloc(1, sizeof(*st));
    st->callback = callback;
    st->uarg = uarg;

    for (ii = 0; ii < 2 && ii < niov; ii++) {
        bi->iov[ii] = iov[ii];
    }

    for (; ii < 2; ii++) {
        bi->iov[ii].iov_base = NULL;
        bi->iov[ii].iov_len = 0;
    }

    bi->root = (void *)st;
    if (io->version >= 2) {
        rdstart = GET_23_FIELD(io, iot->u_io.completion.read);
    } else {
        rdstart = io->v.v1.start_read;
    }

    rv = rdstart(io, sd, R_1to3_callback);
    return rv;
}

static int dummy_bsd_chkclosed(lcb_io_opt_t io, lcb_socket_t s, int f)
{
    (void)io;
    (void)s;
    (void)f;
    return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
}
static int dummy_comp_chkclosed(lcb_io_opt_t io, lcb_sockdata_t *s, int f)
{
    (void)io;
    (void)s;
    (void)f;
    return LCB_IO_SOCKCHECK_STATUS_UNKNOWN;
}

static void init_v23_table(lcbio_TABLE *table, lcb_io_opt_t io)
{
    lcb_io_procs_fn fn = GET_23_FIELD(io, get_procs);
    fn(LCB_IOPROCS_VERSION, &table->loop, &table->timer, &table->u_io.v0.io, &table->u_io.v0.ev,
       &table->u_io.completion, &table->model);

    table->p = io;
    if (table->model == LCB_IOMODEL_COMPLETION) {
        if (!table->u_io.completion.write2) {
            table->u_io.completion.write2 = W_1to3_write;
        }

        if (!table->u_io.completion.read2) {
            table->u_io.completion.read2 = R_1to3_read;
        }
        lcb_assert(table->u_io.completion.read2);
        lcb_assert(table->u_io.completion.write2);
    }

    if (table->model == LCB_IOMODEL_COMPLETION && IOT_V1(table).is_closed == NULL) {
        IOT_V1(table).is_closed = dummy_comp_chkclosed;
    }

    if (table->model == LCB_IOMODEL_EVENT && IOT_V0IO(table).is_closed == NULL) {
        IOT_V0IO(table).is_closed = dummy_bsd_chkclosed;
    }
}

lcbio_TABLE *lcbio_table_new(lcb_io_opt_t io)
{
    lcbio_TABLE *table = calloc(1, sizeof(*table));
    table->p = io;
    table->refcount = 1;

    if (io->version == 2) {
        io->v.v2.iot = table;
        init_v23_table(table, io);
        return table;

    } else if (io->version == 3) {
        /* V3 exists exclusively for back-compat. We need to use a few tricks to
         * determine if we are really v3, or if we've been 'overidden' somehow.
         *
         * To do this, we treat the padding fields (specifically, the event
         * scheduling parts of the padding fields) as sentinel values. The
         * built-in plugins should initialize this to NULL. If a client
         * (e.g. Python SDK) overrides this, the field will no longer be
         * NULL and will be a sign that the event fields have been
         * used by a non-getprocs-aware client.
         */

        io->v.v3.iot = table;
        if (io->v.v0.create_event == NULL) {
            init_v23_table(table, io);
            return table;
        }
    }

    table->timer.create = io->v.v0.create_timer;
    table->timer.destroy = io->v.v0.destroy_timer;
    table->timer.cancel = io->v.v0.delete_timer;
    table->timer.schedule = io->v.v0.update_timer;
    table->loop.start = io->v.v0.run_event_loop;
    table->loop.stop = io->v.v0.stop_event_loop;

    if (io->version == 0 || io->version == 3) {
        lcb_ev_procs *ev = &table->u_io.v0.ev;
        lcb_bsd_procs *bsd = &table->u_io.v0.io;

        table->model = LCB_IOMODEL_EVENT;
        ev->create = io->v.v0.create_event;
        ev->destroy = io->v.v0.destroy_event;
        ev->cancel = io->v.v0.delete_event;
        ev->watch = io->v.v0.update_event;
        bsd->socket0 = io->v.v0.socket;
        bsd->connect0 = io->v.v0.connect;
        bsd->close = io->v.v0.close;
        bsd->recv = io->v.v0.recv;
        bsd->recvv = io->v.v0.recvv;
        bsd->send = io->v.v0.send;
        bsd->sendv = io->v.v0.sendv;
        bsd->is_closed = dummy_bsd_chkclosed;

    } else {
        lcb_completion_procs *cp = &table->u_io.completion;

        table->model = LCB_IOMODEL_COMPLETION;
        cp->socket = io->v.v1.create_socket;
        cp->close = io->v.v1.close_socket;
        cp->connect = io->v.v1.start_connect;
        cp->read = io->v.v1.start_read;
        cp->write = io->v.v1.start_write;
        cp->wballoc = io->v.v1.create_writebuf;
        cp->nameinfo = io->v.v1.get_nameinfo;
        cp->write2 = W_1to3_write;
        cp->read2 = R_1to3_read;
        cp->is_closed = dummy_comp_chkclosed;
    }

    return table;
}

void lcbio_table_unref(lcbio_TABLE *table)
{
    if (--table->refcount) {
        return;
    }

    if (table->dtor) {
        table->dtor(table);
        return;
    }

    if (table->p && table->p->v.v0.need_cleanup) {
        lcb_destroy_io_ops(table->p);
    }

    free(table);
}

void lcbio_table_ref(lcbio_TABLE *table)
{
    ++table->refcount;
}
