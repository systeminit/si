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
#if defined(__GNUC__)
#define JSONSL_API static __attribute__((unused))
#elif defined(_MSC_VER)
#define JSONSL_API static __inline
#else
#define JSONSL_API static
#endif
#include "contrib/jsonsl/jsonsl.c"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include "parser.h"

#define DECLARE_JSONSL_CALLBACK(name)                                                                                  \
    static void name(jsonsl_t, jsonsl_action_t, struct jsonsl_state_st *, const char *)

DECLARE_JSONSL_CALLBACK(row_pop_callback);
DECLARE_JSONSL_CALLBACK(initial_push_callback);
DECLARE_JSONSL_CALLBACK(initial_pop_callback);
DECLARE_JSONSL_CALLBACK(meta_header_complete_callback);
DECLARE_JSONSL_CALLBACK(trailer_pop_callback);

using namespace lcb::jsparse;

/* conform to void */
#define JOBJ_RESPONSE_ROOT (void *)1
#define JOBJ_ROWSET (void *)2

template < typename T > void NORMALIZE_OFFSETS(const char *&buf, T &len)
{
    buf++;
    len--;
}

/**
 * Gets a buffer, given an (absolute) position offset.
 * It will try to get a buffer of size desired. The actual size is
 * returned in 'actual' (and may be less than desired, maybe even 0)
 */
const char *Parser::get_buffer_region(size_t pos, size_t desired, size_t *actual)
{
    const char *ret = current_buf.c_str() + pos - min_pos;
    const char *end = current_buf.c_str() + current_buf.size();
    *actual = end - ret;

    if (min_pos > pos) {
        /* swallowed */
        *actual = 0;
        return NULL;
    }

    lcb_assert(ret < end);
    if (desired < *actual) {
        *actual = desired;
    }
    return ret;
}

/**
 * Consolidate the meta data into a single parsable string..
 */
void Parser::combine_meta()
{
    const char *meta_trailer;
    size_t ntrailer;

    if (meta_complete) {
        return;
    }

    lcb_assert(header_len <= meta_buf.size());

    /* Adjust the length for the first portion */
    meta_buf.resize(header_len);

    /* Append any trailing data */
    meta_trailer = get_buffer_region(last_row_endpos, -1, &ntrailer);
    meta_buf.append(meta_trailer, ntrailer);
    meta_complete = 1;
}

static Parser *get_ctx(jsonsl_t jsn)
{
    return reinterpret_cast< Parser * >(jsn->data);
}

static void meta_header_complete_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state,
                                          const jsonsl_char_t *)
{
    Parser *ctx = get_ctx(jsn);
    ctx->meta_buf.append(ctx->current_buf.c_str(), state->pos_begin);

    ctx->header_len = state->pos_begin;
    jsn->action_callback_PUSH = NULL;
}

static void row_pop_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state, const jsonsl_char_t *)
{
    Parser *ctx = get_ctx(jsn);
    const char *rowbuf;
    size_t szdummy;

    if (ctx->have_error) {
        return;
    }

    ctx->keep_pos = jsn->pos;
    ctx->last_row_endpos = jsn->pos;

    if (state->data == JOBJ_ROWSET) {
        /** The closing ] of "rows" : [ ... ] */
        if (ctx->mode == Parser::MODE_ANALYTICS_DEFERRED) {
            if (ctx->keep_pos > ctx->min_pos) {
                ctx->current_buf.erase(0, ctx->keep_pos - ctx->min_pos);
                ctx->min_pos = ctx->keep_pos;
            }
            ctx->meta_buf.append(ctx->current_buf);
            ctx->header_len = jsn->pos;
            ctx->meta_complete = 1;
            if (ctx->actions) {
                ctx->actions->JSPARSE_on_complete(ctx->meta_buf);
                ctx->actions = NULL;
            }
            return;
        }

        jsn->action_callback_POP = trailer_pop_callback;
        jsn->action_callback_PUSH = NULL;
        if (ctx->rowcount == 0) {
            /* Emulate what meta_header_complete callback does. */

            /* While the entire meta is available to us, the _closing_ part
             * of the meta is handled in a different callback. */
            ctx->meta_buf.append(ctx->current_buf.c_str(), jsn->pos);
            ctx->header_len = jsn->pos;
        }
        return;
    }

    ctx->rowcount++;
    if (!ctx->actions) {
        return;
    }

    rowbuf = ctx->get_buffer_region(state->pos_begin, -1, &szdummy);
    Row dt = {{0}};
    dt.row.iov_base = (void *)rowbuf;
    dt.row.iov_len = jsn->pos - state->pos_begin + 1;
    ctx->actions->JSPARSE_on_row(dt);
}

