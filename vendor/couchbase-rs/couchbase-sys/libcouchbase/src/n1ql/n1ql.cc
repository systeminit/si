/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018-2019 Couchbase, Inc.
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

#include <libcouchbase/couchbase.h>
#include <jsparse/parser.h>
#include "internal.h"
#include "auth-priv.h"
#include "http/http.h"
#include "logging.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <map>
#include <string>
#include <list>

#define LOGFMT "(NR=%p) "
#define LOGID(req) static_cast< const void * >(req)
#define LOGARGS(req, lvl) req->instance->settings, "n1ql", LCB_LOG_##lvl, __FILE__, __LINE__

/**
 * Command structure for N1QL queries. Typically an application will use the
 * lcb_N1QLPARAMS structure to populate the #query and #content_type fields.
 *
 * The #callback field must be specified, and indicates the function the
 * library should call when more response data has arrived.
 */
struct lcb_CMDN1QL_ {
    LCB_CMD_BASE;
    Json::Value root;
    /**Query to be placed in the POST request. The library will not perform
     * any conversions or validation on this string, so it is up to the user
     * (or wrapping library) to ensure that the string is well formed.
     *
     * If using the @ref lcb_N1QLPARAMS structure, the lcb_n1p_mkcmd() function
     * will properly populate this field.
     *
     * In general the string should either be JSON (in which case, the
     * #content_type field should be `application/json`) or url-encoded
     * (in which case the #content_type field should be
     * `application/x-www-form-urlencoded`)
     */
    std::string query;

    /** Callback to be invoked for each row */
    lcb_N1QL_CALLBACK callback;

    /**Request handle. Will be set to the handle which may be passed to
     * lcb_n1ql_cancel() */
    lcb_N1QL_HANDLE **handle;

