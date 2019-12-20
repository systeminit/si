/*
 *     Copyright 2018-2019 Couchbase, Inc.
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

#ifndef SRC_SCRAM_SHA_SCRAM_UILS_H_
#define SRC_SCRAM_SHA_SCRAM_UILS_H_

#include <stddef.h>
#include "cbsasl/cbsasl.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Ensures the seed for OpenSSL's RAND_bytes function is correctly filled.
 * Please note: as we use it only for the generation of the client nonce,
 * we don't need a strong entropy.
 */
void seed_rand(void);

/**
 * Generates a binary nonce of 'buffer_length' bytes at the given buffer address.
 * The buffer must be already allocated with enough space in it.
 */
void generate_nonce(char *buffer, int buffer_length);

/**
 * Computes the number of comma (',') and equal ('=') characters in the input string
 * for further substitution.
 * If return value is negative, it means the buffer contains an invalid (control) character.
 */
int compute_special_chars(const char *buffer, int buffer_length);

/**
 * Copies 'n' bytes from 'src' to 'dest', replacing comma and equal characters by their
 * substitution strings in the destination.
 */
void usernmcpy(char *dest, const char *src, size_t n);

/**
 * Parses the server's first reply to extract the nonce, the salt and the iteration count.
 */
cbsasl_error_t parse_server_challenge(const char *serverin, unsigned int serverinlen, const char **nonce,
                                      unsigned int *noncelength, const char **salt, unsigned int *saltlength,
                                      unsigned int *itcount);

/**
 * Generates the salted password.
 */
cbsasl_error_t generate_salted_password(cbsasl_auth_mechanism_t auth_mech, const cbsasl_secret_t *passwd,
                                        const char *salt, unsigned int saltlen, unsigned int itcount,
                                        unsigned char *outbuffer, unsigned int *outlength);

/**
 * Computes the client proof. It is computed as:
 *
 * ClientKey       := HMAC(SaltedPassword, "Client Key")
 * StoredKey       := H(ClientKey)
 * AuthMessage     := client-first-message-bare + "," +
 *                    server-first-message + "," +
 *                    client-final-message-without-proof
 * ClientSignature := HMAC(StoredKey, AuthMessage)
 * ClientProof     := ClientKey XOR ClientSignature
 */
cbsasl_error_t compute_client_proof(cbsasl_auth_mechanism_t auth_mech, const unsigned char *saltedpassword,
                                    unsigned int saltedpasslen, const char *clientfirstbare, unsigned int cfblen,
                                    const char *serverfirstmess, unsigned int sfmlen,
                                    const char *clientfinalwithoutproof, unsigned int cfwplen, char **authmessage,
                                    char *outclientproof, unsigned int outprooflen);

/**
 * Computes the Server Signature. It is computed as:
 *
 * SaltedPassword  := Hi(Normalize(password), salt, i)
 * ServerKey       := HMAC(SaltedPassword, "Server Key")
 * ServerSignature := HMAC(ServerKey, AuthMessage)
 */
cbsasl_error_t compute_server_signature(cbsasl_auth_mechanism_t auth_mech, const unsigned char *saltedpassword,
                                        unsigned int saltedpasslen, const char *authmessage, char *outserversign,
                                        unsigned int outsignlen);

#ifdef __cplusplus
}
#endif

#endif /* SRC_SCRAM_SHA_SCRAM_UILS_H_ */
