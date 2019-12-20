#include <gtest/gtest.h>
#include "sllist.h"
#include "sllist-inl.h"
#include <list>
#include <stdexcept>

#ifndef ASSERT_NZ
#define ASSERT_NZ(e) ASSERT_NE(0, e)
#endif

#ifndef ASSERT_Z
#define ASSERT_Z(e) ASSERT_EQ(0, e)
#endif

class SListTests : public ::testing::Test
{
};

struct my_elem {
    int value;
    sllist_node slnode;
};

TEST_F(SListTests, testBasic)
{
    sllist_root sl;
    memset(&sl, 0, sizeof(sl));

    ASSERT_TRUE(SLLIST_IS_EMPTY(&sl));
    my_elem elem1, elem2, elem3;

    sllist_append(&sl, &elem1.slnode);
    ASSERT_NZ(sllist_contains(&sl, &elem1.slnode));
    ASSERT_FALSE(SLLIST_IS_EMPTY(&sl));

    sllist_node *tmpnode = SLLIST_FIRST(&sl);
    sllist_remove_head(&sl);
    ASSERT_NE(tmpnode, SLLIST_FIRST(&sl));
    ASSERT_EQ(tmpnode, &elem1.slnode);
    ASSERT_EQ(&elem1, SLLIST_ITEM(tmpnode, struct my_elem, slnode));
    ASSERT_TRUE(SLLIST_IS_EMPTY(&sl));

    sllist_append(&sl, &elem1.slnode);
    sllist_append(&sl, &elem2.slnode);
    sllist_append(&sl, &elem3.slnode);
    ASSERT_EQ(sl.last, &elem3.slnode);
    ASSERT_EQ(SLLIST_FIRST(&sl), &elem1.slnode);

    // Test prepend
    my_elem elem4;
    sllist_prepend(&sl, &elem4.slnode);
    tmpnode = SLLIST_FIRST(&sl);
    ASSERT_EQ(tmpnode, &elem4.slnode);
    sllist_node *cur;
    int itercount = 0;
    SLLIST_ITERBASIC(&sl, cur)
    {
        itercount++;
    }
    ASSERT_EQ(4, itercount);
}

#define BASIC_NELEM 3
TEST_F(SListTests, testBasicIter)
{
    sllist_root sl;
    my_elem elems[BASIC_NELEM];

    memset(&sl, 0, sizeof(sl));
    memset(elems, 0, sizeof(elems));

    for (int ii = 0; ii < BASIC_NELEM; ii++) {
        sllist_append(&sl, &elems[ii].slnode);
    }

    sllist_node *cur;
    int itercount = 0;
    SLLIST_ITERBASIC(&sl, cur)
    {
        my_elem *elem = SLLIST_ITEM(cur, struct my_elem, slnode);
        itercount++;
        elem->value++;
    }

    ASSERT_EQ(BASIC_NELEM, itercount);
    for (int ii = 0; ii < BASIC_NELEM; ii++) {
        ASSERT_EQ(1, elems[ii].value);
    }
}

static void fillDynamicSlist(sllist_root *root, my_elem **ptrs, int nptrs)
{
    sllist_iterator iter;
    SLLIST_ITERFOR(root, &iter)
    {
        sllist_iter_remove(root, &iter);
    }

    for (int ii = 0; ii < nptrs; ii++) {
        free(ptrs[ii]);
        ptrs[ii] = (my_elem *)calloc(1, sizeof(*ptrs[ii]));
        sllist_append(root, &ptrs[ii]->slnode);
    }
}

