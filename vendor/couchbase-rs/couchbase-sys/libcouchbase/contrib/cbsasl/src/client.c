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

#include "config.h"
#include "cbsasl/cbsasl.h"
#include "cram-md5/hmac.h"
#include "scram-sha/scram_utils.h"
#include "util.h"
#include <time.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

CBSASL_PUBLIC_API
cbsasl_error_t cbsasl_client_new(const char *service, const char *serverFQDN, const char *iplocalport,
                                 const char *ipremoteport, const cbsasl_callbacks_t *callbacks, unsigned flags,
                                 cbsasl_conn_t **pconn)
{
    cbsasl_conn_t *conn;

    if (callbacks == NULL) {
        return SASL_BADPARAM;
    }

    conn = calloc(1, sizeof(*conn));
    if (conn == NULL) {
        return SASL_NOMEM;
    }

    conn->client = 1;

    /* Locate the callbacks */
    conn->c.client.get_username = callbacks->username;
    conn->c.client.get_username_ctx = callbacks->context;
    conn->c.client.get_password = callbacks->password;
    conn->c.client.get_password_ctx = callbacks->context;

    if (conn->c.client.get_username == NULL || conn->c.client.get_password == NULL) {
        cbsasl_dispose(&conn);
        return SASL_NOUSER;
    }

    *pconn = conn;

    (void)service;
    (void)serverFQDN;
    (void)iplocalport;
    (void)ipremoteport;
    (void)flags;

    return SASL_OK;
}

