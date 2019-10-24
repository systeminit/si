/*
 * Generic hash table implementation.
 *
 * Copyright (c) 2006  Dustin Sallings <dustin@spy.net>
 */

#ifndef GENHASH_H
#define GENHASH_H 1
#ifdef __cplusplus
extern "C" {
#endif

/*! \mainpage genhash
 *
 * \section intro_sec Introduction
 *
 * genhash is a generic hash table implementation in C.  It's
 * well-tested, freely available (MIT-license) and does what you need.
 *
 * \section docs_sec API Documentation
 *
 * Jump right into <a href="group___core.html">the API docs</a> to get started.
 */

/**
 * \defgroup Core genhash core
 */

/**
 * \addtogroup Core
 * @{
 */

/**
 * Operations on keys and values in the hash table.
 */
struct lcb_hash_ops {
    /**
     * Function to compute a hash for the given value.
     */
    int (*hashfunc)(const void *, lcb_size_t);
    /**
     * Function that returns true if the given keys are equal.
     */
    int (*hasheq)(const void *, lcb_size_t, const void *, lcb_size_t);
    /**
     * Function to duplicate a key for storage.
     */
    void *(*dup_key)(const void *, lcb_size_t);
    /**
     * Function to duplicate a value for storage.
     */
    void *(*dup_value)(const void *, lcb_size_t);
    /**
     * Function to free a key.
     */
    void (*free_key)(void *);
    /**
     * Function to free a value.
     */
    void (*free_value)(void *);
};

/**
 * The hash table structure.
 */
typedef struct _genhash genhash_t ;

/**
 * Type of update performed by an update function.
 */
enum update_type {
    MODIFICATION, /**< This update is modifying an existing entry */
    NEW,           /**< This update is creating a new entry */
    ALLOC_FAILURE
};

/**
 * Create a new generic hashtable.
 *
 * @param est the estimated number of items to store (must be > 0)
 * @param ops the key and value operations
 *
 * @return the new genhash_t or NULL if one cannot be created
 */
genhash_t *genhash_init(lcb_size_t est, struct lcb_hash_ops ops);

/**
 * Free a gen hash.
 *
 * @param h the genhash to free (may be NULL)
 */
void genhash_free(genhash_t *h);

/**
 * Store an item.
 *
 * @param h the genhash
 * @param k the key
 * @param v the value
 */
int genhash_store(genhash_t *h, const void *k, lcb_size_t klen,
                  const void *v, lcb_size_t vlen);

/**
 * Get the most recent value stored for the given key.
 *
 * @param h the genhash
 * @param k the key
 *
 * @return the value, or NULL if one cannot be found
 */
void *genhash_find(genhash_t *h, const void *k, lcb_size_t klen);

/**
 * Delete the most recent value stored for a key.
 *
 * @param h the genhash
 * @param k the key
 *
 * @return the number of items deleted
 */
int genhash_delete(genhash_t *h, const void *k, lcb_size_t klen);

/**
 * Delete all mappings of a given key.
 *
 * @param h the genhash
 * @param k the key
 *
 * @return the number of items deleted
 */
int genhash_delete_all(genhash_t *h, const void *k, lcb_size_t klen);

/**
 * Create or update an item in-place.
 *
 * @param h the genhash
 * @param k the key
 * @param v the new value to store for this key
 *
 * @return an indicator of whether this created a new item or updated
 *         an existing one
 */
enum update_type genhash_update(genhash_t *h, const void *k, lcb_size_t klen,
                                const void *v, lcb_size_t vlen);

/**
 * Create or update an item in-place with a function.
 *
 * @param h hashtable
 * @param key the key of the item
 * @param upd function that will be called with the key and current
 *        value.  Should return the new value.
 * @param fr function to free the return value returned by the update
 *        function
 * @param def default value
 *
 * @return an indicator of whether this created a new item or updated
 *         an existing one
 */
enum update_type genhash_fun_update(genhash_t *h, const void *key, lcb_size_t klen,
                                    void * (*upd)(const void *k, const void *oldv,
                                                  lcb_size_t *ns, void *a),
                                    void (*fr)(void *),
                                    void *arg,
                                    const void *def, lcb_size_t deflen);

/**
 * Iterate all keys and values in a hash table.
 *
 * @param h the genhash
 * @param iterfunc a function that will be called once for every k/v pair
 * @param arg an argument to be passed to the iterfunc on each iteration
 */
void genhash_iter(genhash_t *h,
                  void (*iterfunc)(const void *key, lcb_size_t nkey,
                                   const void *val, lcb_size_t nval,
                                   void *arg),
                  void *arg);

/**
 * Iterate all values for a given key in a hash table.
 *
 * @param h the genhash
 * @param key the key to iterate
 * @param iterfunc a function that will be called once for every k/v pair
 * @param arg an argument to be passed to the iterfunc on each iteration
 */
void genhash_iter_key(genhash_t *h, const void *key, lcb_size_t nkey,
                      void (*iterfunc)(const void *key, lcb_size_t inkey,
                                       const void *val, lcb_size_t inval,
                                       void *arg),
                      void *arg);

/**
 * Get the total number of entries in this hash table.
 *
 * @param h the genhash
 *
 * @return the number of entries in the hash table
 */
int genhash_size(genhash_t *h);

/**
 * Remove all items from a genhash.
 *
 * @param h the genhash
 *
 * @return the number of items removed
 */
int genhash_clear(genhash_t *h);

/**
 * Get the total number of entries in this hash table that map to the given
 * key.
 *
 * @param h the genhash
 * @param k a key
 *
 * @return the number of entries keyed with the given key
 */
int genhash_size_for_key(genhash_t *h, const void *k, lcb_size_t nkey);

/**
 * Convenient hash function for strings.
 *
 * @param k a null-terminated string key.
 *
 * @return a hash value for this string.
 */
int genhash_string_hash(const void *k, lcb_size_t nkey);

/**
 * @}
 */

#ifdef __cplusplus
}
#endif
#endif /* GENHASH_H */
