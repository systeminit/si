/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#include "config.h"
#include <sys/types.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include <libcouchbase/pktfwd.h>
#include <memcached/protocol_binary.h>
#include <iostream>
#include <iomanip>
#include <cstdio>
#include <cerrno>
#include <sstream>
#include <signal.h>
#include "common/options.h"
#include "common/histogram.h"

#include "internal.h"

#include <event2/event.h>
#include <event2/listener.h>
#include <event2/bufferevent.h>
#include <event2/buffer.h>

using namespace cbc;
using namespace cliopts;

static void die(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
    exit(EXIT_FAILURE);
}

static void good_or_die(lcb_STATUS rc, const char *msg = "")
{
    if (rc != LCB_SUCCESS) {
        fprintf(stderr, "%s: %s\n", msg, lcb_strerror_short(rc));
        exit(EXIT_FAILURE);
    }
}

static lcb_INSTANCE *instance = NULL;
static struct event_base *evbase = NULL;
static Histogram hg;

static char app_client_string[] = "cbc-proxy";

#define LOGARGS(lvl) (instance)->settings, "proxy", LCB_LOG_##lvl, __FILE__, __LINE__
#define CL_LOGFMT "<%s:%s> (cl=%p,fd=%d) "
#define CL_LOGID(cl) cl->host, cl->port, (void *)cl, cl->fd

class Configuration
{
  public:
    Configuration() : o_trace("trace"), o_port("port")
    {
        o_trace.abbrev('t').description("Show packet trace on INFO log level");
        o_port.abbrev('p').description("Port for proxy").setDefault(11211);
    }

    ~Configuration() {}

    void addToParser(Parser &parser)
    {
        m_params.addToParser(parser);
        parser.addOption(o_trace);
        parser.addOption(o_port);
    }

    void processOptions() {}

    void fillCropts(lcb_create_st &opts)
    {
        m_params.fillCropts(opts);
    }
    lcb_STATUS doCtls()
    {
        return m_params.doCtls(instance);
    }
    bool useTimings()
    {
        return m_params.useTimings();
    }
    bool shouldDump()
    {
        return m_params.shouldDump();
    }

    bool isTrace()
    {
        return o_trace.result();
    }

    unsigned port()
    {
        return o_port.result();
    }

  private:
    ConnParams m_params;
    BoolOption o_trace;
    UIntOption o_port;
};

static Configuration config;

static struct evconnlistener *listener = NULL;

static void cleanup()
{
    if (instance) {
        if (config.shouldDump()) {
            lcb_dump(instance, stderr, LCB_DUMP_ALL);
        }
        if (config.useTimings()) {
            hg.write();
        }
        if (instance) {
            lcb_destroy(instance);
        }
    }
    if (listener) {
        evconnlistener_free(listener);
    }
    if (evbase) {
        event_base_free(evbase);
    }
}

struct client {
    int fd;
    struct bufferevent *bev;
    char host[NI_MAXHOST + 1];
    char port[NI_MAXSERV + 1];
    long cnt;
};

static void dump_bytes(const struct client *cl, const char *msg, const void *ptr, size_t len)
{
    if (!config.isTrace()) {
        return;
    }

    int width = 16;
    const unsigned char *buf = (const unsigned char *)ptr;
    size_t full_rows = len / width;
    size_t remainder = len % width;
    std::stringstream ss;

    ss << msg << ", " << len
       << " bytes\n"
          "             +-------------------------------------------------+\n"
          "             |  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f |\n"
          "    +--------+-------------------------------------------------+----------------+";

    unsigned int row = 0;
    while (row < full_rows) {
        int row_start_index = row * width;
        // prefix
        ss << "\n    |" << std::setw(8) << std::setfill('0') << std::hex << row_start_index << "|";
        int row_end_index = row_start_index + width;
        // hex
        int i = row_start_index;
        while (i < row_end_index) {
            ss << " " << std::setw(2) << std::setfill('0') << std::hex << (unsigned int)buf[i++];
        }
        ss << " |";
        // ascii
        i = row_start_index;
        while (i < row_end_index) {
            char b = buf[i++];
            if ((b <= 0x1f) || (b >= 0x7f)) {
                ss << '.';
            } else {
                ss << b;
            }
        }
        ss << "|";
        row++;
    }
    if (remainder != 0) {
        int row_start_index = full_rows * width;
        // prefix
        ss << "\n    |" << std::setw(8) << std::setfill('0') << std::hex << row_start_index << "|";
        int row_end_index = row_start_index + remainder;
        // hex
        int i = row_start_index;
        while (i < row_end_index) {
            ss << " " << std::setw(2) << std::setfill('0') << std::hex << (unsigned int)buf[i++];
        }
        i = width - remainder;
        while (i > 0) {
            ss << "   ";
            i--;
        }
        ss << " |";
        // ascii
        i = row_start_index;
        while (i < row_end_index) {
            char b = buf[i++];
            if ((b <= 0x1f) || (b >= 0x7f)) {
                ss << '.';
            } else {
                ss << b;
            }
        }
        i = width - remainder;
        while (i > 0) {
            ss << " ";
            i--;
        }
        ss << "|";
    }
    ss << "\n    +--------+-------------------------------------------------+----------------+";
    lcb_log(LOGARGS(INFO), CL_LOGFMT "%s", CL_LOGID(cl), ss.str().c_str());
}

