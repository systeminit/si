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

/**
 * This file contains the static part of the configure script. Please add
 * all platform specific conditional code to this file.
 *
 * @author Trond Norbye
 */
#ifndef LIBCOUCHBASE_CONFIG_STATIC_H
#define LIBCOUCHBASE_CONFIG_STATIC_H 1

#ifdef HAVE_SYS_TYPES_H
#include <sys/types.h>
#endif

#if !defined HAVE_STDINT_H && defined _WIN32 && defined(_MSC_VER)
#include "win_stdint.h"
#else
#include <stdint.h>
#endif

#ifdef HAVE_SYS_SOCKET_H
#include <sys/socket.h>
#endif

#ifdef HAVE_NETINET_IN_H
#include <netinet/in.h>
#endif

#ifdef HAVE_INTTYPES_H
#ifdef __cplusplus
#define __STDC_FORMAT_MACROS 1
#endif
#include <inttypes.h>
#elif defined(_WIN32)
#ifndef PRIx64
#define PRIx64 "I64x"
#endif
#ifndef PRId64
#define PRId64 "I64d"
#endif
#ifndef PRIu64
#define PRIu64 "I64u"
#endif
#endif

#ifdef HAVE_NETDB_H
#include <netdb.h>
#endif

#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#endif

#ifdef HAVE_SYS_TIME_H
#include <sys/time.h>
#endif

#ifdef HAVE_SYS_UIO_H
#include <sys/uio.h>
#endif

#ifdef HAVE_STRINGS_H
#include <strings.h>
#endif

#ifdef HAVE_FCNTL_H
#include <fcntl.h>
#endif

#ifdef HAVE_DLFCN_H
#include <dlfcn.h>
#endif

#ifdef HAVE_ARPA_INET_H
#include <arpa/inet.h>
#endif

/* Standard C includes */
#include <limits.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>

#ifndef PATH_MAX
#define PATH_MAX 1024
#endif

#ifdef _WIN32
#include <libcouchbase/plugins/io/wsaerr.h>

#ifndef __MINGW32__

#if defined(_MSC_VER) && _MSC_VER < 1900
#define snprintf _snprintf
#endif

#define strcasecmp(a, b) _stricmp(a, b)
#define strncasecmp(a, b, c) _strnicmp(a, b, c)
#undef strdup
#define strdup _strdup
#endif

#else
#define INVALID_SOCKET -1
#define SOCKET_ERROR -1
#endif /* _WIN32 */

#if defined(HAVE_HTONLL)
#define lcb_htonll htonll
#define lcb_ntohll ntohll
#elif defined(WORDS_BIGENDIAN)
#define lcb_ntohll(a) a
#define lcb_htonll(a) a
#else
#define lcb_ntohll(a) lcb_byteswap64(a)
#define lcb_htonll(a) lcb_byteswap64(a)
#endif /* HAVE_HTONLL */

#ifdef __cplusplus
extern "C" {
#endif
extern uint64_t lcb_byteswap64(uint64_t val);
extern uint16_t lcb_byteswap16(uint16_t val);
#ifdef __cplusplus
}
#endif

#define lcb_ntohs(a) lcb_byteswap16(a)
#define lcb_htons(a) lcb_byteswap16(a)

#ifdef linux
#undef ntohs
#undef ntohl
#undef htons
#undef htonl
#endif

#ifndef HAVE_GETHRTIME
#ifdef __cplusplus
extern "C" {
#endif
typedef uint64_t hrtime_t;
extern hrtime_t gethrtime(void);
#ifdef __cplusplus
}
#endif
#endif

#if defined(EWOULDBLOCK) && defined(EAGAIN) && EWOULDBLOCK != EAGAIN
#define USE_EAGAIN 1
#endif

#endif /* LIBCOUCHBASE_CONFIG_STATIC_H */
