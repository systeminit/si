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

#include "connspec.h"
#include "hostlist.h"
#include "strcodecs/strcodecs.h"
#include <stdio.h>
#include <ctype.h>
#include <stdlib.h>

#define SET_ERROR(msg) \
    *errmsg = msg; \
    return LCB_EINVAL;

#define F_HASBUCKET (1<<0)
#define F_HASPASSWD (1<<1)
#define F_HASUSER (1<<2)
#define F_SSLSCHEME (1<<3)
#define F_FILEONLY (1<<4)
#define F_DNSSRV (1<<5)
#define F_DNSSRV_EXPLICIT ( (1<<6) |F_DNSSRV)

using namespace lcb;

static int string_to_porttype(const char *s) {
    if (!strcmp(s, "HTTP")) {
        return LCB_CONFIG_HTTP_PORT;
    } else if (!strcmp(s, "MCD")) {
        return LCB_CONFIG_MCD_PORT;
    } else if (!strcmp(s, "HTTPS")) {
        return LCB_CONFIG_HTTP_SSL_PORT;
    } else if (!strcmp(s, "MCDS")) {
        return LCB_CONFIG_MCD_SSL_PORT;
    } else if (!strcmp(s, "MCCOMPAT")) {
        return LCB_CONFIG_MCCOMPAT_PORT;
    } else {
        return -1;
    }
}

lcb_STATUS
Connspec::parse_hosts(const char *hostbegin,
    const char *hostend, const char **errmsg)
{
    std::string decoded(hostbegin, hostend);
    if (!strcodecs::urldecode(decoded)) {
        SET_ERROR("Couldn't decode from URL encoding!");
    }

    const char *c = decoded.c_str();

    while (*c) {
        // get the current host
        const char *curend;
        unsigned curlen, hostlen, hoststart;
        int rv;

        /* Seek ahead, chopping off any ',' */
        while (*c == ',' || *c == ';') {
            if (*(++c) ==  '\0') {
                return LCB_SUCCESS;
            }
        }

        /* Find the end */
        curend = strpbrk(c, ",;");
        if (!curend) {
            curend = c + strlen(c);
        }
        curlen = curend - c;
        if (!curlen) {
            continue;
        }

        std::string scratch(c, curlen);
        c = curend;

        /* weed out erroneous characters */
        if (scratch.find("://") != std::string::npos) {
            SET_ERROR("Detected '://' inside hostname");
        }

        size_t colonpos = scratch.find(":");
        size_t rcolonpos = scratch.rfind(":");
        std::string port;

        hoststart = 0;
        if (colonpos == std::string::npos) {
            hostlen = scratch.size();
        } else if (colonpos == rcolonpos) {
            if (colonpos == 0 || colonpos == scratch.size() - 1) {
                SET_ERROR("First or last character in spec is colon!");
            } else {
                hostlen = colonpos;
                port = scratch.substr(colonpos + 1);
            }
        } else {
            size_t rbracket = scratch.rfind(']');
            if (scratch[0] == '[' && rbracket != std::string::npos) {
                hoststart = 1;
                hostlen = rbracket - hoststart;
                if (scratch.size() > rbracket + 1) {
                    port = scratch.substr(rbracket + 2);
                }
            } else {
                hostlen = scratch.size();
            }
        }

        if (m_flags & F_DNSSRV_EXPLICIT) {
            if (!m_hosts.empty()) {
                SET_ERROR("Only a single host is allowed with DNS SRV");
            } else if (!port.empty()) {
                SET_ERROR("Port cannot be specified with DNS SRV");
            }
        }

        m_hosts.resize(m_hosts.size() + 1);
        Spechost *dh = &m_hosts.back();
        dh->hostname = scratch.substr(hoststart, hostlen);

        if (port.empty()) {
            continue;
        }

        char hpdummy[256] = { 0 };
        int itmp;
        if (port.size() > sizeof hpdummy) {
            SET_ERROR("Port spec too big!");
        }

        /* we have a port. The format is port=proto */
        rv = sscanf(port.c_str(), "%d=%s", &itmp, hpdummy);
        if (rv == 2) {
            for (char *tmp = hpdummy; *tmp; tmp++) {
                *tmp = toupper(*tmp);
            }
            // Both host and port. Not ambiguous
            if (-1 == (dh->type = string_to_porttype(hpdummy))) {
                SET_ERROR("Unrecognized protocol specified. Recognized are "
                    "HTTP, HTTPS, MCD, MCDS");
            }
        } else if (rv == 1 && m_implicit_port) {
            // Port only, but we have a scheme. No need to set this implicitly
            // in the host object
            if (m_implicit_port == itmp) {
                continue;
            }

            // couchbase scheme with :8091 specification. Just ignore
            if (itmp == LCB_CONFIG_HTTP_PORT &&
                    m_implicit_port == LCB_CONFIG_MCD_PORT) {
                /* Honest 'couchbase://host:8091' mistake */
                continue;
            }
            dh->type = m_implicit_port;
        } else {
            SET_ERROR("Port must be specified with protocol (host:port=proto)");
        }
        dh->port = itmp;
    }
    return LCB_SUCCESS;
}

