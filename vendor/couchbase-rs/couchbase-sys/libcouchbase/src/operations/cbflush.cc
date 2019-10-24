/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#include <http/http.h>

static void flush_cb(lcb_INSTANCE *instance, int, const lcb_RESPBASE *rb)
{
    const lcb_RESPHTTP *resp = (const lcb_RESPHTTP *)rb;
    lcb_RESPCALLBACK callback = lcb_find_callback(instance, LCB_CALLBACK_CBFLUSH);

    const lcb_RESPCBFLUSH *iresp = (const lcb_RESPCBFLUSH *)rb;
    lcb_RESPCBFLUSH fresp = {0};
    fresp = *iresp;
    fresp.rflags |= LCB_RESP_F_FINAL;
    if (resp->rc == LCB_SUCCESS) {
        if (resp->htstatus < 200 || resp->htstatus > 299) {
            fresp.rc = LCB_HTTP_ERROR;
        }
    }
    if (callback) {
        callback(instance, LCB_CALLBACK_CBFLUSH, (lcb_RESPBASE *)&fresp);
    }
}

LIBCOUCHBASE_API
lcb_STATUS lcb_cbflush3(lcb_INSTANCE *instance, void *cookie, const lcb_CMDCBFLUSH *)
{
    lcb_HTTP_HANDLE *htr;
    lcb_STATUS rc;

    std::string urlpath("/pools/default/buckets/");
    urlpath.append(LCBT_SETTING(instance, bucket));
    urlpath.append("/controller/doFlush");

    lcb_CMDHTTP *htcmd;
    lcb_cmdhttp_create(&htcmd, LCB_HTTP_TYPE_MANAGEMENT);
    lcb_cmdhttp_method(htcmd, LCB_HTTP_METHOD_POST);
    lcb_cmdhttp_handle(htcmd, &htr);
    lcb_cmdhttp_path(htcmd, urlpath.c_str(), urlpath.size());

    rc = lcb_http(instance, cookie, htcmd);
    lcb_cmdhttp_destroy(htcmd);

    if (rc != LCB_SUCCESS) {
        return rc;
    }
    htr->set_callback(flush_cb);
    return LCB_SUCCESS;
}
