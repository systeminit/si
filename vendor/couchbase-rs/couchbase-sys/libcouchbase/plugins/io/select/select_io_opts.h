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

#ifndef LIBCOUCHBASE_SELECT_IO_OPTS_H
#define LIBCOUCHBASE_SELECT_IO_OPTS_H 1

#include <libcouchbase/couchbase.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Create an instance of an event handler that utilize libev for
 * event notification.
 *
 * @return status of the operation
 */
LIBCOUCHBASE_API
lcb_STATUS lcb_create_select_io_opts(int version, lcb_io_opt_t *io, void *loop);
#ifdef __cplusplus
}
#endif

#endif
