/*
 * Copyright (c) 2006  Dustin Sallings <dustin@spy.net>
 */

#include <math.h>
#include "internal.h"
#include "genhash.h"

/* Table of 32 primes by their distance from the nearest power of two */
static lcb_size_t prime_size_table[] = {
    3, 7, 13, 23, 47, 97, 193, 383, 769, 1531, 3067, 6143, 12289, 24571, 49157,
    98299, 196613, 393209, 786433, 1572869, 3145721, 6291449, 12582917,
    25165813, 50331653, 100663291, 201326611, 402653189, 805306357,
    1610612741
};

#define TABLE_SIZE ((int)(sizeof(prime_size_table) / sizeof(int)))

struct genhash_entry_t {
    /** The key for this entry */
    void *key;
    /** Size of the key */
    lcb_size_t nkey;
    /** The value for this entry */
    void *value;
    /** Size of the value */
    lcb_size_t nvalue;
    /** Pointer to the next entry */
    struct genhash_entry_t *next;
};

struct _genhash {
    lcb_size_t size;
    struct lcb_hash_ops ops;
    struct genhash_entry_t *buckets[1];
};

static lcb_size_t estimate_table_size(lcb_size_t est);


static void *dup_key(genhash_t *h, const void *key, lcb_size_t klen)
{
    if (h->ops.dup_key != NULL) {
        return h->ops.dup_key(key, klen);
    } else {
        return (void *)key;
    }
}

static void *dup_value(genhash_t *h, const void *value, lcb_size_t vlen)
{
    if (h->ops.dup_value != NULL) {
        return h->ops.dup_value(value, vlen);
    } else {
        return (void *)value;
    }
}

static void free_key(genhash_t *h, void *key)
{
    if (h->ops.free_key != NULL) {
        h->ops.free_key(key);
    }
}

static void free_value(genhash_t *h, void *value)
{
    if (h->ops.free_value != NULL) {
        h->ops.free_value(value);
    }
}

static lcb_size_t estimate_table_size(lcb_size_t est)
{
    lcb_size_t rv = 0;
    while (prime_size_table[rv] < est && rv + 1 < TABLE_SIZE) {
        rv++;
    }
    return prime_size_table[rv];
}

genhash_t *genhash_init(lcb_size_t est, struct lcb_hash_ops ops)
{
    genhash_t *rv = NULL;
    lcb_size_t size = 0;
    if (est < 1) {
        return NULL;
    }

    lcb_assert(ops.hashfunc != NULL);
    lcb_assert(ops.hasheq != NULL);
    lcb_assert((ops.dup_key != NULL && ops.free_key != NULL) || ops.free_key == NULL);
    lcb_assert((ops.dup_value != NULL && ops.free_value != NULL) || ops.free_value == NULL);

    size = estimate_table_size(est);
    rv = calloc(1, sizeof(genhash_t)
                + (size * sizeof(struct genhash_entry_t *)));
    if (rv == NULL) {
        return NULL;
    }
    rv->size = size;
    rv->ops = ops;

    return rv;
}

void genhash_free(genhash_t *h)
{
    if (h != NULL) {
        genhash_clear(h);
        free(h);
    }
}

int genhash_store(genhash_t *h, const void *k, lcb_size_t klen,
                  const void *v, lcb_size_t vlen)
{
    lcb_size_t n = 0;
    struct genhash_entry_t *p;

    lcb_assert(h != NULL);

    n = h->ops.hashfunc(k, klen) % h->size;
    lcb_assert(n < h->size);

    p = calloc(1, sizeof(struct genhash_entry_t));
    if (!p) {
        return -1;
    }

    p->key = dup_key(h, k, klen);
    p->nkey = klen;
    p->value = dup_value(h, v, vlen);
    p->nvalue = vlen;

    p->next = h->buckets[n];
    h->buckets[n] = p;
    return 0;
}

static struct genhash_entry_t *genhash_find_entry(genhash_t *h,
                                                  const void *k,
                                                  lcb_size_t klen)
{
    lcb_size_t n = 0;
    struct genhash_entry_t *p;

    lcb_assert(h != NULL);
    n = h->ops.hashfunc(k, klen) % h->size;
    lcb_assert(n < h->size);

    for (p = h->buckets[n]; p && !h->ops.hasheq(k, klen, p->key, p->nkey); p = p->next);
    return p;
}

void *genhash_find(genhash_t *h, const void *k, lcb_size_t klen)
{
    struct genhash_entry_t *p;
    void *rv = NULL;

    p = genhash_find_entry(h, k, klen);

    if (p) {
        rv = p->value;
    }
    return rv;
}

enum update_type genhash_update(genhash_t *h, const void *k, lcb_size_t klen,
                                const void *v, lcb_size_t vlen)
{
    struct genhash_entry_t *p;
    enum update_type rv = 0;

    p = genhash_find_entry(h, k, klen);

    if (p) {
        free_value(h, p->value);
        p->value = dup_value(h, v, vlen);
        rv = MODIFICATION;
    } else {
        if (-1 == genhash_store(h, k, klen, v, vlen)) {
            return ALLOC_FAILURE;
        }
        rv = NEW;
    }

    return rv;
}

