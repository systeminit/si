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

class CCBC_103 : public ::testing::Test
{
};

typedef struct {
    lcb_list_t list;
} event_t;

typedef struct {
    event_t events;
} io_cookie;

TEST_F(CCBC_103, lists)
{
    io_cookie instance;
    event_t e1, e2, e3, e4;

    lcb_list_init(&instance.events.list);
    lcb_list_append(&instance.events.list, &e1.list);
    ASSERT_EQ(&e1.list, instance.events.list.prev);

    lcb_list_append(&instance.events.list, &e2.list);
    ASSERT_EQ(&e2.list, instance.events.list.prev);

    lcb_list_append(&instance.events.list, &e3.list);
    ASSERT_EQ(&e3.list, instance.events.list.prev);

    lcb_list_append(&instance.events.list, &e4.list);
    ASSERT_EQ(&e4.list, instance.events.list.prev);

    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e4.list));

    // Try to unlink the one in the middle
    lcb_list_delete(&e2.list);
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e4.list));

    // Try to unlink the last one
    lcb_list_delete(&e1.list);
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e4.list));

    // try to unlink the current head
    lcb_list_delete(&e4.list);
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e4.list));

    // try to unlink the last one
    lcb_list_delete(&e3.list);
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(0, lcb_list_contains(&instance.events.list, &e4.list));

    // And we should be able to add all back
    lcb_list_append(&instance.events.list, &e1.list);
    lcb_list_append(&instance.events.list, &e2.list);
    lcb_list_append(&instance.events.list, &e3.list);
    lcb_list_append(&instance.events.list, &e4.list);
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e1.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e2.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e3.list));
    ASSERT_EQ(1, lcb_list_contains(&instance.events.list, &e4.list));
}
