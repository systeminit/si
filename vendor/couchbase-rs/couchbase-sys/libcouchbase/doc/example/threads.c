/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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

#include <pthread.h>
#include <libcouchbase/couchbase.h>

typedef struct {
    lcb_INSTANCE *instance;
    pthread_mutex_t mutex;
} my_CTX;

/*
 * This function uses the same instance between threads. A lock
 * is required for every operation
 */
static void *
thrfunc_locked(void *arg)
{
    my_CTX *ctx = arg;
    lcb_CMDGET cmd = { 0 };
    LCB_CMD_SET_KEY(&cmd, "Hello", strlen("Hello"));

    pthread_mutex_lock(&ctx->mutex);
    lcb_get3(ctx->instance, NULL, &cmd);
    lcb_wait(ctx->instance);
    pthread_mutex_unlock(&ctx->mutex);
    return NULL;
}

/*
 * This function uses an instance per thread. Since no other thread
 * is using the instance, locking is not required
 */
static void *
thrfunc_unlocked(void *arg)
{
    lcb_INSTANCE *instance;
    lcb_create(&instance, NULL);
    lcb_connect(instance);
    lcb_wait(instance);
    LCB_CMDGET cmd = { 0 };
    lcb_get3(instance, NULL, &cmd);
    lcb_wait(instance);
    lcb_destroy(instance);
    return NULL;
}

int main(void)
{
    pthread_t thrs[10];
    my_CTX ctx;
    int ii;

    lcb_create(&ctx.instance, NULL);
    lcb_connect(ctx.instance);
    lcb_wait(ctx.instance);
    pthread_mutex_init(&ctx.mutex, NULL);

    for (ii = 0; ii < 10; ii++) {
        pthread_create(&thrs[ii], NULL, thrfunc_locked, &ctx);
    }

    for (ii = 0; ii < 10; ii++) {
        void *ign;
        pthread_join(thrs[ii], &ign);
    }

    lcb_destroy(ctx.instance);
    pthread_mutex_destroy(&ctx.mutex);

    for (ii = 0; ii < 10; ii++) {
        pthread_create(&thrs[ii], NULL, thrfunc_unlocked, NULL);
    }
    for (ii = 0; ii < 10; ii++) {
        void *ign;
        pthread_join(thrs[ii], &ign);
    }
    return 0;
}
