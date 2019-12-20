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

#include "ssl_iot_common.h"
#include <openssl/err.h>
/**
 * Event-Style SSL Wrapping.
 *
 * This wraps the IO Table for SSL I/O
 *
 *
 * Writes and reads will always use SSL_write and SSL_read methods respectively.
 * I/O will be preemptively scheduled whenever:
 *
 * - SSL_want_read() is true
 * - The wbio is not empty
 */

typedef struct {
    IOTSSL_COMMON_FIELDS
    void *event;          /**< Event pointer (parent->create_event) */
    void *arg;            /**< Argument to pass for user-defined callback */
    short requested;      /**< User defined event flags */
    short fakewhich;      /**< Flags to deliver immediately */
    lcb_ioE_callback ucb; /**< User defined callback */
    int entered;          /**< Whether we are inside a handler */
    int closed;
    lcb_socket_t fd; /**< Socket descriptor */
    lcbio_pTIMER as_fake;
    lcb_SIZE last_nw; /**< Last failed call to SSL_write() */
} lcbio_ESSL;

#ifdef USE_EAGAIN
#define C_EAGAIN                                                                                                       \
    EWOULDBLOCK:                                                                                                       \
    case EAGAIN
#else
#define C_EAGAIN EWOULDBLOCK
#endif

#define ES_FROM_IOPS(iops) (lcbio_ESSL *)(IOTSSL_FROM_IOPS(iops))
#define MINIMUM(a, b) a < b ? a : b

static int maybe_error(lcbio_ESSL *es, int rv)
{
    return iotssl_maybe_error((lcbio_XSSL *)es, rv);
}

static void event_handler(lcb_socket_t, short, void *);

#define SCHEDULE_PENDING_SAFE(es)                                                                                      \
    if (!(es)->entered) {                                                                                              \
        schedule_pending(es);                                                                                          \
    }
/* Schedule watch events:
 * - If we have SSL data to be written, the watcher is activated for WRITE_EVENT
 * - If SSL_pending() is true and the user has requested read, the as_fake
 *   handler is triggered
 */
static void schedule_pending(lcbio_ESSL *es)
{
    short avail = LCB_WRITE_EVENT;
    /* Bitflags of events that the SSL pointer needs in order to progress */
    short wanted = 0;

    IOTSSL_PENDING_PRECHECK(es->ssl);
    if (IOTSSL_IS_PENDING(es->ssl)) {
        /* have user data in buffer */
        avail |= LCB_READ_EVENT;
    }

    if (SSL_want_read(es->ssl)) {
        /* SSL need data from the network */
        wanted |= LCB_READ_EVENT;
    }

    if (BIO_ctrl_pending(es->wbio)) {
        /* have data to flush */
        wanted |= LCB_WRITE_EVENT;
    }

    /* set the events to deliver on the next 'fake' event. This will be all
     * the available events AND'ed with all the events the user cared about */
    es->fakewhich = avail;
    if (es->fakewhich & es->requested) {
        lcbio_async_signal(es->as_fake);
    }

    if ((es->requested & LCB_READ_EVENT) && (avail & LCB_READ_EVENT) == 0) {
        /* if the user wanted application data but nothing is currently
         * available in the SSL's internal buffer, request a read as demanded
         * by the application. */
        wanted |= LCB_READ_EVENT;
    }

    /* Schedule events to watch for here */
    IOT_V0EV(es->orig).watch(IOT_ARG(es->orig), es->fd, es->event, wanted, es, event_handler);
}

/* Reads encrypted data from the socket into SSL */
static int read_ssl_data(lcbio_ESSL *es)
{
    int nr;
    lcbio_pTABLE iot = es->orig;

#if LCB_CAN_OPTIMIZE_SSL_BIO
    BUF_MEM *rmb;

    /* This block is an optimization over BIO_write to avoid copying the memory
     * to a temporary buffer and _then_ copying it into the BIO */

    BIO_get_mem_ptr(es->rbio, &rmb);
#endif

    while (1) {
#if LCB_CAN_OPTIMIZE_SSL_BIO
        /* I don't know why this is here, but it's found inside BIO_write */
        BIO_clear_retry_flags(es->rbio);
        iotssl_bm_reserve(rmb);
        nr = IOT_V0IO(iot).recv(IOT_ARG(iot), es->fd, rmb->data + rmb->length, rmb->max - rmb->length, 0);
#else
#define BUFSZ 4096
        char buf[BUFSZ];
        nr = IOT_V0IO(iot).recv(IOT_ARG(iot), es->fd, buf, BUFSZ, 0);
#endif

        if (nr > 0) {
#if LCB_CAN_OPTIMIZE_SSL_BIO
            /* Extend the BIO used length */
            rmb->length += nr;
#else
            BIO_write(es->rbio, buf, nr);
#endif
        } else if (nr == 0) {
            es->closed = 1;
            return -1;
        } else {
            switch (IOT_ERRNO(iot)) {
                case C_EAGAIN:
                    return 0;
                case EINTR:
                    continue;
                default:
                    return -1;
            }
        }
    }
    /* CONSTCOND */
    return 0;
}

