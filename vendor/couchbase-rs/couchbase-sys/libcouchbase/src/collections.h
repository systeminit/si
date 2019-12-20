/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2019-2019 Couchbase, Inc.
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

#ifndef LCB_COLLECTIONS_H
#define LCB_COLLECTIONS_H

#ifdef __cplusplus
extern "C" {
#endif /** __cplusplus */

typedef lcb_STATUS (*lcb_COLLCACHE_CALLBACK)(uint32_t cid, lcb_INSTANCE *instance, void *cookie, const void *arg);
typedef lcb_STATUS (*lcb_COLLCACHE_ARG_CLONE)(const void *src, void **dst);
typedef lcb_STATUS (*lcb_COLLCACHE_ARG_DTOR)(void *arg);

lcb_STATUS collcache_exec(const char *scope, size_t nscope, const char *collection, size_t ncollection,
                          lcb_INSTANCE *instance, void *cookie, lcb_COLLCACHE_CALLBACK cb,
                          lcb_COLLCACHE_ARG_CLONE clone, lcb_COLLCACHE_ARG_DTOR dtor, const void *arg);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
#include <map>
#include <string>

lcb_STATUS collcache_exec_str(std::string collection, lcb_INSTANCE *instance, void *cookie, lcb_COLLCACHE_CALLBACK cb,
                              lcb_COLLCACHE_ARG_CLONE clone, lcb_COLLCACHE_ARG_DTOR dtor, const void *arg);

namespace lcb
{
class CollectionCache
{
    std::map< std::string, uint32_t > cache_n2i;
    std::map< uint32_t, std::string > cache_i2n;

  public:
    CollectionCache();

    ~CollectionCache();

    bool get(std::string path, uint32_t *cid);

    void put(std::string path, uint32_t cid);

    std::string id_to_name(uint32_t cid);

    void erase(uint32_t cid);
};
} // namespace lcb
typedef lcb::CollectionCache lcb_COLLCACHE;
#else
typedef struct lcb_CollectionCache_st lcb_COLLCACHE;
#endif

#endif
