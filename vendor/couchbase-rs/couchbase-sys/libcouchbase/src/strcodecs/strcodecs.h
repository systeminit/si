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

#ifndef LCB_STRCODECS_H
#define LCB_STRCODECS_H
#include <ctype.h>
#include <libcouchbase/couchbase.h>

#ifdef __cplusplus
extern "C" {
#endif
lcb_STATUS lcb_urlencode_path(const char *path, lcb_size_t npath, char **out, lcb_size_t *nout);

/**
 * Decode a string from 'percent-encoding'
 * @param in The input string
 * @param[in,out] out The output buffer.
 *                If upon entry, out is not-NULL, it is assumed to be a buffer
 *                containing sufficient size for the percent encoding (up to
 *                3x the size of input). Otherwise on exit this will contain
 *                a malloc'd buffer which should be free()d when no longer
 *                required.
 * @param n The size of the input buffer. May be -1 if NUL-terminated
 * @return 0 if converted successfuly, -1 on error
 */
int lcb_urldecode(const char *in, char *out, lcb_SSIZE n);

/**
 * Base64 encode a string into an output buffer.
 * @param src string to encode
 * @param len size of source buffer
 * @param dst destination buffer
 * @param sz size of destination buffer
 * @return 0 if success, -1 if the destination buffer isn't big enough
 */
int lcb_base64_encode(const char *src, lcb_SIZE len, char *dst, lcb_SIZE sz);

/**
 * Base64 encode a string into an output buffer.
 * @param src string to encode
 * @param len size of source buffer
 * @param dst destination buffer
 * @param sz size of destination buffer
 * @return 0 if success, -1 if function wasn't able to allocate enough memory
 */
int lcb_base64_encode2(const char *src, lcb_SIZE len, char **dst, lcb_SIZE *sz);

lcb_SSIZE lcb_base64_decode(const char *src, lcb_SIZE nsrc, char *dst, lcb_SIZE ndst);
lcb_SSIZE lcb_base64_decode2(const char *src, lcb_SIZE nsrc, char **dst, lcb_SIZE *ndst);

void lcb_base64_encode_iov(lcb_IOV *iov, unsigned niov, unsigned nb, char **dst, int *ndst);

/**
 * Encodes a string suitable for being passed as either a key or value in an
 * "HTTP Form" per application/x-www-form-urlencoded
 * @param s The input string
 * @param n The size of the input
 * @param out The output buffer - should be at least 3x the input length
 * @return The number of bytes actually used in the output buffer.
 */
size_t lcb_formencode(const char *s, size_t n, char *out);

int lcb_leb128_encode(lcb_U32 value, lcb_U8 *buf);

#ifdef __cplusplus
}

