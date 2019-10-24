/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2016-2019 Couchbase, Inc.
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
#define NOMINMAX
#include <libcouchbase/ixmgmt.h>
#include <string>
#include <set>
#include <algorithm>

#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include "lcbio/lcbio.h"
#include "lcbio/timer-ng.h"
#include "settings.h"
#include "internal.h"

#define LOGFMT "(mgreq=%p) "
#define LOGID(req) static_cast< const void * >(req)
#define LOGARGS(req, lvl) (req)->m_instance->settings, "ixmgmt", LCB_LOG_##lvl, __FILE__, __LINE__

using std::string;
using std::vector;

static const char *ixtype_2_str(unsigned ixtype)
{
    if (ixtype == LCB_N1XSPEC_T_GSI) {
        return "gsi";
    } else if (ixtype == LCB_N1XSPEC_T_VIEW) {
        return "view";
    } else {
        return NULL;
    }
}

struct IndexOpCtx {
    lcb_N1XMGMTCALLBACK callback;
    void *cookie;
};

struct ErrorSpec {
    string msg;
    unsigned code;
};

template < typename T > void my_delete(T p)
{
    delete p;
}

template < typename T > lcb_STATUS extract_n1ql_errors(const char *s, size_t n, T &err_out)
{
    Json::Value jresp;
    if (!Json::Reader().parse(s, s + n, jresp)) {
        return LCB_PROTOCOL_ERROR;
    }
    if (jresp["status"].asString() == "success") {
        return LCB_SUCCESS;
    }

    Json::Value &errors = jresp["errors"];
    if (errors.isNull()) {
        return LCB_SUCCESS;
    } else if (!errors.isArray()) {
        return LCB_PROTOCOL_ERROR;
    }

    if (errors.empty()) {
        return LCB_SUCCESS;
    }

    for (Json::ArrayIndex ii = 0; ii < errors.size(); ++ii) {
        const Json::Value &err = errors[ii];
        if (!err.isObject()) {
            continue; // expected an object!
        }
        ErrorSpec spec;
        spec.msg = err["msg"].asString();
        spec.code = err["code"].asUInt();
        err_out.insert(err_out.end(), spec);
    }
    return LCB_ERROR;
}

static lcb_STATUS get_n1ql_error(const char *s, size_t n)
{
    std::vector< ErrorSpec > dummy;
    return extract_n1ql_errors(s, n, dummy);
}

// Called for generic operations and establishes existence or lack thereof
static void cb_generic(lcb_INSTANCE *instance, int, const lcb_RESPN1QL *resp)
{
    // Get the real cookie..
    if (!(resp->rflags & LCB_RESP_F_FINAL)) {
        return;
    }

    IndexOpCtx *ctx = reinterpret_cast< IndexOpCtx * >(resp->cookie);
    lcb_RESPN1XMGMT w_resp = {0};
    w_resp.cookie = ctx->cookie;

    if ((w_resp.rc = resp->rc) == LCB_SUCCESS || resp->rc == LCB_HTTP_ERROR) {
        // Check if the top-level N1QL response succeeded, and then
        // descend to determine additional errors. This is primarily
        // required to support EEXIST for GSI primary indexes

        vector< ErrorSpec > errors;
        lcb_STATUS rc = extract_n1ql_errors(resp->row, resp->nrow, errors);
        if (rc == LCB_ERROR) {
            w_resp.rc = LCB_QUERY_ERROR;
            for (size_t ii = 0; ii < errors.size(); ++ii) {
                const std::string &msg = errors[ii].msg;
                if (msg.find("already exist") != string::npos) {
                    w_resp.rc = LCB_KEY_EEXISTS; // Index entry already exists
                } else if (msg.find("not found") != string::npos) {
                    w_resp.rc = LCB_KEY_ENOENT;
                }
            }
        } else {
            w_resp.rc = rc;
        }
    }

    w_resp.inner = resp;
    w_resp.specs = NULL;
    w_resp.nspecs = 0;
    ctx->callback(instance, LCB_CALLBACK_IXMGMT, &w_resp);
    delete ctx;
}

