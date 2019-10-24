/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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

#ifndef LCBIO_H
#define LCBIO_H
#include "connect.h"
#include "manager.h"
#include "ioutils.h"
#include "ctx.h"
#endif

/**
 * @defgroup lcbio I/O
 * @brief IO Core
 *
 * @details
 *
 * This module represents the I/O core of libcouchbase. Effort has been made
 * so that this module in theory is usable outside of libcouchbase.
 *
 * # Architectural Overview
 *
 * The I/O core (_LCBIO_) has been designed to support different I/O models and
 * operating environments, with the goal of being able to integrate natively
 * into such environments with minimal performance loss. Integration is acheived
 * through several layers. The first layer is the _IOPS_ system defined in
 * <libcouchbase/iops.h> and defines integration APIs for different I/O models.
 *
 * Afterwards, this is flattened and normalized into an _IO Table_ (`lcbio_TABLE`)
 * which serves as a context and abstraction layer for unifying those two APIs
 * where applicable.
 *
 * Finally the "End-user" APIs (in this case end-user means the application which
 * requests a TCP connection or I/O on a socket) employs the
 * `lcbio_connect` and `lcbio_CTX` systems to provide a uniform interface, the
 * end result being that the underlying I/O model is completely abstracted.
 */
