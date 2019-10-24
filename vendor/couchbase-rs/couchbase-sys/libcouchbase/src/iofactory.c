/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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

#define LCB_IOPS_V12_NO_DEPRECATE 1 /* For Ruby */

#include "internal.h"
#include "plugins/io/select/select_io_opts.h"
#include <libcouchbase/plugins/io/bsdio-inl.c>

#ifdef LCB_EMBED_PLUGIN_LIBEVENT
LIBCOUCHBASE_API lcb_STATUS lcb_create_libevent_io_opts(int, lcb_io_opt_t *, void *);
#endif

typedef lcb_STATUS (*create_func_t)(int version, lcb_io_opt_t *io, void *cookie);

#ifdef _WIN32
LIBCOUCHBASE_API
lcb_STATUS lcb_iocp_new_iops(int, lcb_io_opt_t *, void *);
#define DEFAULT_IOPS LCB_IO_OPS_WINIOCP
#else
#define DEFAULT_IOPS LCB_IO_OPS_LIBEVENT
#endif

typedef struct {
    /** The "base" name of the plugin */
    const char *base;

    /** Corresponding type */
    lcb_io_ops_type_t iotype;

    /** Filename */
    const char *soname;

    /** Symbol used to initialize the plugin */
    const char *symbol;

    /** Function to create the iops (if builtin) */
    create_func_t create;

    /** Static buffers if reading from the environment */
    char s_soname[PATH_MAX];
    char s_symbol[256];
} plugin_info;

#ifdef __APPLE__
#define PLUGIN_SO(NAME) "libcouchbase_" NAME ".dylib"
#elif defined(_WIN32)
/** Trailing period intentional. See docs for LoadLibrary */
#if (_DEBUG && _MSC_VER)
#define PLUGIN_SO(NAME) "libcouchbase_" NAME "_d.dll."
#else
#define PLUGIN_SO(NAME) "libcouchbase_" NAME ".dll."
#endif /* _DEBUG */
#else
#define PLUGIN_SO(NAME) "libcouchbase_" NAME ".so"
#endif

#define PLUGIN_SYMBOL(NAME) "lcb_create_" NAME "_io_opts"

#define BUILTIN_CORE(name, type, create)                                                                               \
    {                                                                                                                  \
        name, type, NULL, NULL, create, {0},                                                                           \
        {                                                                                                              \
            0                                                                                                          \
        }                                                                                                              \
    }

#define BUILTIN_DL(name, type)                                                                                         \
    {                                                                                                                  \
        name, type, PLUGIN_SO(name), PLUGIN_SYMBOL(name), NULL, {0},                                                   \
        {                                                                                                              \
            0                                                                                                          \
        }                                                                                                              \
    }

static plugin_info builtin_plugins[] = {BUILTIN_CORE("select", LCB_IO_OPS_SELECT, lcb_create_select_io_opts),
                                        BUILTIN_CORE("winsock", LCB_IO_OPS_WINSOCK, lcb_create_select_io_opts),

#ifdef _WIN32
                                        BUILTIN_CORE("iocp", LCB_IO_OPS_WINIOCP, lcb_iocp_new_iops),
#endif

#ifdef LCB_EMBED_PLUGIN_LIBEVENT
                                        BUILTIN_CORE("libevent", LCB_IO_OPS_LIBEVENT, lcb_create_libevent_io_opts),
#else
                                        BUILTIN_DL("libevent", LCB_IO_OPS_LIBEVENT),
#endif

                                        BUILTIN_DL("libev", LCB_IO_OPS_LIBEV),
                                        BUILTIN_DL("libuv", LCB_IO_OPS_LIBUV),

                                        {NULL, LCB_IO_OPS_INVALID, NULL, NULL, NULL, {0}, {0}}};

/**
 * Checks the environment for plugin information.
 * Returns:
 *   1  information found and valid
 *   0  not found
 *   -1 error
 */
static int get_env_plugin_info(plugin_info *info)
{

    plugin_info *cur = NULL;
    memset(info, 0, sizeof(*info));

    if (!lcb_getenv_nonempty_multi(info->s_soname, sizeof(info->s_soname), "LIBCOUCHBASE_EVENT_PLUGIN_NAME",
                                   "LCB_IOPS_NAME", NULL)) {
        return 0;
    }

    for (cur = builtin_plugins; cur->base; cur++) {
        if (strlen(cur->base) != strlen(info->s_soname)) {
            continue;
        }

        if (strcmp(cur->base, info->s_soname) == 0) {
            memcpy(info, cur, sizeof(*cur));
            return 1;
        }
    }

    if (!lcb_getenv_nonempty_multi(info->s_symbol, sizeof(info->s_symbol), "LIBCOUCHBASE_EVENT_PLUGIN_SYMBOL",
                                   "LCB_IOPS_SYMBOL", NULL)) {
        return -1;
    }

    info->soname = info->s_soname;
    info->symbol = info->s_symbol;
    return 1;
}

