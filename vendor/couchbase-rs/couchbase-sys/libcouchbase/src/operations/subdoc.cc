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
#include "internal.h"
#include <vector>
#include <string>

LIBCOUCHBASE_API size_t lcb_respsubdoc_result_size(const lcb_RESPSUBDOC *resp)
{
    return resp->nres;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_result_status(const lcb_RESPSUBDOC *resp, size_t index)
{
    if (index >= resp->nres) {
        return LCB_OPTIONS_CONFLICT;
    }
    return resp->res[index].status;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_result_value(const lcb_RESPSUBDOC *resp, size_t index, const char **value,
                                                        size_t *value_len)
{
    if (index >= resp->nres) {
        return LCB_OPTIONS_CONFLICT;
    }
    *value = (const char *)resp->res[index].value;
    *value_len = resp->res[index].nvalue;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_status(const lcb_RESPSUBDOC *resp)
{
    return resp->rc;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_error_context(const lcb_RESPSUBDOC *resp, const char **ctx, size_t *ctx_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_context(LCB_CALLBACK_GET, (const lcb_RESPBASE *)resp);
    if (val) {
        *ctx = val;
        *ctx_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_error_ref(const lcb_RESPSUBDOC *resp, const char **ref, size_t *ref_len)
{
    if ((resp->rflags & LCB_RESP_F_ERRINFO) == 0) {
        return LCB_KEY_ENOENT;
    }
    const char *val = lcb_resp_get_error_ref(LCB_CALLBACK_GET, (const lcb_RESPBASE *)resp);
    if (val) {
        *ref = val;
        *ref_len = strlen(val);
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_cookie(const lcb_RESPSUBDOC *resp, void **cookie)
{
    *cookie = resp->cookie;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_cas(const lcb_RESPSUBDOC *resp, uint64_t *cas)
{
    *cas = resp->cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_key(const lcb_RESPSUBDOC *resp, const char **key, size_t *key_len)
{
    *key = (const char *)resp->key;
    *key_len = resp->nkey;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_respsubdoc_mutation_token(const lcb_RESPSUBDOC *resp, lcb_MUTATION_TOKEN *token)
{
    const lcb_MUTATION_TOKEN *mt = lcb_resp_get_mutation_token(LCB_CALLBACK_SDMUTATE, (const lcb_RESPBASE *)resp);
    if (token && mt) {
        *token = *mt;
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_create(lcb_SUBDOCOPS **operations, size_t capacity)
{
    lcb_SUBDOCOPS *res = (lcb_SUBDOCOPS *)calloc(1, sizeof(lcb_SUBDOCOPS));
    res->nspecs = capacity;
    res->specs = (lcb_SDSPEC *)calloc(res->nspecs, sizeof(lcb_SDSPEC));
    *operations = res;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_destroy(lcb_SUBDOCOPS *operations)
{
    if (operations) {
        if (operations->specs) {
            size_t ii;
            for (ii = 0; ii < operations->nspecs; ii++) {
                if (operations->specs[ii].sdcmd == LCB_SDCMD_COUNTER) {
                    free((void *)operations->specs[ii].value.u_buf.contig.bytes);
                }
            }
        }
        free(operations->specs);
    }
    free(operations);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_timeout(lcb_CMDSUBDOC *cmd, uint32_t timeout)
{
    cmd->timeout = timeout;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_cas(lcb_CMDSUBDOC *cmd, uint64_t cas)
{
    cmd->cas = cas;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_get(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags, const char *path,
                                              size_t path_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_GET;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_exists(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                 const char *path, size_t path_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_EXISTS;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_replace(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                  const char *path, size_t path_len, const char *value,
                                                  size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_REPLACE;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_dict_add(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                   const char *path, size_t path_len, const char *value,
                                                   size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_DICT_ADD;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_dict_upsert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                      const char *path, size_t path_len, const char *value,
                                                      size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_DICT_UPSERT;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_first(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                          const char *path, size_t path_len, const char *value,
                                                          size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_ARRAY_ADD_FIRST;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_last(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                         const char *path, size_t path_len, const char *value,
                                                         size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_ARRAY_ADD_LAST;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_add_unique(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                           const char *path, size_t path_len, const char *value,
                                                           size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_ARRAY_ADD_UNIQUE;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_array_insert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                       const char *path, size_t path_len, const char *value,
                                                       size_t value_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_ARRAY_INSERT;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_counter(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                  const char *path, size_t path_len, int64_t delta)
{
    operations->specs[index].sdcmd = LCB_SDCMD_COUNTER;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    char *value = (char *)calloc(22, sizeof(char));
    size_t value_len = snprintf(value, 21, "%" PRId64, delta);
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_remove(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                 const char *path, size_t path_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_REMOVE;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_get_count(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                    const char *path, size_t path_len)
{
    operations->specs[index].sdcmd = LCB_SDCMD_GET_COUNT;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_PATH(&operations->specs[index], path, path_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_get(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags)
{
    operations->specs[index].sdcmd = LCB_SDCMD_GET_FULLDOC;
    operations->specs[index].options = flags;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_add(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                      const char *value, size_t value_len)
{
    operations->options |= LCB_CMDSUBDOC_F_INSERT_DOC;
    operations->specs[index].sdcmd = LCB_SDCMD_SET_FULLDOC;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_upsert(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                         const char *value, size_t value_len)
{
    operations->options |= LCB_CMDSUBDOC_F_UPSERT_DOC;
    operations->specs[index].sdcmd = LCB_SDCMD_SET_FULLDOC;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_replace(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags,
                                                          const char *value, size_t value_len)
{
    operations->options &= ~(LCB_CMDSUBDOC_F_INSERT_DOC | LCB_CMDSUBDOC_F_UPSERT_DOC);
    operations->specs[index].sdcmd = LCB_SDCMD_SET_FULLDOC;
    operations->specs[index].options = flags;
    LCB_SDSPEC_SET_VALUE(&operations->specs[index], value, value_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_subdocops_fulldoc_remove(lcb_SUBDOCOPS *operations, size_t index, uint32_t flags)
{
    operations->specs[index].sdcmd = LCB_SDCMD_REMOVE_FULLDOC;
    operations->specs[index].options = flags;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_create(lcb_CMDSUBDOC **cmd)
{
    *cmd = (lcb_CMDSUBDOC *)calloc(1, sizeof(lcb_CMDSUBDOC));
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_clone(const lcb_CMDSUBDOC *cmd, lcb_CMDSUBDOC **copy)
{
    LCB_CMD_CLONE(lcb_CMDSUBDOC, cmd, copy);
    if (cmd->specs) {
        lcb_CMDSUBDOC *ret = *copy;
        ret->specs = (lcb_SDSPEC *)calloc(ret->nspecs, sizeof(lcb_SDSPEC));
        memcpy((void *)ret->specs, cmd->specs, sizeof(lcb_SDSPEC) * ret->nspecs);
        size_t ii;
        for (ii = 0; ii < cmd->nspecs; ii++) {
            lcb_SDSPEC *spec = (lcb_SDSPEC *)&ret->specs[ii];
            size_t npath = spec->path.contig.nbytes;
            if (npath) {
                void *path = calloc(npath, sizeof(char));
                memcpy(path, spec->path.contig.bytes, npath);
                LCB_SDSPEC_SET_PATH(spec, path, npath);
            }
            size_t nval = spec->value.u_buf.contig.nbytes;
            if (nval) {
                void *val = calloc(nval, sizeof(char));
                memcpy(val, spec->value.u_buf.contig.bytes, nval);
                LCB_SDSPEC_SET_VALUE(spec, val, nval);
            }
        }
    }
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_destroy(lcb_CMDSUBDOC *cmd)
{
    if (cmd->cmdflags & LCB_CMD_F_CLONE) {
        if (cmd->specs) {
            size_t ii;
            for (ii = 0; ii < cmd->nspecs; ii++) {
                size_t npath = cmd->specs[ii].path.contig.nbytes;
                if (npath) {
                    free((void *)cmd->specs[ii].path.contig.bytes);
                }
                size_t nvalue = cmd->specs[ii].value.u_buf.contig.nbytes;
                if (nvalue) {
                    free((void *)cmd->specs[ii].value.u_buf.contig.bytes);
                }
            }
            free((void *)cmd->specs);
        }
    }
    LCB_CMD_DESTROY_CLONE(cmd);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_parent_span(lcb_CMDSUBDOC *cmd, lcbtrace_SPAN *span)
{
    cmd->pspan = span;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_collection(lcb_CMDSUBDOC *cmd, const char *scope, size_t scope_len,
                                                     const char *collection, size_t collection_len)
{
    cmd->scope = scope;
    cmd->nscope = scope_len;
    cmd->collection = collection;
    cmd->ncollection = collection_len;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_key(lcb_CMDSUBDOC *cmd, const char *key, size_t key_len)
{
    LCB_CMD_SET_KEY(cmd, key, key_len);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_operations(lcb_CMDSUBDOC *cmd, const lcb_SUBDOCOPS *operations)
{
    cmd->cmdflags |= operations->options;
    cmd->specs = operations->specs;
    cmd->nspecs = operations->nspecs;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_expiration(lcb_CMDSUBDOC *cmd, uint32_t expiration)
{
    cmd->exptime = expiration;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_durability(lcb_CMDSUBDOC *cmd, lcb_DURABILITY_LEVEL level)
{
    cmd->dur_level = level;
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API lcb_STATUS lcb_cmdsubdoc_create_if_missing(lcb_CMDSUBDOC *cmd, int flag)
{
    if (flag) {
        cmd->cmdflags |= LCB_CMDSUBDOC_F_UPSERT_DOC;
    } else {
        cmd->cmdflags &= ~LCB_CMDSUBDOC_F_UPSERT_DOC;
    }
    return LCB_SUCCESS;
}

static lcb_size_t get_value_size(mc_PACKET *packet)
{
    if (packet->flags & MCREQ_F_HASVALUE) {
        if (packet->flags & MCREQ_F_VALUE_IOV) {
            return packet->u_value.multi.total_length;
        } else {
            return packet->u_value.single.size;
        }
    } else {
        return 0;
    }
}

namespace SubdocCmdTraits
{
enum Options {
    EMPTY_PATH = 1 << 0,
    ALLOW_EXPIRY = 1 << 1,
    HAS_VALUE = 1 << 2,
    ALLOW_MKDIRP = 1 << 3,
    IS_LOOKUP = 1 << 4,
    // Must encapsulate in 'multi' spec
    NO_STANDALONE = 1 << 5
};

struct Traits {
    const unsigned allow_empty_path;
    const unsigned allow_expiry;
    const unsigned has_value;
    const unsigned allow_mkdir_p;
    const unsigned is_lookup;
    const uint8_t opcode;

    inline bool valid() const
    {
        return opcode != PROTOCOL_BINARY_CMD_INVALID;
    }

    inline unsigned mode() const
    {
        return is_lookup ? LCB_SDMULTI_MODE_LOOKUP : LCB_SDMULTI_MODE_MUTATE;
    }

    inline bool chk_allow_empty_path(uint32_t options) const
    {
        if (allow_empty_path) {
            return true;
        }
        if (!is_lookup) {
            return false;
        }

        return (options & LCB_SUBDOCOPS_F_XATTRPATH) != 0;
    }

    inline Traits(uint8_t op, unsigned options)
        : allow_empty_path(options & EMPTY_PATH), allow_expiry(options & ALLOW_EXPIRY), has_value(options & HAS_VALUE),
          allow_mkdir_p(options & ALLOW_MKDIRP), is_lookup(options & IS_LOOKUP), opcode(op)
    {
    }
};

static const Traits Get(PROTOCOL_BINARY_CMD_SUBDOC_GET, IS_LOOKUP);

static const Traits Exists(PROTOCOL_BINARY_CMD_SUBDOC_EXISTS, IS_LOOKUP);

static const Traits GetCount(PROTOCOL_BINARY_CMD_SUBDOC_GET_COUNT, IS_LOOKUP | EMPTY_PATH);

static const Traits DictAdd(PROTOCOL_BINARY_CMD_SUBDOC_DICT_ADD, ALLOW_EXPIRY | HAS_VALUE);

static const Traits DictUpsert(PROTOCOL_BINARY_CMD_SUBDOC_DICT_UPSERT, ALLOW_EXPIRY | HAS_VALUE | ALLOW_MKDIRP);

static const Traits Remove(PROTOCOL_BINARY_CMD_SUBDOC_DELETE, ALLOW_EXPIRY);

static const Traits ArrayInsert(PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_INSERT, ALLOW_EXPIRY | HAS_VALUE);

static const Traits Replace(PROTOCOL_BINARY_CMD_SUBDOC_REPLACE, ALLOW_EXPIRY | HAS_VALUE);

static const Traits ArrayAddFirst(PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_FIRST,
                                  ALLOW_EXPIRY | HAS_VALUE | EMPTY_PATH | ALLOW_MKDIRP);

static const Traits ArrayAddLast(PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_LAST,
                                 ALLOW_EXPIRY | HAS_VALUE | EMPTY_PATH | ALLOW_MKDIRP);

static const Traits ArrayAddUnique(PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_ADD_UNIQUE,
                                   ALLOW_EXPIRY | HAS_VALUE | EMPTY_PATH | ALLOW_MKDIRP);

static const Traits Counter(PROTOCOL_BINARY_CMD_SUBDOC_COUNTER, ALLOW_EXPIRY | HAS_VALUE | ALLOW_MKDIRP);

static const Traits GetDoc(PROTOCOL_BINARY_CMD_GET, IS_LOOKUP | EMPTY_PATH | NO_STANDALONE);

static const Traits SetDoc(PROTOCOL_BINARY_CMD_SET, EMPTY_PATH | NO_STANDALONE);

static const Traits DeleteDoc(PROTOCOL_BINARY_CMD_DELETE, EMPTY_PATH | NO_STANDALONE);

static const Traits Invalid(PROTOCOL_BINARY_CMD_INVALID, 0);

const Traits &find(unsigned mode)
{
    switch (mode) {
        case LCB_SDCMD_REPLACE:
            return Replace;
        case LCB_SDCMD_DICT_ADD:
            return DictAdd;
        case LCB_SDCMD_DICT_UPSERT:
            return DictUpsert;
        case LCB_SDCMD_ARRAY_ADD_FIRST:
            return ArrayAddFirst;
        case LCB_SDCMD_ARRAY_ADD_LAST:
            return ArrayAddLast;
        case LCB_SDCMD_ARRAY_ADD_UNIQUE:
            return ArrayAddUnique;
        case LCB_SDCMD_ARRAY_INSERT:
            return ArrayInsert;
        case LCB_SDCMD_GET:
            return Get;
        case LCB_SDCMD_EXISTS:
            return Exists;
        case LCB_SDCMD_GET_COUNT:
            return GetCount;
        case LCB_SDCMD_REMOVE:
            return Remove;
        case LCB_SDCMD_COUNTER:
            return Counter;
        case LCB_SDCMD_GET_FULLDOC:
            return GetDoc;
        case LCB_SDCMD_SET_FULLDOC:
            return SetDoc;
        case LCB_SDCMD_REMOVE_FULLDOC:
            return DeleteDoc;
        default:
            return Invalid;
    }
}
} // namespace SubdocCmdTraits

namespace SubdocPathFlags
{
static const uint8_t MKDIR_P = 0x01;
static const uint8_t XATTR = 0x04;
static const uint8_t EXPAND_MACROS = 0x010;
} // namespace SubdocPathFlags

namespace SubdocDocFlags
{
static const uint8_t MKDOC = 0x01;
static const uint8_t ADDDOC = 0x02;
static const uint8_t ACCESS_DELETED = 0x04;
} // namespace SubdocDocFlags

static size_t get_valbuf_size(const lcb_VALBUF &vb)
{
    if (vb.vtype == LCB_KV_COPY || vb.vtype == LCB_KV_CONTIG) {
        return vb.u_buf.contig.nbytes;
    } else {
        if (vb.u_buf.multi.total_length) {
            return vb.u_buf.multi.total_length;
        } else {
            size_t tmp = 0;
            for (size_t ii = 0; ii < vb.u_buf.multi.niov; ++ii) {
                tmp += vb.u_buf.multi.iov[ii].iov_len;
            }
            return tmp;
        }
    }
}

static uint8_t make_path_flags(const uint32_t user)
{
    uint8_t flags = 0;
    if (user & LCB_SUBDOCOPS_F_MKINTERMEDIATES) {
        flags |= SubdocPathFlags::MKDIR_P;
    }
    if (user & LCB_SUBDOCOPS_F_XATTRPATH) {
        flags |= SubdocPathFlags::XATTR;
    }
    if (user & LCB_SUBDOCOPS_F_XATTR_MACROVALUES) {
        flags |= SubdocPathFlags::XATTR | SubdocPathFlags::EXPAND_MACROS;
    }
    return flags;
}

static uint8_t make_doc_flags(const uint32_t user)
{
    uint8_t flags = 0;
    if (user & LCB_CMDSUBDOC_F_INSERT_DOC) {
        flags |= SubdocDocFlags::ADDDOC;
    }
    if (user & LCB_CMDSUBDOC_F_UPSERT_DOC) {
        flags |= SubdocDocFlags::MKDOC;
    }
    if (user & LCB_CMDSUBDOC_F_ACCESS_DELETED) {
        flags |= SubdocDocFlags::ACCESS_DELETED;
    }
    return flags;
}

struct MultiBuilder {
    static unsigned infer_mode(const lcb_CMDSUBDOC *cmd)
    {
        if (cmd->nspecs == 0) {
            return 0;
        }
        const SubdocCmdTraits::Traits &trait = SubdocCmdTraits::find(cmd->specs[0].sdcmd);
        if (!trait.valid()) {
            return 0;
        }
        return trait.mode();
    }

    MultiBuilder(const lcb_CMDSUBDOC *cmd_) : cmd(cmd_), payload_size(0)
    {
        mode = infer_mode(cmd_);
        size_t ebufsz = is_lookup() ? cmd->nspecs * 4 : cmd->nspecs * 8;
        extra_body = new char[ebufsz];
        bodysz = 0;
    }

    ~MultiBuilder()
    {
        if (extra_body != NULL) {
            delete[] extra_body;
        }
    }

    inline MultiBuilder(const MultiBuilder &);

    // IOVs which are fed into lcb_VALBUF for subsequent use
    const lcb_CMDSUBDOC *cmd;
    std::vector< lcb_IOV > iovs;
    char *extra_body;
    size_t bodysz;

    // Total size of the payload itself
    size_t payload_size;

    unsigned mode;

    bool is_lookup() const
    {
        return mode == LCB_SDMULTI_MODE_LOOKUP;
    }

    bool is_mutate() const
    {
        return mode == LCB_SDMULTI_MODE_MUTATE;
    }

    void maybe_setmode(const SubdocCmdTraits::Traits &t)
    {
        if (mode == 0) {
            mode = t.mode();
        }
    }

    template < typename T > void add_field(T itm, size_t len)
    {
        const char *b = reinterpret_cast< const char * >(&itm);
        memcpy(extra_body + bodysz, b, len);
        bodysz += len;
    }

    const char *extra_mark() const
    {
        return extra_body + bodysz;
    }

    void add_extras_iov(const char *last_begin)
    {
        const char *p_end = extra_mark();
        add_iov(last_begin, p_end - last_begin);
    }

    void add_iov(const void *b, size_t n)
    {
        if (!n) {
            return;
        }

        lcb_IOV iov;
        iov.iov_base = const_cast< void * >(b);
        iov.iov_len = n;
        iovs.push_back(iov);
        payload_size += n;
    }

    void add_iov(const lcb_VALBUF &vb)
    {
        if (vb.vtype == LCB_KV_CONTIG || vb.vtype == LCB_KV_COPY) {
            add_iov(vb.u_buf.contig.bytes, vb.u_buf.contig.nbytes);
        } else {
            for (size_t ii = 0; ii < vb.u_buf.contig.nbytes; ++ii) {
                const lcb_IOV &iov = vb.u_buf.multi.iov[ii];
                if (!iov.iov_len) {
                    continue; // Skip it
                }
                payload_size += iov.iov_len;
                iovs.push_back(iov);
            }
        }
    }

    inline lcb_STATUS add_spec(const lcb_SDSPEC *);
};

lcb_STATUS MultiBuilder::add_spec(const lcb_SDSPEC *spec)
{
    const SubdocCmdTraits::Traits &trait = SubdocCmdTraits::find(spec->sdcmd);
    if (!trait.valid()) {
        return LCB_UNKNOWN_SDCMD;
    }
    maybe_setmode(trait);

    if (trait.mode() != mode) {
        return LCB_OPTIONS_CONFLICT;
    }

    const char *p_begin = extra_mark();
    // opcode
    add_field(trait.opcode, 1);
    // flags
    add_field(make_path_flags(spec->options), 1);

    uint16_t npath = static_cast< uint16_t >(spec->path.contig.nbytes);
    if (!npath && !trait.chk_allow_empty_path(spec->options)) {
        return LCB_EMPTY_PATH;
    }

    // Path length
    add_field(static_cast< uint16_t >(htons(npath)), 2);

    uint32_t vsize = 0;
    if (is_mutate()) {
        // Mutation needs an additional 'value' spec.
        vsize = get_valbuf_size(spec->value);
        add_field(static_cast< uint32_t >(htonl(vsize)), 4);
    }

    // Finalize the header..
    add_extras_iov(p_begin);

    // Add the actual path
    add_iov(spec->path.contig.bytes, spec->path.contig.nbytes);
    if (vsize) {
        add_iov(spec->value);
    }
    return LCB_SUCCESS;
}

static lcb_STATUS sd3_single(lcb_INSTANCE *instance, void *cookie, const lcb_CMDSUBDOC *cmd)
{
    // Find the trait
    const lcb_SDSPEC *spec = cmd->specs;
    const SubdocCmdTraits::Traits &traits = SubdocCmdTraits::find(spec->sdcmd);
    lcb_STATUS rc;

    // Any error here is implicitly related to the only spec
    if (cmd->error_index) {
        *cmd->error_index = 0;
    }

    if (!traits.valid()) {
        return LCB_UNKNOWN_SDCMD;
    }

    // Determine if the trait matches the mode. Technically we don't care
    // about this (since it's always a single command) but we do want the
    // API to remain consistent.
    if (cmd->multimode != 0 && cmd->multimode != traits.mode()) {
        return LCB_OPTIONS_CONFLICT;
    }

    if (LCB_KEYBUF_IS_EMPTY(&cmd->key)) {
        return LCB_EMPTY_KEY;
    }
    if (LCB_KEYBUF_IS_EMPTY(&spec->path) && !traits.chk_allow_empty_path(spec->options)) {
        return LCB_EMPTY_PATH;
    }

    lcb_VALBUF valbuf;
    const lcb_VALBUF *valbuf_p = &valbuf;
    lcb_IOV tmpiov[2];
    lcb_FRAGBUF *fbuf = &valbuf.u_buf.multi;

    valbuf.vtype = LCB_KV_IOVCOPY;
    fbuf->iov = tmpiov;
    fbuf->niov = 1;
    fbuf->total_length = 0;
    tmpiov[0].iov_base = const_cast< void * >(spec->path.contig.bytes);
    tmpiov[0].iov_len = spec->path.contig.nbytes;

    if (traits.has_value) {
        if (spec->value.vtype == LCB_KV_COPY) {
            fbuf->niov = 2;
            /* Subdoc value is the second IOV */
            tmpiov[1].iov_base = (void *)spec->value.u_buf.contig.bytes;
            tmpiov[1].iov_len = spec->value.u_buf.contig.nbytes;
        } else {
            /* Assume properly formatted packet */
            valbuf_p = &spec->value;
        }
    }

    uint8_t extlen = 3;
    uint32_t exptime = 0;
    if (cmd->exptime) {
        if (!traits.allow_expiry) {
            return LCB_OPTIONS_CONFLICT;
        }
        exptime = cmd->exptime;
        extlen = 7;
    }

    uint8_t docflags = make_doc_flags(cmd->cmdflags);
    if (docflags) {
        extlen++;
    }

    protocol_binary_request_header hdr = {{0}};
    mc_PACKET *packet;
    mc_PIPELINE *pipeline;
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);
    lcb_U8 ffextlen = 0;

    if (cmd->dur_level) {
        if (new_durability_supported) {
            hdr.request.magic = PROTOCOL_BINARY_AREQ;
            ffextlen = 4;
        } else {
            return LCB_NOT_SUPPORTED;
        }
    }
    rc = mcreq_basic_packet(&instance->cmdq, (const lcb_CMDBASE *)cmd, &hdr, extlen, ffextlen, &packet, &pipeline,
                            MCREQ_BASICPACKET_F_FALLBACKOK);

    if (rc != LCB_SUCCESS) {
        return rc;
    }

    rc = mcreq_reserve_value(pipeline, packet, valbuf_p);
    if (rc != LCB_SUCCESS) {
        mcreq_wipe_packet(pipeline, packet);
        mcreq_release_packet(pipeline, packet);
        return rc;
    }

    MCREQ_PKT_RDATA(packet)->cookie = cookie;
    MCREQ_PKT_RDATA(packet)->start = gethrtime();
    MCREQ_PKT_RDATA(packet)->deadline = MCREQ_PKT_RDATA(packet)->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));

    hdr.request.magic = PROTOCOL_BINARY_REQ;
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.extlen = packet->extlen;
    hdr.request.opaque = packet->opaque;
    hdr.request.opcode = traits.opcode;
    hdr.request.cas = lcb_htonll(cmd->cas);
    hdr.request.bodylen = htonl(hdr.request.extlen + ffextlen + ntohs(hdr.request.keylen) + get_value_size(packet));

    memcpy(SPAN_BUFFER(&packet->kh_span), hdr.bytes, sizeof hdr.bytes);
    if (cmd->dur_level && new_durability_supported) {
        uint8_t meta = (1 << 4) | 3;
        uint8_t level = cmd->dur_level;
        uint16_t timeout = 0;
        memcpy(SPAN_BUFFER(&packet->kh_span) + MCREQ_PKT_BASESIZE, &meta, sizeof(meta));
        memcpy(SPAN_BUFFER(&packet->kh_span) + MCREQ_PKT_BASESIZE + 1, &level, sizeof(level));
        memcpy(SPAN_BUFFER(&packet->kh_span) + MCREQ_PKT_BASESIZE + 2, &timeout, sizeof(timeout));
    }
    char *extras = SPAN_BUFFER(&packet->kh_span) + MCREQ_PKT_BASESIZE + ffextlen;
    // Path length:
    uint16_t enc_pathlen = htons(spec->path.contig.nbytes);
    memcpy(extras, &enc_pathlen, 2);
    extras += 2;

    uint8_t path_flags = make_path_flags(spec->options);
    memcpy(extras, &path_flags, 1);
    extras += 1;

    if (exptime) {
        uint32_t enc_exptime = htonl(exptime);
        memcpy(extras, &enc_exptime, 4);
        extras += 4;
    }

    if (docflags) {
        memcpy(extras, &docflags, 1);
        extras += 1;
    }

    LCB_SCHED_ADD(instance, pipeline, packet);
    return LCB_SUCCESS;
}

static lcb_STATUS subdoc_validate(lcb_INSTANCE *, const lcb_CMDSUBDOC *cmd)
{
    // First validate the command
    if (cmd->nspecs == 0) {
        return LCB_ENO_COMMANDS;
    }

    return LCB_SUCCESS;
}

static lcb_STATUS subdoc_impl(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg)
{
    const lcb_CMDSUBDOC *cmd = (const lcb_CMDSUBDOC *)arg;
    if (LCBT_SETTING(instance, use_collections)) {
        lcb_CMDSUBDOC *mut = const_cast< lcb_CMDSUBDOC * >(cmd);
        mut->cid = cid;
    }

    if (cmd->nspecs == 1) {
        switch (cmd->specs[0].sdcmd) {
            case LCB_SDCMD_GET_FULLDOC:
            case LCB_SDCMD_SET_FULLDOC:
            case LCB_SDCMD_REMOVE_FULLDOC:
                break;
            default:
                return sd3_single(instance, cookie, cmd);
        }
    }

    uint32_t expiry = cmd->exptime;
    uint8_t docflags = make_doc_flags(cmd->cmdflags);
    lcb_STATUS rc = LCB_SUCCESS;

    MultiBuilder ctx(cmd);
    if (cmd->error_index) {
        *cmd->error_index = -1;
    }

    if (expiry && !ctx.is_mutate()) {
        return LCB_OPTIONS_CONFLICT;
    }

    for (size_t ii = 0; ii < cmd->nspecs; ++ii) {
        if (cmd->error_index) {
            *cmd->error_index = ii;
        }
        rc = ctx.add_spec(cmd->specs + ii);
        if (rc != LCB_SUCCESS) {
            return rc;
        }
    }

    mc_PIPELINE *pl;
    mc_PACKET *pkt;
    uint8_t extlen = 0;
    if (expiry) {
        extlen += 4;
    }
    if (docflags) {
        extlen++;
    }

    protocol_binary_request_header hdr;
    int new_durability_supported = LCBT_SUPPORT_SYNCREPLICATION(instance);
    lcb_U8 ffextlen = 0;

    hdr.request.magic = PROTOCOL_BINARY_REQ;
    if (cmd->dur_level) {
        if (new_durability_supported) {
            hdr.request.magic = PROTOCOL_BINARY_AREQ;
            ffextlen = 4;
        } else {
            return LCB_NOT_SUPPORTED;
        }
    }

    if (cmd->error_index) {
        *cmd->error_index = -1;
    }

    rc = mcreq_basic_packet(&instance->cmdq, reinterpret_cast< const lcb_CMDBASE * >(cmd), &hdr, extlen, ffextlen, &pkt,
                            &pl, MCREQ_BASICPACKET_F_FALLBACKOK);

    if (rc != LCB_SUCCESS) {
        return rc;
    }

    lcb_VALBUF vb = {LCB_KV_IOVCOPY};
    vb.u_buf.multi.iov = &ctx.iovs[0];
    vb.u_buf.multi.niov = ctx.iovs.size();
    vb.u_buf.multi.total_length = ctx.payload_size;
    rc = mcreq_reserve_value(pl, pkt, &vb);

    if (rc != LCB_SUCCESS) {
        mcreq_wipe_packet(pl, pkt);
        mcreq_release_packet(pl, pkt);
        return rc;
    }

    // Set the header fields.
    if (ctx.is_lookup()) {
        hdr.request.opcode = PROTOCOL_BINARY_CMD_SUBDOC_MULTI_LOOKUP;
    } else {
        hdr.request.opcode = PROTOCOL_BINARY_CMD_SUBDOC_MULTI_MUTATION;
    }
    hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
    hdr.request.extlen = pkt->extlen;
    hdr.request.opaque = pkt->opaque;
    hdr.request.cas = lcb_htonll(cmd->cas);
    hdr.request.bodylen = htonl(hdr.request.extlen + ffextlen + ntohs(hdr.request.keylen) + ctx.payload_size);
    memcpy(SPAN_BUFFER(&pkt->kh_span), hdr.bytes, sizeof(hdr.bytes));
    if (cmd->dur_level && new_durability_supported) {
        uint8_t meta = (1 << 4) | 3;
        uint8_t level = cmd->dur_level;
        uint16_t timeout = 0;
        memcpy(SPAN_BUFFER(&pkt->kh_span) + MCREQ_PKT_BASESIZE, &meta, sizeof(meta));
        memcpy(SPAN_BUFFER(&pkt->kh_span) + MCREQ_PKT_BASESIZE + 1, &level, sizeof(level));
        memcpy(SPAN_BUFFER(&pkt->kh_span) + MCREQ_PKT_BASESIZE + 2, &timeout, sizeof(timeout));
    }
    if (expiry) {
        expiry = htonl(expiry);
        memcpy(SPAN_BUFFER(&pkt->kh_span) + MCREQ_PKT_BASESIZE + ffextlen, &expiry, 4);
    }
    if (docflags) {
        memcpy(SPAN_BUFFER(&pkt->kh_span) + MCREQ_PKT_BASESIZE + ffextlen + (extlen - 1), &docflags, 1);
    }

    MCREQ_PKT_RDATA(pkt)->cookie = cookie;
    MCREQ_PKT_RDATA(pkt)->start = gethrtime();
    MCREQ_PKT_RDATA(pkt)->deadline = MCREQ_PKT_RDATA(pkt)->start + LCB_US2NS(cmd->timeout ? cmd->timeout : LCBT_SETTING(instance, operation_timeout));
    LCB_SCHED_ADD(instance, pl, pkt);
    return LCB_SUCCESS;
}

LIBCOUCHBASE_API
lcb_STATUS lcb_subdoc(lcb_INSTANCE *instance, void *cookie, const lcb_CMDSUBDOC *cmd)
{
    lcb_STATUS err;

    err = subdoc_validate(instance, cmd);
    if (err != LCB_SUCCESS) {
        return err;
    }

    return collcache_exec(cmd->scope, cmd->nscope, cmd->collection, cmd->ncollection, instance, cookie, subdoc_impl,
                          (lcb_COLLCACHE_ARG_CLONE)lcb_cmdsubdoc_clone, (lcb_COLLCACHE_ARG_DTOR)lcb_cmdsubdoc_destroy,
                          cmd);
}
