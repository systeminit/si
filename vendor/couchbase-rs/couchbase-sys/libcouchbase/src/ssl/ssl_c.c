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
#include "sllist.h"
#include "sllist-inl.h"

/* throw-away write buffer structure (for encoded data) */
typedef struct {
    void *parent;
    char buf[1];
} my_WBUF;

/* throw-away write buffer structure (for application data) */
typedef struct {
    sllist_node slnode;
    lcb_ioC_write2_callback cb;
    void *uarg;
    void *iovroot_;
    lcb_IOV *iov;
    lcb_size_t niov;
} my_WCTX;

typedef struct {
    IOTSSL_COMMON_FIELDS
    lcb_sockdata_t *sd;    /**< Socket pointer */
    lcbio_pTIMER as_read;  /**< For callbacks when SSL_pending > 0 */
    lcbio_pTIMER as_write; /**< For callbacks when SSL_writes succeeds */
    lcb_IOV urd_iov;       /**< User-defined buffer to read in applicataion data */
    void *urd_arg;         /**< User-defined argument for read callback */
    my_WCTX *wctx_cached;
    lcb_ioC_read2_callback urd_cb; /**< User defined read callback */
    sllist_root writes;            /**< List of pending user writes */

    /**
     * Whether a current read request is active. This read request refers to
     * this module reading raw data from the actual underlying socket. The
     * presence of a user-level (i.e. lcbio-invoked) read request is determined
     * by the presence of a non-NULL urd_cb value
     */
    int rdactive;

    int closed; /**< Pending delivery of close */
    int entered;
} lcbio_CSSL;

#define CS_FROM_IOPS(iops) (lcbio_CSSL *)IOTSSL_FROM_IOPS(iops)
#define SCHEDULE_WANT_SAFE(cs)                                                                                         \
    if (!(cs)->entered) {                                                                                              \
        schedule_wants(cs);                                                                                            \
    }

static void appdata_encode(lcbio_CSSL *);
static void appdata_free_flushed(lcbio_CSSL *);
static void appdata_read(lcbio_CSSL *);
static void schedule_wants(lcbio_CSSL *cs);
static int maybe_set_error(lcbio_CSSL *cs, int rv)
{
    return iotssl_maybe_error((lcbio_XSSL *)cs, rv);
}

/* This function goes through all the pending copies of data that was scheduled
 * for write and where the current IOV position is at the end (or niov==0).
 * For each of those routines this function will invoke its write callback
 */
static void appdata_free_flushed(lcbio_CSSL *cs)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(&cs->writes, &iter)
    {
        my_WCTX *cur = SLLIST_ITEM(iter.cur, my_WCTX, slnode);
        if (cur->niov && cs->error == 0) {
            break;
        }
        /* invoke the callback */
        cur->cb(cs->sd, cs->error ? -1 : 0, cur->uarg);
        sllist_iter_remove(&cs->writes, &iter);
        free(cur->iovroot_);
        if (cs->wctx_cached) {
            free(cur);
        } else {
            cs->wctx_cached = cur;
        }
    }
}

/* This function will attempt to encode pending user data into SSL data. This
 * will be output to the wbio. */
static void appdata_encode(lcbio_CSSL *cs)
{
    sllist_node *cur;

    /* each element here represents a used-defined write buffer */
    SLLIST_FOREACH(&cs->writes, cur)
    {
        my_WCTX *ctx = SLLIST_ITEM(cur, my_WCTX, slnode);

        for (; ctx->niov && cs->error == 0; ctx->niov--, ctx->iov++) {
            int rv;

            lcb_assert(ctx->iov->iov_len);
            rv = SSL_write(cs->ssl, ctx->iov->iov_base, ctx->iov->iov_len);
            if (rv > 0) {
                continue;
            } else if (maybe_set_error(cs, rv) == 0) {
                /* SSL_ERROR_WANT_READ. Should schedule a read here.
                 * XXX: Note that this buffer will not be returned to the user
                 * until the _next_ time the appdata_free_flushed function is
                 * invoked; the call chain for appdata_free_flushed is like this:
                 *
                 * start_write2 => async_schedule(async_write) => appdata_free_flushed.
                 * OR
                 * start_write2 => write_callback => appdata_free_flushed
                 */
                SCHEDULE_WANT_SAFE(cs)
                return;
            } else {
                IOTSSL_ERRNO(cs) = EINVAL;
            }
        }
    }
}

static void async_write(void *arg)
{
    lcbio_CSSL *cs = arg;
    appdata_encode(cs);
    schedule_wants(cs);
    appdata_free_flushed(cs);
}

/* Called when SSL data has been written to the socket */
static void write_callback(lcb_sockdata_t *sd, int status, void *arg)
{
    my_WBUF *wb = arg;
    lcbio_CSSL *cs = wb->parent;

    if (status) {
        IOTSSL_ERRNO(cs) = IOT_ERRNO(cs->orig);
        cs->error = 1;
    }

    free(wb);

    appdata_free_flushed(cs);
    lcbio_table_unref(&cs->base_);
    (void)sd;
}

