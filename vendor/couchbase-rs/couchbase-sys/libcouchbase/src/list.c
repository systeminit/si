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

void lcb_list_init(lcb_list_t *list)
{
    list->next = list;
    list->prev = list;
}

static void list_insert(lcb_list_t *prev, lcb_list_t *next, lcb_list_t *item)
{
    item->next = next;
    item->prev = prev;
    next->prev = item;
    prev->next = item;
}

void lcb_list_prepend(lcb_list_t *list, lcb_list_t *item)
{
    list_insert(list, list->next, item);
}

void lcb_list_append(lcb_list_t *list, lcb_list_t *item)
{
    list_insert(list->prev, list, item);
}

static void list_eject(lcb_list_t *prev, lcb_list_t *next)
{
    next->prev = prev;
    prev->next = next;
}

void lcb_list_delete(lcb_list_t *item)
{
    list_eject(item->prev, item->next);
    item->next = item->prev = NULL;
}

lcb_list_t *lcb_list_shift(lcb_list_t *list)
{
    lcb_list_t *item;

    if (LCB_LIST_IS_EMPTY(list)) {
        return NULL;
    }
    item = list->next;
    lcb_list_delete(item);
    return item;
}

lcb_list_t *lcb_list_pop(lcb_list_t *list)
{
    lcb_list_t *item;

    if (LCB_LIST_IS_EMPTY(list)) {
        return NULL;
    }
    item = list->prev;
    lcb_list_delete(item);
    return item;
}

int lcb_list_contains(lcb_list_t *list, lcb_list_t *item)
{
    lcb_list_t *ptr = list->next;

    while (ptr != list && ptr != item) {
        ptr = ptr->next;
    }

    return (ptr == item) ? 1 : 0;
}

void lcb_list_add_sorted(lcb_list_t *list, lcb_list_t *item, lcb_list_cmp_fn cmp)
{
    lcb_list_t *p;

    if (LCB_LIST_IS_EMPTY(list)) {
        list_insert(list->prev, list, item);
    } else {
        LCB_LIST_FOR(p, list)
        {
            if (cmp(item, p) < 0) {
                break;
            }
        }
        list_insert(p->prev, p, item);
    }
}

void lcb_clist_init(lcb_clist_t *cl)
{
    lcb_list_init((lcb_list_t *)cl);
    cl->size = 0;
}
void lcb_clist_append(lcb_clist_t *cl, lcb_list_t *item)
{
    lcb_list_append((lcb_list_t *)cl, item);
    cl->size++;
}
void lcb_clist_prepend(lcb_clist_t *cl, lcb_list_t *item)
{
    lcb_list_prepend((lcb_list_t *)cl, item);
    cl->size++;
}
void lcb_clist_delete(lcb_clist_t *cl, lcb_list_t *item)
{
    lcb_list_delete(item);
    cl->size--;
}
lcb_list_t *lcb_clist_pop(lcb_clist_t *cl)
{
    lcb_list_t *ret = lcb_list_pop((lcb_list_t *)cl);
    if (ret) {
        cl->size--;
    }
    return ret;
}
lcb_list_t *lcb_clist_shift(lcb_clist_t *cl)
{
    lcb_list_t *ret = lcb_list_shift((lcb_list_t *)cl);
    if (ret) {
        cl->size--;
    }
    return ret;
}