static plugin_info *find_plugin_info(lcb_io_ops_type_t iotype)
{
    plugin_info *cur;

    if (iotype == LCB_IO_OPS_DEFAULT) {
        iotype = DEFAULT_IOPS;
    }

    for (cur = builtin_plugins; cur->base; cur++) {
        if (cur->iotype == iotype) {
            return cur;
        }
    }
    return NULL;
}

static void options_from_info(struct lcb_create_io_ops_st *opts, const plugin_info *info)
{
    void *cookie;

    switch (opts->version) {
        case 0:
            cookie = opts->v.v0.cookie;
            break;
        case 1:
            cookie = opts->v.v1.cookie;
            break;
        case 2:
            cookie = opts->v.v2.cookie;
            break;
        default:
            lcb_assert("unknown options version" && 0);
            cookie = NULL;
    }

    if (info->create) {
        opts->version = 2;
        opts->v.v2.create = info->create;
        opts->v.v2.cookie = cookie;
        return;
    }

    opts->version = 1;
    opts->v.v1.sofile = info->soname;
    opts->v.v1.symbol = info->symbol;
    opts->v.v1.cookie = cookie;
}

static lcb_STATUS create_v2(lcb_io_opt_t *io, const struct lcb_create_io_ops_st *options);

struct plugin_st {
    void *dlhandle;
    union {
        create_func_t create;
        void *voidptr;
    } func;
};

#ifndef _WIN32
static lcb_STATUS get_create_func(const char *image, const char *symbol, struct plugin_st *plugin, int do_warn)
{
    void *dlhandle = dlopen(image, RTLD_NOW | RTLD_LOCAL);
    if (dlhandle == NULL) {
        if (do_warn) {
            fprintf(stderr, "[libcouchbase] dlopen of %s failed with '%s'\n", image, dlerror());
        }
        return LCB_DLOPEN_FAILED;
    }

    memset(plugin, 0, sizeof(*plugin));
    plugin->func.create = NULL;
    plugin->func.voidptr = dlsym(dlhandle, symbol);

    if (plugin->func.voidptr == NULL) {
        if (do_warn) {
            fprintf(stderr, "[libcouchbase] dlsym (%s) -> (%s) failed: %s\n", image, symbol, dlerror());
        }
        dlclose(dlhandle);
        dlhandle = NULL;
        return LCB_DLSYM_FAILED;

    } else {
        plugin->dlhandle = dlhandle;
    }
    return LCB_SUCCESS;
}

static void close_dlhandle(void *handle)
{
    dlclose(handle);
}
#else
static lcb_STATUS get_create_func(const char *image, const char *symbol, struct plugin_st *plugin, int do_warn)
{
    HMODULE hLibrary = LoadLibrary(image);
    FARPROC hFunction;

    memset(plugin, 0, sizeof(*plugin));

    if (!hLibrary) {
        if (do_warn) {
            fprintf(stderr, "LoadLibrary of %s failed with code %d\n", image, (int)GetLastError());
        }
        return LCB_DLOPEN_FAILED;
    }

    hFunction = GetProcAddress(hLibrary, symbol);
    if (!hFunction) {
        if (do_warn) {
            fprintf(stderr, "GetProcAddress (%s) -> (%s) failed with code %d\n", image, symbol, (int)GetLastError());
        }
        FreeLibrary(hLibrary);
        return LCB_DLSYM_FAILED;
    }

    plugin->func.create = (create_func_t)hFunction;
    plugin->dlhandle = hLibrary;
    return LCB_SUCCESS;
}

static void close_dlhandle(void *handle)
{
    FreeLibrary((HMODULE)handle);
}
#endif

static int want_dl_debug = 0; /* global variable */
static lcb_STATUS create_v1(lcb_io_opt_t *io, const struct lcb_create_io_ops_st *options);

