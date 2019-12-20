/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2013-2019 Couchbase, Inc.
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

#include "hostlist.h"
#include <stdio.h>
#include <ctype.h>
#include <limits.h>
#include <algorithm>
#include <string>

using namespace lcb;


Hostlist::~Hostlist()
{
    reset_strlist();
}

void
Hostlist::reset_strlist()
{
    for (size_t ii = 0; ii < hoststrs.size(); ++ii) {
        if (hoststrs[ii] != NULL) {
            delete[] hoststrs[ii];
        }
    }
    hoststrs.clear();
}

lcb_STATUS
lcb_host_parse(lcb_host_t *host, const char *spec, int speclen, int deflport)
{
    std::vector<char> zspec;
    char *host_s;
    char *port_s;
    char *delim;
    bool ipv6 = false;

    /** Parse the host properly */
    if (speclen < 0) {
        speclen = strlen(spec);
    } else if (speclen == 0) {
        return LCB_INVALID_HOST_FORMAT;
    }

    if (deflport < 1) {
        return LCB_INVALID_HOST_FORMAT;
    }

    zspec.assign(spec, spec + speclen);
    zspec.push_back('\0');
    host_s = &zspec[0];

    if ( (delim = strstr(host_s, "://"))) {
        host_s = delim + 3;
    }

    if ((delim = strstr(host_s, "/"))) {
        *delim = '\0';
    }

    port_s = strstr(host_s, ":");
    if (port_s != NULL && strstr(port_s + 1, ":") != NULL) {
        ipv6 = true;
        // treat as IPv6 address
        if (host_s[0] == '[') {
            host_s++;
            char *hostend = strstr(host_s, "]");
            if (hostend == NULL) {
                return LCB_INVALID_HOST_FORMAT;
            }
            port_s = hostend + 1;
            if (*port_s != ':' || (size_t)(port_s - host_s) >= strlen(host_s)) {
                port_s = NULL;
            }
            *hostend = '\0';
        } else {
            port_s = NULL;
        }
    }
    if (port_s != NULL) {
        char *endp;
        long ll;

        *port_s = '\0';
        port_s++;

        if (! *port_s) {
            return LCB_INVALID_HOST_FORMAT;
        }

        ll = strtol(port_s, &endp, 10);
        if (ll == LONG_MIN || ll == LONG_MAX) {
            return LCB_INVALID_HOST_FORMAT;
        }

        if (*endp) {
            return LCB_INVALID_HOST_FORMAT;
        }

    } else {
        port_s = const_cast<char*>("");
    }


    if (strlen(host_s) > sizeof(host->host)-1 ||
            strlen(port_s) > sizeof(host->port)-1 ||
            *host_s == '\0') {
        return LCB_INVALID_HOST_FORMAT;
    }

    size_t ii, hostlen = strlen(host_s);
    for (ii = 0; ii < hostlen; ii++) {
        if (isalnum(host_s[ii])) {
            continue;
        }
        switch (host_s[ii]) {
        case '.':
        case '-':
        case '_':
            break;
        case ':':
        case '[':
        case ']':
            if (ipv6) {
                break;
            }
        /* fallthrough */
        default:
            return LCB_INVALID_HOST_FORMAT;
        }
    }

    strcpy(host->host, host_s);
    if (*port_s) {
        strcpy(host->port, port_s);
    } else {
        sprintf(host->port, "%d", deflport);
    }
    host->ipv6 = ipv6;

    return LCB_SUCCESS;
}

int lcb_host_equals(const lcb_host_t *a, const lcb_host_t *b)
{
    return strcmp(a->host, b->host) == 0 && strcmp(a->port, b->port) == 0;
}

bool
Hostlist::exists(const lcb_host_t& host) const
{
    for (size_t ii = 0; ii < hosts.size(); ++ii) {
        if (lcb_host_equals(&host, &hosts[ii])) {
            return true;
        }
    }
    return false;
}

bool
Hostlist::exists(const char *s) const
{
    lcb_host_t tmp = {"", "", 0};
    if (lcb_host_parse(&tmp, s, -1, 1) != LCB_SUCCESS) {
        return false;
    }
    return exists(tmp);
}

void
Hostlist::add(const lcb_host_t& host)
{
    if (exists(host)) {
        return;
    }
    hosts.push_back(host);
    reset_strlist();
}

lcb_STATUS
Hostlist::add(const char *hostport, long len, int deflport)
{
    lcb_STATUS err = LCB_SUCCESS;

    if (len < 0) {
        len = strlen(hostport);
    }

    std::string ss(hostport, len);
    if (ss.empty()) {
        return LCB_SUCCESS;
    }
    if (ss[ss.length()-1] != ';') {
        ss += ';';
    }

    const char *curstart = ss.c_str();
    const char *delim;
    while ( (delim = strstr(curstart, ";"))) {
        lcb_host_t curhost = {"", "", 0};
        size_t curlen;

        if (delim == curstart) {
            curstart++;
            continue;
        }

        /** { 'f', 'o', 'o', ';' } */
        curlen = delim - curstart;

        err = lcb_host_parse(&curhost, curstart, curlen, deflport);
        if (err != LCB_SUCCESS) {
            return err;
        }
        add(curhost);
        curstart = delim + 1;
    }
    return LCB_SUCCESS;
}

lcb_host_t *
Hostlist::next(bool wrap)
{
    lcb_host_t *ret;
    if (empty()) {
        return NULL;
    }
    if (ix == size()) {
        if (wrap) {
            ix = 0;
        } else {
            return NULL;
        }
    }

    ret = &hosts[ix];
    ix++;
    return ret;
}

void
Hostlist::randomize()
{
    std::random_shuffle(hosts.begin(), hosts.end());
    reset_strlist();
}

void Hostlist::ensure_strlist() {
    if (hoststrs.size()) {
        return;
    }
    for (size_t ii = 0; ii < hosts.size(); ii++) {
        const lcb_host_t& host = hosts[ii];
        std::string ss;
        if (host.ipv6) {
            ss.append("[").append(host.host).append("]");
        } else {
            ss.append(host.host);
        }
        ss.append(":").append(host.port);
        char *newstr = new char[ss.size() + 1];
        newstr[ss.size()] = '\0';
        memcpy(newstr, ss.c_str(), ss.size());
        hoststrs.push_back(newstr);
    }
    hoststrs.push_back(NULL);
}

Hostlist&
Hostlist::assign(const Hostlist& src)
{
    clear();
    reset_strlist();
    for (size_t ii = 0; ii < src.size(); ++ii) {
        add(src.hosts[ii]);
    }
    return *this;
}
