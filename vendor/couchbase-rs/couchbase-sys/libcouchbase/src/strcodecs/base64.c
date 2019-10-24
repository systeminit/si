/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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

#include "strcodecs.h"
#include <string.h>
#include <stdlib.h>

/*
 * Function to base64 encode a text string as described in RFC 4648
 *
 * @author Trond Norbye
 */

/**
 * An array of the legal charracters used for direct lookup
 */
static const lcb_uint8_t code[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Encode up to 3 characters to 4 output character.
 *
 * @param s pointer to the input stream
 * @param d pointer to the output stream
 * @param num the number of characters from s to encode
 * @return 0 upon success, -1 otherwise.
 */
static int encode_rest(const lcb_uint8_t *s, lcb_uint8_t *d, lcb_SIZE num)
{
    lcb_uint32_t val = 0;

    switch (num) {
        case 2:
            val = (lcb_uint32_t)((*s << 16) | (*(s + 1) << 8));
            break;
        case 1:
            val = (lcb_uint32_t)((*s << 16));
            break;
        default:
            return -1;
    }

    d[3] = '=';

    if (num == 2) {
        d[2] = code[(val >> 6) & 63];
    } else {
        d[2] = '=';
    }

    d[1] = code[(val >> 12) & 63];
    d[0] = code[(val >> 18) & 63];

    return 0;
}

/**
 * Encode 3 characters to 4 output character.
 *
 * @param s pointer to the input stream
 * @param d pointer to the output stream
 */
static int encode_triplet(const lcb_uint8_t *s, lcb_uint8_t *d)
{
    lcb_uint32_t val = (lcb_uint32_t)((*s << 16) | (*(s + 1) << 8) | (*(s + 2)));
    d[3] = code[val & 63];
    d[2] = code[(val >> 6) & 63];
    d[1] = code[(val >> 12) & 63];
    d[0] = code[(val >> 18) & 63];

    return 0;
}

/**
 * Base64 encode a string into an output buffer.
 * @param src string to encode
 * @param dst destination buffer
 * @param sz size of destination buffer
 * @return 0 if success, -1 if the destination buffer isn't big enough
 */
int lcb_base64_encode(const char *src, lcb_SIZE len, char *dst, lcb_SIZE sz)
{
    lcb_SIZE triplets = len / 3;
    lcb_SIZE rest = len % 3;
    lcb_SIZE ii;
    const lcb_uint8_t *in = (const lcb_uint8_t *)src;
    lcb_uint8_t *out = (lcb_uint8_t *)dst;

    if (sz < (lcb_SIZE)((triplets + 1) * 4)) {
        return -1;
    }

    for (ii = 0; ii < triplets; ++ii) {
        if (encode_triplet(in, out) != 0) {
            return -1;
        }
        in += 3;
        out += 4;
    }

    if (rest > 0) {
        if (encode_rest(in, out, rest) != 0) {
            return -1;
        }
        out += 4;
    }
    *out = '\0';

    return 0;
}

int lcb_base64_encode2(const char *src, lcb_SIZE nsrc, char **dst, lcb_SIZE *ndst)
{
    lcb_SIZE len = (nsrc / 3 + 1) * 4 + 1;
    char *ptr = calloc(len, sizeof(char));
    int rc = lcb_base64_encode(src, nsrc, ptr, len);
    if (rc == 0) {
        *ndst = strlen(ptr);
        *dst = ptr;
    } else {
        free(ptr);
    }
    return rc;
}

void lcb_base64_encode_iov(lcb_IOV *iov, unsigned niov, unsigned nb, char **dst, int *ndst)
{
    lcb_SIZE nsrc = 0;
    lcb_SIZE len;
    char *ptr;
    lcb_SIZE io;

    for (io = 0; io < niov; io++) {
        nsrc += iov[io].iov_len;
    }
    if (nb < nsrc) {
        nsrc = nb;
    }
    len = (nsrc / 3 + 1) * 4 + 1;
    ptr = calloc(len, sizeof(char));

    {
        lcb_SIZE triplets = nsrc / 3;
        lcb_SIZE rest = nsrc % 3;
        lcb_uint8_t *out = (lcb_uint8_t *)ptr;
        lcb_SIZE iop, ii;
        lcb_uint8_t triplet[3];

        io = 0;
        iop = 0;

        for (ii = 0; ii < triplets; ii++) {
            int tt;

            for (tt = 0; tt < 3; tt++) {
                if (iop >= iov[io].iov_len) {
                    io++;
                    iop = 0;
                }
                triplet[tt] = ((const lcb_uint8_t *)iov[io].iov_base)[iop++];
            }
            encode_triplet(triplet, out);
            out += 4;
        }

        if (rest > 0) {
            for (ii = 0; ii < rest; ii++) {
                if (iop >= iov[io].iov_len) {
                    io++;
                    iop = 0;
                }
                triplet[ii] = ((const lcb_uint8_t *)iov[io].iov_base)[iop++];
            }
            encode_rest(triplet, out, rest);
        }
        *out = '\0';
    }

    *ndst = strlen(ptr);
    *dst = ptr;
}

static int code2val(char c)
{
    if (c >= 'A' && c <= 'Z') {
        return c - 'A';
    }
    if (c >= 'a' && c <= 'z') {
        return c - 'a' + 26;
    }
    if (c >= '0' && c <= '9') {
        return c - '0' + 52;
    }
    if (c == '+') {
        return 62;
    }
    if (c == '/') {
        return 63;
    }
    return -1;
}

lcb_SSIZE lcb_base64_decode(const char *src, lcb_SIZE nsrc, char *dst, lcb_SIZE ndst)
{
    lcb_SIZE offset = 0;
    lcb_SSIZE idx = 0;

    if (nsrc == 0) {
        *dst = '\0';
        return 0;
    }

    while (offset < nsrc) {
        int val, ins;
        lcb_U32 value;

        if (isspace((int)*src)) {
            ++offset;
            ++src;
            continue;
        }

        // We need at least 4 bytes
        if ((offset + 4) > nsrc) {
            return -1;
        }

        val = code2val(src[0]);
        if (val < 0) {
            return -1;
        }
        value = val << 18;
        val = code2val(src[1]);
        if (val < 0) {
            return -1;
        }
        value |= val << 12;

        ins = 3; /* number of characters to insert */
        if (src[2] == '=') {
            ins = 1;
        } else {
            value |= code2val(src[2]) << 6;
            if (src[3] == '=') {
                ins = 2;
            } else {
                value |= code2val(src[3]);
            }
        }

        if ((lcb_SIZE)idx >= ndst) {
            return -1;
        }
        dst[idx++] = (char)(value >> 16);
        if (ins > 1) {
            if ((lcb_SIZE)idx >= ndst) {
                return -1;
            }
            dst[idx++] = (char)(value >> 8);
            if (ins > 2) {
                if ((lcb_SIZE)idx >= ndst) {
                    return -1;
                }
                dst[idx++] = (char)(value);
            }
        }

        src += 4;
        offset += 4;
    }
    dst[idx + 1] = '\0';

    return idx;
}

lcb_SSIZE lcb_base64_decode2(const char *src, lcb_SIZE nsrc, char **dst, lcb_SIZE *ndst)
{
    // To reduce the number of reallocations, start by reserving an
    // output buffer of 75% of the input size (and add 3 to avoid dealing
    // with zero)
    lcb_SIZE len = nsrc * 3 / 4 + 3;
    char *ptr = calloc(len, sizeof(char));

    lcb_SSIZE rc = lcb_base64_decode(src, nsrc, ptr, len);
    if (rc < 0) {
        free(ptr);
    } else {
        *ndst = rc;
        *dst = ptr;
    }
    return rc;
}
