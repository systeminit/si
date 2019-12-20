/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2015-2019 Couchbase, Inc.
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

#ifndef LCB_HTTPPRIV_H
#define LCB_HTTPPRIV_H

#include <libcouchbase/couchbase.h>
#include <lcbio/lcbio.h>
#include <lcbio/timer-ng.h>
#include <lcbio/timer-cxx.h>
#include <lcbht/lcbht.h>
#include "contrib/http_parser/http_parser.h"
#include "http.h"
#include <string>
#include <vector>
#include <set>

namespace lcb
{
namespace http
{

// Simple object for header key and value
struct Header {
    std::string key;
    std::string value;
    Header(const std::string &key_, const std::string &value_) : key(key_), value(value_) {}
};

struct Request {
    /**
     * Initializes the request. This simply copies the relevant fields from the
     * body and initializes instance members to their default values. The static
     * ::create() method should be used instead to construct a new object
     */
    inline Request(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDHTTP *cmd);

    /** Creates a new request object and verifies the input (setup_inputs()) */
    static Request *create(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDHTTP *cmd, lcb_STATUS *rc);

    /** Pause IO on this request */
    void pause();

    /** Resume previously paused IO */
    void resume();

    /** Cancel and finish this request, suppressing callbacks */
    void cancel();

    /**
     * Start all operations required to make this request. This should be
     * called once all inputs have been completed. If successful, I/O operations
     * will begin (this function calls start_io())
     */
    lcb_STATUS submit();

    bool has_pending_redirect() const
    {
        return !pending_redirect.empty();
    }
    /**
     * @return The effective timeout. This is either the user timeout or the
     * default timeout for the API type
     */
    uint32_t timeout() const;
    bool is_data_request() const
    {
        return reqtype == LCB_HTTP_TYPE_N1QL || reqtype == LCB_HTTP_TYPE_VIEW || reqtype == LCB_HTTP_TYPE_FTS ||
               reqtype == LCB_HTTP_TYPE_PING || reqtype == LCB_HTTP_TYPE_CBAS;
    }

    /**
     * @return If this request is in an ONGOING state, meaning no I/O errors
     * and is not finished.
     */
    bool is_ongoing() const
    {
        return status == ONGOING;
    }

    /**
     * Sets up inputs from the command. This should really be in the
     * constructor, however we also need a return value. The ::create()
     * static method creates a new object and calls this function to verify
     * the inputs
     */
    inline lcb_STATUS setup_inputs(const lcb_CMDHTTP *cmd);

    /**
     * Gets a target node for the destination API in the form of host:port.
     *
     * This function will also inspect previously used nodes and skip over ones
     * which are problematic, in the case of retries.
     *
     * @param[out] rc error code
     * @return string if there is a node, or NULL if there is an error. Check rc
     *         for error details
     */
    const char *get_api_node(lcb_STATUS &rc);
    const char *get_api_node()
    {
        lcb_STATUS dummy;
        return get_api_node(dummy);
    }

    /**
     * Sets the URL for the field
     * @param base The URL base (i.e. http://foo.com)
     * @param nbase Length of the base field
     * @param rest User input (the path)
     * @param nrest Length of the path
     * @return LCB_SUCCESS on success, error if failure
     *
     * If all parameters are 0 then this function will use the already-contained
     * ::url field and validate/parse the inputs
     */
    inline lcb_STATUS assign_url(const char *base, size_t nbase, const char *rest, size_t nrest);

    /**
     * Helper method to get a URL field
     * @param field The field constant (UF_XXX)
     * @param target The string to contain the result
     */
    inline void assign_from_urlfield(http_parser_url_fields field, std::string &target);

    /**
     * Add a new header to the list of request headers
     * @param key header key
     * @param value header value
     */
    void add_header(const std::string &key, const std::string &value)
    {
        request_headers.push_back(Header(key, value));
    }

    // Helper methods to populate request buffer
    inline void add_to_preamble(const char *);
    inline void add_to_preamble(const std::string &);
    inline void add_to_preamble(const Header &);

    /**
     * Starts the IO on the current request. This really belongs in submit(),
     * but submit() handles building the request while start_io() sets up
     * the actual connection
     */
    lcb_STATUS start_io(lcb_host_t &);

    /**
     * Release any I/O resources attached to this request.
     */
    void close_io();

    // Helper functions for parsing response data from network
    inline int handle_parse_chunked(const char *buf, unsigned nbuf);
    inline void assign_response_headers(const lcb::htparse::Response &);

    /**
     * Called when a redirect has happened. pending_redirect must not be empty.
     * This will transfer control to the redirect call. If there is an error
     * during redirect, the appropriate finish() API will be called by this
     * function.
     *
     * Note that the new URL should be present in the ::pending_redirect
     * field
     */
    void redirect();

