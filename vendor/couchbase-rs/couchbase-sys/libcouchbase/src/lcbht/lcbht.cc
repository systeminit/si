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

#include "lcbht.h"
#include "contrib/http_parser/http_parser.h"
#include "settings.h"

using namespace lcb::htparse;

Parser::Parser(lcb_settings_st *settings_)
    : http_parser(), settings(settings_), lastcall(CB_NONE), last_body(0), last_bodylen(0), paused(false), is_ex(false)
{
    lcb_settings_ref(settings);
    reset();
}

Parser::~Parser()
{
    lcb_settings_unref(settings);
}

static int on_hdr_key(http_parser *pb, const char *s, size_t n)
{
    return Parser::from_htp(pb)->on_hdr_key(s, n);
}
int Parser::on_hdr_key(const char *s, size_t n)
{
    if (lastcall != CB_HDR_KEY) {
        /* new key */
        resp.headers.push_back(MimeHeader());
    }

    resp.headers.back().key.append(s, n);
    lastcall = CB_HDR_KEY;
    return 0;
}

static int on_hdr_value(http_parser *pb, const char *s, size_t n)
{
    return Parser::from_htp(pb)->on_hdr_value(s, n);
}

int Parser::on_hdr_value(const char *s, size_t n)
{
    MimeHeader *header = &resp.headers.back();
    header->value.append(s, n);
    lastcall = CB_HDR_VALUE;
    return 0;
}

static int on_hdr_done(http_parser *pb)
{
    return Parser::from_htp(pb)->on_hdr_done();
}
int Parser::on_hdr_done()
{
    resp.state |= S_HTSTATUS | S_HEADER;

    /* extract the status */
    resp.status = http_parser::status_code;
    lastcall = CB_HDR_DONE;
    return 0;
}

static int on_body(http_parser *pb, const char *s, size_t n)
{
    return Parser::from_htp(pb)->on_body(s, n);
}
int Parser::on_body(const char *s, size_t n)
{
    if (is_ex) {
        last_body = s;
        last_bodylen = n;
        paused = true;
        _lcb_http_parser_pause(this, 1);
    } else {
        resp.body.append(s, n);
    }

    lastcall = CB_BODY;
    resp.state |= S_BODY;
    return 0;
}

static int on_msg_done(http_parser *pb)
{
    return Parser::from_htp(pb)->on_msg_done();
}

int Parser::on_msg_done()
{
    resp.state |= S_DONE;
    return 0;
}

static struct http_parser_settings Parser_Settings = {NULL, /* msg_begin */
                                                      NULL, /* on_url */
                                                      ::on_hdr_key, ::on_hdr_value, ::on_hdr_done,
                                                      ::on_body,    ::on_msg_done};

unsigned Parser::parse(const void *data_, size_t ndata)
{
    is_ex = false;
    size_t nb = _lcb_http_parser_execute(this, &Parser_Settings, reinterpret_cast< const char * >(data_), ndata);

    if (nb != ndata) {
        resp.state |= S_ERROR;
    }

    return resp.state;
}

unsigned Parser::parse_ex(const void *data_, unsigned ndata, unsigned *nused, unsigned *nbody, const char **pbody)
{
    is_ex = true;
    size_t nb = _lcb_http_parser_execute(this, &Parser_Settings, reinterpret_cast< const char * >(data_), ndata);
    if (nb != ndata) {
        if (paused) {
            _lcb_http_parser_pause(this, 0);
            paused = false;
        } else {
            resp.state |= S_ERROR;
            return resp.state;
        }
    }

    *nused = nb;
    *nbody = last_bodylen;
    *pbody = last_body;

    last_body = NULL;
    last_bodylen = 0;
    return resp.state;
}

bool Parser::can_keepalive() const
{
    if (!(resp.state & S_DONE)) {
        return 0;
    }

    if (resp.state & S_ERROR) {
        return 0;
    }

    return _lcb_http_should_keep_alive(const_cast< http_parser * >(static_cast< const http_parser * >(this)));
}

void Parser::reset()
{
    resp.clear();
    _lcb_http_parser_init(this, HTTP_RESPONSE);
}

const MimeHeader *Response::get_header(const std::string &key) const
{
    std::list< MimeHeader >::const_iterator it;
    for (it = headers.begin(); it != headers.end(); ++it) {
        if (it->key == key) {
            return &*it;
        }
    }
    return NULL;
}