/**
 * Dispatch the actual operation using a N1QL query
 * @param instance
 * @param cookie User cookie
 * @param u_callback User callback (to assign to new context)
 * @param i_callback Internal callback to be invoked when N1QL response
 *        is done
 * @param s N1QL request payload
 * @param n N1QL request length
 * @param obj Internal context. Created with new if NULL
 * @return
 *
 * See other overload for passing just the query string w/o extra parameters
 */
template < typename T >
lcb_STATUS dispatch_common(lcb_INSTANCE *instance, const void *cookie, lcb_N1XMGMTCALLBACK u_callback,
                           lcb_N1QL_CALLBACK i_callback, const char *s, size_t n, T *obj)
{
    lcb_STATUS rc = LCB_SUCCESS;
    bool our_alloc = false;
    struct {
        lcb_INSTANCE *m_instance;
    } ixwrap = {instance}; // For logging

    if (obj == NULL) {
        obj = new T();
        our_alloc = true;
    }

    if (!(obj->callback = u_callback)) {
        rc = LCB_EINVAL;
        goto GT_ERROR;
    }

    obj->cookie = const_cast< void * >(cookie);

    lcb_CMDN1QL *cmd;
    lcb_cmdn1ql_create(&cmd);
    lcb_cmdn1ql_query(cmd, s, n);
    lcb_cmdn1ql_callback(cmd, i_callback);
    lcb_log(LOGARGS(&ixwrap, DEBUG), LOGFMT "Issuing query %.*s", LOGID(obj), (int)n, s);
    rc = lcb_n1ql(instance, obj, cmd);
    lcb_cmdn1ql_destroy(cmd);

GT_ERROR:
    if (rc != LCB_SUCCESS && our_alloc) {
        delete obj;
    }
    return rc;
}

template < typename T >
lcb_STATUS dispatch_common(lcb_INSTANCE *instance, const void *cookie, lcb_N1XMGMTCALLBACK u_callback,
                           lcb_N1QL_CALLBACK i_callback, const string &ss, T *obj = NULL)
{
    Json::Value root;
    root["statement"] = ss;
    string reqbuf = Json::FastWriter().write(root);
    return dispatch_common< T >(instance, cookie, u_callback, i_callback, reqbuf.c_str(), reqbuf.size() - 1 /*newline*/,
                                obj);
}

// Class to back the storage for the actual lcb_IXSPEC without doing too much
// mind-numbing buffer copies. Maybe this can be done via a macro instead?
class IndexSpec : public lcb_N1XSPEC
{
  public:
    IndexSpec(const char *s, size_t n) : lcb_N1XSPEC()
    {
        load_json(s, n);
    }
    inline IndexSpec(const lcb_N1XSPEC *spec);
    static inline void to_key(const lcb_N1XSPEC *spec, std::string &out);
    bool is_primary() const
    {
        return flags & LCB_N1XSPEC_F_PRIMARY;
    }
    bool is_defer() const
    {
        return flags & LCB_N1XSPEC_F_DEFER;
    }
    void ensure_keyspace(lcb_INSTANCE *instance)
    {
        if (nkeyspace) {
            return;
        }
        keyspace = LCBT_SETTING(instance, bucket);
        nkeyspace = strlen(keyspace);
    }

  private:
    // Load fields from a JSON string
    inline void load_json(const char *s, size_t n);

    // Load all fields
    inline size_t load_fields(const Json::Value &root, bool do_copy);

    size_t total_fields_size(const Json::Value &src)
    {
        return load_fields(src, false);
    }

    // Load field from a JSON object
    inline size_t load_json_field(const Json::Value &root, const char *name, const char **tgt_ptr, size_t *tgt_len,
                                  bool do_copy);

    // Load field from another pointer
    void load_field(const char **dest, const char *src, size_t n)
    {
        m_buf.append(src, n);
        if (n) {
            *dest = &m_buf.c_str()[m_buf.size() - n];
        } else {
            *dest = NULL;
        }
    }

    string m_buf;
    IndexSpec(const IndexSpec &);
};