/* Read application data from SSL's rbio buffer. Invokes the user callback
 * for the current read operation if there is data */
static void appdata_read(lcbio_CSSL *cs)
{
    /* either an error or an actual read event */
    int nr;
    lcb_ioC_read2_callback cb = cs->urd_cb;
    if (!cb) {
        return;
    }
    lcb_assert(!cs->rdactive);
    nr = SSL_read(cs->ssl, cs->urd_iov.iov_base, cs->urd_iov.iov_len);
    if (nr > 0) {
        /* nothing */
    } else if (cs->closed || nr == 0) {
        nr = 0;
    } else if (maybe_set_error(cs, nr) == 0) {
        return;
    }

    cs->urd_cb = NULL;
    cb(cs->sd, nr, cs->urd_arg);
}

/* Invoked when SSL data has been read from the socket */
static void read_callback(lcb_sockdata_t *sd, lcb_ssize_t nr, void *arg)
{
#if LCB_CAN_OPTIMIZE_SSL_BIO
    lcbio_CSSL *cs = arg;
#else
    my_WBUF *rb = arg;
    lcbio_CSSL *cs = rb->parent;
#endif

    cs->rdactive = 0;
    cs->entered++;

    if (nr > 0) {
#if LCB_CAN_OPTIMIZE_SSL_BIO
        BUF_MEM *mb;

        BIO_clear_retry_flags(cs->rbio);
        BIO_get_mem_ptr(cs->rbio, &mb);
        mb->length += nr;
#else
        BIO_write(cs->rbio, rb->buf, nr);
#endif

    } else if (nr == 0) {
        cs->closed = 1;
        cs->error = 1;

    } else {
        cs->error = 1;
        IOTSSL_ERRNO(cs) = IOT_ERRNO(cs->orig);
    }
#if !LCB_CAN_OPTIMIZE_SSL_BIO
    free(rb);
#endif

    appdata_encode(cs);
    appdata_read(cs);

    cs->entered--;
    schedule_wants(cs);
    lcbio_table_unref(&cs->base_);
    (void)sd;
}

/* This function schedules any I/O on the actual socket. It writes encoded
 * data and requests to read decoded data */
static void schedule_wants(lcbio_CSSL *cs)
{
    size_t npend = BIO_ctrl_pending(cs->wbio);
    char dummy;

    int has_appdata = 0;

    if (SSL_peek(cs->ssl, &dummy, 1) == 1) {
        has_appdata = 1;
    }

    if (npend) {
        /* Have pending data to write. The buffer is copied here because the
         * BIO structure doesn't support "lockdown" semantics like netbuf/rdb
         * do. We might transplant this with a different sort of BIO eventually..
         */
        my_WBUF *wb = malloc(sizeof(*wb) + npend);
        lcb_IOV iov;
        BIO_read(cs->wbio, wb->buf, npend);
        iov.iov_base = wb->buf;
        iov.iov_len = npend;
        wb->parent = cs;

        /* Increment the reference count. This is decremented when we get back
         * the callback. The goal is that a pending internal SSL_write() should
         * keep the object alive despite the user having called lcbio_table_unref()
         * on us.
         */
        lcbio_table_ref(&cs->base_);
        IOT_V1(cs->orig).write2(IOT_ARG(cs->orig), cs->sd, &iov, 1, wb, write_callback);
    }

    /* Only schedule additional reads if we're not already in the process of a
     * read */

    if (cs->rdactive == 0) {
        if (cs->error) {
            /* This can happen if we got an SSL error in performing something
             * within this callback.
             *
             * In this case, just signal "as-if" a read happened. appdata_read
             * will do the right thing if there is no read callback, and will
             * return an error if SSL_read() fails (which it should).
             */
            lcbio_async_signal(cs->as_read);

        } else if (SSL_want_read(cs->ssl) || (cs->urd_cb && has_appdata == 0)) {
            /* request more data from the socket */
            lcb_IOV iov;
#if LCB_CAN_OPTIMIZE_SSL_BIO
            BUF_MEM *mb;
#else
#define BUFSZ 4096
            my_WBUF *rb = malloc(sizeof(*rb) + BUFSZ);
            rb->parent = cs;
#endif

            cs->rdactive = 1;
            lcbio_table_ref(&cs->base_);
#if LCB_CAN_OPTIMIZE_SSL_BIO
            BIO_get_mem_ptr(cs->rbio, &mb);
            iotssl_bm_reserve(mb);
            iov.iov_base = mb->data + mb->length;
            iov.iov_len = mb->max - mb->length;
            IOT_V1(cs->orig).read2(IOT_ARG(cs->orig), cs->sd, &iov, 1, cs, read_callback);
#else
            iov.iov_base = rb->buf;
            iov.iov_len = BUFSZ;
            IOT_V1(cs->orig).read2(IOT_ARG(cs->orig), cs->sd, &iov, 1, rb, read_callback);
#endif
        }
    }
}

