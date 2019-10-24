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

/**
 * @file
 * libuv integration with libcouchbase
 * @author Mark Nunberg
 */

/**
 * @ingroup lcb-io-plugin-api
 * @defgroup lcb-libuv libuv
 * @brief libuv integration
 *
 * @details
 * libuv is a cross platform event framework using a completion-based programming
 * model. Since most distributions do not offer up-to-date libuv binary packages
 * and because libuv is constantly evolving itself, binary packages are not
 * provided. The source code for this plugin is actually shipped with the
 * headers of libcouchbase.
 *
 * If you built libuv together with libcouchbase (and thus there exists a
 * library called `libcouchbase_libuv` then you may simply include this file
 * and initialize the iops structure.
 *
 * Otherwise you should ensure that `<libcouchbase/plugins/io/libuv/plugin-libuv.c>`
 * is compiled into an object, _and_ that the `LCBUV_EMBEDDED_SOURCE` macro
 * is defined for both the compiled object and any code in your application
 * that is using the module, thus:
 *
 * @code{.c}
 * //uv-stub.c
 * #include <libcouchbase/plugins/io/libuv/plugin-libuv.c>
 * @endcode
 *
 * Then, in your application
 * @code{.c}
 * #include <libcouchbase/libuv_io_opts.h>
 *
 * lcb_io_opt_t *io;
 * lcbuv_options_t options;
 * options.v.v0.loop = uv_default_loop();
 * options.v.v0.startsop_noop = 1;
 * lcb_create_libuv_io_opts(0, &io, &options);
 * @endcode
 *
 * And then compile as
 * @code
 * $ gcc -o myapp uv-stub.c main.c -DLCBUV_EMBEDDED_SOURCE
 * @endcode
 *
 * @addtogroup lcb-libuv
 * @{
 */

#ifndef LCB_PLUGIN_UV_H
#define LCB_PLUGIN_UV_H
#ifdef __cplusplus
extern "C" {
#endif

#include <libcouchbase/couchbase.h>
#include <uv.h>

#ifdef LCBUV_EMBEDDED_SOURCE
#define LCBUV_API
#else
#define LCBUV_API LIBCOUCHBASE_API
#endif

/**Options passed to the iops constructure. You will most likely want
 * to set the 'startsop_noop' field to true if you are using an async
 * application.*/
typedef struct lcbuv_options_st {
    int version;
    union {
        struct {
            /** External loop to be used (if not default) */
            uv_loop_t *loop;

            /** Whether run_event_loop/stop_event_loop should do anything */
            int startsop_noop;
        } v0;
    } v;
} lcbuv_options_t;

/**
 * Use this if using an existing uv_loop_t
 * @param version Set this to `0`
 * @param [out] io a pointer to an io pointer. Will be populated on success
 * @param options the options to be passed. From libcouchbase this is a
 * `void*` parameter.
 */
LCBUV_API
lcb_STATUS lcb_create_libuv_io_opts(int version, lcb_io_opt_t *io, lcbuv_options_t *options);

#ifdef __cplusplus
}
#endif
#endif

/**@}*/
