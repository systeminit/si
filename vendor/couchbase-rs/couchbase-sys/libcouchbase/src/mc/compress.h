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

#include "mcreq.h"
#include "settings.h"
#ifndef LCB_MCCOMPRESS_H
#define LCB_MCCOMPRESS_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Stores a compressed payload into a packet
 * @param pl The pipeline which hosts the packet
 * @param pkt The packet which hosts the value
 * @param vbuf The user input to be compressed
 * @param settings The instance settings
 * @param should_compress The pointer, which stores zero if the value is not compressed
 * @return 0 if successful, nonzero on error.
 */
int mcreq_compress_value(mc_PIPELINE *pl, mc_PACKET *pkt, const lcb_VALBUF *vbuf, lcb_settings *settings,
                         int *should_compress);

/**
 * Inflate a compressed value
 * @param compressed The value to inflate
 * @param ncompressed Size of value to inflate
 * @param[out] bytes The inflated value
 * @param[out] nbytes The size of the inflated value
 * @param[in/out] freeptr Pointer initialized to NULL (or an malloc'd buffer)
 * which on output should point to a malloc'd buffer to be freed() when no
 * longer required.
 * @return 0 if successful, nonzero on error.
 */
int mcreq_inflate_value(const void *compressed, lcb_SIZE ncompressed, const void **bytes, lcb_SIZE *nbytes,
                        void **freeptr);

#ifdef __cplusplus
}
#endif
#endif