#include <string>
namespace lcb
{
namespace strcodecs
{
template < typename Ti, typename To > bool urldecode(Ti first, Ti last, To out, size_t &nout)
{
    for (; first != last && *first != '\0'; ++first) {
        if (*first == '%') {
            char nextbuf[3] = {0};
            size_t jj = 0;
            first++;
            nextbuf[0] = *first;
            for (; first != last && jj < 2; ++jj) {
                nextbuf[jj] = *first;
                if (jj != 1) {
                    first++;
                }
            }
            if (jj != 2) {
                return false;
            }

            unsigned octet = 0;
            if (sscanf(nextbuf, "%2X", &octet) != 1) {
                return false;
            }

            *out = static_cast< char >(octet);
        } else {
            *out = *first;
        }

        out++;
        nout++;
    }
    return true;
}

inline bool urldecode(const char *input, char *output)
{
    const char *endp = input + strlen(input);
    size_t nout = 0;
    if (urldecode(input, endp, output, nout)) {
        output[nout] = '\0';
        return true;
    }
    return false;
}

inline bool urldecode(char *in_out)
{
    return urldecode(in_out, in_out);
}

inline bool urldecode(std::string &s)
{
    size_t n = 0;
    if (urldecode(s.begin(), s.end(), s.begin(), n)) {
        s.resize(n);
        return true;
    }
    return false;
}

namespace priv
{
inline bool is_legal_urichar(char c)
{
    unsigned char uc = (unsigned char)c;
    if (isalpha(uc) || isdigit(uc)) {
        return true;
    }
    switch (uc) {
        case '-':
        case '_':
        case '.':
        case '~':
        case '!':
        case '*':
        case '\'':
        case '(':
        case ')':
        case ';':
        case ':':
        case '@':
        case '&':
        case '=':
        case '+':
        case '$':
        case ',':
        case '/':
        case '?':
        case '#':
        case '[':
        case ']':
            return true;
        default:
            break;
    }
    return false;
}

template < typename T > inline bool is_already_escape(T first, T last)
{
    first++; // ignore '%'
    size_t jj;
    for (jj = 0; first != last && jj < 2; ++jj, ++first) {
        if (!isxdigit(*first)) {
            return false;
        }
    }
    if (jj != 2) {
        return false;
    }
    return true;
}
} // namespace priv

template < typename Ti, typename To > bool urlencode(Ti first, Ti last, To &o, bool check_encoded = true)
{
    // If re-encoding detection is enabled, this flag indicates not to
    // re-encode
    bool skip_encoding = false;

    for (; first != last; ++first) {
        if (!skip_encoding && check_encoded) {
            if (*first == '%') {
                skip_encoding = priv::is_already_escape(first, last);
            } else if (*first == '+') {
                skip_encoding = true;
            }
        }
        if (skip_encoding || priv::is_legal_urichar(*first)) {
            if (skip_encoding && *first != '%' && !priv::is_legal_urichar(*first)) {
                return false;
            }

            o.insert(o.end(), first, first + 1);
        } else {
            unsigned int c = static_cast< unsigned char >(*first);
            size_t numbytes;

            if ((c & 0x80) == 0) { /* ASCII character */
                numbytes = 1;
            } else if ((c & 0xE0) == 0xC0) { /* 110x xxxx */
                numbytes = 2;
            } else if ((c & 0xF0) == 0xE0) { /* 1110 xxxx */
                numbytes = 3;
            } else if ((c & 0xF8) == 0xF0) { /* 1111 0xxx */
                numbytes = 4;
            } else {
                return false;
            }

            do {
                char buf[4];
                sprintf(buf, "%%%02X", static_cast< unsigned char >(*first));
                o.insert(o.end(), &buf[0], &buf[0] + 3);
            } while (--numbytes && ++first != last);
        }
    }
    return true;
}
template < typename Tin, typename Tout > bool urlencode(const Tin &in, Tout &out)
{
    return urlencode(in.begin(), in.end(), out);
}

/* See: https://url.spec.whatwg.org/#urlencoded-serializing: */
/*
 * 0x2A
 * 0x2D
 * 0x2E
 * 0x30 to 0x39
 * 0x41 to 0x5A
 * 0x5F
 * 0x61 to 0x7A
 *  Append a code point whose value is byte to output.
 * Otherwise
 *  Append byte, percent encoded, to output.
 */
template < typename Ti, typename To > void formencode(Ti first, Ti last, To &out)
{
    for (; first != last; ++first) {
        unsigned char c = *first;
        if (isalnum(c)) {
            out.insert(out.end(), first, first + 1);
            continue;
        } else if (c == ' ') {
            char tmp = '+';
            out.insert(out.end(), &tmp, &tmp + 1);
        } else if ((c == 0x2A || c == 0x2D || c == 0x2E) || (c >= 0x30 && c <= 0x39) || (c >= 0x41 && c <= 0x5A) ||
                   (c == 0x5F) || (c >= 0x60 && c <= 0x7A)) {
            out.insert(out.end(), static_cast< char >(c));
        } else {
            char buf[3] = {0};
            out.insert(out.end(), '%');
            sprintf(buf, "%02X", c);
            out.insert(out.end(), &buf[0], &buf[0] + 2);
        }
    }
}

} // namespace strcodecs
} // namespace lcb
#endif
#endif
