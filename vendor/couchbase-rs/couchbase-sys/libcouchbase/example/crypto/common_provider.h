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

#ifndef _COMMON_PROVIDER_H
#define _COMMON_PROVIDER_H

#include <libcouchbase/couchbase.h>

extern char *common_aes256_key_id;

#define AES256_KEY_SIZE 32
#define AES256_IV_SIZE 16

extern uint8_t common_aes256_key[AES256_KEY_SIZE];
extern uint8_t common_aes256_iv[AES256_IV_SIZE];

extern uint8_t *common_hmac_sha256_key;

#endif
