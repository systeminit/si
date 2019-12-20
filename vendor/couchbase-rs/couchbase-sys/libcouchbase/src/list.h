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

#ifndef LIBCOUCHBASE_LIST_H
#define LIBCOUCHBASE_LIST_H 1

#ifdef __cplusplus
extern "C" {
#endif

/* Circular list implementation
 *
 * Could be used to implement queues, stacks etc.
 *
 * Declare list in your structure:
 *
 *      typedef struct {
 *          lcb_list_t list;
 *          lcb_uint32_t msec;
 *          void (*callback)(struct timer *tm);
 *      } lcb_timer_t;
 *
 * Initialize head of list:
 *
 *      lcb_timer_t tm;
 *      lcb_list_init(&tm.list);
 *
 * Add new items:
 *
 *      lcb_timer_t *t;
 *      t->msec = 2000;
 *      t->callback = my_cb;
 *      lcb_list_append(&tm.list, t);
 *
 * Iterate over items:
 *
 *      lcb_timer_t *t, *n;
 *
 *      for (ii = tm.list.next; ii != &tm.list; ii = ii->next) {
 *          t = LCB_LIST_ITEM(ii, lcb_timer_t, list);
 *          printf("timeout: %d\n", t->msec);
 *      }
 *
 *      LCB_LIST_FOR(ii, &tm.list) {
 *          t = LCB_LIST_ITEM(ii, lcb_timer_t, list);
 *          printf("timeout: %d\n", t->msec);
 *      }
 *
 *      LCB_LIST_SAFE_FOR(ii, n, &tm.list) {
 *          t = LCB_LIST_ITEM(ii, lcb_timer_t, list);
 *          printf("timeout: %d\n", t->msec);
 *      }
 */
typedef struct lcb_list_s lcb_list_t;
struct lcb_list_s {
    lcb_list_t *next;
    lcb_list_t *prev;
};

typedef struct lcb_clist_s {
    lcb_list_t *next;
    lcb_list_t *prev;
    lcb_size_t size;
} lcb_clist_t;

typedef int (*lcb_list_cmp_fn)(lcb_list_t *a, lcb_list_t *b);

void lcb_list_init(lcb_list_t *list);
void lcb_list_prepend(lcb_list_t *list, lcb_list_t *item);
void lcb_list_append(lcb_list_t *list, lcb_list_t *item);
void lcb_list_delete(lcb_list_t *item);
lcb_list_t *lcb_list_shift(lcb_list_t *list);
lcb_list_t *lcb_list_pop(lcb_list_t *list);
int lcb_list_contains(lcb_list_t *list, lcb_list_t *item);
void lcb_list_add_sorted(lcb_list_t *list, lcb_list_t *item, lcb_list_cmp_fn cmp);

/** Definitions for type safety. Rather than macros */
void lcb_clist_init(lcb_clist_t *);
void lcb_clist_append(lcb_clist_t *, lcb_list_t *);
void lcb_clist_prepend(lcb_clist_t *, lcb_list_t *);
void lcb_clist_delete(lcb_clist_t *, lcb_list_t *);
lcb_list_t *lcb_clist_shift(lcb_clist_t *);
lcb_list_t *lcb_clist_pop(lcb_clist_t *);

#define LCB_LIST_IS_EMPTY(list) ((list) == (list)->next && (list) == (list)->prev)

#define LCB_LIST_ITEM(ptr, type, member) ((type *)(void *)((char *)(ptr)-offsetof(type, member)))

#define LCB_LIST_FOR(pos, list) for (pos = (list)->next; pos != (list); pos = pos->next)

#define LCB_LIST_SAFE_FOR(pos, n, list) for (pos = (list)->next, n = pos->next; pos != (list); pos = n, n = pos->next)

#define LCB_LIST_HAS_NEXT(ll, item) ((item)->next != ll)

#define LCB_CLIST_SIZE(cl) (cl)->size

#define LCB_LIST_TAIL(list) ((LCB_LIST_IS_EMPTY(list)) ? NULL : (list)->prev)

#define LCB_LIST_HEAD(list) ((LCB_LIST_IS_EMPTY(list)) ? NULL : (list)->next)

#ifdef __cplusplus
}
#endif
#endif
