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

#include "internal.h"
#include "contrib/genhash/genhash.h"

/**
 * Structures for common hash table operations
 */

static int hasheq(const void *a, lcb_size_t n_a, const void *b, lcb_size_t n_b)
{
    if (n_a != n_b) {
        return 0;
    }

    return memcmp(a, b, n_a) == 0;
}
/**
 * Structure for a no-copy hash table
 */
static struct lcb_hash_ops hashops_nocopy = {
    genhash_string_hash, /* hashfunc */
    hasheq,              /* hasheq */
    NULL,                /* dupKey */
    NULL,                /* dupValue */
    NULL,                /* freeKey */
    NULL,                /* freeValue */
};

genhash_t *lcb_hashtable_nc_new(lcb_size_t est)
{
    return genhash_init(est, hashops_nocopy);
}

static int u32_hash(const void *p, lcb_size_t n)
{
    (void)p;
    return n;
}

static int u32_eq(const void *a, lcb_size_t n_a, const void *b, lcb_size_t n_b)
{
    (void)a;
    (void)b;
    return n_a == n_b;
}

static struct lcb_hash_ops hashops_u32 = {u32_hash, u32_eq, NULL, NULL, NULL, NULL};

genhash_t *lcb_hashtable_szt_new(lcb_size_t est)
{
    return genhash_init(est, hashops_u32);
}
