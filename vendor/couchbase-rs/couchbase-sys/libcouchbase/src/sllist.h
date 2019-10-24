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

#ifndef LCB_SLIST_H
#define LCB_SLIST_H

struct slist_node_st;
typedef struct slist_node_st {
    struct slist_node_st *next;
} sllist_node;

typedef struct {
    sllist_node first_prev;
    sllist_node *last;
} sllist_root;

/**
 * Indicates whether the list is empty or not
 */
#define SLLIST_FIRST(list) (list)->first_prev.next
#define SLLIST_LAST(list) (list)->last

#define SLLIST_IS_EMPTY(list) (SLLIST_LAST(list) == NULL)
#define SLLIST_IS_ONE(list) (SLLIST_FIRST(list) && SLLIST_FIRST(list) == SLLIST_LAST(list))

/**
 * Iterator for list. This can be used as the 'for' statement; as such this
 * macro should look like such:
 *
 *  slist_node *ii;
 *  SLIST_FOREACH(list, ii) {
 *      my_item *item = LCB_LIST_ITEM(my_item, ii, slnode);
 *  }
 *
 *  @param list the list to iterate
 *  @param pos a local variable to use as the iterator
 */
#define SLLIST_FOREACH(list, pos) for (pos = SLLIST_FIRST(list); pos; pos = pos->next)

typedef struct sllist_iterator_st {
    sllist_node *cur;
    sllist_node *prev;
    sllist_node *next;
    int removed;
} sllist_iterator;

#define sllist_iter_end(list, iter) ((iter)->cur == NULL)

#define SLLIST_ITEM(ptr, type, member) ((type *)(void *)((char *)(ptr)-offsetof(type, member)))

#define SLLIST_ITERFOR(list, iter)                                                                                     \
    for (slist_iter_init(list, iter); !sllist_iter_end(list, iter); slist_iter_incr(list, iter))

#define SLLIST_ITERBASIC(list, elem) for (elem = SLLIST_FIRST(list); elem; elem = elem->next)

#endif
