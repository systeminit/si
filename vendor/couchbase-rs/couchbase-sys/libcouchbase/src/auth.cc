/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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

#include <libcouchbase/couchbase.h>
#include "auth-priv.h"

using namespace lcb;

lcb_AUTHENTICATOR *
lcbauth_new()
{
    return new Authenticator();
}

lcb_STATUS
lcbauth_add_pass(lcb_AUTHENTICATOR *auth, const char *u, const char *p, int flags)
{
    return auth->add(u, p, flags);
}

lcb_STATUS
Authenticator::add(const char *u, const char *p, int flags)
{
    if (!u) {
        return LCB_EINVAL;
    }

    if (!(flags & (LCBAUTH_F_BUCKET|LCBAUTH_F_CLUSTER))) {
        return LCB_EINVAL;
    }

    if (m_mode == LCBAUTH_MODE_RBAC && (flags & LCBAUTH_F_BUCKET)) {
        return LCB_OPTIONS_CONFLICT;
    }

    if (flags & LCBAUTH_F_CLUSTER) {
        if (p) {
            m_username = u;
            m_password = p;
        } else {
            m_username.clear();
            m_password.clear();
        }
    }

    if (flags & LCBAUTH_F_BUCKET) {
        if (p) {
            m_buckets[u] = p;
        } else {
            m_buckets.erase(u);
        }
    }

    return LCB_SUCCESS;
}

static const std::string EmptyString;

const std::string Authenticator::username_for(const char *host, const char *port, const char *bucket) const
{
    switch (m_mode) {
        case LCBAUTH_MODE_RBAC:
            return m_username;
        case LCBAUTH_MODE_DYNAMIC:
            if (m_usercb != NULL) {
                return m_usercb(m_cookie, host, port, bucket);
            }
            break;
        case LCBAUTH_MODE_CLASSIC:
            // Find bucket specific credentials:
            const Map::const_iterator it = m_buckets.find(bucket);
            if (it != m_buckets.end()) {
                return it->first;
            }
            break;
    }
    return EmptyString;
}

const std::string Authenticator::password_for(const char *host, const char *port, const char *bucket) const
{
    switch (m_mode) {
        case LCBAUTH_MODE_RBAC:
            return m_password;
        case LCBAUTH_MODE_DYNAMIC:
            if (m_passcb != NULL) {
                return m_passcb(m_cookie, host, port, bucket);
            }
            break;
        case LCBAUTH_MODE_CLASSIC:
            const Map::const_iterator it = m_buckets.find(bucket);
            if (it != m_buckets.end()) {
                return it->second;
            }
            break;
    }
    return EmptyString;
}

void
lcbauth_ref(lcb_AUTHENTICATOR *auth)
{
    auth->incref();
}

void
lcbauth_unref(lcb_AUTHENTICATOR *auth)
{
    auth->decref();
}

Authenticator::Authenticator(const Authenticator &other)
    : m_buckets(other.m_buckets), m_username(other.m_username), m_password(other.m_password), m_refcount(1),
      m_mode(other.m_mode), m_usercb(other.m_usercb), m_passcb(other.m_passcb), m_cookie(other.m_cookie)
{
}

lcb_AUTHENTICATOR *
lcbauth_clone(const lcb_AUTHENTICATOR *src) {
    return new Authenticator(*src);
}

lcb_STATUS
lcbauth_set_mode(lcb_AUTHENTICATOR *src, lcbauth_MODE mode) {
    return src->set_mode(mode);
}

lcb_STATUS lcbauth_set_callbacks(lcb_AUTHENTICATOR *auth, void *cookie, lcb_AUTHCALLBACK usercb,
                                  lcb_AUTHCALLBACK passcb)
{
    return auth->set_callbacks(cookie, usercb, passcb);
}