static void pktfwd_callback(lcb_INSTANCE *, const void *cookie, lcb_STATUS err, lcb_PKTFWDRESP *resp)
{
    good_or_die(err, "Failed to forward a packet");

    struct client *cl = (struct client *)cookie;
    struct evbuffer *output = bufferevent_get_output(cl->bev);
    for (unsigned ii = 0; ii < resp->nitems; ii++) {
        dump_bytes(cl, "response", resp->iovs[ii].iov_base, resp->iovs[ii].iov_len);
        evbuffer_expand(output, resp->iovs[ii].iov_len);
        evbuffer_add(output, resp->iovs[ii].iov_base, resp->iovs[ii].iov_len);
    }
}

extern "C" {
#define DEFINE_ROW_CALLBACK(cbname, resptype)                                                                          \
    static void cbname(lcb_INSTANCE *, int, const resptype *resp)                                                      \
    {                                                                                                                  \
        char key[100] = {0};                                                                                           \
        size_t nkey;                                                                                                   \
        struct client *cl = (struct client *)resp->cookie;                                                             \
                                                                                                                       \
        protocol_binary_response_header header = {};                                                                   \
        header.response.magic = PROTOCOL_BINARY_RES;                                                                   \
        header.response.opcode = PROTOCOL_BINARY_CMD_STAT;                                                             \
                                                                                                                       \
        struct evbuffer *output = bufferevent_get_output(cl->bev);                                                     \
                                                                                                                       \
        if (resp->rflags & LCB_RESP_F_FINAL) {                                                                         \
            memcpy(key, "meta", 4);                                                                                    \
        } else {                                                                                                       \
            snprintf(key, sizeof(key), "row-%ld", cl->cnt++);                                                          \
        }                                                                                                              \
        nkey = strlen(key);                                                                                            \
        header.response.keylen = htons(nkey);                                                                          \
        header.response.bodylen = htonl(resp->nrow + nkey);                                                            \
                                                                                                                       \
        evbuffer_expand(output, resp->nrow + sizeof(header.bytes));                                                    \
        dump_bytes(cl, "response", header.bytes, sizeof(header.bytes));                                                \
        evbuffer_add(output, header.bytes, sizeof(header.bytes));                                                      \
        dump_bytes(cl, "response", key, nkey);                                                                         \
        evbuffer_add(output, key, nkey);                                                                               \
        dump_bytes(cl, "response", resp->row, resp->nrow);                                                             \
        evbuffer_add(output, resp->row, resp->nrow);                                                                   \
                                                                                                                       \
        if (resp->rflags & LCB_RESP_F_FINAL) {                                                                         \
            header.response.keylen = 0;                                                                                \
            header.response.bodylen = 0;                                                                               \
            evbuffer_expand(output, sizeof(header.bytes));                                                             \
            dump_bytes(cl, "response", header.bytes, sizeof(header.bytes));                                            \
            evbuffer_add(output, header.bytes, sizeof(header.bytes));                                                  \
        }                                                                                                              \
    }

DEFINE_ROW_CALLBACK(n1ql_callback, lcb_RESPN1QL)
DEFINE_ROW_CALLBACK(fts_callback, lcb_RESPFTS)
}

