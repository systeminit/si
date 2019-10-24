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

#include "common_provider.h"

char *common_aes256_key_id = "mykeyid";

uint8_t *common_hmac_sha256_key = "myauthpassword";

uint8_t common_aes256_key[AES256_KEY_SIZE] = "!mysecretkey#9^5usdk39d&dlf)03sL";
uint8_t common_aes256_iv[AES256_IV_SIZE] = {0x65, 0xe7, 0x66, 0xbe, 0x35, 0xb2, 0xd2, 0x52,
                                            0x2b, 0x2e, 0x7e, 0x8e, 0x99, 0x9,  0x8d, 0xa9};