static int Cssl_read2(lcb_io_opt_t iops, lcb_sockdata_t *sd, lcb_IOV *iov, lcb_size_t niov, void *uarg,
                      lcb_ioC_read2_callback callback)
{
    lcbio_CSSL *cs = CS_FROM_IOPS(iops);
    cs->urd_iov = *iov;
    cs->urd_arg = uarg;
    cs->urd_cb = callback;

    IOTSSL_PENDING_PRECHECK(cs->ssl);
    if (IOTSSL_IS_PENDING(cs->ssl)) {
        /* have data to be read. Fast path here */
        lcbio_async_signal(cs->as_read);
    } else {
        SCHEDULE_WANT_SAFE(cs);
    }

    (void)niov;
    (void)sd;
    return 0;
}

static int Cssl_write2(lcb_io_opt_t io, lcb_sockdata_t *sd, lcb_IOV *iov, lcb_size_t niov, void *uarg,
                       lcb_ioC_write2_callback callback)
{
    lcbio_CSSL *cs = CS_FROM_IOPS(io);
    my_WCTX *wc;

    /* We keep one of these cached inside the cs structure so we don't have
     * to make a new malloc for each write */
    if (cs->wctx_cached) {
        wc = cs->wctx_cached;
        cs->wctx_cached = NULL;
        memset(wc, 0, sizeof *wc);
    } else {
        wc = calloc(1, sizeof(*wc));
    }

    /* assign the common parameters */
    wc->uarg = uarg;
    wc->cb = callback;

    /* If the socket does not have a pending error and there are no other
     * writes before this, then try to write the current buffer immediately. */
    if (cs->error == 0 && SLLIST_IS_EMPTY(&cs->writes)) {
        unsigned ii;
        for (ii = 0; ii < niov; ++ii) {
            int rv = SSL_write(cs->ssl, iov->iov_base, iov->iov_len);
            if (rv > 0) {
                iov++;
                niov--;
            } else {
                maybe_set_error(cs, rv);
                break;
            }
        }
    }

    /* We add this now in order for the SLLIST_IS_EMPTY to be false before, if
     * no other items were pending */
    sllist_append(&cs->writes, &wc->slnode);

    /* If we have some IOVs remaining then it means we couldn't write all the
     * data. If so, reschedule and place in the queue for later */
    if (niov && cs->error == 0) {
        wc->niov = niov;
        wc->iov = malloc(sizeof(*iov) * wc->niov);
        wc->iovroot_ = wc->iov;
        memcpy(wc->iov, iov, sizeof(*iov) * niov);
        /* This function will try to schedule the proper events. We need at least
         * one SSL_write() in order to advance the state machine. In the future
         * we could determine if we performed a previous SSL_write above */
        appdata_encode(cs);
    }

    /* In most cases we will want to deliver the "flushed" notification */
    lcbio_async_signal(cs->as_write);
    (void)sd;
    return 0;
}

static unsigned Cssl_close(lcb_io_opt_t io, lcb_sockdata_t *sd)
{
    lcbio_CSSL *cs = CS_FROM_IOPS(io);
    IOT_V1(cs->orig).close(IOT_ARG(cs->orig), sd);
    cs->error = 1;
    if (!SLLIST_IS_EMPTY(&cs->writes)) {
        /* It is possible that a prior call to SSL_write returned an SSL_want_read
         * and the next subsequent call to the underlying read API returned an
         * error. For this reason we signal to the as_write function (which
         * then calls the appdata_free_flushed function) in case we have such
         * leftover data.
         */
        lcbio_async_signal(cs->as_write);
    }
    return 0;
}

static void Cssl_dtor(void *arg)
{
    lcbio_CSSL *cs = arg;
    lcb_assert(SLLIST_IS_EMPTY(&cs->writes));
    lcbio_timer_destroy(cs->as_read);
    lcbio_timer_destroy(cs->as_write);
    iotssl_destroy_common((lcbio_XSSL *)cs);
    free(cs->wctx_cached);
    free(arg);
}

lcbio_pTABLE lcbio_Cssl_new(lcbio_pTABLE orig, lcb_sockdata_t *sd, SSL_CTX *sctx)
{
    lcbio_CSSL *ret = calloc(1, sizeof(*ret));
    lcbio_pTABLE iot = &ret->base_;
    ret->sd = sd;
    ret->as_read = lcbio_timer_new(orig, ret, (void (*)(void *))appdata_read);
    ret->as_write = lcbio_timer_new(orig, ret, async_write);
    ret->base_.dtor = Cssl_dtor;

    iot->u_io.completion.read2 = Cssl_read2;
    iot->u_io.completion.write2 = Cssl_write2;
    iot->u_io.completion.close = Cssl_close;
    iotssl_init_common((lcbio_XSSL *)ret, orig, sctx);
    return iot;
}