    lcb_CMDN1QL_() : callback(NULL), handle(NULL)
    {
        RESET_CMD_BASE(this);
    }
};

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_status(const lcb_RESPN1QL *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_cookie(const lcb_RESPN1QL *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_row(const lcb_RESPN1QL *resp, const char **row, size_t *row_len)
{
    *row = resp->row;
    *row_len = resp->nrow;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_http_response(const lcb_RESPN1QL *resp, const lcb_RESPHTTP **http)
{
    *http = resp->htresp;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respn1ql_handle(const lcb_RESPN1QL *resp, lcb_N1QL_HANDLE **handle)
{
    *handle = resp->handle;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API int lcb_respn1ql_is_final(const lcb_RESPN1QL *resp)
{
    return resp->rflags & LCB_RESP_F_FINAL;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_create(lcb_CMDN1QL **cmd)
{
    *cmd = new lcb_CMDN1QL();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_destroy(lcb_CMDN1QL *cmd)
{
    if (cmd) {
        delete cmd;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_timeout(lcb_CMDN1QL *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_reset(lcb_CMDN1QL *cmd)
{
    RESET_CMD_BASE(cmd);
    cmd->root = Json::Value();
    cmd->query = "";
    cmd->callback = NULL;
    cmd->handle = NULL;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_parent_span(lcb_CMDN1QL *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_callback(lcb_CMDN1QL *cmd, lcb_N1QL_CALLBACK callback)
{
    cmd->callback = callback;
    return LCB_SUCCESS;
}

#define fix_strlen(s, n)                                                                                               \
    if (n == (size_t)-1) {                                                                                             \
        n = strlen(s);                                                                                                 \
    }

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_payload(lcb_CMDN1QL *cmd, const char **payload, size_t *payload_len)
{
    cmd->query = Json::FastWriter().write(cmd->root);
    *payload = cmd->query.c_str();
    *payload_len = cmd->query.size();
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_query(lcb_CMDN1QL *cmd, const char *query, size_t query_len)
{
    fix_strlen(query, query_len);
    Json::Value value;
    if (!Json::Reader().parse(query, query + query_len, value)) {
        return LCB_EINVAL;
    }
    cmd->root = value;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_statement(lcb_CMDN1QL *cmd, const char *statement, size_t statement_len)
{
    fix_strlen(statement, statement_len);
    cmd->root["statement"] = std::string(statement, statement_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_named_param(lcb_CMDN1QL *cmd, const char *name, size_t name_len,
                                                    const char *value, size_t value_len)
{
    std::string key = "$" + std::string(name, name_len);
    return lcb_cmdn1ql_option(cmd, key.c_str(), key.size(), value, value_len);
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_positional_param(lcb_CMDN1QL *cmd, const char *value, size_t value_len)
{
    fix_strlen(value, value_len);
    Json::Value jval;
    if (!Json::Reader().parse(value, value + value_len, jval)) {
        return LCB_EINVAL;
    }
    cmd->root["args"].append(jval);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_adhoc(lcb_CMDN1QL *cmd, int adhoc)
{
    if (adhoc) {
        cmd->cmdflags &= ~LCB_CMDN1QL_F_PREPCACHE;
    } else {
        cmd->cmdflags |= LCB_CMDN1QL_F_PREPCACHE;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_client_context_id(lcb_CMDN1QL *cmd, const char* value, size_t value_len)
{
    cmd->root["client_context_id"] = std::string(value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pretty(lcb_CMDN1QL *cmd, int pretty)
{
    cmd->root["pretty"] = pretty != 0;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_readonly(lcb_CMDN1QL *cmd, int readonly)
{
    cmd->root["readonly"] = readonly ? true : false;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_scan_cap(lcb_CMDN1QL *cmd, int value)
{
    cmd->root["scan_cap"] = Json::valueToString(value);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pipeline_cap(lcb_CMDN1QL *cmd, int value)
{
    cmd->root["pipeline_cap"] = Json::valueToString(value);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_pipeline_batch(lcb_CMDN1QL *cmd, int value)
{
    cmd->root["pipeline_batch"] = Json::valueToString(value);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency(lcb_CMDN1QL *cmd, lcb_N1QL_CONSISTENCY mode)
{
    if (mode == LCB_N1QL_CONSISTENCY_NONE) {
        cmd->root.removeMember("scan_consistency");
    } else if (mode == LCB_N1QL_CONSISTENCY_REQUEST) {
        cmd->root["scan_consistency"] = "request_plus";
    } else if (mode == LCB_N1QL_CONSISTENCY_STATEMENT) {
        cmd->root["scan_consistency"] = "statement_plus";
    }
    return LCB_SUCCESS;
}

static void encode_mutation_token(Json::Value &sparse, const lcb_MUTATION_TOKEN *sv)
{
    char buf[64] = {0};
    sprintf(buf, "%u", sv->vbid_);
    Json::Value &cur_sv = sparse[buf];

    cur_sv[0] = static_cast< Json::UInt64 >(sv->seqno_);
    sprintf(buf, "%llu", (unsigned long long)sv->uuid_);
    cur_sv[1] = buf;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency_token_for_keyspace(lcb_CMDN1QL *cmd, const char *keyspace,
                                                                       size_t keyspace_len,
                                                                       const lcb_MUTATION_TOKEN *token)
{
    if (!LCB_MUTATION_TOKEN_ISVALID(token)) {
        return LCB_EINVAL;
    }

    cmd->root["scan_consistency"] = "at_plus";
    encode_mutation_token(cmd->root["scan_vectors"][std::string(keyspace, keyspace_len)], token);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_consistency_tokens(lcb_CMDN1QL *cmd, lcb_INSTANCE *instance)
{
    lcbvb_CONFIG *vbc;
    lcb_STATUS rc = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc);
    if (rc != LCB_SUCCESS) {
        return rc;
    }

    const char *bucketname;
    rc = lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_BUCKETNAME, &bucketname);
    if (rc != LCB_SUCCESS) {
        return rc;
    }

    Json::Value *sv_json = NULL;

    size_t vbmax = vbc->nvb;
    for (size_t ii = 0; ii < vbmax; ++ii) {
        lcb_KEYBUF kb;
        kb.type = LCB_KV_VBID;
        kb.vbid = ii;
        const lcb_MUTATION_TOKEN *mt = lcb_get_mutation_token(instance, &kb, &rc);
        if (rc == LCB_SUCCESS && mt != NULL) {
            if (sv_json == NULL) {
                sv_json = &cmd->root["scan_vectors"][bucketname];
                cmd->root["scan_consistency"] = "at_plus";
            }
            encode_mutation_token(*sv_json, mt);
        }
    }

    if (!sv_json) {
        return LCB_KEY_ENOENT;
    }

    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_option(lcb_CMDN1QL *cmd, const char *name, size_t name_len, const char *value,
                                               size_t value_len)
{
    fix_strlen(name, name_len);
    fix_strlen(value, value_len);
    Json::Reader rdr;
    Json::Value jsonValue;
    bool rv = rdr.parse(value, value + value_len, jsonValue);
    if (!rv) {
        return LCB_EINVAL;
    }

    cmd->root[std::string(name, name_len)] = jsonValue;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdn1ql_handle(lcb_CMDN1QL *cmd, lcb_N1QL_HANDLE **handle)
{
    cmd->handle = handle;
    return LCB_SUCCESS;
}

// Indicate that the 'creds' field is to be used.
#define F_CMDN1QL_CREDSAUTH 1 << 15

class Plan
{
  private:
    friend struct lcb_N1QLCACHE_st;
    std::string key;
    std::string planstr;
    Plan(const std::string &k) : key(k) {}

  public:
    /**
     * Applies the plan to the output 'bodystr'. We don't assign the
     * Json::Value directly, as this appears to be horribly slow. On my system
     * an assignment took about 200ms!
     * @param body The request body (e.g. N1QLREQ::json)
     * @param[out] bodystr the actual request payload
     */
    void apply_plan(Json::Value &body, std::string &bodystr) const
    {
        body.removeMember("statement");
        bodystr = Json::FastWriter().write(body);

        // Assume bodystr ends with '}'
        size_t pos = bodystr.rfind('}');
        bodystr.erase(pos);

        if (body.size() > 0) {
            bodystr.append(",");
        }
        bodystr.append(planstr);
        bodystr.append("}");
    }

  private:
    /**
     * Assign plan data to this entry
     * @param plan The JSON returned from the PREPARE request
     */
    void set_plan(const Json::Value &plan, bool include_encoded_plan)
    {
        // Set the plan as a string
        planstr = "\"prepared\":";
        planstr += Json::FastWriter().write(plan["name"]);
        if (include_encoded_plan) {
            planstr += ",";
            planstr += "\"encoded_plan\":";
            planstr += Json::FastWriter().write(plan["encoded_plan"]);
        }
    }
};

// LRU Cache structure..
struct lcb_N1QLCACHE_st {
    typedef std::list< Plan * > LruCache;
    typedef std::map< std::string, LruCache::iterator > Lookup;

    Lookup by_name;
    LruCache lru;

    /** Maximum number of entries in LRU cache. This is fixed at 5000 */
    static size_t max_size()
    {
        return 5000;
    }

    /**
     * Adds an entry for a given key
     * @param key The key to add
     * @param json The prepared statement returned by the server
     * @return the newly added plan.
     */
    const Plan &add_entry(const std::string &key, const Json::Value &json, bool include_encoded_plan = true)
    {
        if (lru.size() == max_size()) {
            // Purge entry from end
            remove_entry(lru.back()->key);
        }

        // Remove old entry, if present
        remove_entry(key);

        lru.push_front(new Plan(key));
        by_name[key] = lru.begin();
        lru.front()->set_plan(json, include_encoded_plan);
        return *lru.front();
    }

    /**
     * Gets the entry for a given key
     * @param key The statement (key) to look up
     * @return a pointer to the plan if present, NULL if no entry exists for key
     */
    const Plan *get_entry(const std::string &key)
    {
        Lookup::iterator m = by_name.find(key);
        if (m == by_name.end()) {
            return NULL;
        }

        const Plan *cur = *m->second;

        // Update LRU:
        lru.splice(lru.begin(), lru, m->second);
        // Note, updating of iterators is not required since splice doesn't
        // invalidate iterators.
        return cur;
    }

    /** Removes an entry with the given key */
    void remove_entry(const std::string &key)
    {
        Lookup::iterator m = by_name.find(key);
        if (m == by_name.end()) {
            return;
        }
        // Remove entry from map
        LruCache::iterator m2 = m->second;
        delete *m2;
        by_name.erase(m);
        lru.erase(m2);
    }

    /** Clears the LRU cache */
    void clear()
    {
        for (LruCache::iterator ii = lru.begin(); ii != lru.end(); ++ii) {
            delete *ii;
        }
        lru.clear();
        by_name.clear();
    }

    ~lcb_N1QLCACHE_st()
    {
        clear();
    }
};

typedef struct lcb_N1QL_HANDLE_ : lcb::jsparse::Parser::Actions {
    const lcb_RESPHTTP *cur_htresp;
    lcb_HTTP_HANDLE *htreq;
    lcb::jsparse::Parser *parser;
    const void *cookie;
    lcb_N1QL_CALLBACK callback;
    lcb_INSTANCE *instance;
    lcb_STATUS lasterr;
    lcb_U32 flags;
    lcb_U32 timeout;
    // How many rows were received. Used to avoid parsing the meta
    size_t nrows;

    /** The PREPARE query itself */
    struct lcb_N1QL_HANDLE_ *prepare_req;

    /** Request body as received from the application */
    Json::Value json;
    const Json::Value &json_const() const
    {
        return json;
    }

    /** String of the original statement. Cached here to avoid jsoncpp lookups */
    std::string statement;

    /** Whether we're retrying this */
    bool was_retried;

    /** Is this query to Analytics for N1QL service */
    bool is_cbas;

    lcbtrace_SPAN *span;

    lcb_N1QLCACHE &cache()
    {
        return *instance->n1ql_cache;
    }

    /**
     * Creates the sub-N1QLREQ for the PREPARE statement. This inspects the
     * current request (see ::json) and copies it so that we execute the
     * PREPARE instead of the actual query.
     * @return see issue_htreq()
     */
    inline lcb_STATUS request_plan();

    /**
     * Use the plan to execute the given query, and issues the query
     * @param plan The plan itself
     * @return see issue_htreq()
     */
    inline lcb_STATUS apply_plan(const Plan &plan);

    /**
     * Issues the HTTP request for the query
     * @param payload The body to send
     * @return Error code from lcb's http subsystem
     */
    inline lcb_STATUS issue_htreq(const std::string &payload);

    lcb_STATUS issue_htreq()
    {
        std::string s = Json::FastWriter().write(json);
        return issue_htreq(s);
    }

    /**
     * Attempt to retry the query. This will inspect the meta (if present)
     * for any errors indicating that a failure might be a result of a stale
     * plan, and if this query was retried already.
     * @return true if the retry was successful.
     */
    inline bool maybe_retry();

    /**
     * Returns true if payload matches retry conditions.
     */
    inline bool has_retriable_error(const Json::Value &root);

    /**
     * Did the application request this query to use prepared statements
     * @return true if using prepared statements
     */
    inline bool use_prepcache() const
    {
        return flags & LCB_CMDN1QL_F_PREPCACHE;
    }

    /**
     * Pass a row back to the application
     * @param resp The response. This is populated with state information
     *  from the current query
     * @param is_last Whether this is the last row. If this is the last, then
     *  the RESP_F_FINAL flag is set, and no further callbacks will be invoked
     */
    inline void invoke_row(lcb_RESPN1QL *resp, bool is_last);

    /**
     * Fail an application-level query because the prepared statement failed
     * @param orig The response from the PREPARE request
     * @param err The error code
     */
    inline void fail_prepared(const lcb_RESPN1QL *orig, lcb_STATUS err);

    inline lcb_N1QL_HANDLE_(lcb_INSTANCE *obj, const void *user_cookie, const lcb_CMDN1QL *cmd);
    inline ~lcb_N1QL_HANDLE_();

    // Parser overrides:
    void JSPARSE_on_row(const lcb::jsparse::Row &row)
    {
        lcb_RESPN1QL resp = {0};
        resp.row = static_cast< const char * >(row.row.iov_base);
        resp.nrow = row.row.iov_len;
        nrows++;
        invoke_row(&resp, false);
    }
    void JSPARSE_on_error(const std::string &)
    {
        lasterr = LCB_PROTOCOL_ERROR;
    }
    void JSPARSE_on_complete(const std::string &)
    {
        // Nothing
    }

} N1QLREQ;

static bool parse_json(const char *s, size_t n, Json::Value &res)
{
    return Json::Reader().parse(s, s + n, res);
}

lcb_N1QLCACHE *lcb_n1qlcache_create(void)
{
    return new lcb_N1QLCACHE;
}

void lcb_n1qlcache_destroy(lcb_N1QLCACHE *cache)
{
    delete cache;
}

void lcb_n1qlcache_clear(lcb_N1QLCACHE *cache)
{
    cache->clear();
}

// Special function for debugging. This returns the name and encoded form of
// the plan
void lcb_n1qlcache_getplan(lcb_N1QLCACHE *cache, const std::string &key, std::string &out)
{
    const Plan *plan = cache->get_entry(key);
    if (plan != NULL) {
        Json::Value tmp(Json::objectValue);
        plan->apply_plan(tmp, out);
    }
}

static const char *wtf_magic_strings[] = {
    "index deleted or node hosting the index is down - cause: queryport.indexNotFound",
    "Index Not Found - cause: queryport.indexNotFound", NULL};

bool N1QLREQ::has_retriable_error(const Json::Value &root)
{
    if (!root.isObject()) {
        return false;
    }
    const Json::Value &errors = root["errors"];
    if (!errors.isArray()) {
        return false;
    }
    Json::Value::const_iterator ii;
    for (ii = errors.begin(); ii != errors.end(); ++ii) {
        const Json::Value &cur = *ii;
        if (!cur.isObject()) {
            continue; // eh?
        }
        const Json::Value &jmsg = cur["msg"];
        const Json::Value &jcode = cur["code"];
        unsigned code = 0;
        if (jcode.isNumeric()) {
            code = jcode.asUInt();
            switch (code) {
                    /* n1ql */
                case 4040: /* statement not found */
                case 4050:
                case 4070:
                    /* analytics */
                case 23000:
                case 23003:
                case 23007:
                    lcb_log(LOGARGS(this, TRACE), LOGFMT "Will retry request. code: %d", LOGID(this), code);
                    return true;
                default:
                    break;
            }
        }
        if (jmsg.isString()) {
            const char *jmstr = jmsg.asCString();
            for (const char **curs = wtf_magic_strings; *curs; curs++) {
                if (!strstr(jmstr, *curs)) {
                    lcb_log(LOGARGS(this, TRACE), LOGFMT "Will retry request. code: %d, msg: %s", LOGID(this), code,
                            jmstr);
                    return true;
                }
            }
        }
    }
    return false;
}

bool N1QLREQ::maybe_retry()
{
    // Examines the buffer to determine the type of error
    Json::Value root;
    lcb_IOV meta;

    if (callback == NULL) {
        // Cancelled
        return false;
    }

    if (nrows) {
        // Has results:
        return false;
    }

    if (was_retried) {
        return false;
    }

    if (!use_prepcache()) {
        // Didn't use our built-in caching (maybe using it from elsewhere?)
        return false;
    }

    was_retried = true;
    parser->get_postmortem(meta);
    if (!parse_json(static_cast< const char * >(meta.iov_base), meta.iov_len, root)) {
        return false; // Not JSON
    }
    if (!has_retriable_error(root)) {
        return false;
    }

    lcb_log(LOGARGS(this, ERROR), LOGFMT "Repreparing statement. Index or version mismatch.", LOGID(this));

    // Let's see if we can actually retry. First remove the existing prepared
    // entry:
    cache().remove_entry(statement);

    if ((lasterr = request_plan()) == LCB_SUCCESS) {
        // We'll be parsing more rows later on..
        delete parser;
        parser = new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_N1QL, this);
        return true;
    }

    return false;
}

void N1QLREQ::invoke_row(lcb_RESPN1QL *resp, bool is_last)
{
    resp->cookie = const_cast< void * >(cookie);
    resp->htresp = cur_htresp;
    resp->handle = this;

    if (is_last) {
        lcb_IOV meta;
        resp->rflags |= LCB_RESP_F_FINAL;
        resp->rc = lasterr;
        parser->get_postmortem(meta);
        resp->row = static_cast< const char * >(meta.iov_base);
        resp->nrow = meta.iov_len;
    }

    if (callback) {
        callback(instance, LCB_CALLBACK_N1QL, resp);
    }
    if (is_last) {
        callback = NULL;
    }
}

lcb_N1QL_HANDLE_::~lcb_N1QL_HANDLE_()
{
    if (htreq) {
        lcb_http_cancel(instance, htreq);
        htreq = NULL;
    }

    if (callback) {
        lcb_RESPN1QL resp = {0};
        invoke_row(&resp, 1);
    }

    if (span) {
        if (htreq) {
            lcbio_CTX *ctx = htreq->ioctx;
            if (ctx) {
                std::string remote;
                if (htreq->ipv6) {
                    remote = "[" + std::string(htreq->host) + "]:" + std::string(htreq->port);
                } else {
                    remote = std::string(htreq->host) + ":" + std::string(htreq->port);
                }
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_PEER_ADDRESS, remote.c_str());
                lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_LOCAL_ADDRESS,
                                          lcbio__inet_ntop(&ctx->sock->info->sa_local).c_str());
            }
        }
        lcbtrace_span_finish(span, LCBTRACE_NOW);
        span = NULL;
    }

    if (parser) {
        delete parser;
    }
    if (prepare_req) {
        lcb_n1ql_cancel(instance, prepare_req);
    }
}

static void chunk_callback(lcb_INSTANCE *instance, int ign, const lcb_RESPBASE *rb)
{
    const lcb_RESPHTTP *rh = (const lcb_RESPHTTP *)rb;
    N1QLREQ *req = static_cast< N1QLREQ * >(rh->cookie);

    (void)ign;
    (void)instance;

    req->cur_htresp = rh;
    if (rh->rc != LCB_SUCCESS || rh->htstatus != 200) {
        if (req->lasterr == LCB_SUCCESS || rh->htstatus != 200) {
            req->lasterr = rh->rc ? rh->rc : LCB_HTTP_ERROR;
        }
    }

    if (rh->rflags & LCB_RESP_F_FINAL) {
        req->htreq = NULL;
        if (!req->maybe_retry()) {
            delete req;
        }
        return;
    } else if (req->callback == NULL) {
        /* Cancelled. Similar to the block above, except the http request
         * should remain alive (so we can cancel it later on) */
        delete req;
        return;
    }
    req->parser->feed(static_cast< const char * >(rh->body), rh->nbody);
}

void N1QLREQ::fail_prepared(const lcb_RESPN1QL *orig, lcb_STATUS err)
{
    lcb_log(LOGARGS(this, ERROR), LOGFMT "Prepare failed!", LOGID(this));

    lcb_RESPN1QL newresp = *orig;
    newresp.rflags = LCB_RESP_F_FINAL;
    newresp.cookie = const_cast< void * >(cookie);
    newresp.rc = err;
    if (err == LCB_SUCCESS) {
        newresp.rc = LCB_ERROR;
    }

    if (callback != NULL) {
        callback(instance, LCB_CALLBACK_N1QL, &newresp);
        callback = NULL;
    }
    delete this;
}

// Received internally for PREPARE
static void prepare_rowcb(lcb_INSTANCE *instance, int, const lcb_RESPN1QL *row)
{
    lcb_N1QL_HANDLE_ *origreq = reinterpret_cast< lcb_N1QL_HANDLE_ * >(row->cookie);

    lcb_n1ql_cancel(instance, origreq->prepare_req);
    origreq->prepare_req = NULL;

    if (row->rc != LCB_SUCCESS || (row->rflags & LCB_RESP_F_FINAL)) {
        origreq->fail_prepared(row, row->rc);
    } else {
        // Insert into cache
        Json::Value prepared;
        if (!parse_json(row->row, row->nrow, prepared)) {
            lcb_log(LOGARGS(origreq, ERROR), LOGFMT "Invalid JSON returned from PREPARE", LOGID(origreq));
            origreq->fail_prepared(row, LCB_PROTOCOL_ERROR);
            return;
        }

        bool eps = LCBVB_CCAPS(LCBT_VBCONFIG(instance)) & LCBVB_CCAP_N1QL_ENHANCED_PREPARED_STATEMENTS;
        // Insert plan into cache
        lcb_log(LOGARGS(origreq, DEBUG), LOGFMT "Got %sprepared statement. Inserting into cache and reissuing",
            LOGID(origreq), eps ? "(enhanced) " : "");
        const Plan &ent = origreq->cache().add_entry(origreq->statement, prepared, !eps);

        // Issue the query with the newly prepared plan
        lcb_STATUS rc = origreq->apply_plan(ent);
        if (rc != LCB_SUCCESS) {
            origreq->fail_prepared(row, rc);
        }
    }
}

lcb_STATUS N1QLREQ::issue_htreq(const std::string &body)
{
    std::string content_type("application/json");

    lcb_CMDHTTP *htcmd;
    if (is_cbas) {
        lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_CBAS);
    } else {
        lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_N1QL);
    }
    lcb_cmdhttp_body(htcmd, body.c_str(), body.size());
    lcb_cmdhttp_content_type(htcmd, content_type.c_str(), content_type.size());
    lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);
    lcb_cmdhttp_streaming(htcmd, true);
    lcb_cmdhttp_timeout(htcmd, timeout);
    lcb_cmdhttp_handle(htcmd, &htreq);
    if (flags & F_CMDN1QL_CREDSAUTH) {
        lcb_cmdhttp_skip_auth_header(htcmd, true);
    }

    lcb_STATUS rc = lcb_http(instance, this, htcmd);
    lcb_cmdhttp_destroy(htcmd);
    if (rc == LCB_SUCCESS) {
        htreq->set_callback(chunk_callback);
    }
    return rc;
}

lcb_STATUS N1QLREQ::request_plan()
{
    Json::Value newbody(Json::objectValue);
    newbody["statement"] = "PREPARE " + statement;
    lcb_CMDN1QL newcmd;
    newcmd.callback = prepare_rowcb;
    newcmd.cmdflags = LCB_CMDN1QL_F_JSONQUERY;
    newcmd.handle = &prepare_req;
    newcmd.root = newbody;
    if (flags & F_CMDN1QL_CREDSAUTH) {
        newcmd.cmdflags |= LCB_CMD_F_MULTIAUTH;
    }

    return lcb_n1ql(instance, this, &newcmd);
}

lcb_STATUS N1QLREQ::apply_plan(const Plan &plan)
{
    lcb_log(LOGARGS(this, DEBUG), LOGFMT "Using prepared plan", LOGID(this));
    std::string bodystr;
    plan.apply_plan(json, bodystr);
    return issue_htreq(bodystr);
}

lcb_U32 lcb_n1qlreq_parsetmo(const std::string &s)
{
    double num;
    int nchars, rv;

    rv = sscanf(s.c_str(), "%lf%n", &num, &nchars);
    if (rv != 1) {
        return 0;
    }
    std::string mults = s.substr(nchars);

    // Get the actual timeout value in microseconds. Note we can't use the macros
    // since they will truncate the double value.
    if (mults == "s") {
        return num * static_cast< double >(LCB_S2US(1));
    } else if (mults == "ms") {
        return num * static_cast< double >(LCB_MS2US(1));
    } else if (mults == "h") {
        return num * static_cast< double >(LCB_S2US(3600));
    } else if (mults == "us") {
        return num;
    } else if (mults == "m") {
        return num * static_cast< double >(LCB_S2US(60));
    } else if (mults == "ns") {
        return LCB_NS2US(num);
    } else {
        return 0;
    }
}

lcb_N1QL_HANDLE_::lcb_N1QL_HANDLE_(lcb_INSTANCE *obj, const void *user_cookie, const lcb_CMDN1QL *cmd)
    : cur_htresp(NULL), htreq(NULL), parser(new lcb::jsparse::Parser(lcb::jsparse::Parser::MODE_N1QL, this)),
      cookie(user_cookie), callback(cmd->callback), instance(obj), lasterr(LCB_SUCCESS), flags(cmd->cmdflags),
      timeout(0), nrows(0), prepare_req(NULL), was_retried(false), is_cbas(false), span(NULL)
{
    if (cmd->handle) {
        *cmd->handle = this;
    }

    if (flags & LCB_CMDN1QL_F_JSONQUERY) {
        json = cmd->root;
    } else {
        std::string encoded = Json::FastWriter().write(cmd->root);
        if (!parse_json(encoded.c_str(), encoded.size(), json)) {
            lasterr = LCB_EINVAL;
            return;
        }
    }

    if (flags & LCB_CMDN1QL_F_ANALYTICSQUERY) {
        is_cbas = true;
    }
    if (is_cbas && (flags & LCB_CMDN1QL_F_PREPCACHE)) {
        lasterr = LCB_OPTIONS_CONFLICT;
        return;
    }

    const Json::Value &j_statement = json_const()["statement"];
    if (j_statement.isString()) {
        statement = j_statement.asString();
    } else if (!j_statement.isNull()) {
        lasterr = LCB_EINVAL;
        return;
    }

    timeout = LCBT_SETTING(obj, n1ql_timeout);
    if (cmd->timeout) {
        timeout = cmd->timeout;
    }
    Json::Value &tmoval = json["timeout"];
    if (tmoval.isNull()) {
        char buf[64] = {0};
        sprintf(buf, "%uus", timeout);
        tmoval = buf;
    } else if (tmoval.isString()) {
        timeout = lcb_n1qlreq_parsetmo(tmoval.asString());
    } else {
        // Timeout is not a string!
        lasterr = LCB_EINVAL;
        return;
    }

    // Determine if we need to add more credentials.
    // Because N1QL multi-bucket auth will not work on server versions < 4.5
    // using JSON encoding, we need to only use the multi-bucket auth feature
    // if there are actually multiple credentials to employ.
    const lcb::Authenticator &auth = *instance->settings->auth;
    if (auth.buckets().size() > 1 && (cmd->cmdflags & LCB_CMD_F_MULTIAUTH)) {
        flags |= F_CMDN1QL_CREDSAUTH;
        Json::Value &creds = json["creds"];
        lcb::Authenticator::Map::const_iterator ii = auth.buckets().begin();
        if (!(creds.isNull() || creds.isArray())) {
            lasterr = LCB_EINVAL;
            return;
        }
        for (; ii != auth.buckets().end(); ++ii) {
            if (ii->second.empty()) {
                continue;
            }
            Json::Value &curCreds = creds.append(Json::Value(Json::objectValue));
            curCreds["user"] = ii->first;
            curCreds["pass"] = ii->second;
        }
    }
    if (instance->settings->tracer) {
        char id[20] = {0};
        snprintf(id, sizeof(id), "%p", (void *)this);
        span = lcbtrace_span_start(instance->settings->tracer, LCBTRACE_OP_DISPATCH_TO_SERVER, LCBTRACE_NOW, NULL);
        lcbtrace_span_add_tag_str(span, LCBTRACE_TAG_OPERATION_ID, id);
        lcbtrace_span_add_system_tags(span, instance->settings,
                                      is_cbas ? LCBTRACE_TAG_SERVICE_ANALYTICS : LCBTRACE_TAG_SERVICE_N1QL);
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_n1ql(lcb_INSTANCE *instance, void *cookie, const lcb_CMDN1QL *cmd)
{
    lcb_STATUS err;
    N1QLREQ *req = NULL;

    if ((cmd->query.empty() && cmd->root.empty()) || cmd->callback == NULL) {
        return LCB_EINVAL;
    }
    req = new lcb_N1QL_HANDLE_(instance, cookie, cmd);
    if (!req) {
        err = LCB_CLIENT_ENOMEM;
        goto GT_DESTROY;
    }
    if ((err = req->lasterr) != LCB_SUCCESS) {
        goto GT_DESTROY;
    }

    if (cmd->cmdflags & LCB_CMDN1QL_F_PREPCACHE) {
        if (req->statement.empty()) {
            err = LCB_EINVAL;
            goto GT_DESTROY;
        }

        const Plan *cached = req->cache().get_entry(req->statement);
        if (cached != NULL) {
            if ((err = req->apply_plan(*cached)) != LCB_SUCCESS) {
                goto GT_DESTROY;
            }
        } else {
            lcb_log(LOGARGS(req, DEBUG), LOGFMT "No cached plan found. Issuing prepare", LOGID(req));
            if ((err = req->request_plan()) != LCB_SUCCESS) {
                goto GT_DESTROY;
            }
        }
    } else {
        // No prepare
        if ((err = req->issue_htreq()) != LCB_SUCCESS) {
            goto GT_DESTROY;
        }
    }

    return LCB_SUCCESS;

GT_DESTROY:
    if (cmd->handle) {
        *cmd->handle = NULL;
    }

    if (req) {
        req->callback = NULL;
        delete req;
    }
    return err;
}

LIBCOUCHBASE_API lcb_STATUS lcb_n1ql_cancel(lcb_INSTANCE *instance, lcb_N1QL_HANDLE *handle)
{
    // Note that this function is just an elaborate way to nullify the
    // callback. We are very particular about _not_ cancelling the underlying
    // http request, because the handle's deletion is controlled
    // from the HTTP callback, which checks if the callback is NULL before
    // deleting.
    // at worst, deferring deletion to the http response might cost a few
    // extra network reads; whereas this function itself is intended as a
    // bailout for unexpected destruction.

    if (handle->prepare_req) {
        lcb_n1ql_cancel(instance, handle->prepare_req);
        handle->prepare_req = NULL;
    }
    handle->callback = NULL;
    return LCB_SUCCESS;
}
