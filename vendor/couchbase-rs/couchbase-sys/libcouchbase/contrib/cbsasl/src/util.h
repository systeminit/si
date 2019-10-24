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

#ifndef CBSASL_UTIL_H_
#define CBSASL_UTIL_H_ 1
#include <config.h>

/* Encode hexadecimal representation of bytes from src into dest.
 * Will write srclen * 2 bytes. */
void cbsasl_hex_encode(char *dest, const char *src, size_t srclen);

/* Compare a and b without revealing their content by short-circuiting */
int cbsasl_secure_compare(const char *a, size_t alen, const char *b, size_t blen);

cbsasl_error_t cbsasl_secure_random(char *dest, size_t len);


#endif /*  CBSASL_UTIL_H_ */
