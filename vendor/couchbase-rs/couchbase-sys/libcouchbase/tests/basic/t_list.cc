/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#include "config.h"
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include "list.h"

typedef struct {
    lcb_list_t list;
    const char *desc;
} todo_t;

class List : public ::testing::Test
{
};

TEST_F(List, basicTests)
{
    todo_t root;

    memset(&root, 0, sizeof(todo_t));
    lcb_list_init(&root.list);
    EXPECT_EQ(&root.list, root.list.next);
    EXPECT_EQ(&root.list, root.list.prev);

    todo_t t0 = {{NULL, NULL}, "break"};
    lcb_list_append(&root.list, &t0.list);
    EXPECT_EQ(&t0.list, root.list.next);
    EXPECT_EQ(&t0.list, root.list.prev);

    lcb_list_delete(&t0.list);
    EXPECT_EQ(NULL, t0.list.next);
    EXPECT_EQ(NULL, t0.list.prev);
    EXPECT_EQ(&root.list, root.list.next);
    EXPECT_EQ(&root.list, root.list.prev);

    todo_t t1 = {{NULL, NULL}, "write"};
    EXPECT_STREQ("write", t1.desc);
    lcb_list_append(&root.list, &t1.list);
    EXPECT_EQ(&t1.list, root.list.next);
    EXPECT_EQ(&t1.list, root.list.prev);

    todo_t t2 = {{NULL, NULL}, "test"};
    EXPECT_STREQ("test", t2.desc);
    lcb_list_append(&root.list, &t2.list);
    EXPECT_EQ(&t1.list, root.list.next);
    EXPECT_EQ(&t2.list, root.list.prev);

    todo_t t3 = {{NULL, NULL}, "refactor"};
    EXPECT_STREQ("refactor", t3.desc);
    lcb_list_append(&root.list, &t3.list);
    EXPECT_EQ(&t1.list, root.list.next);
    EXPECT_EQ(&t3.list, root.list.prev);

    todo_t t4 = {{NULL, NULL}, "read"};
    EXPECT_STREQ("read", t4.desc);
    lcb_list_prepend(&root.list, &t4.list);
    EXPECT_EQ(&t4.list, root.list.next);
    EXPECT_EQ(&t3.list, root.list.prev);

    lcb_list_t *ii = root.list.next;
    todo_t *tt;

    tt = LCB_LIST_ITEM(ii, todo_t, list);
    EXPECT_STREQ("read", tt->desc);
    ii = ii->next;
    tt = LCB_LIST_ITEM(ii, todo_t, list);
    EXPECT_STREQ("write", tt->desc);
    ii = ii->next;
    tt = LCB_LIST_ITEM(ii, todo_t, list);
    EXPECT_STREQ("test", tt->desc);
    ii = ii->next;
    tt = LCB_LIST_ITEM(ii, todo_t, list);
    EXPECT_STREQ("refactor", tt->desc);

    lcb_list_t *nn;
    LCB_LIST_SAFE_FOR(ii, nn, &root.list)
    {
        tt = LCB_LIST_ITEM(ii, todo_t, list);
        lcb_list_delete(&tt->list);
        memset(tt, 0, sizeof(todo_t));
    }
    EXPECT_EQ(&root.list, root.list.next);
    EXPECT_EQ(&root.list, root.list.prev);
}

typedef struct {
    lcb_list_t list;
    int number;
} num_t;

int ascending(lcb_list_t *a, lcb_list_t *b)
{
    num_t *aa, *bb;

    aa = LCB_LIST_ITEM(a, num_t, list);
    bb = LCB_LIST_ITEM(b, num_t, list);
    if (aa->number > bb->number) {
        return 1;
    } else if (aa->number < bb->number) {
        return -1;
    } else {
        return 0;
    }
}

TEST_F(List, sortedListTest)
{
    num_t root;

    memset(&root, 0, sizeof(num_t));
    lcb_list_init(&root.list);

    num_t n0 = {{NULL, NULL}, 0};
    lcb_list_add_sorted(&root.list, &n0.list, ascending);
    num_t n3 = {{NULL, NULL}, 3};
    lcb_list_add_sorted(&root.list, &n3.list, ascending);
    num_t n2 = {{NULL, NULL}, 2};
    lcb_list_add_sorted(&root.list, &n2.list, ascending);
    num_t n7 = {{NULL, NULL}, 7};
    lcb_list_add_sorted(&root.list, &n7.list, ascending);
    num_t n1 = {{NULL, NULL}, 1};
    lcb_list_add_sorted(&root.list, &n1.list, ascending);

    lcb_list_t *ii = root.list.next;
    num_t *nn;
    nn = LCB_LIST_ITEM(ii, num_t, list);
    EXPECT_EQ(0, nn->number);
    ii = ii->next;
    nn = LCB_LIST_ITEM(ii, num_t, list);
    EXPECT_EQ(1, nn->number);
    ii = ii->next;
    nn = LCB_LIST_ITEM(ii, num_t, list);
    EXPECT_EQ(2, nn->number);
    ii = ii->next;
    nn = LCB_LIST_ITEM(ii, num_t, list);
    EXPECT_EQ(3, nn->number);
    ii = ii->next;
    nn = LCB_LIST_ITEM(ii, num_t, list);
    EXPECT_EQ(7, nn->number);
    ii = ii->next;
}
