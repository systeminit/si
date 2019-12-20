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

#include "cbsasl/cbsasl.h"
#include "util.h"
#include <stdlib.h>

CBSASL_PUBLIC_API
void cbsasl_dispose(cbsasl_conn_t **conn)
{
    if (*conn != NULL) {
        if ((*conn)->client) {
            free((*conn)->c.client.userdata);
            free((*conn)->c.client.nonce);
            free((*conn)->c.client.client_first_message_bare);
            free((*conn)->c.client.saltedpassword);
            free((*conn)->c.client.auth_message);
        } else {
            free((*conn)->c.server.username);
            free((*conn)->c.server.config);
            free((*conn)->c.server.sasl_data);
        }

        free(*conn);
        *conn = NULL;
    }
}

static const char *hexchar = "0123456789abcdef";
void cbsasl_hex_encode(char *dest, const char *src, size_t srclen)
{
    size_t i;
    for (i = 0; i < srclen; i++) {
        dest[i * 2] = hexchar[(src[i] >> 4) & 0xF];
        dest[i * 2 + 1] = hexchar[src[i] & 0xF];
    }
}
