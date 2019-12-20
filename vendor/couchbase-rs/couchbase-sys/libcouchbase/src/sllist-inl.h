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

#include "sllist.h"
#include <libcouchbase/assert.h>
#include <stdlib.h>
#include <stdio.h>
#ifndef INLINE
#ifdef _MSC_VER
#define INLINE __inline
#elif __GNUC__
#define INLINE __inline__
#else
#define INLINE inline
#endif /* MSC_VER */
#endif /* !INLINE */

static INLINE int sllist_contains(sllist_root *list, sllist_node *item)
{
    sllist_node *ll;
    SLLIST_FOREACH(list, ll)
    {
        if (item == ll) {
            return 1;
        }
    }
    return 0;
}

static INLINE unsigned sllist_get_size(sllist_root *list)
{
    unsigned ret = 0;
    sllist_node *ll;
    SLLIST_FOREACH(list, ll)
    {
        ret++;
    }
    return ret;
}

/* #define SLLIST_DEBUG */

#ifdef SLLIST_DEBUG
#define slist_sanity_insert(l, n) lcb_assert(!slist_contains(l, n))
#else
#define slist_sanity_insert(l, n)
#endif

static INLINE void slist_iter_init_at(sllist_node *node, sllist_iterator *iter)
{
    iter->cur = node->next;
    iter->prev = node;
    iter->removed = 0;

    if (iter->cur) {
        iter->next = iter->cur->next;
    } else {
        iter->next = NULL;
    }
}

static INLINE void slist_iter_init(sllist_root *list, sllist_iterator *iter)
{
    slist_iter_init_at(&list->first_prev, iter);
}

static INLINE void slist_iter_incr(sllist_root *list, sllist_iterator *iter)
{
    if (!iter->removed) {
        iter->prev = iter->prev->next;
    } else {
        iter->removed = 0;
    }

    if ((iter->cur = iter->next)) {
        iter->next = iter->cur->next;
    } else {
        iter->next = NULL;
    }

    lcb_assert(iter->cur != iter->prev);

    (void)list;
}

static INLINE void sllist_iter_remove(sllist_root *list, sllist_iterator *iter)
{
    iter->prev->next = iter->next;

    /** GCC strict aliasing. Yay. */
    if (iter->prev->next == NULL && iter->prev == &list->first_prev) {
        list->last = NULL;
    } else if (iter->cur == list->last && iter->next == NULL) {
        /* removing the last item */
        list->last = iter->prev;
    }
    iter->removed = 1;
}

static INLINE void sllist_remove_head(sllist_root *list)
{
    if (!SLLIST_FIRST(list)) {
        return;
    }

    SLLIST_FIRST(list) = SLLIST_FIRST(list)->next;

    if (!SLLIST_FIRST(list)) {
        list->last = NULL;
    }
}

static INLINE void sllist_remove(sllist_root *list, sllist_node *item)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(list, &iter)
    {
        if (iter.cur == item) {
            sllist_iter_remove(list, &iter);
            return;
        }
    }
    fprintf(stderr, "SLLIST: Requested to remove item %p which is not in %p\n", (void *)list, (void *)item);
    lcb_assert(0);
}

static INLINE void sllist_append(sllist_root *list, sllist_node *item)
{
    if (SLLIST_IS_EMPTY(list)) {
        SLLIST_FIRST(list) = list->last = item;
        item->next = NULL;
    } else {
        slist_sanity_insert(list, item);
        list->last->next = item;
        list->last = item;
    }
    item->next = NULL;
}

static INLINE void sllist_prepend(sllist_root *list, sllist_node *item)
{
    if (SLLIST_IS_EMPTY(list)) {
        SLLIST_FIRST(list) = list->last = item;
    } else {
        slist_sanity_insert(list, item);
        item->next = SLLIST_FIRST(list);
        SLLIST_FIRST(list) = item;
    }
}

static void sllist_insert(sllist_root *list, sllist_node *prev, sllist_node *item)
{
    item->next = prev->next;
    prev->next = item;
    if (item->next == NULL) {
        list->last = item;
    }
}

static INLINE void sllist_insert_sorted(sllist_root *list, sllist_node *item,
                                        int (*compar)(sllist_node *, sllist_node *))
{
    sllist_iterator iter;
    SLLIST_ITERFOR(list, &iter)
    {
        int rv = compar(item, iter.cur);
        /** if the item we have is before the current, prepend it here */
        if (rv <= 0) {
            sllist_insert(list, iter.prev, item);
            return;
        }
    }
    sllist_append(list, item);
}
