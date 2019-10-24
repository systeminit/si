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

/**
 * HTTP-based 'REST' configuration. This module works by connecting to the
 * REST API port (and trying various other nodes) until it receives a
 * configuration.
 */
#ifndef LCB_CLPROVIDER_HTTP_H
#define LCB_CLPROVIDER_HTTP_H

#include "config.h"
#include "hostlist.h"
#include "clconfig.h"
#include <lcbht/lcbht.h>

#define REQBUCKET_COMPAT_PREFIX "/pools/default/bucketsStreaming/"
#define REQBUCKET_TERSE_PREFIX "/pools/default/bs/"

#define REQPOOLS_URI "/pools/"
#define HOSTHDR_FMT "Host: %s:%s\r\n"
#define LAST_HTTP_HEADER "X-Libcouchbase: " LCB_CLIENT_ID "\r\n"
#define CONFIG_DELIMITER "\n\n\n\n"

namespace lcb
{
namespace clconfig
{
struct HttpProvider : Provider {
    HttpProvider(Confmon *);
    ~HttpProvider();

    void reset_stream_state();

    void delayed_disconn();
    void delayed_reconnect();
    void on_timeout();
    lcb_STATUS on_io_error(lcb_STATUS origerr);

    /**
     * Closes the current connection and removes the disconn timer along with it
     */
    void close_current();

    bool is_v220_compat() const;

    lcb_STATUS connect_next();

    /* Overrides */
    bool pause();
    lcb_STATUS refresh();
    ConfigInfo *get_cached();
    void config_updated(lcbvb_CONFIG *);
    void configure_nodes(const lcb::Hostlist &);
    const lcb::Hostlist *get_nodes() const;
    void dump(FILE *) const;
    lcb_STATUS setup_request_header(const lcb_host_t &host);
    /* END Overrides */

    /** Base configuration structure */
    lcbio_pCONNSTART creq;
    lcbio_CTX *ioctx;
    lcb::htparse::Parser *htp;

    /**
     * Buffer to use for writing our request header. Recreated for each
     * connection because of the Host: header
     */
    std::string request_buf;

    /**
     * We only recreate the connection if our current stream 'times out'. This
     * timer waits until the current stream times out and then proceeds to the
     * next connection.
     */
    lcb::io::Timer< HttpProvider, &HttpProvider::delayed_disconn > disconn_timer;
    lcb::io::Timer< HttpProvider, &HttpProvider::on_timeout > io_timer;
    lcb::io::Timer< HttpProvider, &HttpProvider::delayed_reconnect > as_reconnect;

    /** List of hosts to try */
    lcb::Hostlist *nodes;

    /** The cached configuration. */
    ConfigInfo *current_config;
    ConfigInfo *last_parsed;

    int generation;
    bool try_nexturi;
    int uritype;
};

} // namespace clconfig
} // namespace lcb
#endif /* LCB_CLPROVIDER_HTTP_H */