CBSASL_PUBLIC_API
cbsasl_error_t cbsasl_client_start(cbsasl_conn_t *conn, const char *mechlist, void **prompt_need,
                                   const char **clientout, unsigned int *clientoutlen, const char **mech,
                                   int allow_scram_sha)
{
    if (conn->client == 0) {
        return SASL_BADPARAM;
    }

    *mech = NULL;
    if (allow_scram_sha) {
#if !defined(LCB_NO_SSL) && defined(HAVE_PKCS5_PBKDF2_HMAC)
        // we use SCRAM-SHA only if OpenSSL is linked and has support for PBKDF2_HMAC functions
        if (strstr(mechlist, MECH_SCRAM_SHA512) != NULL) {
            *mech = MECH_SCRAM_SHA512;
            conn->c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA512;
        } else if (strstr(mechlist, MECH_SCRAM_SHA256) != NULL) {
            *mech = MECH_SCRAM_SHA256;
            conn->c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA256;
        } else if (strstr(mechlist, MECH_SCRAM_SHA1) != NULL) {
            *mech = MECH_SCRAM_SHA1;
            conn->c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA1;
        }
#endif
    }
    if (*mech == NULL) {
        if (strstr(mechlist, MECH_CRAM_MD5) != NULL) {
            *mech = MECH_CRAM_MD5;
            conn->c.client.auth_mech = SASL_AUTH_MECH_CRAM_MD5;
        } else if (strstr(mechlist, MECH_PLAIN) != NULL) {
            *mech = MECH_PLAIN;
            conn->c.client.auth_mech = SASL_AUTH_MECH_PLAIN;
        } else {
            return SASL_NOMECH;
        }
    }

    switch (conn->c.client.auth_mech) {
        case SASL_AUTH_MECH_PLAIN: {
            const char *usernm = NULL;
            unsigned int usernmlen;
            cbsasl_secret_t *pass;
            cbsasl_error_t ret;

            ret = conn->c.client.get_username(conn->c.client.get_username_ctx, CBSASL_CB_USER, &usernm, &usernmlen);
            if (ret != SASL_OK) {
                return ret;
            }

            ret = conn->c.client.get_password(conn, conn->c.client.get_password_ctx, CBSASL_CB_PASS, &pass);
            if (ret != SASL_OK) {
                return ret;
            }

            conn->c.client.userdata = calloc(usernmlen + 1 + pass->len + 1, 1);
            if (conn->c.client.userdata == NULL) {
                return SASL_NOMEM;
            }

            memcpy(conn->c.client.userdata + 1, usernm, usernmlen);
            memcpy(conn->c.client.userdata + usernmlen + 2, pass->data, pass->len);
            *clientout = conn->c.client.userdata;
            *clientoutlen = (unsigned int)(usernmlen + 2 + pass->len);
            break;
        }
        case SASL_AUTH_MECH_SCRAM_SHA1:
        case SASL_AUTH_MECH_SCRAM_SHA256:
        case SASL_AUTH_MECH_SCRAM_SHA512: {
            const char *usernm = NULL;
            unsigned int usernmlen;
            int usernmspeccharsnb;
            char binnonce[SCRAM_NONCE_SIZE]; // binary nonce
            cbsasl_error_t ret;

            ret = conn->c.client.get_username(conn->c.client.get_username_ctx, CBSASL_CB_USER, &usernm, &usernmlen);
            if (ret != SASL_OK) {
                return ret;
            }
            usernmspeccharsnb = compute_special_chars(usernm, usernmlen);
            if (usernmspeccharsnb < 0) {
                // invalid characters in the username
                return SASL_BADPARAM;
            }

            generate_nonce(binnonce, SCRAM_NONCE_SIZE);
            conn->c.client.nonce = calloc(SCRAM_NONCE_SIZE * 2 + 1, 1);
            if (conn->c.client.nonce == NULL) {
                return SASL_NOMEM;
            }
            // stores binary nonce in hexadecimal format into conn->c.client.nonce array
            cbsasl_hex_encode(conn->c.client.nonce, binnonce, SCRAM_NONCE_SIZE);
            conn->c.client.nonce[SCRAM_NONCE_SIZE * 2] = '\0';

#define GS2_HEADER "n,,n=" // "no binding" + start of name attribute
#define NONCE_ATTR ",r="   // start of nonce attribute
            conn->c.client.userdata = calloc(
                strlen(GS2_HEADER) + usernmlen + usernmspeccharsnb * 2 + strlen(NONCE_ATTR) + SCRAM_NONCE_SIZE * 2, 1);
            if (conn->c.client.userdata == NULL) {
                return SASL_NOMEM;
            }

            memcpy(conn->c.client.userdata, GS2_HEADER, strlen(GS2_HEADER));
            if (!usernmspeccharsnb) {
                // no special char, we can do a direct copy
                memcpy(conn->c.client.userdata + strlen(GS2_HEADER), usernm, usernmlen);
            } else {
                // copy with substitution of special characters
                usernmcpy(conn->c.client.userdata + strlen(GS2_HEADER), usernm, usernmlen);
            }
            memcpy(conn->c.client.userdata + strlen(GS2_HEADER) + usernmlen + usernmspeccharsnb * 2, NONCE_ATTR,
                   strlen(NONCE_ATTR));
            memcpy(conn->c.client.userdata + strlen(GS2_HEADER) + usernmlen + usernmspeccharsnb * 2 +
                       strlen(NONCE_ATTR),
                   conn->c.client.nonce, SCRAM_NONCE_SIZE * 2);

            *clientout = conn->c.client.userdata;
            *clientoutlen = (unsigned int)(strlen(GS2_HEADER) + usernmlen + usernmspeccharsnb * 2 + strlen(NONCE_ATTR) +
                                           SCRAM_NONCE_SIZE * 2);

            // save the client first message for later step (without the first three characters)
            conn->c.client.client_first_message_bare = calloc(*clientoutlen - 3 + 1, 1); // +1 for the binary zero
            if (conn->c.client.client_first_message_bare == NULL) {
                return SASL_NOMEM;
            }
            memcpy(conn->c.client.client_first_message_bare, *clientout + 3, *clientoutlen - 3);
            // no need to add a final binary zero, as calloc already sets the memory to zero
            break;
        }
        case SASL_AUTH_MECH_CRAM_MD5:
            // no data in the first CRAM-MD5 message
            *clientout = NULL;
            *clientoutlen = 0;
            break;
    }

    (void)prompt_need;
    return SASL_OK;
}