/* Writes encrypted data from SSL over to the network */
static int flush_ssl_data(lcbio_ESSL *es)
{
    BUF_MEM *wmb;
    char *tmp_p;
    int tmp_len, nw;
    lcbio_pTABLE iot = es->orig;

    BIO_get_mem_ptr(es->wbio, &wmb);
    tmp_p = wmb->data;
    tmp_len = wmb->length;

    /* We use this block of code over BIO_read() as we have no guarantee that
     * we'll be able to flush all the bytes received from BIO_read(), and
     * BIO has no way to "put back" some bytes. Thus this block is not an
     * optimization but a necessity.
     *
     * tmp_len is the number of bytes originally inside the BUF_MEM structure.
     * It is decremented each time we write data from the network.
     *
     * The loop here terminates until we get a WOULDBLOCK from the socket or we
     * have no more data left to write.
     */
    while (tmp_len) {
        nw = IOT_V0IO(iot).send(IOT_ARG(iot), es->fd, tmp_p, tmp_len, 0);
        if (nw > 0) {
            tmp_len -= nw;
            tmp_p += nw;
            continue;
        } else if (nw == 0) {
            return -1;
        } else {
            switch (IOT_ERRNO(iot)) {
                case C_EAGAIN:
                    goto GT_WRITE_DONE;
                case EINTR:
                    continue;
                default:
                    return -1;
            }
        }
    }

/* TODO: This block is inefficient as it results in a bunch of memmove()
 * calls. While we could have done this inline with the send() call this
 * would make future optimization more difficult. */
GT_WRITE_DONE:
#if !LCB_CAN_OPTIMIZE_SSL_BIO
    BIO_get_mem_ptr(es->wbio, &wmb);
#endif
    while (wmb->length > (size_t)tmp_len) {
        char dummy[4096];
        unsigned to_read = MINIMUM(wmb->length - tmp_len, sizeof dummy);
        BIO_read(es->wbio, dummy, to_read);
#if !LCB_CAN_OPTIMIZE_SSL_BIO
        BIO_get_mem_ptr(es->wbio, &wmb);
#endif
    }
    BIO_clear_retry_flags(es->wbio);
    return 0;
}

/* This is the raw event handler called from the underlying IOPS */
static void event_handler(lcb_socket_t fd, short which, void *arg)
{
    lcbio_ESSL *es = arg;
    int rv = 0;
    int u_which;
    es->entered++;

    if (which & LCB_READ_EVENT) {
        rv = read_ssl_data(es);
    }
    if (rv == 0 && (which & LCB_WRITE_EVENT)) {
        rv = flush_ssl_data(es);
    }

    if (rv == -1) {
        es->error = 1;

        /* stop internal watcher */
        IOT_V0EV(es->orig).watch(IOT_ARG(es->orig), es->fd, es->event, 0, NULL, NULL);

        /* send/recv will detect es->error and return -1/EINVAL appropriately */
        if (es->requested && es->ucb) {
            es->ucb(fd, es->requested, es->arg);
        }
        es->entered--;
        return;
    }

    /* deliver stuff back to the user: */
    u_which = 0;
    if (es->requested & LCB_READ_EVENT) {
        u_which |= LCB_READ_EVENT;
    }
    if (es->requested & LCB_WRITE_EVENT) {
        u_which |= LCB_WRITE_EVENT;
    }
    if (es->ucb && u_which) {
        es->ucb(fd, u_which & es->requested, es->arg);
    }

    /* socket closed. Don't reschedule */
    if (es->fd == -1) {
        es->entered--;
        return;
    }

    es->entered--;
    schedule_pending(es);
}

/* User events are delivered out-of-sync with SSL events. This is mainly with
 * respect to write events. */
static void fake_signal(void *arg)
{
    lcbio_ESSL *es = arg;
    short which = es->fakewhich;
    es->fakewhich = 0;
    es->entered++;

    /* invoke the callback */
    which &= es->requested;
    if (which && es->ucb) {
        es->ucb(es->fd, which, es->arg);
    }
    es->entered--;
    schedule_pending(es);
}