static int parse_error_callback(jsonsl_t jsn, jsonsl_error_t, struct jsonsl_state_st *, jsonsl_char_t *)
{
    Parser *ctx = get_ctx(jsn);
    ctx->have_error = 1;

    /* invoke the callback */
    if (ctx->actions) {
        ctx->actions->JSPARSE_on_error(ctx->current_buf);
        ctx->actions = NULL;
    }
    return 0;
}

static void trailer_pop_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state, const jsonsl_char_t *)
{
    Parser *ctx = get_ctx(jsn);

    if (state->data != JOBJ_RESPONSE_ROOT) {
        return;
    }
    ctx->combine_meta();
    if (ctx->actions) {
        ctx->actions->JSPARSE_on_complete(ctx->meta_buf);
        ctx->actions = NULL;
    }
}

static void initial_pop_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state, const jsonsl_char_t *)
{
    Parser *ctx = get_ctx(jsn);
    unsigned long len;

    if (ctx->have_error) {
        return;
    }
    if (JSONSL_STATE_IS_CONTAINER(state)) {
        return;
    }
    if (state->type != JSONSL_T_HKEY) {
        return;
    }

    const char *key = ctx->current_buf.c_str() + state->pos_begin;
    len = jsn->pos - state->pos_begin;
    NORMALIZE_OFFSETS(key, len);
    ctx->last_hk.assign(key, len);
}

/**
 * This is called for the first few tokens, where we are still searching
 * for the row set.
 */
static void initial_push_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state, const jsonsl_char_t *)
{
    Parser *ctx = (Parser *)jsn->data;
    jsonsl_jpr_match_t match = JSONSL_MATCH_UNKNOWN;

    if (ctx->have_error) {
        return;
    }

    if (JSONSL_STATE_IS_CONTAINER(state)) {
        jsonsl_jpr_match_state(jsn, state, ctx->last_hk.c_str(), ctx->last_hk.size(), &match);
    }
    ctx->last_hk.clear();

    if (ctx->mode == Parser::MODE_ANALYTICS_DEFERRED) {
        ctx->initialized = 1;
    }

    if (ctx->initialized == 0) {
        if (state->type != JSONSL_T_OBJECT) {
            ctx->have_error = 1;
            return;
        }

        if (match != JSONSL_MATCH_POSSIBLE) {
            ctx->have_error = 1;
            return;
        }
        /* tag the state */
        state->data = JOBJ_RESPONSE_ROOT;
        ctx->initialized = 1;
        return;
    }

    if (state->type == JSONSL_T_LIST && match == JSONSL_MATCH_POSSIBLE) {
        /* we have a match, e.g. "rows:[]" */
        jsn->action_callback_POP = row_pop_callback;
        jsn->action_callback_PUSH = meta_header_complete_callback;
        state->data = JOBJ_ROWSET;
    }
}

void Parser::feed(const char *data_, size_t ndata)
{
    size_t old_len = current_buf.size();
    current_buf.append(data_, ndata);
    jsonsl_feed(jsn, current_buf.c_str() + old_len, ndata);

    /* Do we need to cut off some bytes? */
    if (keep_pos > min_pos) {
        current_buf.erase(0, keep_pos - min_pos);
    }

    min_pos = keep_pos;
}

const char *Parser::jprstr_for_mode(Mode mode)
{
    switch (mode) {
        case MODE_VIEWS:
            return "/rows/^";
        case MODE_N1QL:
        case MODE_ANALYTICS:
            return "/results/^";
        case MODE_ANALYTICS_DEFERRED:
            return "/^";
        case MODE_FTS:
            return "/hits/^";
        default:
            lcb_assert(0 && "Invalid mode passed!");
            return "/";
    }
}

Parser::Parser(Mode mode_, Parser::Actions *actions_)
    : jsn(jsonsl_new(512)), jsn_rdetails(jsonsl_new(32)), jpr(jsonsl_jpr_new(jprstr_for_mode(mode_), NULL)),
      mode(mode_), have_error(0), initialized(0), meta_complete(0), rowcount(0), min_pos(0), keep_pos(0), header_len(0),
      last_row_endpos(0), cxx_data(), actions(actions_)
{

    jsonsl_jpr_match_state_init(jsn, &jpr, 1);
    jsonsl_reset(jsn);
    jsonsl_reset(jsn_rdetails);
    current_buf.clear();
    meta_buf.clear();
    last_hk.clear();

    /* Initially all callbacks are enabled so that we can search for the
     * rows array. */
    jsn->action_callback_POP = initial_pop_callback;
    jsn->action_callback_PUSH = initial_push_callback;
    jsn->error_callback = parse_error_callback;
    if (mode == MODE_ANALYTICS_DEFERRED) {
        jsn->max_callback_level = 3;
    } else {
        jsn->max_callback_level = 4;
    }
    jsn->data = this;
    jsonsl_enable_all_callbacks(jsn);
}

