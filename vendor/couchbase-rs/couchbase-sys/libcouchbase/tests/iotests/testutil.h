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
#ifndef TESTS_TESTUTIL_H
#define TESTS_TESTUTIL_H 1

#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include <string.h>
struct Item {
    void assign(const lcb_RESPGET *resp)
    {
        err = lcb_respget_status(resp);

        const char *p;
        size_t n;

        lcb_respget_key(resp, &p, &n);
        key.assign(p, n);
        lcb_respget_value(resp, &p, &n);
        val.assign(p, n);
        lcb_respget_flags(resp, &flags);
        lcb_respget_cas(resp, &cas);
        lcb_respget_datatype(resp, &datatype);
    }

    void assign(const lcb_RESPSTORE *resp)
    {
        err = lcb_respstore_status(resp);

        const char *p;
        size_t n;

        lcb_respstore_key(resp, &p, &n);
        key.assign(p, n);
        lcb_respstore_cas(resp, &cas);
    }

    void assign(const lcb_RESPREMOVE *resp)
    {
        err = lcb_respremove_status(resp);

        const char *p;
        size_t n;

        lcb_respremove_key(resp, &p, &n);
        key.assign(p, n);
        lcb_respremove_cas(resp, &cas);
    }

    /**
     * Extract the key and CAS from a response.
     */
    template < typename T > void assignKC(const T *resp)
    {
        key.assign((const char *)resp->key, resp->nkey);
        cas = resp->cas;
        err = resp->rc;
    }

    Item()
    {
        key.clear();
        val.clear();

        err = LCB_SUCCESS;
        flags = 0;
        cas = 0;
        datatype = 0;
        exp = 0;
    }

    Item(const std::string &key, const std::string &value = "", lcb_cas_t cas = 0)
    {

        this->key = key;
        this->val = value;
        this->cas = cas;

        flags = 0;
        datatype = 0;
        exp = 0;
    }

    friend std::ostream &operator<<(std::ostream &out, const Item &item);

    /**
     * Dump the string representation of the item to standard output
     */
    void dump()
    {
        std::cout << *this;
    }

    std::string key;
    std::string val;
    lcb_uint32_t flags;
    lcb_cas_t cas;
    lcb_datatype_t datatype;
    lcb_STATUS err;
    lcb_time_t exp;
};

struct KVOperation {
    /** The resultant item */
    Item result;

    /** The request item */
    const Item *request;

    /** whether the callback was at all received */
    unsigned callCount;

    /** Acceptable errors during callback */
    std::set< lcb_STATUS > allowableErrors;

    /** Errors received from error handler */
    std::set< lcb_STATUS > globalErrors;

    void assertOk(lcb_STATUS err);

    KVOperation(const Item *request)
    {
        this->request = request;
        this->ignoreErrors = false;
        callCount = 0;
    }

    void clear()
    {
        result = Item();
        callCount = 0;
        allowableErrors.clear();
        globalErrors.clear();
    }

    void store(lcb_INSTANCE *instance);
    void get(lcb_INSTANCE *instance);
    void remove(lcb_INSTANCE *instance);

    void cbCommon(lcb_STATUS error)
    {
        callCount++;
        if (error != LCB_SUCCESS) {
            globalErrors.insert(error);
        }
        assertOk(error);
    }

    static void handleInstanceError(lcb_INSTANCE *, lcb_STATUS, const char *);
    bool ignoreErrors;

  private:
    void enter(lcb_INSTANCE *);
    void leave(lcb_INSTANCE *);
    const void *oldCookie;

    struct {
        lcb_RESPCALLBACK get;
        lcb_RESPCALLBACK store;
        lcb_RESPCALLBACK rm;
    } callbacks;
};

void storeKey(lcb_INSTANCE *instance, const std::string &key, const std::string &value);
void removeKey(lcb_INSTANCE *instance, const std::string &key);
void getKey(lcb_INSTANCE *instance, const std::string &key, Item &item);

/**
 * Generate keys which will trigger all the servers in the map.
 */
void genDistKeys(lcbvb_CONFIG *vbc, std::vector< std::string > &out);
void genStoreCommands(const std::vector< std::string > &keys, std::vector< lcb_CMDSTORE * > &cmds);
/**
 * This doesn't _actually_ attempt to make sense of an operation. It simply
 * will try to keep the event loop alive.
 */
void doDummyOp(lcb_INSTANCE *instance);

#endif