static int start_watch(lcb_io_opt_t iops, lcb_socket_t sock, void *event, short which, void *uarg,
                       lcb_ioE_callback callback)
{
    lcbio_ESSL *es = ES_FROM_IOPS(iops);
    es->arg = uarg;
    es->requested = which;
    es->ucb = callback;

    SCHEDULE_PENDING_SAFE(es);

    (void)event;
    (void)sock;

    return 0;
}

static void stop_watch(lcb_io_opt_t iops, lcb_socket_t sock, void *event)
{
    start_watch(iops, sock, event, 0, NULL, NULL);
}

/** socket routines go here now.. */
static lcb_ssize_t Essl_recv(lcb_io_opt_t iops, lcb_socket_t sock, void *buf, lcb_size_t nbuf, int ign)
{
    lcbio_ESSL *es = ES_FROM_IOPS(iops);
    int rv = SSL_read(es->ssl, buf, nbuf);

    if (es->error) {
        IOTSSL_ERRNO(es) = EINVAL;
        return -1;
    }

    if (rv >= 0) {
        /* data or clean shutdown */
        return rv;
    } else if (es->closed) {
        return 0;
    } else if (maybe_error(es, rv) != 0) {
        IOTSSL_ERRNO(es) = EINVAL;
    } else {
        IOTSSL_ERRNO(es) = EWOULDBLOCK;
    }
    (void)ign;
    (void)sock;
    return -1;
}

static lcb_ssize_t Essl_send(lcb_io_opt_t iops, lcb_socket_t sock, const void *buf, lcb_size_t nbuf, int ign)
{
    lcbio_ESSL *es = ES_FROM_IOPS(iops);
    int rv;
    (void)ign;
    (void)sock;

    if (es->error) {
        IOTSSL_ERRNO(es) = EINVAL;
        return -1;
    }

    rv = SSL_write(es->ssl, buf, nbuf);
    if (rv >= 0) {
        /* still need to schedule data to get flushed to the network */
        SCHEDULE_PENDING_SAFE(es);
        return rv;
    } else if (maybe_error(es, rv)) {
        IOTSSL_ERRNO(es) = EINVAL;
        return -1;
    } else {
        IOTSSL_ERRNO(es) = EWOULDBLOCK;
        return -1;
    }
}

static lcb_ssize_t Essl_recvv(lcb_io_opt_t iops, lcb_socket_t sock, lcb_IOV *iov, lcb_size_t niov)
{
    (void)niov;
    return Essl_recv(iops, sock, iov->iov_base, iov->iov_len, 0);
}

static lcb_ssize_t Essl_sendv(lcb_io_opt_t iops, lcb_socket_t sock, lcb_IOV *iov, lcb_size_t niov)
{
    (void)niov;
    return Essl_send(iops, sock, iov->iov_base, iov->iov_len, 0);
}

static void Essl_close(lcb_io_opt_t iops, lcb_socket_t fd)
{
    lcbio_ESSL *es = ES_FROM_IOPS(iops);
    IOT_V0IO(es->orig).close(IOT_ARG(es->orig), fd);
    es->fd = -1;
}

static void Essl_dtor(void *arg)
{
    lcbio_ESSL *es = arg;
    IOT_V0EV(es->orig).destroy(IOT_ARG(es->orig), es->event);
    lcbio_timer_destroy(es->as_fake);
    iotssl_destroy_common((lcbio_XSSL *)es);
    free(es);
}

lcbio_pTABLE lcbio_Essl_new(lcbio_pTABLE orig, lcb_socket_t fd, SSL_CTX *sctx)
{
    lcbio_ESSL *es = calloc(1, sizeof(*es));
    lcbio_TABLE *iot = &es->base_;
    es->fd = fd;
    es->as_fake = lcbio_timer_new(orig, es, fake_signal);
    es->event = IOT_V0EV(orig).create(IOT_ARG(orig));
    iot->u_io.v0.ev.watch = start_watch;
    iot->u_io.v0.ev.cancel = stop_watch;
    iot->u_io.v0.io.recv = Essl_recv;
    iot->u_io.v0.io.send = Essl_send;
    iot->u_io.v0.io.recvv = Essl_recvv;
    iot->u_io.v0.io.sendv = Essl_sendv;
    iot->u_io.v0.io.close = Essl_close;
    iot->dtor = Essl_dtor;
    iotssl_init_common((lcbio_XSSL *)es, orig, sctx);
    return iot;
}
