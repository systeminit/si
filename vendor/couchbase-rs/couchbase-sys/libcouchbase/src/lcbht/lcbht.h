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

#ifndef LCB_HTTP_H
#define LCB_HTTP_H

#include <libcouchbase/couchbase.h>
#include "contrib/http_parser/http_parser.h"
#include <list>
#include <string>

struct lcb_settings_st;

/**
 * @file
 * HTTP Response parsing.
 *
 * This file provides HTTP/1.0 compatible response parsing semantics, supporting
 * the Content-Length header.
 *
 * Specifically this may be used to parse incoming HTTP streams into a single
 * body.
 */

namespace lcb
{
namespace htparse
{

struct MimeHeader {
    std::string key;
    std::string value;
};

struct Response {
    void clear()
    {
        status = 0;
        state = 0;
        headers.clear();
        body.clear();
    }

    /**
     * Get a header value for a key
     * @param response The response
     * @param key The key to look up
     * @return A string containing the value. If the header has no value then the
     * empty string will be returned. If the header does not exist NULL will be
     * returned.
     */
    const MimeHeader *get_header(const std::string &key) const;

    /**
     * Get a header value for a key
     * @param key The key to look up
     * @return A string containing the value. If the header has no value then the
     * empty string will be returned. If the header does not exist NULL will be
     * returned.
     */
    const char *get_header_value(const std::string &key) const
    {
        const MimeHeader *header = get_header(key);
        if (header) {
            return header->value.c_str();
        }
        return NULL;
    }

    unsigned short status; /**< HTTP Status code */
    unsigned state;
    typedef std::list< MimeHeader > HeaderList;
    HeaderList headers;
    std::string body; /**< Body */
};

class Parser : private http_parser
{
  public:
    /**
     * Initialize the parser object
     * @param settings the settings structure used for logging
     */
    Parser(lcb_settings_st *);
    ~Parser();

    /** Response state */
    enum State {
        S_NONE = 0,
        S_HTSTATUS = 1 << 0, /**< Have HTTP status */
        S_HEADER = 1 << 1,   /**< Have HTTP header */
        S_BODY = 1 << 2,     /**< Have HTTP body */
        S_DONE = 1 << 3,     /**< Have a full message */

        /**Have a parse error. Note this is not the same as a HTTP error */
        S_ERROR = 1 << 4
    };

    /**
     * Parse incoming data into a message
     * @param data Pointer to new data
     * @param ndata Size of the data
     *
     * @return The current state of the parser. If `state & LCBHT_S_DONE` then
     * the current response should be handled before continuing.
     * If `state & LCBHT_S_ERROR` then there was an error parsing the contents
     * as it violated the HTTP protocol.
     */
    unsigned parse(const void *data, size_t ndata);

    /**
     * Parse incoming data without buffering
     * @param data The data to parse
     * @param ndata Length of the data
     * @param[out] nused How much of the data was actually consumed
     * @param[out] nbody Size of the body pointer
     * @param[out] pbody a pointer for the body
     *
     * @return See lcbht_set_bufmode for the meaning of this value
     *
     * @note It is not an error if `pbody` is NULL. It may mean that the parse state
     * is still within the headers and there is no body to parse yet.
     *
     * This function is intended to be used in a loop, until there is no input
     * remaining. The use of the `nused` pointer is to determine by how much the
     * `data` pointer should be incremented (and the `ndata` decremented) for the
     * next call. When this function returns with a non-error status, `pbody`
     * will contain a pointer to a buffer of data (but see note above) which can
     * then be processed by the application.
     *
     * @code{.c++}
     * char **body, *input;
     * unsigned inlen = get_input_len(), nused, bodylen;
     * unsigned res;
     * do {
     *   res = parser->parse_ex(input, inlen, &nused, &nbody, &body);
     *   if (res & Parser::S_ERROR) {
     *     // handle error
     *     break;
     *   }
     *   if (nbody) {
     *     // handle body
     *   }
     *   input += nused;
     *   inlen -= nused;
     * } while (!(res & Parser::S_DONE));
     * @endcode
     */
    unsigned parse_ex(const void *data, unsigned ndata, unsigned *nused, unsigned *nbody, const char **pbody);

    /**
     * Obtain the current response being processed.
     * @return a reference to a response object. The response object is only valid
     * until the next call into another parser API
     */
    Response &get_cur_response()
    {
        return resp;
    }

    /**
     * Determine whether HTTP/1.1 keepalive is enabled on the connection
     * @return true if keepalive is enabled, false otherwise.
     */
    bool can_keepalive() const;

    void reset();

    // Callbacks:
    inline int on_hdr_key(const char *, size_t);
    inline int on_hdr_value(const char *, size_t);
    inline int on_hdr_done();
    inline int on_body(const char *, size_t);
    inline int on_msg_done();

    static Parser *from_htp(http_parser *p)
    {
        return static_cast< Parser * >(p);
    }

  private:
    Response resp;
    lcb_settings_st *settings;

    enum last_call_type { CB_NONE, CB_HDR_KEY, CB_HDR_VALUE, CB_HDR_DONE, CB_BODY, CB_MSG_DONE };
    last_call_type lastcall;

    /* For parse_ex */
    const char *last_body;
    unsigned last_bodylen;

    bool paused;
    bool is_ex;
};

} // namespace htparse
} // namespace lcb
#endif
