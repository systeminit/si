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

#ifndef SRC_CRAM_MD5_HMAC_H_
#define SRC_CRAM_MD5_HMAC_H_ 1
#define DIGEST_LENGTH 16

/**
 * Perform hmac on md5
 *
 * The code in this function is based on the code provided in rfc 2104.
 * http://www.ietf.org/rfc/rfc2104.txt
 */
void cbsasl_hmac_md5(unsigned char *text,
              int text_len,
              unsigned char *key,
              int keylen,
              unsigned char *digest);

#endif  /* SRC_CRAM_MD5_HMAC_H_ */
