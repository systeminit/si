/**
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
 **/

#ifndef LCB_VIEWROW_H_
#define LCB_VIEWROW_H_

#include <libcouchbase/couchbase.h>
#include "contrib/jsonsl/jsonsl.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <string>

namespace lcb
{
namespace jsparse
{

struct Parser;

struct Row {
    lcb_IOV docid;
    lcb_IOV key;
    lcb_IOV value;
    lcb_IOV row;
    lcb_IOV geo;
};

struct Parser {
    enum Mode { MODE_VIEWS, MODE_N1QL, MODE_FTS, MODE_ANALYTICS, MODE_ANALYTICS_DEFERRED };

    struct Actions {
        /**
         * Called when a row is received.
         * This is a row of view data. You can parse this as JSON from your
         * favorite decoder/converter
         */
        virtual void JSPARSE_on_row(const Row &) = 0;

        /**
         * A JSON parse error occured. The payload will contain string data. This
         * may be JSON (but this is not likely).
         * The callback will be delivered twice. First when the error is noticed,
         * and second at the end (instead of a COMPLETE callback)
         */
        virtual void JSPARSE_on_error(const std::string &buf) = 0;

        /**
         * All the rows have been returned. In this case, the data is the 'meta'.
         * This is a valid JSON payload which was returned from the server.
         * The "rows" : [] array will be empty.
         */
        virtual void JSPARSE_on_complete(const std::string &meta) = 0;

        virtual ~Actions() {}
    };

    /**
     * Creates a new vrow context object.
     * You must set callbacks on this object if you wish it to be useful.
     * You must feed it data (calling vrow_feed) as well. The data may be fed
     * in chunks and callbacks will be invoked as each row is read.
     */
    Parser(Mode mode, Actions *actions_);
    ~Parser();

    /**
     * Feeds data into the vrow. The callback may be invoked multiple times
     * in this function. In the context of normal lcb usage, this will typically
     * be invoked from within an http_data_callback.
     */
    void feed(const char *s, size_t n);
    void feed(const std::string &s)
    {
        feed(s.c_str(), s.size());
    }

    /**
     * Parse the row buffer into its constituent parts. This should be called
     * if you want to split the row into its basic 'docid', 'key' and 'value'
     * fields
     * @param vp The parser to use
     * @param vr The row to parse. This assumes the row's "row" field is properly
     * set.
     */
    void parse_viewrow(Row &vr);

    /**
     * Get the raw contents of the current buffer. This can be used to debug errors.
     *
     * Note that the buffer may be partial or malformed or otherwise unsuitable
     * for structured inspection, but may help human observers debug problems.
     *
     * @param out The iov structure to contain the buffer/offset
     */
    void get_postmortem(lcb_IOV &out) const;

    inline const char *get_buffer_region(size_t pos, size_t desired, size_t *actual);
    inline void combine_meta();
    inline static const char *jprstr_for_mode(Mode);

    jsonsl_t jsn;            /**< Parser for the row itself */
    jsonsl_t jsn_rdetails;   /**< Parser for the row details */
    jsonsl_jpr_t jpr;        /**< jsonpointer match object */
    std::string meta_buf;    /**< String containing the skeleton (outer layer) */
    std::string current_buf; /**< Scratch/read buffer */
    std::string last_hk;     /**< Last hashkey */

    lcb_U8 mode;

    /* flags. This should be an int with a bunch of constant flags */
    lcb_U8 have_error;
    lcb_U8 initialized;
    lcb_U8 meta_complete;
    unsigned rowcount;

    /* absolute position offset corresponding to the first byte in current_buf */
    size_t min_pos;

    /* minimum (absolute) position to keep */
    size_t keep_pos;

    /**
     * size of the metadata header chunk (i.e. everything until the opening
     * bracket of "rows" [
     */
    size_t header_len;

    /**
     * Position of last row returned. If there are no subsequent rows, this
     * signals the beginning of the metadata trailer
     */
    size_t last_row_endpos;

    /**
     * std::string to contain parsed document ID.
     */
    Json::Value cxx_data;

    /* callback to invoke */
    Actions *actions;
};

} // namespace jsparse
} // namespace lcb
#endif /* LCB_VIEWROW_H_ */
