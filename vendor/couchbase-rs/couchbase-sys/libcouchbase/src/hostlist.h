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

#ifndef LCB_HOSTLIST_H
#define LCB_HOSTLIST_H

#include "config.h"
#include <libcouchbase/couchbase.h>

/**
 * Structure representing a host. This contains the string representation
 * of a host and a port.
 */
typedef struct lcb_host_st {
    char host[NI_MAXHOST + 1];
    char port[NI_MAXSERV + 1];
    int ipv6;
} lcb_host_t;

#define LCB_HOST_FMT LCB_LOG_SPEC("%s%s%s:%s")
#define LCB_HOST_ARG(__settings, __host)                                                                               \
    ((__settings && __settings->log_redaction) ? LCB_LOG_SD_OTAG : ""), ((__host)->ipv6 ? "[" : ""), (__host)->host,   \
        ((__host)->ipv6 ? "]" : ""), (__host)->port,                                                                   \
        ((__settings && __settings->log_redaction) ? LCB_LOG_SD_CTAG : "")

/**
 * Structure representing a list of hosts. This has an internal iteration
 * index which is used to cycle between 'good' and 'bad' hosts.
 */
#ifndef __cplusplus
struct hostlist_st;
typedef struct hostlist_st *hostlist_t;
#else
#include <vector>

namespace lcb
{
struct Hostlist {
    Hostlist() : ix(0) {}
    ~Hostlist();

    /**
     * Adds a string to the hostlist. See lcb_host_parse for details.
     * Note that if the host already exists (see 'lcb_host_equals') it will
     * not be added
     * @param s the string to parse
     * @param len the length of the string
     * @param deflport If `s` does not contain an explicit port, use this
     *        port instead.
     * @return LCB_EINVAL if the host string is not valid
     */
    lcb_STATUS add(const char *s, long len, int deflport);
    lcb_STATUS add(const char *s, int deflport)
    {
        return add(s, -1, deflport);
    }
    void add(const lcb_host_t &);

    bool exists(const lcb_host_t &) const;
    bool exists(const char *hostport) const;

    /**
     * Return the next host in the list.
     * @param wrap If the internal iterator has reached its limit, this
     * indicates whether it should be reset, or if it should return NULL
     * @return a new host if available, or NULL if the list is empty or the
     * iterator is finished.
     */
    lcb_host_t *next(bool wrap);
    bool finished() const;

    size_t size() const
    {
        return hosts.size();
    }
    bool empty() const
    {
        return hosts.empty();
    }
    Hostlist &assign(const Hostlist &other);

    /** Clears the hostlist */
    void clear()
    {
        hosts.clear();
        reset_strlist();
        ix = 0;
    }

    /** Randomize the hostlist by shuffling the order. */
    void randomize();

    /**
     * String list handling functions. These are used to return the hostlist via
     * the API where we return a char*[] terminated by a NULL pointer.
     */

    /** Ensure that the string list contains at least one entry */
    void ensure_strlist();

    /** Frees the current list of strings */
    void reset_strlist();

    const char *const *get_strlist()
    {
        ensure_strlist();
        return &hoststrs[0];
    }

    unsigned int ix;
    const lcb_host_t &operator[](size_t ix_) const
    {
        return hosts[ix_];
    }

    std::vector< lcb_host_t > hosts;
    std::vector< const char * > hoststrs;
};
} // namespace lcb
typedef lcb::Hostlist *hostlist_t;

struct hostlist_st : lcb::Hostlist {
    hostlist_st() : Hostlist() {}
};
#endif

#ifdef __cplusplus
extern "C" {
#endif
/**
 * Parses a string into a hostlist
 * @param host the target host to populate
 * @param spec a string to parse. This may either be an IP/host or an
 * IP:Port pair.
 * @param speclen the length of the string. If this is -1 then it is assumed
 * to be NUL-terminated and strlen will be used
 * @param deflport If a port is not found in the spec, then this port will
 * be used
 *
 * @return LCB_EINVAL if the host format is invalid
 */
lcb_STATUS lcb_host_parse(lcb_host_t *host, const char *spec, int speclen, int deflport);

/** Wrapper around lcb_host_parse() which accepts a NUL-terminated string
 * @param host the host to populate
 * @param spec a NUL-terminated string to parse
 * @param deflport the default port to use if the `spec` does not contain a port
 * @see lcb_host_parse()
 */
#define lcb_host_parsez(host, spec, deflport) lcb_host_parse(host, spec, -1, deflport)

/**
 * Compares two hosts for equality.
 * @param a host to compare
 * @param b other host to compare
 * @return true if equal, false if different.
 */
int lcb_host_equals(const lcb_host_t *a, const lcb_host_t *b);

#ifdef __cplusplus
}
#endif
#endif