    /**
     * Called to initialize a response object. Sets up standard boilerplate
     * variables for the response
     */
    void init_resp(lcb_RESPHTTP *resp);

    /** Called by finish() to refresh the config, if necessary */
    void maybe_refresh_config(lcb_STATUS rc);

    /** Finish this request, invoking the final callback if necessary */
    void finish(lcb_STATUS rc);

    /**
     * Finish or retry this request, depending on the state of the response
     * If the request can be retried, IO will begin again, otherwise it will
     * be finished via a call to finish()
     **/
    void finish_or_retry(lcb_STATUS rc);

    /**
     * Change the callback for this request. This is used to indicate that
     * a custom internal callback is used, rather than the one installed via
     * lcb_install_callback3
     * @param callback_ The internal callback to invoke
     */
    void set_callback(lcb_RESPCALLBACK callback_)
    {
        callback = callback_;
    }

    /**
     * Let the request finish its normal course, suppressing any callbacks.
     * Unlike cancel(), this does not dispatch to finish. Finish is called
     * when any final I/O has been completed, which may happen after the
     * instance is already destroyed
     */
    void block_callback()
    {
        status |= NOLCB | CBINVOKED;
    }

    /** Decrement refcount. The object is destroyed when the refcount hits 0 */
    void decref();

    /** Increment refcount */
    void incref()
    {
        refcount++;
    }

    lcb_INSTANCE *instance; /**< Library handle */
    std::string url;        /**<Base URL: http://host:port/path?query*/
    std::string host;       /**< Host, derived from URL */
    std::string port;       /**< Port, derived from URL */
    bool ipv6;

    std::string pending_redirect; /**< New redirected URL */

    const std::vector< char > body; /**< Input body (for POST/PUT) */

    /** Request buffer (excluding body). Reassembled from inputs */
    std::vector< char > preamble;

    struct http_parser_url url_info;  /**< Parser info for the URL */
    const lcb_HTTP_METHOD method;     /**< Request method constant */
    const bool chunked;               /**< Whether to invoke callback for each data chunk */
    bool paused;                      /**< See pause() and resume() */
    const void *const command_cookie; /** User context for callback */
    size_t refcount;                  /** Initialized to 1. See incref() and decref() */
    int redircount;                   /** Times this request was redirected */

    /**
     * Whether this request has delivered data to the user. This is relevant
     * in cases where a retry is requested. If any data has been passed at
     * all, we cannot retry the request.
     */
    bool passed_data;

    /** Sparse map indicating which nodes the request was already sent to */
    std::vector< int > used_nodes;

    /**
     * Last revision ID of vBucket config. If the current revision does not
     * match this number, the ::used_nodes field is cleared
     */
    int last_vbcrev;

    const lcb_HTTP_TYPE reqtype; /**< HTTP API type */

    enum State {
        /**
         * The request is still ongoing. Callbacks are still active.
         * Note that this essentially means the absence of any flags :)
         */
        ONGOING = 0,

        /**
         * This flag is set when the on_complete callback has been invoked. This
         * is used as a marker to prevent us from calling that callback more than
         * once per request
         */
        CBINVOKED = 1 << 0,

        /**
         * This flag is set by lcb_http_request_finish, and indicates that the
         * request is no longer active per se. This means that while the request
         * may still be valid in memory, it is simply waiting for any pending I/O
         * operations to close, so the reference count can hit zero.
         */
        FINISHED = 1 << 1,

        /**
         * Internal flag used to indicate that finish() should not not attempt
         * to modify any instance-level globals. This is currently used
         * from within lcb_destroy()
         */
        NOLCB = 1 << 2
    };
    int status;                            /**< OR'd flags of ::State */
    std::vector< Header > request_headers; /**< List of request headers */

    /**
     * Response headers for callback (array of char*). Buffers are mapped to
     * ::response_headers
     */
    std::vector< const char * > response_headers_clist;

    /** Backing buffers for response headers */
    std::vector< lcb::htparse::MimeHeader > response_headers;

    /** Callback to invoke */
    lcb_RESPCALLBACK callback;

    // IO variables
    lcbio_pTABLE io;
    lcbio_pCTX ioctx;
    lcbio_pTIMER timer;
    lcb::io::ConnectionRequest *creq;

    /** HTTP Protocol parser */
    lcb::htparse::Parser *parser;

    /** overrides default timeout if nonzero */
    const uint32_t user_timeout;

    hrtime_t start; /**< Start time */
    lcbio_SERVICE service;
};

} // namespace http
} // namespace lcb

struct lcb_HTTP_HANDLE_ : public lcb::http::Request {
};

#endif /* HEADER GUARD */