enum update_type genhash_fun_update(genhash_t *h,
                                    const void *k,
                                    lcb_size_t klen,
                                    void * (*upd)(const void *,
                                                  const void *,
                                                  lcb_size_t *,
                                                  void *),
                                    void (*fr)(void *),
                                    void *arg,
                                    const void *def,
                                    lcb_size_t deflen)
{
    struct genhash_entry_t *p;
    enum update_type rv = 0;
    lcb_size_t newSize = 0;

    p = genhash_find_entry(h, k, klen);

    if (p) {
        void *newValue = upd(k, p->value, &newSize, arg);
        free_value(h, p->value);
        p->value = dup_value(h, newValue, newSize);
        fr(newValue);
        rv = MODIFICATION;
    } else {
        void *newValue = upd(k, def, &newSize, arg);
        genhash_store(h, k, klen, newValue, newSize);
        fr(newValue);
        rv = NEW;
    }

    (void)deflen;
    return rv;
}

static void free_item(genhash_t *h, struct genhash_entry_t *i)
{
    lcb_assert(i);
    free_key(h, i->key);
    free_value(h, i->value);
    free(i);
}

int genhash_delete(genhash_t *h, const void *k, lcb_size_t klen)
{
    struct genhash_entry_t *deleteme = NULL;
    lcb_size_t n = 0;
    int rv = 0;

    lcb_assert(h != NULL);
    n = h->ops.hashfunc(k, klen) % h->size;
    lcb_assert(n < h->size);

    if (h->buckets[n] != NULL) {
        /* Special case the first one */
        if (h->ops.hasheq(h->buckets[n]->key, h->buckets[n]->nkey, k, klen)) {
            deleteme = h->buckets[n];
            h->buckets[n] = deleteme->next;
        } else {
            struct genhash_entry_t *p = NULL;
            for (p = h->buckets[n]; deleteme == NULL && p->next != NULL; p = p->next) {
                if (h->ops.hasheq(p->next->key, p->next->nkey, k, klen)) {
                    deleteme = p->next;
                    p->next = deleteme->next;
                }
            }
        }
    }
    if (deleteme != NULL) {
        free_item(h, deleteme);
        rv++;
    }

    return rv;
}

int genhash_delete_all(genhash_t *h, const void *k, lcb_size_t klen)
{
    int rv = 0;
    while (genhash_delete(h, k, klen) == 1) {
        rv++;
    }
    return rv;
}

void genhash_iter(genhash_t *h,
                  void (*iterfunc)(const void *key, lcb_size_t nkey,
                                   const void *val, lcb_size_t nval,
                                   void *arg), void *arg)
{
    lcb_size_t i = 0;
    struct genhash_entry_t *p = NULL;
    lcb_assert(h != NULL);

    for (i = 0; i < h->size; i++) {
        for (p = h->buckets[i]; p != NULL; p = p->next) {
            iterfunc(p->key, p->nkey, p->value, p->nvalue, arg);
        }
    }
}

int genhash_clear(genhash_t *h)
{
    lcb_size_t i = 0;
    int rv = 0;
    lcb_assert(h != NULL);

    for (i = 0; i < h->size; i++) {
        while (h->buckets[i]) {
            struct genhash_entry_t *p = NULL;
            p = h->buckets[i];
            h->buckets[i] = p->next;
            free_item(h, p);
        }
    }

    return rv;
}

static void count_entries(const void *key,
                          lcb_size_t klen,
                          const void *val,
                          lcb_size_t vlen,
                          void *arg)
{
    int *count = (int *)arg;
    (*count)++;
    (void)key;
    (void)klen;
    (void)val;
    (void)vlen;
}

int genhash_size(genhash_t *h)
{
    int rv = 0;
    lcb_assert(h != NULL);
    genhash_iter(h, count_entries, &rv);
    return rv;
}

int genhash_size_for_key(genhash_t *h, const void *k, lcb_size_t klen)
{
    int rv = 0;
    lcb_assert(h != NULL);
    genhash_iter_key(h, k, klen, count_entries, &rv);
    return rv;
}

void genhash_iter_key(genhash_t *h, const void *key, lcb_size_t klen,
                      void (*iterfunc)(const void *key, lcb_size_t klen,
                                       const void *val, lcb_size_t vlen,
                                       void *arg), void *arg)
{
    lcb_size_t n = 0;
    struct genhash_entry_t *p = NULL;

    lcb_assert(h != NULL);
    n = h->ops.hashfunc(key, klen) % h->size;
    lcb_assert(n < h->size);

    for (p = h->buckets[n]; p != NULL; p = p->next) {
        if (h->ops.hasheq(key, klen, p->key, p->nkey)) {
            iterfunc(p->key, p->nkey, p->value, p->nvalue, arg);
        }
    }
}

int genhash_string_hash(const void *p, lcb_size_t nkey)
{
    int rv = 5381;
    int i = 0;
    char *str = (char *)p;

    for (i = 0; i < (int)nkey; i++) {
        lcb_assert(str[i]);
        rv = ((rv << 5) + rv) ^ str[i];
    }

    return rv;
}