TEST_F(SListTests, testExtendedIter)
{
    sllist_root sl;
    my_elem *elemp[BASIC_NELEM] = {NULL};
    memset(&sl, 0, sizeof(sl));

    fillDynamicSlist(&sl, elemp, BASIC_NELEM);

    // Delete all elements from the list
    sllist_iterator iter;
    SLLIST_ITERFOR(&sl, &iter)
    {
        my_elem *elem = SLLIST_ITEM(iter.cur, struct my_elem, slnode);
        sllist_iter_remove(&sl, &iter);
        memset(elem, 0xff, sizeof(*elem));
        free(elem);
    }

    ASSERT_TRUE(SLLIST_IS_EMPTY(&sl));
    memset(elemp, 0, sizeof(*elemp) * BASIC_NELEM);

    // Delete only the first element of the list. Repopulate
    fillDynamicSlist(&sl, elemp, BASIC_NELEM);
    SLLIST_ITERFOR(&sl, &iter)
    {
        my_elem *elem = SLLIST_ITEM(iter.cur, struct my_elem, slnode);
        if (elem == elemp[0]) {
            sllist_iter_remove(&sl, &iter);
            memset(elem, 0xff, sizeof(*elem));
            free(elem);
            elemp[0] = NULL;
        }
    }

    int itercount = 0;
    SLLIST_ITERFOR(&sl, &iter)
    {
        sllist_iter_remove(&sl, &iter);
        itercount++;
    }
    ASSERT_EQ(BASIC_NELEM - 1, itercount);
    ASSERT_TRUE(SLLIST_IS_EMPTY(&sl));

    // Delete only the middle element
    fillDynamicSlist(&sl, elemp, BASIC_NELEM);
    SLLIST_ITERFOR(&sl, &iter)
    {
        my_elem *elem = SLLIST_ITEM(iter.cur, struct my_elem, slnode);
        if (elem == elemp[1]) {
            sllist_iter_remove(&sl, &iter);
            memset(elem, 0xff, sizeof(*elem));
            free(elem);
            elemp[1] = NULL;
        }
    }
    ASSERT_FALSE(SLLIST_IS_EMPTY(&sl));

    // Delete only the last element
    fillDynamicSlist(&sl, elemp, BASIC_NELEM);
    SLLIST_ITERFOR(&sl, &iter)
    {
        my_elem *elem = SLLIST_ITEM(iter.cur, struct my_elem, slnode);
        if (elem == elemp[BASIC_NELEM - 1]) {
            sllist_iter_remove(&sl, &iter);
            memset(elem, 0xff, sizeof(*elem));
            free(elem);
            elemp[BASIC_NELEM - 1] = NULL;
        }
    }
    ASSERT_FALSE(SLLIST_IS_EMPTY(&sl));
    SLLIST_ITERFOR(&sl, &iter)
    {
        my_elem *elem = SLLIST_ITEM(iter.cur, struct my_elem, slnode);
        sllist_iter_remove(&sl, &iter);
        free(elem);
    }
}

struct NumberedItem {
    sllist_node slnode;
    int value;
};
static int ni_compare(sllist_node *a, sllist_node *b)
{
    NumberedItem *na = SLLIST_ITEM(a, NumberedItem, slnode);
    NumberedItem *nb = SLLIST_ITEM(b, NumberedItem, slnode);
    return na->value - nb->value;
}
TEST_F(SListTests, testSort)
{
    sllist_root l;
    memset(&l, 0, sizeof(l));
    NumberedItem items[10];
    for (unsigned ii = 0; ii < 10; ii++) {
        items[ii].value = ii;
        sllist_insert_sorted(&l, &items[ii].slnode, ni_compare);
    }

    int last = -1;
    sllist_node *cur;
    SLLIST_FOREACH(&l, cur)
    {
        NumberedItem *ni = SLLIST_ITEM(cur, NumberedItem, slnode);
        ASSERT_EQ(last, ni->value - 1);
        last = ni->value;
    }

    /** Insert another item */
    NumberedItem big1;
    big1.value = 100;
    sllist_insert_sorted(&l, &big1.slnode, ni_compare);
    ASSERT_EQ(l.last, &big1.slnode);

    NumberedItem small1;
    small1.value = -100;
    sllist_insert_sorted(&l, &small1.slnode, ni_compare);
    ASSERT_EQ(SLLIST_FIRST(&l), &small1.slnode);

    NumberedItem middle1;
    middle1.value = 5;
    sllist_insert_sorted(&l, &middle1.slnode, ni_compare);
    NumberedItem *ni_next = SLLIST_ITEM(middle1.slnode.next, NumberedItem, slnode);
    ASSERT_EQ(5, ni_next->value);

    ni_next = SLLIST_ITEM(items[3].slnode.next, NumberedItem, slnode);
    ASSERT_EQ(&middle1.slnode, ni_next->slnode.next);
}

template < typename T, sllist_node T::*m = &T::slnode > class SList : public sllist_root
{
  public:
    SList()
    {
        clear();
    };

    void append(T &memb)
    {
        sllist_append(this, getNode(memb));
    }

    void prepend(T &memb)
    {
        sllist_prepend(this, getNode(memb));
    }

    void insert(T &memb, int (*compar)(sllist_node *, sllist_node *))
    {
        sllist_insert_sorted(this, getNode(memb), compar);
    }

    bool contains(T &memb)
    {
        return sllist_contains(this, getNode(memb)) != 0;
    }

    size_t size()
    {
        return sllist_get_size(this);
    }

    void remove(T &memb)
    {
        sllist_remove(this, getNode(memb));
    }

    bool empty()
    {
        return SLLIST_IS_EMPTY(this);
    }

    void clear()
    {
        SLLIST_FIRST(this) = NULL;
        last = NULL;
    }

    T &front()
    {
        if (empty()) {
            throw std::out_of_range("List is empty");
        }
        return *llToItem(SLLIST_FIRST(this));
    }

    T &back()
    {
        if (empty()) {
            throw std::out_of_range("List is empty");
        }
        return *llToItem(last);
    }

    T &operator[](int ix)
    {
        int cur = 0;
        sllist_node *ll;
        SLLIST_FOREACH(this, ll)
        {
            if (cur++ == ix) {
                return *llToItem(ll);
            }
        }
        throw std::out_of_range("No such index");
    }

  private:
    sllist_node *getNode(T &memb) const
    {
        return &(memb.*m);
    }

    T *llToItem(sllist_node *node)
    {
        // Get the offset
        T *obj = 0;
        sllist_node *offset = &(obj->*m);
        char *diff = (char *)offset;
        return (T *)(((char *)node) - diff);
    }
};

