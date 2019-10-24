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

#include "hmac.h"
#include "md5.h"
#include <string.h>

/**
 * The code in this function is based on the code provided in rfc 2104.
 * http://www.ietf.org/rfc/rfc2104.txt
 */
void cbsasl_hmac_md5(unsigned char *text,
              int textlen,
              unsigned char *key,
              int keylen,
              unsigned char *digest)
{
    MD5_CTX context;
    unsigned char k_ipad[65];
    unsigned char k_opad[65];
    unsigned char tk[16];
    int i;

    if (keylen > 64) {
        MD5_CTX ctx;
        cbsasl_MD5_Init(&ctx);
        cbsasl_MD5_Update(&ctx, key, keylen);
        cbsasl_MD5_Final(tk, &ctx);
        key = tk;
        keylen = 16;
    }

    memset(k_ipad, 0, sizeof(k_ipad));
    memset(k_opad, 0, sizeof(k_opad));
    memcpy(k_ipad, key, keylen);
    memcpy(k_opad, key, keylen);

    for (i = 0; i < 64; i++) {
        k_ipad[i] ^= 0x36;
        k_opad[i] ^= 0x5c;
    }

    /* Perform inner md5 */
    cbsasl_MD5_Init(&context);
    cbsasl_MD5_Update(&context, k_ipad, 64);
    cbsasl_MD5_Update(&context, text, textlen);
    cbsasl_MD5_Final(digest, &context);

    /* Perform outer md5 */
    cbsasl_MD5_Init(&context);
    cbsasl_MD5_Update(&context, k_opad, 64);
    cbsasl_MD5_Update(&context, digest, 16);
    cbsasl_MD5_Final(digest, &context);
}
