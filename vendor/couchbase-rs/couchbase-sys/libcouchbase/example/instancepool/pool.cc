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

#include "pool.h"

using namespace lcb;
using std::queue;
using std::vector;

Pool::Pool(const lcb_create_st &options, size_t nitems) : initial_size(0)
{
    pthread_mutex_init(&mutex, NULL);
    pthread_cond_init(&cond, NULL);
    for (size_t ii = 0; ii < nitems; ii++) {
        lcb_INSTANCE *cur;
        lcb_STATUS err = lcb_create(&cur, &options);
        if (err != LCB_SUCCESS) {
            throw err;
        }

        instances.push(cur);
        all_instances.push_back(cur);
        initial_size++;
    }
}

lcb_STATUS Pool::connect()
{
    vector< lcb_INSTANCE * >::const_iterator ii = all_instances.begin();
    for (; ii != all_instances.end(); ii++) {
        lcb_STATUS err;
        initialize(*ii);
        if ((err = lcb_connect(*ii)) != LCB_SUCCESS) {
            return err;
        }
        lcb_wait(*ii);
        if ((err = lcb_get_bootstrap_status(*ii)) != LCB_SUCCESS) {
            return err;
        }
    }
    return LCB_SUCCESS;
}

Pool::~Pool()
{
    pthread_mutex_lock(&mutex);
    while (instances.size() < initial_size) {
        pthread_cond_wait(&cond, &mutex);
    }
    vector< lcb_INSTANCE * >::const_iterator ii = all_instances.begin();
    for (; ii != all_instances.end(); ii++) {
        lcb_destroy(*ii);
    }
    pthread_mutex_unlock(&mutex);
    pthread_mutex_destroy(&mutex);
    pthread_cond_destroy(&cond);
}

lcb_INSTANCE *Pool::pop()
{
    lcb_INSTANCE *ret = NULL;

    // Need to lock the mutex to the pool structure itself
    pthread_mutex_lock(&mutex);

    while (instances.empty()) {
        pthread_cond_wait(&cond, &mutex);
    }

    ret = instances.front();
    instances.pop();
    pthread_mutex_unlock(&mutex);

    // Note that the instance itself does not need a mutex as long as it is not
    // used between multiple threads concurrently.
    return ret;
}

void Pool::push(lcb_INSTANCE *instance)
{
    pthread_mutex_lock(&mutex);
    instances.push(instance);
    pthread_cond_signal(&cond);
    pthread_mutex_unlock(&mutex);
}
