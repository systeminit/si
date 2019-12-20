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
#ifndef LIBCOUCHBASE_ASSERT_H
#define LIBCOUCHBASE_ASSERT_H 1

#ifdef NDEBUG
#include <stdio.h>
#include <stdlib.h>

#define lcb_assert(a)                                                                                                  \
    if (!(a)) {                                                                                                        \
        fprintf(stderr, "FATAL ERROR:\n");                                                                             \
        fprintf(stderr, "    libcouchbase experienced an unrecoverable error");                                        \
        fprintf(stderr, " and terminates the program\n");                                                              \
        fprintf(stderr, "    to avoid undefined behavior.\n");                                                         \
        fprintf(stderr, "    The program should have generated a ");                                                   \
        fprintf(stderr, "\"corefile\" which may used\n");                                                              \
        fprintf(stderr, "    to gather more information about the problem.\n");                                        \
        fprintf(stderr, "    If your system doesn't create \"corefiles\" I ");                                         \
        fprintf(stderr, "can tell you that the\n");                                                                    \
        fprintf(stderr, "    assertion failed in %s at line %d\n", __FILE__, __LINE__);                                \
    }
#else
#include <assert.h>
#define lcb_assert(a) assert(a)
#endif

#endif