static void conn_readcb(struct bufferevent *bev, void *cookie)
{
    struct client *cl = (struct client *)cookie;
    struct evbuffer *input;
    size_t len;

    input = bufferevent_get_input(bev);
    len = evbuffer_get_length(input);
    if (len < 24) {
        lcb_log(LOGARGS(DEBUG), CL_LOGFMT "not enough data for header", CL_LOGID(cl));
        return;
    }

    protocol_binary_request_header header;
    evbuffer_copyout(input, &header, sizeof(header));
    lcb_U32 bodylen = ntohl(header.request.bodylen);

    size_t pktlen = sizeof(header) + bodylen;
    len = evbuffer_get_length(input);
    if (len < pktlen) {
        lcb_log(LOGARGS(DEBUG), CL_LOGFMT "not enough data for packet", CL_LOGID(cl));
        return;
    }
    void *pkt = malloc(pktlen);
    evbuffer_remove(input, pkt, pktlen);

    lcb_sched_enter(instance);
    dump_bytes(cl, "request", pkt, pktlen);
    if (header.request.opcode == PROTOCOL_BINARY_CMD_STAT) {
        lcb_U8 extlen = ntohs(header.request.extlen);
        lcb_U16 keylen = ntohs(header.request.keylen);
        if (keylen < 5) {
            goto FWD;
        }
        char *key = (char *)pkt + sizeof(header) + extlen;
        lcb_STATUS rc;
        if (memcmp(key, "n1ql ", 5) == 0) {
            lcb_CMDN1QL *cmd;
            lcb_cmdn1ql_create(&cmd);

            rc = lcb_cmdn1ql_statement(cmd, key + 5, keylen - 5);
            if (rc != LCB_SUCCESS) {
                lcb_log(LOGARGS(INFO), CL_LOGFMT "failed to set query for N1QL", CL_LOGID(cl));
                goto FWD;
            }
            lcb_cmdn1ql_callback(cmd, n1ql_callback);
            cl->cnt = 0;
            rc = lcb_n1ql(instance, cl, cmd);
            lcb_cmdn1ql_destroy(cmd);
            if (rc != LCB_SUCCESS) {
                lcb_log(LOGARGS(INFO), CL_LOGFMT "failed to schedule N1QL command", CL_LOGID(cl));
                goto FWD;
            }
            goto DONE;
        } else if (memcmp(key, "fts ", 4) == 0) {
            lcb_CMDFTS *cmd;
            lcb_cmdfts_create(&cmd);
            lcb_cmdfts_query(cmd, key + 4, keylen - 4);
            lcb_cmdfts_callback(cmd, fts_callback);
            rc = lcb_fts(instance, cl, cmd);
            lcb_cmdfts_destroy(cmd);
            cl->cnt = 0;
            if (rc != LCB_SUCCESS) {
                lcb_log(LOGARGS(INFO), CL_LOGFMT "failed to schedule FTS command", CL_LOGID(cl));
                goto FWD;
            }
            goto DONE;
        }
    }
FWD : {
    lcb_CMDPKTFWD cmd = {0};
    cmd.vb.vtype = LCB_KV_COPY;
    cmd.vb.u_buf.contig.bytes = pkt;
    cmd.vb.u_buf.contig.nbytes = pktlen;
    good_or_die(lcb_pktfwd3(instance, cl, &cmd), "Failed to forward packet");
}
DONE:
    lcb_sched_leave(instance);
}

static void conn_eventcb(struct bufferevent *bev, short events, void *cookie)
{
    struct client *cl = (struct client *)cookie;

    if (events & BEV_EVENT_EOF) {
        lcb_log(LOGARGS(INFO), CL_LOGFMT "connection closed", CL_LOGID(cl));
        bufferevent_free(bev);
        delete cl;
    } else if (events & BEV_EVENT_ERROR) {
        lcb_log(LOGARGS(ERROR), CL_LOGFMT "got an error on the connection: %s\n", CL_LOGID(cl), strerror(errno));
        bufferevent_free(bev);
        delete cl;
    } else {
        lcb_log(LOGARGS(DEBUG), CL_LOGFMT "ignore event 0x%02x", CL_LOGID(cl), events);
    }
}