CBSASL_PUBLIC_API
cbsasl_error_t cbsasl_client_step(cbsasl_conn_t *conn, const char *serverin, unsigned int serverinlen, void **not_used,
                                  const char **clientout, unsigned int *clientoutlen)
{
    const char *usernm = NULL;
    unsigned int usernmlen;
    cbsasl_secret_t *pass;
    cbsasl_error_t ret;

    (void)not_used;

    if (conn->client == 0) {
        return SASL_BADPARAM;
    }

    if (conn->c.client.auth_mech == SASL_AUTH_MECH_PLAIN) {
        /* Shouldn't be called during plain auth */
        return SASL_BADPARAM;
    }

    ret = conn->c.client.get_username(conn->c.client.get_username_ctx, CBSASL_CB_USER, &usernm, &usernmlen);
    if (ret != SASL_OK) {
        return ret;
    }

    ret = conn->c.client.get_password(conn, conn->c.client.get_password_ctx, CBSASL_CB_PASS, &pass);
    if (ret != SASL_OK) {
        return ret;
    }

    free(conn->c.client.userdata);
    conn->c.client.userdata = NULL;
    switch (conn->c.client.auth_mech) {
        case SASL_AUTH_MECH_CRAM_MD5: {
            unsigned char digest[DIGEST_LENGTH];
            char md5string[DIGEST_LENGTH * 2];
            conn->c.client.userdata = calloc(usernmlen + 1 + sizeof(md5string) + 1, 1);
            if (conn->c.client.userdata == NULL) {
                return SASL_NOMEM;
            }

            cbsasl_hmac_md5((unsigned char *)serverin, serverinlen, pass->data, pass->len, digest);
            cbsasl_hex_encode(md5string, (char *)digest, DIGEST_LENGTH);
            memcpy(conn->c.client.userdata, usernm, usernmlen);
            conn->c.client.userdata[usernmlen] = ' ';
            memcpy(conn->c.client.userdata + usernmlen + 1, md5string, sizeof(md5string));
            break;
        }
        case SASL_AUTH_MECH_SCRAM_SHA1:
        case SASL_AUTH_MECH_SCRAM_SHA256:
        case SASL_AUTH_MECH_SCRAM_SHA512: {
            if (!conn->c.client.auth_message) {
                const char *combinednonce = NULL; // nonce extracted from server's first reply
                unsigned int noncelen = 0;
                const char *salt = NULL; // salt extracted from server's first reply
                unsigned int saltlen = 0;
                unsigned int itcount = 0;
                unsigned char saltedpassword[CBSASL_SHA512_DIGEST_SIZE]; // max digest size
                unsigned int saltedpasslen = 0;
                unsigned int prooflen = 0; // proof size in base64

                ret =
                    parse_server_challenge(serverin, serverinlen, &combinednonce, &noncelen, &salt, &saltlen, &itcount);
                if (ret != SASL_OK) {
                    return ret;
                }
                if (!combinednonce || !noncelen || !salt || !saltlen || !itcount) {
                    // missing or invalid value in server challenge
                    return SASL_BADPARAM;
                }
                if (noncelen < SCRAM_NONCE_SIZE * 2 ||
                    memcmp(combinednonce, conn->c.client.nonce, SCRAM_NONCE_SIZE * 2)) {
                    // the combined nonce doesn't start with the client nonce we sent previously
                    return SASL_BADPARAM;
                }
                // ok, now we can compute the client proof
                ret = generate_salted_password(conn->c.client.auth_mech, pass, salt, saltlen, itcount, saltedpassword,
                                               &saltedpasslen);
                if (ret != SASL_OK) {
                    return ret;
                }
                // save salted password for later use
                conn->c.client.saltedpassword = calloc(saltedpasslen, 1);
                if (conn->c.client.saltedpassword == NULL) {
                    return SASL_NOMEM;
                }
                memcpy(conn->c.client.saltedpassword, saltedpassword, saltedpasslen);
                conn->c.client.saltedpasslen = saltedpasslen;

// before building the client proof, we start building the client final message,
// as it is used for the computation of the proof
// The final message starts with the base64-encoded GS2 header from the initial message.
// As we always use "n,,", we can hardcode directly its base64-counterpart, so "biws".
#define FINAL_HEADER "c=biws,r="
#define PROOF_ATTR ",p="
                switch (conn->c.client.auth_mech) {
                    case SASL_AUTH_MECH_SCRAM_SHA1:
                        prooflen = (CBSASL_SHA1_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    case SASL_AUTH_MECH_SCRAM_SHA256:
                        prooflen = (CBSASL_SHA256_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    case SASL_AUTH_MECH_SCRAM_SHA512:
                        prooflen = (CBSASL_SHA512_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    default:
                        break;
                }
                conn->c.client.userdata =
                    calloc(strlen(FINAL_HEADER) + noncelen + strlen(PROOF_ATTR) + prooflen + 1, 1);
                if (conn->c.client.userdata == NULL) {
                    return SASL_NOMEM;
                }
                memcpy(conn->c.client.userdata, FINAL_HEADER, strlen(FINAL_HEADER));
                memcpy(conn->c.client.userdata + strlen(FINAL_HEADER), combinednonce, noncelen);
                memcpy(conn->c.client.userdata + strlen(FINAL_HEADER) + noncelen, PROOF_ATTR, strlen(PROOF_ATTR));

                ret = compute_client_proof(
                    conn->c.client.auth_mech, saltedpassword, saltedpasslen, conn->c.client.client_first_message_bare,
                    strlen(conn->c.client.client_first_message_bare), serverin, serverinlen, conn->c.client.userdata,
                    strlen(FINAL_HEADER) + noncelen, &(conn->c.client.auth_message),
                    conn->c.client.userdata + strlen(FINAL_HEADER) + noncelen + strlen(PROOF_ATTR), prooflen + 1);
                if (ret != SASL_OK) {
                    return ret;
                }
            } else {
                // auth_message should not be already set
                return SASL_FAIL;
            }
            break;
        }
        default:
            break;
    }
    *clientout = conn->c.client.userdata;
    *clientoutlen = strlen(conn->c.client.userdata);

    return SASL_CONTINUE;
}

cbsasl_error_t cbsasl_client_check(cbsasl_conn_t *conn, const char *serverin, unsigned int serverinlen)
{
    switch (conn->c.client.auth_mech) {
        case SASL_AUTH_MECH_SCRAM_SHA1:
        case SASL_AUTH_MECH_SCRAM_SHA256:
        case SASL_AUTH_MECH_SCRAM_SHA512: {
            if (conn->c.client.auth_message) {
                char serversign[(CBSASL_SHA512_DIGEST_SIZE / 3 + 1) * 4 + 1]; // max sign len
                unsigned int serversignlen = 0;
                cbsasl_error_t ret;

                // Last step: we have to verify the server's proof.
                // In case of positive answer from the server, its final reply must start with "v=".
                if (serverinlen <= 2 || memcmp(serverin, "v=", 2)) {
                    return SASL_FAIL;
                }
                switch (conn->c.client.auth_mech) {
                    case SASL_AUTH_MECH_SCRAM_SHA1:
                        serversignlen = (CBSASL_SHA1_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    case SASL_AUTH_MECH_SCRAM_SHA256:
                        serversignlen = (CBSASL_SHA256_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    case SASL_AUTH_MECH_SCRAM_SHA512:
                        serversignlen = (CBSASL_SHA512_DIGEST_SIZE / 3 + 1) * 4;
                        break;
                    default:
                        break;
                }
                ret = compute_server_signature(conn->c.client.auth_mech, conn->c.client.saltedpassword,
                                               conn->c.client.saltedpasslen, conn->c.client.auth_message, serversign,
                                               sizeof(serversign));
                if (ret != SASL_OK) {
                    return ret;
                }

                // ok, we can now compare the two values
                if ((serverinlen - 2 < serversignlen) || (memcmp(serverin + 2, serversign, serversignlen))) {
                    return SASL_FAIL;
                }
            } else {
                // we have an issue: auth_message should not have been cleared
                return SASL_FAIL;
            }
            break;
        }
        case SASL_AUTH_MECH_CRAM_MD5:
        case SASL_AUTH_MECH_PLAIN:
        default:
            // nothing to do
            break;
    }

    return SASL_OK;
}