lcb_STATUS
Connspec::parse_options(
    const char *options_, const char *specend, const char **errmsg)
{
    while (options_ != NULL && options_ < specend) {
        unsigned curlen;
        const char *curend;

        if (*options_ == '&') {
            options_++;
            continue;
        }

        curend = strchr(options_, '&');
        if (!curend) {
            curend = specend;
        }

        curlen = curend - options_;
        std::vector<char> optpair(options_, options_ + curlen);
        optpair.push_back('\0');

        options_ = curend+1;

        char *key = &optpair[0];
        char *value = strchr(key, '=');
        if (!value) {
            SET_ERROR("Option must be specified as a key=value pair");
        }

        *(value++) = '\0';
        if (!*value) {
            SET_ERROR("Value cannot be empty");
        }
        if (! (strcodecs::urldecode(value) && strcodecs::urldecode(key))) {
            SET_ERROR("Couldn't decode key or value!");
        }
        if (!strcmp(key, "bootstrap_on")) {
            m_transports.clear();
            if (!strcmp(value, "cccp")) {
                m_transports.insert(LCB_CONFIG_TRANSPORT_CCCP);
            } else if (!strcmp(value, "http")) {
                m_transports.insert(LCB_CONFIG_TRANSPORT_HTTP);
            } else if (!strcmp(value, "all")) {
                m_transports.insert(LCB_CONFIG_TRANSPORT_CCCP);
                m_transports.insert(LCB_CONFIG_TRANSPORT_HTTP);
            } else if (!strcmp(value, "file_only")) {
                m_flags |= LCB_CONNSPEC_F_FILEONLY;
            } else {
                SET_ERROR("Value for bootstrap_on must be 'cccp', 'http', or 'all'");
            }
        } else if (!strcmp(key, "username") || !strcmp(key, "user")) {
            if (! (m_flags & F_HASUSER)) {
                m_username = value;
            }
        } else if (!strcmp(key, "password") || !strcmp(key, "pass")) {
            if (! (m_flags & F_HASPASSWD)) {
                m_password = value;
            }
        } else if (!strcmp(key, "ssl")) {
            if (!strcmp(value, "off")) {
                if (m_flags & F_SSLSCHEME) {
                    SET_ERROR("SSL scheme specified, but ssl=off found in options");
                }
                m_sslopts &= (~LCB_SSL_ENABLED);
            } else if (!strcmp(value, "on")) {
                m_sslopts |= LCB_SSL_ENABLED;
            } else if (!strcmp(value, "no_verify")) {
                m_sslopts |= LCB_SSL_ENABLED|LCB_SSL_NOVERIFY;
            } else if (!strcmp(value, "no_global_init")) {
                m_sslopts |= LCB_SSL_NOGLOBALINIT;
            } else {
                SET_ERROR("Invalid value for 'ssl'. Choices are on, off, and no_verify");
            }
        } else if (!strcmp(key, "truststorepath")) {
            if (! (m_flags & F_SSLSCHEME)) {
                SET_ERROR("Trust store path must be specified with SSL host or scheme");
            }
            m_truststorepath = value;
        } else if (!strcmp(key, "certpath")) {
            if (! (m_flags & F_SSLSCHEME)) {
                SET_ERROR("Certificate path must be specified with SSL host or scheme");
            }
            m_certpath = value;
        } else if (!strcmp(key, "keypath")) {
            if (! (m_flags & F_SSLSCHEME)) {
                SET_ERROR("Private key path must be specified with SSL host or scheme");
            }
            m_keypath = value;
        } else if (!strcmp(key, "console_log_level")) {
            if (sscanf(value, "%d", &m_loglevel) != 1) {
                SET_ERROR("console_log_level must be a numeric value");
            }
        } else if (!strcmp(key, "log_redaction")) {
            int btmp = 0;
            if (!strcmp(value, "on") || !strcmp(value, "true")) {
                btmp = 1;
            } else if (!strcmp(value, "off") || !strcmp(value, "false")) {
                btmp = 0;
            } else if (sscanf(value, "%d", &btmp) != 1) {
                SET_ERROR("log_redaction must have numeric (boolean) value");
            }
            m_logredact = btmp != 0;
        } else if (!strcmp(key, "dnssrv")) {
            if ((m_flags & F_DNSSRV_EXPLICIT) == F_DNSSRV_EXPLICIT) {
                SET_ERROR("Cannot use dnssrv scheme with dnssrv option");
            }
            int btmp = 0;
            if (!strcmp(value, "on") || !strcmp(value, "true")) {
                btmp = 1;
            } else if (!strcmp(value, "off") || !strcmp(value, "false")) {
                btmp = 0;
            } else if (sscanf(value, "%d", &btmp) != 1) {
                SET_ERROR("dnssrv must have numeric (boolean) value");
            }
            if (btmp) {
                m_flags |= F_DNSSRV;
            } else {
                m_flags &= ~F_DNSSRV_EXPLICIT;
            }
        } else if (!strcmp(key, "ipv6")) {
            if (!strcmp(value, "only")) {
                m_ipv6 = LCB_IPV6_ONLY;
            } else if (!strcmp(value, "disabled")) {
                m_ipv6 = LCB_IPV6_DISABLED;
            } else if (!strcmp(value, "allow")) {
                m_ipv6 = LCB_IPV6_ALLOW;
            } else {
                SET_ERROR("Value for ipv6 must be 'disabled', 'allow', or 'only'");
            }
        } else {
            m_ctlopts.push_back(std::make_pair(key, value));
        }
    }
    if (!m_keypath.empty() && m_certpath.empty()) {
        SET_ERROR("Private key path must be specified with certificate path");
    }

    return LCB_SUCCESS;
}

