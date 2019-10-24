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

#ifndef LCB_CONNSPEC_H
#define LCB_CONNSPEC_H

#include <libcouchbase/couchbase.h>
#include "config.h"

#include <string>
#include <vector>
#include <set>
#include "hostlist.h"

#ifdef _MSC_VER
/*
 * Disable DLL interface warning. This isn't an issue since this API is
 * private anyway
 */
#pragma warning(push)
#pragma warning(disable : 4251)
#endif

namespace lcb
{
struct Spechost {
    Spechost() : port(0), type(0) {}
    lcb_U16 port;
    short type;
    std::string hostname;
    bool isSSL() const
    {
        return type == LCB_CONFIG_MCD_SSL_PORT || type == LCB_CONFIG_HTTP_SSL_PORT;
    }
    bool isHTTPS() const
    {
        return type == LCB_CONFIG_HTTP_SSL_PORT;
    }
    bool isHTTP() const
    {
        return type == LCB_CONFIG_HTTP_PORT;
    }
    bool isMCD() const
    {
        return type == LCB_CONFIG_MCD_PORT;
    }
    bool isMCDS() const
    {
        return type == LCB_CONFIG_MCD_SSL_PORT;
    }
    bool isTypeless() const
    {
        return type == 0;
    }

    bool isAnyMcd() const
    {
        return isMCD() || isMCDS() || type == LCB_CONFIG_MCCOMPAT_PORT;
    }
    bool isAnyHttp() const
    {
        return isHTTP() || isHTTPS();
    }
};

#define LCB_CONNSPEC_F_FILEONLY (1 << 4)

class LCB_CLASS_EXPORT Connspec
{
  public:
    typedef std::vector< std::pair< std::string, std::string > > Options;
    Connspec()
        : m_sslopts(0), m_implicit_port(0), m_loglevel(0), m_logredact(false), m_transports(), m_flags(0),
          m_ipv6(LCB_IPV6_DISABLED), m_logger(NULL)
    {
    }

    lcb_STATUS parse(const char *connstr, const char **errmsg = NULL);
    lcb_STATUS load(const lcb_create_st &);

    bool has_bsmode(int mode) const
    {
        return m_transports.find(mode) != m_transports.end();
    }
    bool is_bs_udef() const
    {
        return !m_transports.empty() || (m_flags & LCB_CONNSPEC_F_FILEONLY);
    }
    bool is_bs_http() const
    {
        return has_bsmode(LCB_CONFIG_TRANSPORT_HTTP);
    }
    bool is_bs_cccp() const
    {
        return has_bsmode(LCB_CONFIG_TRANSPORT_CCCP);
    }
    bool is_bs_file() const
    {
        return m_flags & LCB_CONNSPEC_F_FILEONLY;
    }

    /**
     * Whether a DNS SRV lookup can be performed on this connection string.
     * @return true if a DNS SRV lookup is possible, or false if there is
     * a parameter or format of the connection string preventing a lookup
     */
    bool can_dnssrv() const;

    /**
     * Whether the explicit `couchbase{s}+dnssrv` internal scheme is used
     */
    bool is_explicit_dnssrv() const;
    uint16_t default_port() const
    {
        return m_implicit_port;
    }
    const std::vector< Spechost > &hosts() const
    {
        return m_hosts;
    }
    const std::string &bucket() const
    {
        return m_bucket;
    }
    const std::string &username() const
    {
        return m_username;
    }
    const std::string &password() const
    {
        return m_password;
    }
    const std::string &truststorepath() const
    {
        return m_truststorepath;
    }
    const std::string &certpath() const
    {
        return m_certpath;
    }
    const std::string &keypath() const
    {
        return m_keypath;
    }
    unsigned sslopts() const
    {
        return m_sslopts;
    }
    const Options &options() const
    {
        return m_ctlopts;
    }
    lcb_logprocs *logger() const
    {
        return m_logger;
    }
    unsigned loglevel() const
    {
        return m_loglevel;
    }
    bool logredact() const
    {
        return m_logredact;
    }
    const std::string &connstr() const
    {
        return m_connstr;
    }
    void clear_hosts()
    {
        m_hosts.clear();
    }
    void add_host(const Spechost &host)
    {
        m_hosts.push_back(host);
    }
    lcb_ipv6_t ipv6_policy() const
    {
        return m_ipv6;
    }

  private:
    Options m_ctlopts;
    std::string m_bucket;
    std::string m_username;
    std::string m_password;
    std::string m_truststorepath;
    std::string m_certpath;
    std::string m_keypath;
    std::string m_connstr;
    unsigned m_sslopts; /**< SSL Options */
    std::vector< Spechost > m_hosts;
    lcb_U16 m_implicit_port; /**< Implicit port, based on scheme */
    int m_loglevel;          /* cached loglevel */
    bool m_logredact;

    inline lcb_STATUS parse_options(const char *options, const char *optend, const char **errmsg);
    inline lcb_STATUS parse_hosts(const char *hostbegin, const char *hostend, const char **errmsg);

    std::set< int > m_transports;
    unsigned m_flags; /**< Internal flags */
    lcb_ipv6_t m_ipv6;
    lcb_logprocs *m_logger;
};

#define LCB_SPECSCHEME_RAW "couchbase+explicit://"
#define LCB_SPECSCHEME_MCD "couchbase://"
#define LCB_SPECSCHEME_MCD_SSL "couchbases://"
#define LCB_SPECSCHEME_HTTP "http://"
#define LCB_SPECSCHEME_HTTP_SSL "https-internal://"
#define LCB_SPECSCHEME_MCCOMPAT "memcached://"
#define LCB_SPECSCHEME_SRV "couchbase+dnssrv://"
#define LCB_SPECSCHEME_SRV_SSL "couchbases+dnssrv://"

// Standalone functionality:
lcb_STATUS dnssrv_query(const char *name, Hostlist &hostlist);

Hostlist *dnssrv_getbslist(const char *addr, bool is_ssl, lcb_STATUS &errout);

} // namespace lcb

#endif

#ifdef _MSC_VER
#pragma warning(pop)
#endif