LIBCOUCHBASE_API
lcb_STATUS lcb_destroy_io_ops(lcb_io_opt_t io)
{
    if (io) {
        void *dlhandle = io->dlhandle;
        if (io->destructor) {
            io->destructor(io);
        }
        if (dlhandle) {
            close_dlhandle(dlhandle);
        }
    }

    return LCB_SUCCESS;
}

/**
 * Note, the 'pi' is just a context variable to ensure the pointers copied
 * to the options are valid. It is *not* meant to be inspected.
 */
static lcb_STATUS generate_options(plugin_info *pi, const struct lcb_create_io_ops_st *user,
                                   struct lcb_create_io_ops_st *ours, lcb_io_ops_type_t *type)
{
    if (user) {
        memcpy(ours, user, sizeof(*user));

    } else {
        memset(ours, 0, sizeof(*ours));
        ours->version = 0;
        ours->v.v0.type = LCB_IO_OPS_DEFAULT;
    }

    if (ours->version > 0) {
        if (type) {
            *type = LCB_IO_OPS_INVALID;
        }
        /* we don't handle non-v0 options */
        return LCB_SUCCESS;
    }

    if (ours->v.v0.type == LCB_IO_OPS_DEFAULT) {
        int rv;
        memset(pi, 0, sizeof(*pi));

        rv = get_env_plugin_info(pi);
        if (rv > 0) {
            options_from_info(ours, pi);

            if (type) {
                *type = pi->iotype;
            }

        } else if (rv < 0) {
            return LCB_BAD_ENVIRONMENT;

        } else {
            plugin_info *pip = find_plugin_info(LCB_IO_OPS_DEFAULT);
            lcb_assert(pip);

            if (type) {
                *type = pip->iotype;
            }

            options_from_info(ours, pip);

            /* if the plugin is dynamically loadable, we need to
             * fallback to select(2) plugin in case we cannot find the
             * create function */
            if (ours->version == 1) {
                struct plugin_st plugin;
                int want_debug;
                lcb_STATUS ret;

                if (lcb_getenv_boolean_multi("LIBCOUCHBASE_DLOPEN_DEBUG", "LCB_DLOPEN_DEBUG", NULL)) {
                    want_debug = 1;
                } else {
                    want_debug = want_dl_debug;
                }
                ret = get_create_func(ours->v.v1.sofile, ours->v.v1.symbol, &plugin, want_debug);
                if (ret != LCB_SUCCESS) {
                    char path[PATH_MAX];
                    /* try to look up the so-file in the libdir */
                    snprintf(path, PATH_MAX, "%s/%s", LCB_LIBDIR, ours->v.v1.sofile);
                    ret = get_create_func(path, ours->v.v1.symbol, &plugin, want_debug);
                }
                if (ret != LCB_SUCCESS) {
                    if (type) {
                        *type = LCB_IO_OPS_SELECT;
                    }
                    ours->version = 2;
                    ours->v.v2.create = lcb_create_select_io_opts;
                    ours->v.v2.cookie = NULL;
                }
            }
        }
        return LCB_SUCCESS;

    } else {
        /** Not default, ignore environment */
        plugin_info *pip = find_plugin_info(ours->v.v0.type);
        if (!pip) {
            return LCB_NOT_SUPPORTED;
        }
        options_from_info(ours, pip);
        if (type) {
            *type = pip->iotype;
        }
        return LCB_SUCCESS;
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_create_io_ops(lcb_io_opt_t *io, const struct lcb_create_io_ops_st *io_opts)
{

    struct lcb_create_io_ops_st options;
    lcb_STATUS err;
    plugin_info pi;
    memset(&options, 0, sizeof(options));

    err = lcb_initialize_socket_subsystem();
    if (err != LCB_SUCCESS) {
        return err;
    }

    err = generate_options(&pi, io_opts, &options, NULL);
    if (err != LCB_SUCCESS) {
        return err;
    }

    if (options.version == 1) {
        err = create_v1(io, &options);
    } else if (options.version == 2) {
        err = create_v2(io, &options);
    } else {
        return LCB_NOT_SUPPORTED;
    }

    if (err != LCB_SUCCESS) {
        return err;
    }
    /*XXX:
     * This block of code here because the Ruby SDK relies on undocumented
     * functionality of older versions of libcouchbase in which its send/recv
     * functions assert that the number of IOV elements passed is always going
     * to be 2.
     *
     * This works around the issue by patching the send/recv functions of
     * the ruby implementation at load-time.
     *
     * This block of code will go away once the Ruby SDK is fixed and a released
     * version has been out for enough time that it won't break common existing
     * deployments.
     */
    if (io_opts && io_opts->version == 1 && io_opts->v.v1.symbol != NULL) {
        if (strstr(io_opts->v.v1.symbol, "cb_create_ruby")) {
            wire_lcb_bsd_impl(*io);
        }
    }
    return LCB_SUCCESS;
}

static lcb_STATUS create_v1(lcb_io_opt_t *io, const struct lcb_create_io_ops_st *options)
{
    struct plugin_st plugin;
    int want_debug;
    lcb_STATUS ret;

    if (lcb_getenv_boolean_multi("LIBCOUCHBASE_DLOPEN_DEBUG", "LCB_DLOPEN_DEBUG", NULL)) {
        want_debug = 1;
    } else {
        want_debug = want_dl_debug;
    }
    ret = get_create_func(options->v.v1.sofile, options->v.v1.symbol, &plugin, want_debug);
    if (ret != LCB_SUCCESS) {
        /* try to look up the symbol in the current image */
        lcb_STATUS ret2 = get_create_func(NULL, options->v.v1.symbol, &plugin, want_debug);
        if (ret2 != LCB_SUCCESS) {
#ifndef _WIN32
            char path[PATH_MAX];
            /* try to look up the so-file in the libdir */
            snprintf(path, PATH_MAX, "%s/%s", LCB_LIBDIR, options->v.v1.sofile);
            ret2 = get_create_func(path, options->v.v1.symbol, &plugin, want_debug);
#endif
            if (ret2 != LCB_SUCCESS) {
                /* return original error to allow caller to fix it */
                return ret;
            }
        }
    }

    ret = plugin.func.create(0, io, options->v.v1.cookie);
    if (ret != LCB_SUCCESS) {
        if (options->v.v1.sofile != NULL) {
            close_dlhandle(plugin.dlhandle);
        }
        return LCB_CLIENT_ENOMEM;
    } else {
        lcb_io_opt_t iop = *io;
        iop->dlhandle = plugin.dlhandle;
        /* check if plugin selected compatible version */
        if (iop->version < 0 || iop->version > 3) {
            lcb_destroy_io_ops(iop);
            return LCB_PLUGIN_VERSION_MISMATCH;
        }
    }

    return LCB_SUCCESS;
}

static lcb_STATUS create_v2(lcb_io_opt_t *io, const struct lcb_create_io_ops_st *options)
{
    lcb_STATUS ret;

    ret = options->v.v2.create(0, io, options->v.v2.cookie);
    if (ret != LCB_SUCCESS) {
        return ret;
    } else {
        lcb_io_opt_t iop = *io;
        /* check if plugin selected compatible version */
        if (iop->version < 0 || iop->version > 3) {
            lcb_destroy_io_ops(iop);
            return LCB_PLUGIN_VERSION_MISMATCH;
        }
    }

    return LCB_SUCCESS;
}

lcb_STATUS lcb_iops_cntl_handler(int mode, lcb_INSTANCE *instance, int cmd, void *arg)
{
    (void)instance;

    switch (cmd) {
        case LCB_CNTL_IOPS_DEFAULT_TYPES: {
            struct lcb_create_io_ops_st options;
            struct lcb_cntl_iops_info_st *info = arg;
            lcb_STATUS err;
            plugin_info pi;

            memset(&options, 0, sizeof(options));
            if (mode != LCB_CNTL_GET) {
                return LCB_NOT_SUPPORTED;
            }

            if (info->version != 0) {
                return LCB_EINVAL;
            }

            info->v.v0.os_default = DEFAULT_IOPS;

            err = generate_options(&pi, info->v.v0.options, &options, &info->v.v0.effective);

            if (err != LCB_SUCCESS) {
                return LCB_ERROR;
            }

            return LCB_SUCCESS;
        }

        case LCB_CNTL_IOPS_DLOPEN_DEBUG: {
            int *usr = arg;
            if (mode == LCB_CNTL_SET) {
                want_dl_debug = *usr;
            } else {
                *usr = want_dl_debug;
            }
            return LCB_SUCCESS;
        }

        default:
            return LCB_EINVAL;
    }
}

/* In-library wrapper version */
LIBCOUCHBASE_API
void lcb_iops_wire_bsd_impl2(lcb_bsd_procs *procs, int version)
{
    wire_lcb_bsd_impl2(procs, version);
}