lcb_STATUS
Connspec::parse(const char *connstr_, const char **errmsg)
{
    lcb_STATUS err = LCB_SUCCESS;
    const char *errmsg_s; /* stack based error message pointer */
    const char *hlend; /* end of hosts list */
    const char *bucket_s = NULL; /* beginning of bucket (path) string */
    const char *options_ = NULL; /* beginning of options (query) string */
    const char *specend = NULL; /* end of spec */
    unsigned speclen; /* length of spec string */
    const char *found_scheme = NULL;

    if (!errmsg) {
        errmsg = &errmsg_s;
    }

    if (!connstr_) {
        connstr_ = "couchbase://";
    }

    m_connstr = connstr_;

#define SCHEME_MATCHES(scheme_const) \
    strncmp(connstr_, scheme_const, sizeof(scheme_const)-1) == 0 && \
    (found_scheme = scheme_const)

    if (SCHEME_MATCHES(LCB_SPECSCHEME_MCD_SSL)) {
        m_implicit_port = LCB_CONFIG_MCD_SSL_PORT;
        m_sslopts |= LCB_SSL_ENABLED;
        m_flags |= F_SSLSCHEME;

    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_HTTP_SSL)) {
        m_implicit_port = LCB_CONFIG_HTTP_SSL_PORT;
        m_sslopts |= LCB_SSL_ENABLED;
        m_flags |= F_SSLSCHEME;

    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_HTTP)) {
        m_implicit_port = LCB_CONFIG_HTTP_PORT;

    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_MCD)) {
        m_implicit_port = LCB_CONFIG_MCD_PORT;

    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_RAW)) {
        m_implicit_port = 0;
    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_MCCOMPAT)) {
        m_implicit_port = LCB_CONFIG_MCCOMPAT_PORT;
    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_SRV)) {
        m_implicit_port = LCB_CONFIG_MCD_PORT;
        m_flags |= F_DNSSRV_EXPLICIT;
    } else if (SCHEME_MATCHES(LCB_SPECSCHEME_SRV_SSL)) {
        m_implicit_port = LCB_CONFIG_MCD_SSL_PORT;
        m_sslopts |= LCB_SSL_ENABLED;
        m_flags |= F_SSLSCHEME | F_DNSSRV_EXPLICIT;
    } else {
        /* If we don't have a scheme at all: */
        if (strstr(connstr_, "://")) {
            SET_ERROR("String must begin with 'couchbase://, 'couchbases://', or 'http://'");
        } else {
            found_scheme = "";
            m_implicit_port = LCB_CONFIG_HTTP_PORT;
        }
    }

    connstr_ += strlen(found_scheme);
    speclen = strlen(connstr_);
    specend = connstr_ + speclen;

    /* Hosts end where either options or the bucket itself begin */
    if ((hlend = strpbrk(connstr_, "?/"))) {
        if (*hlend == '?') {
            /* Options first */
            options_ = hlend + 1;

        } else if (*hlend == '/') {
            /* Bucket first. Options follow bucket */
            bucket_s = hlend + 1;
            if ((options_ = strchr(bucket_s, '?'))) {
                options_++;
            }
        }
    } else {
        hlend = specend;
    }

    if (bucket_s != NULL) {
        unsigned blen;
        const char *b_end = options_ ? options_-1 : specend;
        /* scan each of the options */
        blen = b_end - bucket_s;

        m_bucket.assign(bucket_s, bucket_s + blen);
        if (!(m_flags & F_HASBUCKET)) {
            if (!strcodecs::urldecode(m_bucket)) {
                SET_ERROR("Couldn't decode bucket string");
            }
        }
        if (m_bucket.empty()) {
            SET_ERROR("Bucket name is set to empty");
        }
    } else {
        m_bucket = "default";
    }

    if ((err = parse_hosts(connstr_, hlend, errmsg)) != LCB_SUCCESS) {
        goto GT_DONE;
    }

    if (m_hosts.empty()) {
        m_hosts.resize(m_hosts.size()+1);
        m_hosts.back().hostname = "localhost";
    } else if (m_hosts.size() == 1 && m_hosts[0].isTypeless()) {
        m_flags |= F_DNSSRV;
    }

    if (options_ != NULL) {
        if ((err = parse_options(options_, specend, errmsg)) != LCB_SUCCESS) {
            goto GT_DONE;
        }
    }
    GT_DONE:
    return err;
}