LIBCOUCHBASE_API
lcb_STATUS lcb_n1x_create(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XMGMT *cmd)
{
    string ss;
    IndexSpec spec(&cmd->spec);
    spec.ensure_keyspace(instance);

    ss = "CREATE";
    if (spec.is_primary()) {
        ss += " PRIMARY";
    } else if (!spec.nname) {
        return LCB_EMPTY_KEY;
    }
    ss.append(" INDEX");
    if (spec.nname) {
        ss.append(" `").append(spec.name, spec.nname).append("` ");
    }
    ss.append(" ON `").append(spec.keyspace, spec.nkeyspace).append("`");

    if (!spec.is_primary()) {
        if (!spec.nfields) {
            return LCB_EMPTY_KEY;
        }

        // See if we can parse 'fields' properly. First, try to parse as
        // JSON:
        Json::Value fields_arr;
        Json::Reader r;
        if (!r.parse(spec.fields, spec.fields + spec.nfields, fields_arr)) {
            // Invalid JSON!
            return LCB_EINVAL;
        }

        ss.append(" (");
        if (fields_arr.isArray()) {
            if (!fields_arr.size()) {
                return LCB_EMPTY_KEY;
            }
            for (size_t ii = 0; ii < fields_arr.size(); ++ii) {
                static Json::Value empty;
                const Json::Value &field = fields_arr.get(ii, empty);
                if (!field.isString()) {
                    return LCB_EINVAL;
                }
                ss.append(field.asString());
                if (ii != fields_arr.size() - 1) {
                    ss.append(",");
                }
            }
        } else if (fields_arr.isString()) {
            std::string field_list = fields_arr.asString();
            if (field_list.empty()) {
                return LCB_EMPTY_KEY;
            }
            ss.append(field_list);
        } else {
            return LCB_EINVAL;
        }
        ss.append(") ");
    }

    if (spec.ncond) {
        if (spec.is_primary()) {
            return LCB_EINVAL;
        }
        ss.append(" WHERE ").append(spec.cond, spec.ncond).append(" ");
    }

    if (spec.ixtype) {
        const char *ixtype = ixtype_2_str(spec.ixtype);
        if (!ixtype) {
            return LCB_EINVAL;
        }
        ss.append(" USING ").append(ixtype);
    }

    if (spec.is_defer()) {
        ss.append(" WITH {\"defer_build\": true}");
    }

    return dispatch_common< IndexOpCtx >(instance, cookie, cmd->callback, cb_generic, ss);
}

class ListIndexCtx : public IndexOpCtx
{
  public:
    vector< IndexSpec * > specs;

    virtual void invoke(lcb_INSTANCE *instance, lcb_RESPN1XMGMT *resp)
    {
        finish(instance, resp);
    }

    void finish(lcb_INSTANCE *instance, lcb_RESPN1XMGMT *resp = NULL)
    {
        lcb_RESPN1XMGMT w_resp = {0};
        if (resp == NULL) {
            resp = &w_resp;
            resp->rc = LCB_SUCCESS;
        }
        resp->cookie = cookie;
        lcb_N1XSPEC **speclist = reinterpret_cast< lcb_N1XSPEC ** >(&specs[0]);
        resp->specs = speclist;
        resp->nspecs = specs.size();
        callback(instance, LCB_CALLBACK_IXMGMT, resp);
        delete this;
    }

    virtual ~ListIndexCtx()
    {
        for (size_t ii = 0; ii < specs.size(); ++ii) {
            delete specs[ii];
        }
        specs.clear();
    }
};

static void cb_index_list(lcb_INSTANCE *instance, int, const lcb_RESPN1QL *resp)
{
    ListIndexCtx *ctx = reinterpret_cast< ListIndexCtx * >(resp->cookie);
    if (!(resp->rflags & LCB_RESP_F_FINAL)) {
        ctx->specs.push_back(new IndexSpec(resp->row, resp->nrow));
        return;
    }

    lcb_RESPN1XMGMT w_resp = {0};
    if ((w_resp.rc = resp->rc) == LCB_SUCCESS) {
        w_resp.rc = get_n1ql_error(resp->row, resp->nrow);
    }
    w_resp.inner = resp;
    ctx->invoke(instance, &w_resp);
}

