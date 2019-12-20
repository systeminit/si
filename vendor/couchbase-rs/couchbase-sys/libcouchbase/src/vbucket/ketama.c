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

#define PROTOTYPES 1

#define MD5Final vb__MD5_final
#define MD5Init vb__MD5Init
#define MD5Update vb__MD5Update

#include <stdlib.h>
#include "rfc1321/md5c-inl.h"
#include "hash.h"

void vb__hash_md5(const char *key, size_t key_length, unsigned char *result)
{
    MD5_CTX ctx;

    MD5Init(&ctx);
    MD5Update(&ctx, (unsigned char *)key, (unsigned int)key_length);
    MD5Final(result, &ctx);
}

void *vb__hash_md5_update(void *ctx, const char *key, size_t key_length)
{
    if (ctx == NULL) {
        ctx = calloc(1, sizeof(MD5_CTX));
        MD5Init(ctx);
    }
    MD5Update(ctx, (unsigned char *)key, (unsigned int)key_length);
    return ctx;
}

void vb__hash_md5_final(void *ctx, unsigned char *result)
{
    if (ctx == NULL) {
        return;
    }
    MD5Final(result, ctx);
    free(ctx);
}

uint32_t vb__hash_ketama(const char *key, size_t key_length)
{
    unsigned char digest[16];

    vb__hash_md5(key, key_length, digest);

    return (uint32_t)((digest[3] << 24) | (digest[2] << 16) | (digest[1] << 8) | digest[0]);
}