#define MAYBEDUP(s) ((s) && (*s)) ? strdup(s) : NULL

static lcb_STATUS
convert_hosts(std::string& outstr, const char *instr, int deflport)
{
    Hostlist hl;
    lcb_STATUS rc = hl.add(instr, -1, deflport);
    if (rc != LCB_SUCCESS) {
        return rc;
    }

    for (size_t ii = 0; ii < hl.size(); ii++) {
        const lcb_host_t& src = hl[ii];
        int port, rv;
        outstr.append(src.host);
        rv = sscanf(src.port, "%d", &port);
        if (rv && port != deflport) {
            const char *proto;
            char tmpbuf[256];
            if (deflport == LCB_CONFIG_MCD_PORT) {
                proto = "mcd";
            } else {
                proto = "http";
            }
            sprintf(tmpbuf, ":%d=%s", port, proto);
            outstr.append(tmpbuf);
        }
        outstr.append(",");
    }
    return LCB_SUCCESS;
}

#define TRYDUP(s) (s) ? strdup(s) : NULL
lcb_STATUS
Connspec::load(const lcb_create_st& cropts)
{
    const char *errmsg;
    const struct lcb_create_st2 *cr2 = &cropts.v.v2;
    lcb_STATUS err = LCB_SUCCESS;

    /* handle overrides */
    if (cr2->bucket && *cr2->bucket) {
        m_flags |= F_HASBUCKET;
        m_bucket = cr2->bucket;
    }

    if (cr2->user && *cr2->user) {
        m_flags |= F_HASUSER;
        m_username = cr2->user;
    }

    if (cr2->passwd && *cr2->passwd) {
        m_flags |= F_HASPASSWD;
        m_password = cr2->passwd;
    }

    if (cropts.version == 4) {
        m_logger = cropts.v.v4.logger;
    }

    if (cropts.version == 3 || cropts.version == 4) {
        return parse(cropts.v.v3.connstr, &errmsg);
    }

    if (cropts.version > 2 || cropts.version < 0) {
        return LCB_NOT_SUPPORTED;
    }

    m_connstr = LCB_SPECSCHEME_RAW;
    if (cr2->host) {
        err = convert_hosts(m_connstr, cr2->host, LCB_CONFIG_HTTP_PORT);
        if (err != LCB_SUCCESS) {
            goto GT_DONE;
        }
    }

    if (cropts.version == 2 && cr2->mchosts) {
        err = convert_hosts(m_connstr, cr2->mchosts, LCB_CONFIG_MCD_PORT);
        if (err != LCB_SUCCESS) {
            goto GT_DONE;
        }
    }

    if (cropts.version < 3 && cr2->bucket) {
        m_connstr += "/";
        m_connstr += cr2->bucket;
    }

    m_connstr += '?';
    err = parse(m_connstr.c_str(), &errmsg);
    if (err == LCB_SUCCESS && cropts.version == 2 && cr2->transports) {
        /* copy over bootstrap list */
        for (size_t ii = 0; ii < LCB_CONFIG_TRANSPORT_MAX; ii++) {
            if (cr2->transports[ii] == LCB_CONFIG_TRANSPORT_LIST_END) {
                break;
            }
            m_transports.insert(cr2->transports[ii]);
        }
    }

    GT_DONE:
    return err;
}

bool
Connspec::can_dnssrv() const {
    return m_flags & F_DNSSRV;
}

bool
Connspec::is_explicit_dnssrv() const {
    return (m_flags & F_DNSSRV_EXPLICIT) == F_DNSSRV_EXPLICIT;
}