void Parser::get_postmortem(lcb_IOV &out) const
{
    if (meta_complete) {
        out.iov_base = const_cast< char * >(meta_buf.c_str());
        out.iov_len = meta_buf.size();
    } else {
        out.iov_base = const_cast< char * >(current_buf.c_str());
        out.iov_len = current_buf.size();
    }
}

Parser::~Parser()
{
    jsonsl_jpr_match_state_cleanup(jsn);
    jsonsl_destroy(jsn);
    jsonsl_destroy(jsn_rdetails);
    jsonsl_jpr_destroy(jpr);
}

typedef struct {
    const char *root;
    lcb_IOV *next_iov;
    Row *datum;
    Parser *parent;
} miniparse_ctx;

static void parse_json_docid(lcb_IOV *iov, Parser *parent)
{
    Json::Reader r;
    const char *s = static_cast< char * >(iov->iov_base);
    const char *s_end = s + iov->iov_len;
    Json::Value &jvp = parent->cxx_data;
    bool rv = r.parse(s, s_end, jvp);
    if (!rv) {
        // fprintf(stderr, "libcouchbase: Failed to parse document ID as JSON!\n");
        return;
    }

    s = NULL;
    s_end = NULL;

    lcb_assert(jvp.isString());

    // Re-use s and s_end values for the string value itself
    if (!jvp.getString(&s, &s_end)) {
        // fprintf(stderr, "libcouchbase: couldn't get string value!\n");
        iov->iov_base = NULL;
        iov->iov_len = 0;
    }
    iov->iov_base = const_cast< char * >(s);
    iov->iov_len = s_end - s;
}

static void miniparse_callback(jsonsl_t jsn, jsonsl_action_t, struct jsonsl_state_st *state, const jsonsl_char_t *at)
{
    miniparse_ctx *ctx = reinterpret_cast< miniparse_ctx * >(jsn->data);
    lcb_IOV *iov;

    if (state->level == 1) {
        return;
    }

    /* Is a hashkey? */
    if (state->type == JSONSL_T_HKEY) {
        size_t nhk = state->pos_cur - state->pos_begin;

        nhk--;

#define IS_ROWFIELD(s) (nhk == sizeof(s) - 1 && !strncmp(s, at - (sizeof(s) - 1), sizeof(s) - 1))

        if (IS_ROWFIELD("id")) {
            /* "id" */
            ctx->next_iov = &ctx->datum->docid;
        } else if (IS_ROWFIELD("key")) {
            /* "key" */
            ctx->next_iov = &ctx->datum->key;
        } else if (IS_ROWFIELD("value")) {
            /* "value" */
            ctx->next_iov = &ctx->datum->value;
        } else if (IS_ROWFIELD("geometry")) {
            ctx->next_iov = &ctx->datum->geo;
        } else {
            ctx->next_iov = NULL;
        }
#undef IS_ROWFIELD
        return;
    }

    if (ctx->next_iov == NULL) {
        return;
    }

    iov = ctx->next_iov;

    if (JSONSL_STATE_IS_CONTAINER(state)) {
        iov->iov_base = (void *)(ctx->root + state->pos_begin);
        iov->iov_len = (jsn->pos - state->pos_begin) + 1;
    } else if (iov == &ctx->datum->docid) {
        if (state->nescapes) {
            iov->iov_base = (void *)(ctx->root + state->pos_begin);
            iov->iov_len = (state->pos_cur - state->pos_begin) + 1;
            parse_json_docid(iov, ctx->parent);
        } else {
            iov->iov_base = (void *)(ctx->root + state->pos_begin + 1);
            iov->iov_len = (state->pos_cur - state->pos_begin) - 1;
        }
    } else {
        iov->iov_base = (void *)(ctx->root + state->pos_begin);
        iov->iov_len = state->pos_cur - state->pos_begin;
        if (state->type == JSONSL_T_STRING) {
            iov->iov_len++;
        }
    }
}

void Parser::parse_viewrow(Row &vr)
{
    miniparse_ctx ctx = {NULL};
    ctx.datum = &vr;
    ctx.root = static_cast< const char * >(vr.row.iov_base);
    ctx.parent = this;

    jsonsl_reset(jsn_rdetails);

    jsonsl_enable_all_callbacks(jsn_rdetails);
    jsn_rdetails->max_callback_level = 3;
    jsn_rdetails->action_callback_POP = miniparse_callback;
    jsn_rdetails->data = &ctx;

    jsonsl_feed(jsn_rdetails, static_cast< const char * >(vr.row.iov_base), vr.row.iov_len);
}