TEST_F(SListTests, testSandwichSort)
{
    // moar sort tests
    SList< NumberedItem > sl;
    NumberedItem itm_1, itm_2, itm_3;
    itm_1.value = 1;
    itm_2.value = 2;
    itm_3.value = 3;

    // 1, 3, 2
    sl.insert(itm_1, ni_compare);
    ASSERT_TRUE(sl.contains(itm_1));
    ASSERT_EQ(1, sl.size());
    ASSERT_EQ(1, sl[0].value);

    sl.insert(itm_3, ni_compare);
    ASSERT_TRUE(sl.contains(itm_3));
    ASSERT_EQ(1, sl[0].value);
    ASSERT_EQ(3, sl[1].value);

    sl.insert(itm_2, ni_compare);
    ASSERT_TRUE(sl.contains(itm_2));
    ASSERT_EQ(3, sl.size());
    ASSERT_EQ(1, sl[0].value);
    ASSERT_EQ(2, sl[1].value);
    ASSERT_EQ(3, sl[2].value);

    sl.clear();
    // Insert 3,2,1
    sl.insert(itm_3, ni_compare);
    sl.insert(itm_2, ni_compare);
    sl.insert(itm_1, ni_compare);
    ASSERT_EQ(1, sl[0].value);
    ASSERT_EQ(2, sl[1].value);
    ASSERT_EQ(3, sl[2].value);
}

TEST_F(SListTests, testPrependSort)
{
    SList< NumberedItem > sl;
    NumberedItem itm_1, itm_2, itm_3;
    itm_1.value = 1;
    itm_2.value = 2;
    itm_3.value = 3;

    // 2, 3, 1
    sl.insert(itm_2, ni_compare);
    sl.insert(itm_3, ni_compare);
    ASSERT_EQ(2, sl.size());
    ASSERT_EQ(2, sl[0].value);
    ASSERT_EQ(3, sl[1].value);

    // Prepend item 1
    sl.insert(itm_1, ni_compare);
    ASSERT_EQ(3, sl.size());
    ASSERT_EQ(1, sl[0].value);
    ASSERT_EQ(2, sl[1].value);
    ASSERT_EQ(3, sl[2].value);
}

// Ensure removing the tail item inside an iterator does what we want it to
TEST_F(SListTests, testRemoveTailIter)
{
    NumberedItem itm_1, itm_2, itm_3;
    itm_1.value = 1;
    itm_2.value = 2;
    itm_3.value = 3;
    SList< NumberedItem > sl;
    sl.append(itm_1);
    sl.append(itm_2);
    sl.append(itm_3);

    ASSERT_EQ(1, sl.front().value);
    ASSERT_EQ(3, sl.back().value);

    sllist_iterator iter;
    bool removed = false;
    SLLIST_ITERFOR(&sl, &iter)
    {
        if (iter.cur == &itm_3.slnode) {
            sllist_iter_remove(&sl, &iter);
            removed = true;
            break;
        }
    }

    ASSERT_TRUE(removed);
    ASSERT_EQ(2, sl.size());
    ASSERT_EQ(1, sl.front().value);
    ASSERT_EQ(2, sl.back().value);
}

TEST_F(SListTests, testRemoveEmptyTailIter)
{
    NumberedItem itm_1;
    SList< NumberedItem > sl;
    sl.append(itm_1);
    sllist_iterator iter;
    SLLIST_ITERFOR(&sl, &iter)
    {
        sllist_iter_remove(&sl, &iter);
    }
    ASSERT_TRUE(sl.empty());
}

TEST_F(SListTests, testRemoveFirstIter)
{
    NumberedItem itm_1, itm_2, itm_3;
    SList< NumberedItem > sl;
    itm_1.value = 1;
    itm_2.value = 2;
    itm_3.value = 3;
    sl.append(itm_1);
    sl.append(itm_2);
    sl.append(itm_3);
    sllist_iterator iter;
    SLLIST_ITERFOR(&sl, &iter)
    {
        if (iter.cur == &itm_1.slnode) {
            sllist_iter_remove(&sl, &iter);
        }
    }

    ASSERT_EQ(2, sl.size());
    ASSERT_EQ(2, sl.front().value);
    ASSERT_EQ(3, sl.back().value);
}
