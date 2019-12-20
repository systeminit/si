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

#ifndef LCB_ASPEND_H
#define LCB_ASPEND_H

#ifdef __cplusplus
#include <set>
typedef std::set< void * > lcb_ASPEND_SETTYPE;
#else
typedef void lcb_ASPEND_SETTYPE;
#endif

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @page Asynchronous Pending Queue
 *
 * This defines the API for asynchronous requests which should block calls to
 * lcb_wait() or similar. This is a replacement for the explicit hashsets used
 * in lcb_INSTANCE.
 *
 * Items are added to the pending queue via lcb_aspend_add(). They may be
 * removed either explicitly via lcb_aspend_del() or implicitly when the
 * instance is destroyed.
 *
 * An exception to this rule is the special LCB_PENDTYPE_COUNTER which does
 * not associate a specific pointer with it.
 */

/** Pending item type */
typedef enum {
    LCB_PENDTYPE_HTTP = 0,   /**< item is of type lcb_http_request_t */
    LCB_PENDTYPE_DURABILITY, /**< item is of type lcb_durability_set_t */
    LCB_PENDTYPE_COUNTER,    /**< just increment/decrement the counter */
    LCB_PENDTYPE_MAX
} lcb_ASPENDTYPE;

/** Items for pending operations */
typedef struct {
    lcb_ASPEND_SETTYPE *items[LCB_PENDTYPE_MAX];
    unsigned count;
} lcb_ASPEND;

/**
 * Initialize the pending queues
 * @param ops
 */
void lcb_aspend_init(lcb_ASPEND *ops);

/**
 * Clean up any resources used by the pending queues
 * @param ops
 */
void lcb_aspend_cleanup(lcb_ASPEND *ops);

/**
 * Add an opaque pointer of a given type to a pending queue
 * @param ops
 * @param type The type of pointer to add
 * @param item The item to add
 */
void lcb_aspend_add(lcb_ASPEND *ops, lcb_ASPENDTYPE type, const void *item);

/**
 * Remove an item from the queue and decrement the pending count
 * @param ops
 * @param type The type of item to remove
 * @param item The item to remove
 *
 * @attention If the item is not found inside the queue then the count is
 * _not_ decremented. An exception to this rule is the LCB_PENDTYPE_COUNTER
 * type which does not have a pointer associated with it. In this case the
 * counter is always decremented.
 */
void lcb_aspend_del(lcb_ASPEND *ops, lcb_ASPENDTYPE type, const void *item);

/**
 * Determine whether there are pending items in any of the queues
 * @param ops
 */
#define lcb_aspend_pending(ops) ((ops)->count > 0)

#ifdef __cplusplus
}
#endif
#endif
