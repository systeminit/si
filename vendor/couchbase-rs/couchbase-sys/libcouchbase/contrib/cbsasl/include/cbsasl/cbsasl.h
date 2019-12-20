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

#ifndef INCLUDE_CBSASL_CBSASL_H_
#define INCLUDE_CBSASL_CBSASL_H_ 1
#define CBSASL_PUBLIC_API

#ifdef __cplusplus
extern "C" {
#endif

#define MECH_PLAIN         "PLAIN"
#define MECH_CRAM_MD5      "CRAM-MD5"
#define MECH_SCRAM_SHA1    "SCRAM-SHA1"
#define MECH_SCRAM_SHA256  "SCRAM-SHA256"
#define MECH_SCRAM_SHA512  "SCRAM-SHA512"

// size in bytes - to double if the nonce is displayed in hexadecimal
#define SCRAM_NONCE_SIZE   8

#define CBSASL_SHA1_DIGEST_SIZE 20
#define CBSASL_SHA256_DIGEST_SIZE 32
#define CBSASL_SHA512_DIGEST_SIZE 64

    typedef enum cbsasl_error {
        SASL_OK,
        SASL_CONTINUE,
        SASL_FAIL,
        SASL_NOMEM,
        SASL_BADPARAM,
        SASL_NOMECH,
        SASL_NOUSER
    }
    cbsasl_error_t;

    typedef enum cbsasl_auth_mechanism {
        SASL_AUTH_MECH_PLAIN,
        SASL_AUTH_MECH_CRAM_MD5, // deprecated
        SASL_AUTH_MECH_SCRAM_SHA1,
        SASL_AUTH_MECH_SCRAM_SHA256,
        SASL_AUTH_MECH_SCRAM_SHA512
    }
    cbsasl_auth_mechanism_t;

    typedef struct {
        unsigned long len;
        unsigned char data[1];
    } cbsasl_secret_t;

    typedef struct cbsasl_conn_st cbsasl_conn_t;

    typedef struct {
        void *context;
        int (*username)(void *context, int id, const char **result, unsigned int *len);
        int (*password)(cbsasl_conn_t *conn, void *context, int id, cbsasl_secret_t **psecret);
    } cbsasl_callbacks_t;

    typedef cbsasl_error_t (*cbsasl_init_fn)(void);
    typedef cbsasl_error_t (*cbsasl_start_fn)(cbsasl_conn_t *);
    typedef cbsasl_error_t (*cbsasl_step_fn)(cbsasl_conn_t *, const char *,
                                             unsigned, const char **, unsigned *);

    typedef struct cbsasl_mechs {
        const char *name;
        cbsasl_init_fn init;
        cbsasl_start_fn start;
        cbsasl_step_fn step;
    } cbsasl_mechs_t;

    struct cbsasl_client_conn_t {
        char *userdata;
        cbsasl_auth_mechanism_t auth_mech; // authentication mechanism
        int (*get_username)(void *context, int id, const char **result,
                            unsigned int *len);
        void *get_username_ctx;
        int (*get_password)(cbsasl_conn_t *conn, void *context, int id,
                            cbsasl_secret_t **psecret);
        void *get_password_ctx;
        char *nonce; // client nonce for SCRAM-SHA authentication
        char *client_first_message_bare; // for SCRAM-SHA authentication
        unsigned char *saltedpassword; // for SCRAM-SHA authentication
        unsigned int saltedpasslen; // length of the salted password field
        char *auth_message; // for SCRAM-SHA authentication
    };

    struct cbsasl_server_conn_t {
        char *username;
        char *config;
        char *sasl_data;
        unsigned int sasl_data_len;
        cbsasl_mechs_t mech;
    };

    struct cbsasl_conn_st {
        int client;
        union {
            struct cbsasl_client_conn_t client;
            struct cbsasl_server_conn_t server;
        } c;
    };

    /**
     * Lists all of the mechanisms this sasl server supports
     *
     * @param mechs A string containing all supported mechanism names
     * @param mechslen The length of the mechs string
     *
     * @return Whether or not an error occured while getting the mechanism list
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_list_mechs(const char **mechs,
                                     unsigned *mechslen);

    /**
     * Initializes the sasl server
     *
     * This function initializes the server by loading passwords from the cbsasl
     * password file. This function should only be called once.
     *
     * @return Whether or not the sasl server initialization was successful
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_server_init(void);

    /**
     * Creates a sasl connection and begins authentication
     *
     * When a client receives a request for sasl authentication this function is
     * called in order to initialize the sasl connection based on the mechanism
     * specified.
     *
     * @param conn The connection context for this session
     * @param mechanism The mechanism that will be used for authentication
     *
     * @return Whether or not the mecahnism initialization was successful
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_server_start(cbsasl_conn_t **conn,
                                       const char *mech,
                                       const char *clientin,
                                       unsigned int clientinlen,
                                       unsigned char **serverout,
                                       unsigned int *serveroutlen);

    /**
     * Does username/password authentication
     *
     * After the sasl connection is initialized the step function is called to
     * check credentials.
     *
     * @return Whether or not the sasl step was successful
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_server_step(cbsasl_conn_t *conn,
                                      const char *input,
                                      unsigned inputlen,
                                      const char **output,
                                      unsigned *outputlen);

    /**
     * Final authentication step, if need to be (depending on the mechanism).
     *
     * This is mainly use for SCRAM-SHA authentication family, to verify the
     * server signature. Even if the server accepted the authentication, if
     * we can't verify its signature, we must reject the connection.
     *
     * @return Whether or not the sasl check was successful
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_client_check(cbsasl_conn_t *conn,
                                       const char *input,
                                       unsigned int inputlen);

    /**
     * Frees up funushed sasl connections
     *
     * @param conn The sasl connection to free
     */
    CBSASL_PUBLIC_API
    void cbsasl_dispose(cbsasl_conn_t **pconn);

    /**
     * Refresh the internal data (this may result in loading password
     * databases etc)
     *
     * @return Whether or not the operation was successful
     */
    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_server_refresh(void);

    typedef enum {
        CBSASL_USERNAME = 0,
        CBSASL_CONFIG = 1
    } cbsasl_prop_t;

    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_getprop(cbsasl_conn_t *conn,
                                  cbsasl_prop_t propnum,
                                  const void **pvalue);

    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_setprop(cbsasl_conn_t *conn,
                                  cbsasl_prop_t propnum,
                                  const void *pvalue);

    /* Client API */


    /* define the different callback id's we support */
#define CBSASL_CB_USER 1
#define CBSASL_CB_AUTHNAME 2
#define CBSASL_CB_PASS 3
#define CBSASL_CB_LIST_END 4

    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_client_new(const char *service,
                                     const char *serverFQDN,
                                     const char *iplocalport,
                                     const char *ipremoteport,
                                     const cbsasl_callbacks_t *prompt_supp,
                                     unsigned int flags,
                                     cbsasl_conn_t **pconn);

    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_client_start(cbsasl_conn_t *conn,
                                       const char *mechlist,
                                       void **prompt_need,
                                       const char **clientout,
                                       unsigned int *clientoutlen,
                                       const char **mech,
                                       int allow_scram_sha);

    CBSASL_PUBLIC_API
    cbsasl_error_t cbsasl_client_step(cbsasl_conn_t *conn,
                                      const char *serverin,
                                      unsigned int serverinlen,
                                      void **not_used,
                                      const char **clientout,
                                      unsigned int *clientoutlen);

#ifdef __cplusplus
}
#endif


#endif  /* INCLUDE_CBSASL_CBSASL_H_ */
