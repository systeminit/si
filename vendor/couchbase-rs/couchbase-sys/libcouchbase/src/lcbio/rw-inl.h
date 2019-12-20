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

/**
 * Inline routines for reading and writing socket buffers
 */
#include <errno.h>
#include <limits.h> /* For IOV_MAX */
#include "ctx-log-inl.h"
#include "strcodecs/strcodecs.h"

#ifndef INLINE
#ifdef _MSC_VER
#define INLINE __inline
#elif __GNUC__
#define INLINE __inline__
#else
#define INLINE inline
#endif /* MSC_VER */
#endif /* !INLINE */

#define RWINL_IOVSIZE 32
#if defined(IOV_MAX) && IOV_MAX < RWINL_IOVSIZE
#undef RWINL_IOVSIZE
#define RWINL_IOVSIZE IOV_MAX
#endif

#ifndef USE_EAGAIN
#define C_EAGAIN 0
#else
#define C_EAGAIN EAGAIN
#endif

static INLINE lcbio_IOSTATUS lcbio_E_rdb_slurp(lcbio_CTX *ctx, rdb_IOROPE *ior)
{
    lcb_ssize_t rv;
    lcb_IOV iov[RWINL_IOVSIZE];
    unsigned niov;
    lcbio_TABLE *iot = ctx->io;
    const lcb_U32 rdsize = ctx->sock->settings->read_chunk_size;
    lcb_U32 total_nr = 0;

    do {
        niov = rdb_rdstart(ior, (nb_IOV *)iov, RWINL_IOVSIZE);
    GT_READ:
        rv = IOT_V0IO(iot).recvv(IOT_ARG(iot), CTX_FD(ctx), iov, niov);
        if (rv > 0) {
#ifdef LCB_DUMP_PACKETS
            {
                char *b64 = NULL;
                int nb64 = 0;
                lcb_base64_encode_iov((lcb_IOV *)iov, niov, rv, &b64, &nb64);
                lcb_log(LOGARGS(ctx, TRACE), CTX_LOGFMT "pkt,rcv: size=%d, %.*s", CTX_LOGID(ctx), nb64, nb64, b64);
                free(b64);
            }
#endif
            rdb_rdend(ior, rv);
            if (rdsize && (total_nr += rv) >= rdsize) {
                return LCBIO_PENDING;
            }
        } else if (rv == -1) {
            switch (IOT_ERRNO(iot)) {
                case EWOULDBLOCK:
                case C_EAGAIN:
                    return LCBIO_PENDING;
                case EINTR:
                    goto GT_READ;
                default:
                    ctx->sock->last_error = IOT_ERRNO(iot);
                    return LCBIO_IOERR;
            }
        } else {
            return LCBIO_SHUTDOWN;
        }
    } while (1);
    /* UNREACHED */
    return LCBIO_IOERR;
}

static INLINE lcbio_IOSTATUS lcbio_E_rb_write(lcbio_CTX *ctx, ringbuffer_t *buf)
{
    lcb_IOV iov[2] = {0};
    lcb_ssize_t nw;
    lcbio_TABLE *iot = ctx->io;
    while (buf->nbytes) {
        unsigned niov;
        ringbuffer_get_iov(buf, RINGBUFFER_READ, iov);
#if RWINL_IOVSIZE < 2
        niov = 1;
#else
        niov = iov[1].iov_len ? 2 : 1;
#endif
        nw = IOT_V0IO(iot).sendv(IOT_ARG(iot), CTX_FD(ctx), iov, niov);
        if (nw == -1) {
            switch (IOT_ERRNO(iot)) {
                case EINTR:
                    break;
                case EWOULDBLOCK:
                case C_EAGAIN:
                    return LCBIO_PENDING;
                default:
                    ctx->sock->last_error = IOT_ERRNO(iot);
                    return LCBIO_IOERR;
            }
        }
        if (nw) {
#ifdef LCB_DUMP_PACKETS
            {
                char *b64 = NULL;
                int nb64 = 0;
                lcb_base64_encode_iov((lcb_IOV *)iov, niov, nw, &b64, &nb64);
                lcb_log(LOGARGS(ctx, TRACE), CTX_LOGFMT "pkt,snd: size=%d, %.*s", CTX_LOGID(ctx), nb64, nb64, b64);
                free(b64);
            }
#endif
            ringbuffer_consumed(buf, nw);
            CTX_INCR_METRIC(ctx, bytes_sent, nw);
        }
    }
    return LCBIO_COMPLETED;
}
