/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#ifndef LIBCOUCHBASE_WIN_ERRNO_SOCK_H
#define LIBCOUCHBASE_WIN_ERRNO_SOCK_H 1

#include <errno.h>

#ifndef EWOULDBLOCK
#define EWOULDBLOCK             WSAEWOULDBLOCK
#endif

#ifndef EINPROGRESS
#define EINPROGRESS             WSAEINPROGRESS
#endif

#ifndef EALREADY
#define EALREADY                WSAEALREADY
#endif

#ifndef ENOTSOCK
#define ENOTSOCK                WSAENOTSOCK
#endif

#ifndef EDESTADDRREQ
#define EDESTADDRREQ            WSAEDESTADDRREQ
#endif

#ifndef EMSGSIZE
#define EMSGSIZE                WSAEMSGSIZE
#endif

#ifndef EPROTOTYPE
#define EPROTOTYPE              WSAEPROTOTYPE
#endif

#ifndef ENOPROTOOPT
#define ENOPROTOOPT             WSAENOPROTOOPT
#endif

#ifndef EPROTONOSUPPORT
#define EPROTONOSUPPORT         WSAEPROTONOSUPPORT
#endif

#ifndef ESOCKTNOSUPPORT
#define ESOCKTNOSUPPORT         WSAESOCKTNOSUPPORT
#endif

#ifndef EOPNOTSUPP
#define EOPNOTSUPP              WSAEOPNOTSUPP
#endif

#ifndef ENOPROTOOPT
#define ENOPROTOOPT             WSAENOPROTOOPT
#endif

#ifndef EPROTONOSUPPORT
#define EPROTONOSUPPORT         WSAEPROTONOSUPPORT
#endif

#ifndef ESOCKTNOSUPPORT
#define ESOCKTNOSUPPORT         WSAESOCKTNOSUPPORT
#endif

#ifndef EPFNOSUPPORT
#define EPFNOSUPPORT            WSAEPFNOSUPPORT
#endif

#ifndef EAFNOSUPPORT
#define EAFNOSUPPORT            WSAEAFNOSUPPORT
#endif

#ifndef EADDRINUSE
#define EADDRINUSE              WSAEADDRINUSE
#endif

#ifndef EADDRNOTAVAIL
#define EADDRNOTAVAIL           WSAEADDRNOTAVAIL
#endif

#ifndef ENETDOWN
#define ENETDOWN                WSAENETDOWN
#endif

#ifndef ENETUNREACH
#define ENETUNREACH             WSAENETUNREACH
#endif

#ifndef ENETRESET
#define ENETRESET               WSAENETRESET
#endif

#ifndef ECONNABORTED
#define ECONNABORTED            WSAECONNABORTED
#endif

#ifndef ECONNRESET
#define ECONNRESET              WSAECONNRESET
#endif

#ifndef ENOBUFS
#define ENOBUFS                 WSAENOBUFS
#endif

#ifndef EISCONN
#define EISCONN                 WSAEISCONN
#endif

#ifndef ENOTCONN
#define ENOTCONN                WSAENOTCONN
#endif

#ifndef ESHUTDOWN
#define ESHUTDOWN               WSAESHUTDOWN
#endif

#ifndef ETOOMANYREFS
#define ETOOMANYREFS            WSAETOOMANYREFS
#endif

#ifndef ETIMEDOUT
#define ETIMEDOUT               WSAETIMEDOUT
#endif

#ifndef ECONNREFUSED
#define ECONNREFUSED            WSAECONNREFUSED
#endif

#ifndef ELOOP
#define ELOOP                   WSAELOOP
#endif

/*
#ifndef ENAMETOOLONG
#define ENAMETOOLONG            WSAENAMETOOLONG
#endif
*/

#ifndef EHOSTDOWN
#define EHOSTDOWN               WSAEHOSTDOWN
#endif

#ifndef EHOSTUNREACH
#define EHOSTUNREACH            WSAEHOSTUNREACH
#endif

/*
#ifndef ENOTEMPTY
#define ENOTEMPTY               WSAENOTEMPTY
#endif
*/

#ifndef EPROCLIM
#define EPROCLIM                WSAEPROCLIM
#endif

#ifndef EUSERS
#define EUSERS                  WSAEUSERS
#endif

#ifndef EDQUOT
#define EDQUOT                  WSAEDQUOT
#endif

#ifndef ESTALE
#define ESTALE                  WSAESTALE
#endif

#ifndef EREMOTE
#define EREMOTE                 WSAEREMOTE
#endif

#ifndef EPROTO
#define EPROTO                  WSAEPROTONOSUPPORT
#endif

#ifndef ECANCELED
#define ECANCELED               WSAECANCELLED
#endif

/** Some versions have this; some don't */
#ifndef ENOTSUP
#define ENOTSUP                 -1
#endif

#endif