static lcb_STATUS do_index_list(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XMGMT *cmd,
                                ListIndexCtx *ctx)
{
    string ss;
    IndexSpec spec(&cmd->spec);
    ss = "SELECT idx.* FROM system:indexes idx WHERE";

    if (spec.flags & LCB_N1XSPEC_F_PRIMARY) {
        ss.append(" is_primary=true AND");
    }
    if (spec.nkeyspace) {
        ss.append(" keyspace_id=\"").append(spec.keyspace, spec.nkeyspace).append("\" AND");
    }
    if (spec.nnspace) {
        ss.append(" namespace_id=\"").append(spec.nspace, spec.nnspace).append("\" AND");
    }
    if (spec.ixtype) {
        const char *s_ixtype = ixtype_2_str(spec.ixtype);
        if (s_ixtype == NULL) {
            return LCB_EINVAL;
        }
        ss.append(" using=\"").append(s_ixtype).append("\" AND");
    }
    if (spec.nname) {
        ss.append(" name=\"").append(spec.name, spec.nname).append("\" AND");
    }

    // WHERE <.....> true
    ss.append(" true");
    ss.append(" ORDER BY is_primary DESC, name ASC");

    return dispatch_common< ListIndexCtx >(instance, cookie, cmd->callback, cb_index_list, ss, ctx);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_n1x_list(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XMGMT *cmd)
{
    return do_index_list(instance, cookie, cmd, NULL);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_n1x_drop(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XMGMT *cmd)
{
    string ss;
    IndexSpec spec(&cmd->spec);
    spec.ensure_keyspace(instance);

    if (spec.nname) {
        ss = "DROP INDEX";
        ss.append(" `").append(spec.keyspace, spec.nkeyspace).append("`");
        ss.append(".`").append(spec.name, spec.nname).append("`");
    } else if (spec.flags & LCB_N1XSPEC_F_PRIMARY) {
        ss = "DROP PRIMARY INDEX ON";
        ss.append(" `").append(spec.keyspace, spec.nkeyspace).append("`");
    } else {
        return LCB_EMPTY_KEY;
    }

    if (spec.ixtype) {
        const char *stype = ixtype_2_str(spec.ixtype);
        if (!stype) {
            return LCB_EINVAL;
        }
        ss.append(" USING ").append(stype);
    }

    return dispatch_common< IndexOpCtx >(instance, cookie, cmd->callback, cb_generic, ss);
}

class ListIndexCtx_BuildIndex : public ListIndexCtx
{
  public:
    virtual inline void invoke(lcb_INSTANCE *instance, lcb_RESPN1XMGMT *resp);
    inline lcb_STATUS try_build(lcb_INSTANCE *instance);
};

static void cb_build_submitted(lcb_INSTANCE *instance, int, const lcb_RESPN1QL *resp)
{
    ListIndexCtx *ctx = reinterpret_cast< ListIndexCtx * >(resp->cookie);

    if (resp->rflags & LCB_RESP_F_FINAL) {
        lcb_RESPN1XMGMT w_resp = {0};
        if ((w_resp.rc = resp->rc) == LCB_SUCCESS) {
            w_resp.rc = get_n1ql_error(resp->row, resp->nrow);
        }
        ctx->finish(instance, &w_resp);
    }
}

lcb_STATUS ListIndexCtx_BuildIndex::try_build(lcb_INSTANCE *instance)
{
    vector< IndexSpec * > pending;
    for (size_t ii = 0; ii < specs.size(); ++ii) {
        IndexSpec *spec = specs[ii];
        if (strncmp(spec->state, "pending", spec->nstate) == 0 || strncmp(spec->state, "deferred", spec->nstate) == 0) {
            pending.push_back(spec);
        }
    }

    if (pending.empty()) {
        return LCB_KEY_ENOENT;
    }

    string ss;
    ss = "BUILD INDEX ON `";

    ss.append(pending[0]->keyspace, pending[0]->nkeyspace).append("`");
    ss += '(';
    for (size_t ii = 0; ii < pending.size(); ++ii) {
        ss += '`';
        ss.append(pending[ii]->name, pending[ii]->nname);
        ss += '`';
        if (ii + 1 < pending.size()) {
            ss += ',';
        }
    }
    ss += ')';

    lcb_STATUS rc =
        dispatch_common< ListIndexCtx_BuildIndex >(instance, cookie, callback, cb_build_submitted, ss, this);

    if (rc == LCB_SUCCESS) {
        std::set< IndexSpec * > to_remove(specs.begin(), specs.end());
        for (size_t ii = 0; ii < pending.size(); ++ii) {
            to_remove.erase(pending[ii]);
        }

        std::for_each(to_remove.begin(), to_remove.end(), my_delete< IndexSpec * >);

        specs = pending;
    }
    return rc;
}

void ListIndexCtx_BuildIndex::invoke(lcb_INSTANCE *instance, lcb_RESPN1XMGMT *resp)
{
    if (resp->rc == LCB_SUCCESS && (resp->rc = try_build(instance)) == LCB_SUCCESS) {
        return;
    }
    finish(instance, resp);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_n1x_startbuild(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XMGMT *cmd)
{
    ListIndexCtx_BuildIndex *ctx = new ListIndexCtx_BuildIndex();
    lcb_STATUS rc = do_index_list(instance, cookie, cmd, ctx);
    if (rc != LCB_SUCCESS) {
        delete ctx;
    }
    return rc;
}

struct WatchIndexCtx : public IndexOpCtx {
    // Interval timer
    lcbio_pTIMER m_timer;
    uint32_t m_interval;
    uint64_t m_tsend;
    lcb_INSTANCE *m_instance;
    std::map< std::string, IndexSpec * > m_defspend;
    std::vector< IndexSpec * > m_defsok;

    inline void read_state(const lcb_RESPN1XMGMT *resp);
    inline void reschedule();
    inline lcb_STATUS do_poll();
    inline lcb_STATUS load_defs(const lcb_CMDN1XWATCH *);
    inline WatchIndexCtx(lcb_INSTANCE *, const void *, const lcb_CMDN1XWATCH *);
    inline ~WatchIndexCtx();
    inline void finish(lcb_STATUS rc, const lcb_RESPN1XMGMT *);
};

static void cb_watchix_tm(void *arg)
{
    WatchIndexCtx *ctx = reinterpret_cast< WatchIndexCtx * >(arg);
    uint64_t now = lcb_nstime();
    if (now >= ctx->m_tsend) {
        ctx->finish(LCB_ETIMEDOUT, NULL);
    } else {
        ctx->do_poll();
    }
}

#define DEFAULT_WATCH_TIMEOUT LCB_S2US(30)
#define DEFAULT_WATCH_INTERVAL LCB_MS2US(500)

WatchIndexCtx::WatchIndexCtx(lcb_INSTANCE *instance, const void *cookie_, const lcb_CMDN1XWATCH *cmd)
    : m_instance(instance)
{
    uint64_t now = lcb_nstime();
    uint32_t timeout = cmd->timeout ? cmd->timeout : DEFAULT_WATCH_TIMEOUT;
    m_interval = cmd->interval ? cmd->interval : DEFAULT_WATCH_INTERVAL;
    m_interval = std::min(m_interval, timeout);
    m_tsend = now + LCB_US2NS(timeout);

    this->callback = cmd->callback;
    this->cookie = const_cast< void * >(cookie_);

    m_timer = lcbio_timer_new(instance->iotable, this, cb_watchix_tm);
    lcb_aspend_add(&instance->pendops, LCB_PENDTYPE_COUNTER, NULL);
}

WatchIndexCtx::~WatchIndexCtx()
{
    if (m_timer) {
        lcbio_timer_destroy(m_timer);
    }
    if (m_instance) {
        lcb_aspend_del(&m_instance->pendops, LCB_PENDTYPE_COUNTER, NULL);
        lcb_maybe_breakout(m_instance);
    }

    std::for_each(m_defsok.begin(), m_defsok.end(), my_delete< IndexSpec * >);
    for (std::map< string, IndexSpec * >::iterator ii = m_defspend.begin(); ii != m_defspend.end(); ++ii) {
        delete ii->second;
    }
}

void IndexSpec::to_key(const lcb_N1XSPEC *spec, std::string &s)
{
    // Identity is:
    // {keyspace,name,is_primary,type}
    s.append(spec->nspace, spec->nnspace).append(" ");
    s.append(spec->keyspace, spec->nkeyspace).append(" ");
    s.append(spec->name, spec->nname).append(" ");
    const char *type_s = ixtype_2_str(spec->ixtype);
    if (!type_s) {
        type_s = "<UNKNOWN>";
    }
    s.append(type_s);
}

void WatchIndexCtx::read_state(const lcb_RESPN1XMGMT *resp)
{
    // We examine the indexes here to see which ones have been completed
    // Make them all into an std::map
    if (resp->rc != LCB_SUCCESS) {
        lcb_log(LOGARGS(this, INFO), LOGFMT "Error 0x%x while listing indexes. Rescheduling", LOGID(this), resp->rc);
        reschedule();
        return;
    }

    std::map< std::string, const lcb_N1XSPEC * > in_specs;
    for (size_t ii = 0; ii < resp->nspecs; ++ii) {
        std::string key;
        IndexSpec::to_key(resp->specs[ii], key);
        in_specs[key] = resp->specs[ii];
    }

    std::map< std::string, IndexSpec * >::iterator it_remain = m_defspend.begin();
    while (it_remain != m_defspend.end()) {
        // See if the index is 'online' yet!
        std::map< std::string, const lcb_N1XSPEC * >::iterator res;
        res = in_specs.find(it_remain->first);
        if (res == in_specs.end()) {
            lcb_log(LOGARGS(this, INFO), LOGFMT "Index [%s] not in cluster", LOGID(this), it_remain->first.c_str());
            // We can't find our own index. Someone else deleted it. Bail!
            finish(LCB_KEY_ENOENT, resp);
            return;
        }

        std::string s_state(res->second->state, res->second->nstate);
        if (s_state == "online") {
            lcb_log(LOGARGS(this, DEBUG), LOGFMT "Index [%s] is ready", LOGID(this), it_remain->first.c_str());
            m_defsok.push_back(it_remain->second);
            m_defspend.erase(it_remain++);
        } else {
            ++it_remain;
        }
    }

    if (m_defspend.empty()) {
        finish(LCB_SUCCESS, resp);
    } else {
        reschedule();
    }
}

lcb_STATUS WatchIndexCtx::load_defs(const lcb_CMDN1XWATCH *cmd)
{
    for (size_t ii = 0; ii < cmd->nspec; ++ii) {
        std::string key;
        IndexSpec *extspec = new IndexSpec(cmd->specs[ii]);
        IndexSpec::to_key(extspec, key);
        m_defspend[key] = extspec;
    }
    if (m_defspend.empty()) {
        return LCB_ENO_COMMANDS;
    }
    return LCB_SUCCESS;
}

void WatchIndexCtx::finish(lcb_STATUS rc, const lcb_RESPN1XMGMT *resp)
{
    lcb_RESPN1XMGMT my_resp = {0};
    my_resp.cookie = cookie;
    my_resp.rc = rc;

    if (resp) {
        my_resp.inner = resp->inner;
    }

    lcb_N1XSPEC **speclist = reinterpret_cast< lcb_N1XSPEC ** >(&m_defsok[0]);
    my_resp.specs = speclist;
    my_resp.nspecs = m_defsok.size();
    callback(m_instance, LCB_CALLBACK_IXMGMT, &my_resp);
    delete this;
}

void WatchIndexCtx::reschedule()
{
    // Next interval!
    uint64_t now = lcb_nstime();
    if (now + LCB_US2NS(m_interval) >= m_tsend) {
        finish(LCB_ETIMEDOUT, NULL);
    } else {
        lcbio_timer_rearm(m_timer, m_interval);
    }
}

static void cb_watch_gotlist(lcb_INSTANCE *, int, const lcb_RESPN1XMGMT *resp)
{
    WatchIndexCtx *ctx = reinterpret_cast< WatchIndexCtx * >(resp->cookie);
    ctx->read_state(resp);
}

lcb_STATUS WatchIndexCtx::do_poll()
{
    lcb_CMDN1XMGMT cmd;
    memset(&cmd, 0, sizeof cmd);
    cmd.callback = cb_watch_gotlist;
    lcb_log(LOGARGS(this, DEBUG), LOGFMT "Will check for index readiness of %lu indexes. %lu completed", LOGID(this),
            (unsigned long int)m_defspend.size(), (unsigned long int)m_defsok.size());
    return lcb_n1x_list(m_instance, this, &cmd);
}

LIBCOUCHBASE_API
lcb_STATUS lcb_n1x_watchbuild(lcb_INSTANCE *instance, const void *cookie, const lcb_CMDN1XWATCH *cmd)
{
    WatchIndexCtx *ctx = new WatchIndexCtx(instance, cookie, cmd);
    lcb_STATUS rc;
    if ((rc = ctx->load_defs(cmd)) != LCB_SUCCESS) {
        delete ctx;
        return rc;
    }
    if ((rc = ctx->do_poll()) != LCB_SUCCESS) {
        delete ctx;
        return rc;
    }
    return LCB_SUCCESS;
}

void IndexSpec::load_json(const char *s, size_t n)
{
    Json::Value root;
    // Set the JSON first!
    m_buf.assign(s, n);
    nrawjson = n;

    if (!Json::Reader().parse(s, s + n, root)) {
        rawjson = m_buf.c_str();
        return;
    }

    m_buf.reserve(n + total_fields_size(root));
    load_fields(root, true);

    // Once all the fields are loaded, it's time to actually assign the
    // rawjson field, which is simply the beginning of the buffer
    rawjson = m_buf.c_str();

    // Get the index type
    string ixtype_s = root["using"].asString();
    if (ixtype_s == "gsi") {
        ixtype = LCB_N1XSPEC_T_GSI;
    } else if (ixtype_s == "view") {
        ixtype = LCB_N1XSPEC_T_VIEW;
    }
    if (root["is_primary"].asBool()) {
        flags |= LCB_N1XSPEC_F_PRIMARY;
    }
}

// IndexSpec stuff
IndexSpec::IndexSpec(const lcb_N1XSPEC *spec)
{
    *static_cast< lcb_N1XSPEC * >(this) = *spec;
    if (spec->nrawjson) {
        load_json(spec->rawjson, spec->nrawjson);
        return;
    }
    // Initialize the bufs
    m_buf.reserve(nname + nkeyspace + nnspace + nstate + nfields + nrawjson + nstate + ncond);
    load_field(&rawjson, spec->rawjson, nrawjson);
    load_field(&name, spec->name, nname);
    load_field(&keyspace, spec->keyspace, nkeyspace);
    load_field(&nspace, spec->nspace, nnspace);
    load_field(&state, spec->state, nstate);
    load_field(&fields, spec->fields, nfields);
    load_field(&cond, spec->cond, ncond);
}

size_t IndexSpec::load_fields(const Json::Value &root, bool do_copy)
{
    size_t size = 0;
    size += load_json_field(root, "name", &name, &nname, do_copy);
    size += load_json_field(root, "keyspace_id", &keyspace, &nkeyspace, do_copy);
    size += load_json_field(root, "namespace_id", &nspace, &nnspace, do_copy);
    size += load_json_field(root, "state", &state, &nstate, do_copy);
    size += load_json_field(root, "index_key", &fields, &nfields, do_copy);
    size += load_json_field(root, "condition", &cond, &ncond, do_copy);
    return size;
}

size_t IndexSpec::load_json_field(const Json::Value &root, const char *name_, const char **tgt_ptr, size_t *tgt_len,
                                  bool do_copy)
{
    size_t namelen = strlen(name_);
    const Json::Value *val = root.find(name_, name_ + namelen);
    size_t n = 0;

    if (val == NULL) {
        return 0;
    }

    if (val->isString()) {
        const char *s_begin, *s_end;
        if (val->getString(&s_begin, &s_end) && (n = s_end - s_begin) && do_copy) {
            m_buf.insert(m_buf.end(), s_begin, s_end);
        }
    } else {
        std::string frag = Json::FastWriter().write(*val);
        n = frag.size();
        if (do_copy) {
            m_buf.append(frag);
        }
    }

    if (n) {
        *tgt_ptr = &(m_buf.c_str()[m_buf.size() - n]);
        *tgt_len = n;
    } else {
        *tgt_ptr = NULL;
        *tgt_len = 0;
    }
    return n;
}
