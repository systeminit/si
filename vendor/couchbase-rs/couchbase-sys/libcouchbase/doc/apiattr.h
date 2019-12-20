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

/**
 * @page lcb_attributes Interface Attributes
 *
 * Libcouchbase tries to follow the _release early, release often_
 * philosophy. As part of that we want people to be able to try out new
 * interfaces before we "stick" to them. Unfortunately this means that we
 * discover that we need to change them in incompatible ways.
 *
 * To aid developers to make reasonable decisions on how likely we are to
 * change these interfaces all functions in libcouchbase have an
 * associated Interface Stability tag. If you find undocumented structs,
 * functions or files you should *not* use them. They may be changed in
 * incompatible ways without any notice. Unless explicitly noted the
 * interface stability applies to both source code and binaries.
 *
 * This is a standard definition across all official Couchbase Server SDKs.
 * @see https://developer.couchbase.com/documentation/server/current/sdk/c/compatibility-versions-features.html
 *
 * The following classifications exist:
 *
 * ### Committed
 *
 * A committed interface is the highest grade of stability, and is the
 * preferred attribute level for consumers of the library. Couchbase
 * tries at best effort to preseve committed interfaces between major
 * versions of libcouchbase, and changes to committed interfaces within a
 * major version is highly exceptional. Such exceptions may include
 * situations where the interface may lead to data corruption, security
 * holes etc.
 *
 * _This is the default interface level for an API, unless the API is specifically
 * marked otherwise._
 *
 * ### Uncommitted
 *
 * No commitment is made about the interface (in binary or source
 * form). It may be changed in incompatible ways and dropped from one
 * release to another. The difference between an uncommitted interface
 * and a volatile interface is its maturity and likelyhood of being
 * changed. Uncommitted interfaces may mature into committed interfaces.
 *
 * ### Volatile
 *
 * Volatile interfaces can change at any time and for any reason.
 *
 * Interfaces may be volatile for reasons including:
 *
 * * Interface depends on specific implementation detail within the library
 *   which may change in the future.
 *
 * * Interface depends on specific implementation detail within the server
 *   which may change in the future.
 *
 * * Interface has been introduced as part of a trial phase for the specific
 *   feature.
 *
 * ### Deprecated
 *
 * The interface is subject to be removed from future versions of
 * libcouchbase. Interfaces may be deprecated for a variety of reasons, such
 * as specific bugs found within the API itself, or a more uniform and/or
 * direct method of achieving the same goal.
 *
 * ### Internal
 *
 * Internal interfaces are used internally in libcouchbase and should not
 * be used elsewhere. Doing so may cause libcouchbase to misbehave.
 *
 * Unless otherwise noted, any API not found in the <include/libcouchbase>
 * directory is considered to be internal.
 *
 * Some earlier versions of libcouchbase may have listed APIs as private,
 * when the intended term in the taxonomy is internal.
 *
 * The listing of interfaces may be found here:
 *
 * * @subpage lcb_apiattr_committed
 * * @subpage lcb_apiattr_uncommitted
 * * @subpage lcb_apiattr_volatile
 * * @subpage deprecated
 * * @subpage lcb_apiattr_internal
 **/

/**
 * @page lcb_apiattr_committed Committed Interfaces
 * @see @ref lcb_attributes
 *
 * @page lcb_apiattr_uncommitted Uncomitted Interfaces
 * @see @ref lcb_attributes
 *
 * @page lcb_apiattr_volatile Volatile Interfaces
 * @see @ref lcb_attributes
 *
 * @page deprecated Deprecated Interfaces
 * @see @ref lcb_attributes

 * @page lcb_apiattr_internal Internal Interfaces
 * @see @ref lcb_attributes
 *
 */