static void listener_cb(struct evconnlistener *, evutil_socket_t fd, struct sockaddr *addr, int naddr, void *)
{
    struct bufferevent *bev;
    bev = bufferevent_socket_new(evbase, fd, BEV_OPT_CLOSE_ON_FREE);

    if (!bev) {
        die("Error constructing bufferevent");
    }

    struct client *cl = new client();
    cl->fd = fd;
    cl->bev = bev;
    getnameinfo(addr, naddr, cl->host, sizeof(cl->host), cl->port, sizeof(cl->port), NI_NUMERICHOST | NI_NUMERICSERV);
    bufferevent_setcb(bev, conn_readcb, NULL, conn_eventcb, cl);
    bufferevent_enable(bev, EV_READ | EV_WRITE);
    lcb_log(LOGARGS(INFO), CL_LOGFMT "new client connection", CL_LOGID(cl));
}

static void setup_listener()
{
    struct sockaddr_in sin;

    memset(&sin, 0, sizeof(sin));
    sin.sin_family = AF_INET;
    sin.sin_port = htons(config.port());

    listener = evconnlistener_new_bind(evbase, listener_cb, NULL, LEV_OPT_REUSEABLE | LEV_OPT_CLOSE_ON_FREE, -1,
                                       (struct sockaddr *)&sin, sizeof(sin));
    if (!listener) {
        die("Failed to create proxy listener");
    }
    lcb_log(LOGARGS(INFO), "Listening incoming proxy connections on port %d", config.port());
}

static void bootstrap_callback(lcb_INSTANCE *, lcb_STATUS err)
{
    good_or_die(err, "Failed to bootstrap");
    lcb_log(LOGARGS(INFO), "connected to Couchbase Server");
    setup_listener();
}

static int terminating = 0;
static void sigint_handler(int)
{
    lcb_log(LOGARGS(INFO), "terminating the server");
    if (!terminating) {
        event_base_loopbreak(evbase);
        terminating = 1;
    }
}

static void diag_callback(lcb_INSTANCE *, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPDIAG *resp = (const lcb_RESPDIAG *)rb;
    if (resp->rc != LCB_SUCCESS) {
        fprintf(stderr, "failed: %s\n", lcb_strerror_short(resp->rc));
    } else {
        if (resp->njson) {
            fprintf(stderr, "\n%.*s", (int)resp->njson, resp->json);
        }
    }
}

static void sigquit_handler(int)
{
    lcb_CMDDIAG req = {};
    req.options = LCB_PINGOPT_F_JSONPRETTY;
    req.id = app_client_string;
    lcb_diag(instance, NULL, &req);
}

static void real_main(int argc, char **argv)
{
    Parser parser;

    config.addToParser(parser);
    parser.parse(argc, argv);
    config.processOptions();

    lcb_create_st cropts;
    memset(&cropts, 0, sizeof cropts);
    config.fillCropts(cropts);

    /* bind to external libevent loop */
    evbase = event_base_new();
    struct lcb_create_io_ops_st ciops;
    memset(&ciops, 0, sizeof(ciops));
    ciops.v.v0.type = LCB_IO_OPS_LIBEVENT;
    ciops.v.v0.cookie = evbase;
    good_or_die(lcb_create_io_ops(&cropts.v.v3.io, &ciops), "Failed to create and IO ops strucutre for libevent");

    good_or_die(lcb_create(&instance, &cropts), "Failed to create connection");
    config.doCtls();
    lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_CLIENT_STRING, app_client_string);
    lcb_set_bootstrap_callback(instance, bootstrap_callback);
    lcb_set_pktfwd_callback(instance, pktfwd_callback);
    lcb_install_callback3(instance, LCB_CALLBACK_DIAG, diag_callback);

    good_or_die(lcb_connect(instance), "Failed to connect to cluster");
    if (config.useTimings()) {
        hg.install(instance, stdout);
    }
    std::atexit(cleanup);

    /* setup CTRL-C handler */
    struct sigaction action;
    sigemptyset(&action.sa_mask);
    action.sa_handler = sigint_handler;
    action.sa_flags = 0;
    sigaction(SIGINT, &action, NULL);

    /* setup CTRL-\ handler */
    sigemptyset(&action.sa_mask);
    action.sa_handler = sigquit_handler;
    action.sa_flags = 0;
    sigaction(SIGQUIT, &action, NULL);

    event_base_dispatch(evbase);
}

int main(int argc, char **argv)
{
    try {
        real_main(argc, argv);
        return 0;
    } catch (std::exception &exc) {
        std::cerr << exc.what() << std::endl;
        exit(EXIT_FAILURE);
    }
}
