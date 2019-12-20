/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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

#ifndef VB_ALIASES_H
#define VB_ALIASES_H

#define VB_NODESTR(config, index) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_PLAIN)
#define VB_RESTURL(config, index) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_MGMT, LCBVB_SVCMODE_PLAIN)
#define VB_VIEWSURL(config, index) lcbvb_get_capibase(config, index, LCBVB_SVCMODE_PLAIN)
#define VB_SSLNODESTR(config, index) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_DATA, LCBVB_SVCMODE_SSL)
#define VB_SSLRESTURL(config, index) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_MGMT, LCBVB_SVCMODE_SSL)
#define VB_SSLVIEWSURL(config, index) lcbvb_get_capibase(config, index, LCBVB_SVCMODE_SSL)
#define VB_MEMDSTR(config, index, mode) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_DATA, mode)
#define VB_MGMTSTR(config, index, mode) lcbvb_get_hostport(config, index, LCBVB_SVCTYPE_MGMT, mode)
#define VB_CAPIURL(config, index, mode) lcbvb_get_capibase(config, index, mode)

#define VB_DISTTYPE(config) (config)->dtype
#define VB_NREPLICAS(config) (config)->nrepl
#define VB_NSERVERS(config) (config)->nsrv

#endif
