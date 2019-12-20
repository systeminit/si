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

#ifndef LCB_AUTH_PRIV_H
#define LCB_AUTH_PRIV_H
#include <libcouchbase/auth.h>

#ifdef __cplusplus
#include <string>
#include <map>

namespace lcb
{
class Authenticator
{
  public:
    typedef std::map< std::string, std::string > Map;
    // Gets the "global" username
    const std::string &username() const
    {
        return m_username;
    }

    // Gets the "global" password
    const std::string &password() const
    {
        return m_password;
    }

    // Get the username and password for a specific bucket
    const std::string username_for(const char *host, const char *port, const char *bucket) const;
    const std::string password_for(const char *host, const char *port, const char *bucket) const;

    const Map &buckets() const
    {
        return m_buckets;
    }
    Authenticator() : m_refcount(1), m_mode(LCBAUTH_MODE_CLASSIC), m_usercb(NULL), m_passcb(NULL), m_cookie(NULL) {}
    Authenticator(const Authenticator &);

    size_t refcount() const
    {
        return m_refcount;
    }
    void incref()
    {
        ++m_refcount;
    }
    void decref()
    {
        if (!--m_refcount) {
            delete this;
        }
    }
    lcb_STATUS set_mode(lcbauth_MODE mode_)
    {
        if (mode_ == LCBAUTH_MODE_DYNAMIC && (m_usercb == NULL || m_passcb == NULL)) {
            return LCB_EINVAL;
        }
        if (m_buckets.size() || m_username.size() || m_password.size()) {
            return LCB_ERROR;
        } else {
            m_mode = mode_;
            return LCB_SUCCESS;
        }
    }
    lcbauth_MODE mode() const
    {
        return m_mode;
    }
    lcb_STATUS add(const char *user, const char *pass, int flags);
    lcb_STATUS add(const std::string &user, const std::string &pass, int flags)
    {
        return add(user.c_str(), pass.c_str(), flags);
    }
    lcb_STATUS set_callbacks(void *cookie, lcb_AUTHCALLBACK usercb, lcb_AUTHCALLBACK passcb)
    {
        m_usercb = usercb;
        m_passcb = passcb;
        m_cookie = cookie;
        return LCB_SUCCESS;
    }

  private:
    Map m_buckets;
    std::string m_username;
    std::string m_password;
    size_t m_refcount;
    lcbauth_MODE m_mode;
    lcb_AUTHCALLBACK m_usercb;
    lcb_AUTHCALLBACK m_passcb;
    void *m_cookie;
};
} // namespace lcb
#endif
#endif /* LCB_AUTH_H */
