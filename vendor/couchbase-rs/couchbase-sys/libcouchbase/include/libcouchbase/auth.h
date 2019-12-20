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

#ifndef LCB_AUTH_H
#define LCB_AUTH_H

/**
 * @file
 * Credentials store for Couchbase
 */

#ifdef __cplusplus
namespace lcb
{
class Authenticator;
}
typedef lcb::Authenticator lcb_AUTHENTICATOR;
extern "C" {
#else /* C only! */
typedef struct lcb_AUTHENTICATOR_Cdummy lcb_AUTHENTICATOR;
#endif

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-auth Authentication
 *
 * @details
 *
 * The @ref lcb_AUTHENTICATOR object is how the library stores credentials
 * internally, and may be used in cases where you'd like to manage credentials
 * in an object separate from the library. This interface also provides better
 * clarification between 'old style' (Classic) and new style (RBAC) auth.
 *
 * If you don't have a specific need to have credentials managed in their own.
 * @ref lcb_create_st3::username and @ref lcb_create_st3::passwd fields (note
 * that `username` is only valid on clusters 5.0 and higher):
 *
 * @code{.c}
 * crst.v.v3.username = "user"; // Only for newer clusters
 * crst.v.v3.passwd = "s3cr3t";
 * lcb_create(&instance, &crst);
 * @endcode
 *
 * If you are connecting to a cluster older than 5.0 and would like to issue
 * N1QL queries against multiple password-protected buckets, you can use
 * the @ref LCB_CNTL_BUCKET_CRED setting to "add" more bucket:password pairs
 * to the library. The library will then send these credentials whenever you
 * issue a query with the @ref LCB_CMD_F_MULTIAUTH flag set.
 *
 * @code{.c}
 * lcb_BUCKETCRED creds;
 * creds[0] = "secondBucket";
 * creds[1] = "secondPass";
 * lcb_cntl(instance, LCB_CNTL_SET, LCB_CNTL_BUCKET_CRED, creds);
 * @endcode
 *
 * Or if you have a JSON encoder handy (or are interfacing from a higher level
 * language) you can use the lcb_cntl_string() variant:
 *
 * @code{.c}
 * JsonArray *arr = new_json_array();
 * json_array_push_string("secondBucket");
 * json_array_push_string("secondPass");
 * char *s = json_encode(arr);
 * lcb_cntl_string(instance, "bucket_cred", s);
 * @endcode
 *
 * The json functions in the above example are mockups of however you would
 * actually create a JSON array.
 *
 * @addtogroup lcb-auth
 * @{
 */

/**
 * @class lcb_AUTHENTICATOR
 * Opaque pointer containing credentials for the library.
 */

/**
 * @uncommitted
 *
 * Creates a new authenticator object. You may destroy it using lcbauth_unref().
 * The returned object initially has a refcount of 1.
 *
 * @return A new authenticator object.
 *
 * You must set the mode on this object before adding credentials to it. See
 * @ref lcbauth_set_mode().
 *
 * Once you have added all the credentials to the object, you may assign it
 * (or a copy, see lcbauth_clone()) to a library handle via lcb_set_auth().
 *
 * Setting RBAC Auth:
 *
 * @code{.c}
 * lcb_AUTHENTICATOR *auth = lcbauth_new();
 * lcbauth_set_mode(auth, LCBAUTH_MODE_RBAC);
 * lcbauth_add_pass(auth, "mark", "secret", LCBAUTH_F_CLUSTER);
 *
 * lcb_INSTANCE instance;
 * lcb_create_st crst = { 0 };
 * crst.version = 3;
 * crst.v.v3.connstr = "couchbase://cbhost.com/myBucket";
 * lcb_create(&instance, &crst);
 * lcb_set_auth(instance, auth);
 * lcbauth_unref(auth);
 * @endcode
 *
 * Setting multi-bucket classic auth, also with cluster administrative
 * credentials:
 *
 * @code{.c}
 * lcb_AUTHENTICATOR *auth = lcbauth_new();
 * lcbauth_set_mode(auth, LCBAUTH_MODE_CLASSIC);
 * lcbauth_add_pass(auth, "myBucket", "secret", LCBAUTH_F_BUCKET);
 * lcbauth_add_pass(auth, "otherBucket", "otherSecret", LCBAUTH_F_BUCKET);
 * lcbauth_add_pass(auth, "Administrator", "password", LCBAUTH_F_CLUSTER);
 * lcb_INSTANCE instance;
 * lcb_create_st crst = { 0 };
 * crst.version = 3;
 * crst.v.v3.connstr = "couchbase://cbhost.com/myBucket";
 * lcb_create(&instance, &crst);
 * lcb_set_auth(instance, auth);
 * lcbauth_unref(auth);
 * @endcode
 */
LIBCOUCHBASE_API
lcb_AUTHENTICATOR *lcbauth_new(void);

/**
 * Flags to use when adding a new set of credentials to lcbauth_add_pass
 */
typedef enum {
    /** User/Password is administrative; for cluster */
    LCBAUTH_F_CLUSTER = 1 << 1,

    /**
     * User is bucket name. Password is bucket password. This flag is only
     * used for legacy authentication. Using it with RBAC authentication will
     * return an error
     */
    LCBAUTH_F_BUCKET = 1 << 2
} lcbauth_ADDPASSFLAGS;

/**
 * @uncommitted
 *
 * Add a set of credentials
 * @param auth
 * @param user the username (or bucketname, if LCBAUTH_F_BUCKET is passed)
 * @param pass the password. If the password is NULL, the credential is removed
 * @param flags one of @ref LCBAUTH_F_CLUSTER or @ref LCBAUTH_F_BUCKET. If both
 * flags are combined then the credential will be used for both bucket-level
 * and cluster-level administrative operations
 * (using @ref LCB_HTTP_TYPE_MANAGEMENT).
 * @return LCB_OPTIONS_CONFLICT if @ref LCBAUTH_F_BUCKET is used in conjunction
 * with @ref LCBAUTH_MODE_RBAC.
 *
 * @note
 * You must set the mode of the authenticator using @ref lcbauth_set_mode()
 * before calling this function
 *
 * @note when using @ref LCBAUTH_MODE_RBAC, only @ref LCBAUTH_F_CLUSTER is
 * supported.
 */
LIBCOUCHBASE_API
lcb_STATUS lcbauth_add_pass(lcb_AUTHENTICATOR *auth, const char *user, const char *pass, int flags);

/**
 * @volatile
 *
 * Increments the refcount on the authenticator object
 * @param auth
 *
 * The only time you would want to call this function is when sharing a single
 * @ref lcb_AUTHENTICATOR with multiple @ref lcb_INSTANCE instances. While doing
 * so is theoretically possible, it is not supported or tested.
 */
LIBCOUCHBASE_API
void lcbauth_ref(lcb_AUTHENTICATOR *auth);

/**
 * @uncommitted
 *
 * Decrements the refcount on the authenticator object, freeing it if there
 * are no more owners.
 *
 * @param auth
 */
LIBCOUCHBASE_API
void lcbauth_unref(lcb_AUTHENTICATOR *auth);

/**
 * @uncommitted
 *
 * Makes a copy of an existing lcb_AUTHENTICATOR object. The returned
 * authenticator object has a reference count of 1.
 * @param src the authenticator object to clone
 * @return the cloned authenticator.
 *
 * This function is useful when you wish to copy an existing set of credentials
 * for use with a new client.
 */
LIBCOUCHBASE_API
lcb_AUTHENTICATOR *lcbauth_clone(const lcb_AUTHENTICATOR *src);

/**
 * @private
 *
 * Callback invoked for LCBAUTH_MODE_DYNAMIC type of authenticator.
 *
 * @param cookie The opaque pointer, configured during callbacks setup.
 * @param host The hostname of the service.
 * @param port The port of the service.
 * @param bucket The bucket name.
 * @return password or username, depending on where the callback used
 */
typedef const char *(*lcb_AUTHCALLBACK)(void *cookie, const char *host, const char *port, const char *bucket);

/**
 * @private
 *
 * Sets callback, which will be invoked every time the library needs credentials.
 *
 * @param auth
 * @param cookie the opaque pointer, which will be passed to callbacks
 * @param usercb the callback, which should return user name
 * @param passcb the callback, which should return user name
 */
LIBCOUCHBASE_API
lcb_STATUS lcbauth_set_callbacks(lcb_AUTHENTICATOR *auth, void *cookie, lcb_AUTHCALLBACK usercb,
                                 lcb_AUTHCALLBACK passcb);

typedef enum {
    /**
     * Use "bucket-specific" credentials when authenticating. This is the
     * only way of authenticating up to server version 5.0
     */
    LCBAUTH_MODE_CLASSIC = 0,

    /**
     * Use role-based access control. This allows the same user to have
     * access to multiple buckets with a single set of credentials.
     *
     * Note that if this option is selected, it becomes impossible to use
     * @ref LCBAUTH_F_BUCKET with lcbauth_add_pass()
     */
    LCBAUTH_MODE_RBAC = 1,

    /**
     * @private
     *
     * This mode allows to supply username/password with user-specified
     * callback. See lcbauth_set_callback().
     */
    LCBAUTH_MODE_DYNAMIC = 2
} lcbauth_MODE;

/**
 * @uncommitted
 *
 * Set the mode of this authenticator.
 * @param src the authenticator
 * @param mode the mode to use.
 * @return error if the authenticator already contains credentials.
 *
 * @note
 * This function should be called as early as possible. It is not possible to
 * change the mode after credentials have been added
 */
LIBCOUCHBASE_API
lcb_STATUS lcbauth_set_mode(lcb_AUTHENTICATOR *src, lcbauth_MODE mode);

/** @} */

#ifdef __cplusplus
}
#endif
#endif /* LCB_AUTH_H */
