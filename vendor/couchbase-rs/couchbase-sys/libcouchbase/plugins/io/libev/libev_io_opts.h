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

/**
 * @file
 * libev integration with libcouchbase
 * @author Sergey Avseyev
 */

/**
 * @ingroup lcb-io-plugin-api
 * @defgroup lcb-libev libev
 * @brief libev integration
 *
 * @details
 * libcouchbase_create_libev_io_opts() allows you to create an instance
 * of the ioopts that will utilize libev. You may either supply an event
 * loop (if you'd like to add your own events into the loop), or it will
 * create it's own.
 *
 * @addtogroup lcb-libev
 * @{
 */
#ifndef LIBCOUCHBASE_LIBEV_IO_OPTS_H
#define LIBCOUCHBASE_LIBEV_IO_OPTS_H 1

#include <libcouchbase/couchbase.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Create an instance of an event handler that utilize libev for
 * event notification.
 *
 * @param version Set this to 0. This may be used in the future to allow
 *        variation on the third argument (`void*` currently).
 * @param[out] io a pointer to a newly created and initialized event handler
 * @param loop the event loop (struct ev_loop *) to hook use (please
 *             note that you shouldn't reference the event loop from
 *             multiple threads)
 * @return status of the operation
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_create_libev_io_opts(int version, lcb_io_opt_t *io, void *loop);
#ifdef __cplusplus
}
#endif

/**@}*/
#endif
