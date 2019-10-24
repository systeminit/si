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

#include "scram_utils.h"
#include "config.h"
#include <time.h>
#include <ctype.h>

#include "strcodecs/strcodecs.h"

#ifndef LCB_NO_SSL
#include <openssl/rand.h>
#include <openssl/evp.h>
#include <openssl/hmac.h>
#include <openssl/sha.h>

#ifdef _WIN32
#include <process.h> // for _getpid
#else
#include <unistd.h> // for getpid
#endif

#endif

/**
 * Ensures the seed for the random generator is correctly filled.
 * Please note: as we use it only for the generation of the client nonce,
 * we don't need a strong entropy.
 */
void seed_rand(void)
{
    // To keep the code as much platform-agnostic as possible, we use standard values
    // like PID and current time for seeding the pseudo random generator.
    // The entropy of these values is not good, but that's enough for generating nonces.

#ifdef LCB_NO_SSL
    srand(time(NULL));
#else
    time_t current_time = time(NULL);
    clock_t clk;
#ifdef _WIN32
    int pid;
#else
    pid_t pid;
#endif
    RAND_add(&current_time, sizeof(current_time), 0.0);
    clk = clock();
    RAND_add(&clk, sizeof(clk), 0.0);
#ifdef _WIN32
    pid = _getpid();
#else
    pid = getpid();
#endif // _WIN32
    RAND_add(&pid, sizeof(pid), 0.0);
#endif // LCB_NO_SSL
}

/**
 * Generates a binary nonce of 'buffer_length' bytes at the given buffer address.
 * The buffer must be already allocated with enough space in it.
 */
void generate_nonce(char *buffer, int buffer_length)
{
    if ((NULL == buffer) || (0 == buffer_length)) {
        // invalid input arguments
        return;
    }
    seed_rand();

#ifndef LCB_NO_SSL
    // we try first to use RAND_bytes from OpenSSL
    if (!RAND_bytes((unsigned char *)buffer, buffer_length))
    // RAND_bytes failed: we use the standard rand() function.
#endif
    {
        int aRandom = 0;
        unsigned int aMaxRandBits = 0,
                     aMaxRand = RAND_MAX;           // we have to compute how many bits the rand() function can return
        unsigned int aRandRange = aMaxRandBits / 8; // number of bytes we can extract from a rand() value.
        int i;
        while (aMaxRand >>= 1) {
            aMaxRandBits++;
        }
        // To avoid generating a new random number for each character, we call rand() only once every 5 characters.
        // A 32-bits integer can give 5 values of 6 bits.
        for (i = 0; i < buffer_length; ++i) {
            if (i % aRandRange == 0) {
                // we refill aRandom
                aRandom = rand();
            }
            // we use only the last 8 bits of aRamdom
            buffer[i] = (char)(aRandom & 0xFF);
            aRandom >>= 8; // shift value by 8 bits
        }
    }
}

/**
 * Computes the number of comma (',') and equal ('=') characters in the input string
 * for further substitution.
 * Returns a negative value in case the buffer contains an invalid (control) character.
 */
int compute_special_chars(const char *buffer, int buffer_length)
{
    int result = 0;
    int i;
    for (i = 0; i < buffer_length; ++i) {
        char c = buffer[i];
        if (iscntrl(c)) {
            return -1; // control characters are not allowed
        }
        switch (c) {
            case '=':
            case ',':
                ++result;
                break;
            default:
                break;
        }
    }
    return result;
}

/**
 * Copies 'n' bytes from 'src' to 'dest', replacing comma and equal characters by their
 * substitution strings in the destination.
 */
void usernmcpy(char *dest, const char *src, size_t n)
{
    char *newdest = dest;
    unsigned int i;

    if (NULL == dest || NULL == src || 0 == n) {
        return; // invalid arguments
    }

    for (i = 0; i < n; ++i) {
        char c = src[i];
        switch (c) {
            case '=':
                // '=' character is replaced by "=3D"
                memcpy(newdest, "=3D", 3);
                newdest += 3;
                break;
            case ',':
                // '=' character is replaced by "=2C"
                memcpy(newdest, "=2C", 3);
                newdest += 3;
                break;
            default:
                *newdest = c;
                newdest++;
                break;
        }
    }
}

/**
 * Parses the server's first reply to extract the nonce, the salt and the iteration count.
 */
cbsasl_error_t parse_server_challenge(const char *serverin, unsigned int serverinlen, const char **nonce,
                                      unsigned int *noncelength, const char **salt, unsigned int *saltlength,
                                      unsigned int *itcount)
{
    const char *ptr = serverin;
    const char *oldptr = serverin;
    unsigned int remainlen = serverinlen;

    if (NULL == serverin || 0 == serverinlen) {
        return SASL_BADPARAM;
    }

    // the server challenge is normally composed of 3 attributes, separated by commas
    do {
        unsigned int attrlen; // attribute length
        ptr = memchr(ptr, ',', remainlen);
        if (ptr != NULL) {
            // oldptr points to the beginning of the attribute
            // Ex: "r=xxxxx,s=zzzzzz,i=10"
            //      ^      ^
            //      |      |
            //      |      +-- ptr
            //      +--------- oldptr
            attrlen = ptr - oldptr;
            ptr++; // to skip the comma
        } else {
            attrlen = remainlen;
        }
        if (attrlen <= 2) {
            // invalid attribute
            return SASL_BADPARAM;
        }
        // parse the attribute
        if (oldptr[1] != '=') {
            // invalid attribute: the second character must be an equal sign
            return SASL_BADPARAM;
        }
        switch (oldptr[0]) {
            case 'r': // nonce
                if (*nonce != NULL) {
                    // it looks like we already stored a previous occurrence of the nonce attribute
                    return SASL_BADPARAM;
                }
                *nonce = oldptr + 2;
                *noncelength = attrlen - 2;
                break;
            case 's': // salt
                if (*salt != NULL) {
                    // it looks like we already stored a previous occurrence of the salt attribute
                    return SASL_BADPARAM;
                }
                *salt = oldptr + 2;
                *saltlength = attrlen - 2;
                break;
            case 'i': // iteration count
            {
                // we have to use a temporary char array to parse the iteration count
                char itcountstr[11]; // an integer has maximum ten digits
                if (attrlen - 2 > 10) {
                    // value is larger than an integer
                    return SASL_BADPARAM;
                }
                memcpy(itcountstr, oldptr + 2, attrlen - 2);
                itcountstr[attrlen - 2] = 0;
                *itcount = atoi(itcountstr);
                break;
            }
            default:
                // invalid attribute type
                return SASL_BADPARAM;
        }

        remainlen = remainlen - attrlen - 1;
        oldptr = ptr;
    } while (ptr != NULL);

    return SASL_OK;
}

/**
 * Generates the salted password.
 * The 'outbuffer' buffer must be already allocated with enough space
 * (CBSASL_SHA512_DIGEST_SIZE).
 * As the salted password is binary and may contain the binary zero, we
 * don't put a binary zero at the end of the buffer.
 */
cbsasl_error_t generate_salted_password(cbsasl_auth_mechanism_t auth_mech, const cbsasl_secret_t *passwd,
                                        const char *salt, unsigned int saltlen, unsigned int itcount,
                                        unsigned char *outbuffer, unsigned int *outlength)
{
    // decode the salt from Base64
    char decodedsalt[256];
    int decsaltlen = lcb_base64_decode(salt, saltlen, decodedsalt, sizeof(decodedsalt));
    if (decsaltlen == -1) {
        // could not decode the salt from base64
        return SASL_BADPARAM;
    }

#ifdef HAVE_PKCS5_PBKDF2_HMAC
    switch (auth_mech) {
        case SASL_AUTH_MECH_SCRAM_SHA1:
            PKCS5_PBKDF2_HMAC((const char *)passwd->data, passwd->len, (const unsigned char *)decodedsalt,
                              (unsigned int)decsaltlen, itcount, EVP_sha1(), CBSASL_SHA1_DIGEST_SIZE, outbuffer);
            *outlength = CBSASL_SHA1_DIGEST_SIZE;
            break;
        case SASL_AUTH_MECH_SCRAM_SHA256:
            PKCS5_PBKDF2_HMAC((const char *)passwd->data, passwd->len, (const unsigned char *)decodedsalt,
                              (unsigned int)decsaltlen, itcount, EVP_sha256(), CBSASL_SHA256_DIGEST_SIZE, outbuffer);
            *outlength = CBSASL_SHA256_DIGEST_SIZE;
            break;
        case SASL_AUTH_MECH_SCRAM_SHA512:
            PKCS5_PBKDF2_HMAC((const char *)passwd->data, passwd->len, (const unsigned char *)decodedsalt,
                              (unsigned int)decsaltlen, itcount, EVP_sha512(), CBSASL_SHA512_DIGEST_SIZE, outbuffer);
            *outlength = CBSASL_SHA512_DIGEST_SIZE;
            break;
        default:
            return SASL_BADPARAM;
    }
    return SASL_OK;
#else
    (void)auth_mech;
    (void)passwd;
    (void)salt;
    (void)saltlen;
    (void)itcount;
    (void)outbuffer;
    (void)outlength;

    return SASL_BADPARAM;
#endif
}

#ifndef LCB_NO_SSL
/**
 * Generates a HMAC digest of the key and data by using the given
 * algorithm.
 */
static cbsasl_error_t HMAC_digest(cbsasl_auth_mechanism_t auth_mech, const unsigned char *key, unsigned int keylen,
                                  const unsigned char *data, unsigned int datalen, unsigned char *digest,
                                  unsigned int *digestlen)
{
    switch (auth_mech) {
        case SASL_AUTH_MECH_SCRAM_SHA1:
            if (HMAC(EVP_sha1(), key, keylen, data, datalen, digest, digestlen) == NULL) {
                return SASL_FAIL;
            }
            break;
        case SASL_AUTH_MECH_SCRAM_SHA256:
            if (HMAC(EVP_sha256(), key, keylen, data, datalen, digest, digestlen) == NULL) {
                return SASL_FAIL;
            }
            break;
        case SASL_AUTH_MECH_SCRAM_SHA512:
            if (HMAC(EVP_sha512(), key, keylen, data, datalen, digest, digestlen) == NULL) {
                return SASL_FAIL;
            }
            break;
        default:
            return SASL_BADPARAM;
    }
    return SASL_OK;
}
#endif

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
                                    char *outclientproof, unsigned int outprooflen)
{
#ifndef LCB_NO_SSL
    const char *clientkeystr = "Client Key";
    unsigned char clientkeyhmac[EVP_MAX_MD_SIZE];
    unsigned int hmaclen = 0;
    // we first compute the client key
    // ClientKey       := HMAC(SaltedPassword, "Client Key")
    cbsasl_error_t ret = HMAC_digest(auth_mech, saltedpassword, saltedpasslen, (const unsigned char *)clientkeystr,
                                     strlen(clientkeystr), clientkeyhmac, &hmaclen);
    if (ret != SASL_OK) {
        return ret;
    }

    // then we hashes the client key to compute the storey key
    // StoredKey       := H(ClientKey)
    unsigned char storedkey[EVP_MAX_MD_SIZE];
    unsigned int storedkeylen = 0;
    switch (auth_mech) {
        case SASL_AUTH_MECH_SCRAM_SHA1:
            if (SHA1(clientkeyhmac, hmaclen, storedkey) == NULL) {
                return SASL_FAIL;
            }
            storedkeylen = SHA_DIGEST_LENGTH;
            break;
        case SASL_AUTH_MECH_SCRAM_SHA256:
            if (SHA256(clientkeyhmac, hmaclen, storedkey) == NULL) {
                return SASL_FAIL;
            }
            storedkeylen = SHA256_DIGEST_LENGTH;
            break;
        case SASL_AUTH_MECH_SCRAM_SHA512:
            if (SHA512(clientkeyhmac, hmaclen, storedkey) == NULL) {
                return SASL_FAIL;
            }
            storedkeylen = SHA512_DIGEST_LENGTH;
            break;
        default:
            return SASL_BADPARAM;
    }

    // now we can compute the AuthMessage
    // AuthMessage     := client-first-message-bare + "," +
    //                   server-first-message + "," +
    //                   client-final-message-without-proof
    unsigned int authmesslen = cfblen + 1 + sfmlen + 1 + cfwplen;
    char *authmess = calloc(authmesslen + 1, 1); // +1 for the binary zero
    if (NULL == authmess) {
        return SASL_NOMEM;
    }
    memcpy(authmess, clientfirstbare, cfblen);
    authmess[cfblen] = ',';
    memcpy(authmess + cfblen + 1, serverfirstmess, sfmlen);
    authmess[cfblen + 1 + sfmlen] = ',';
    memcpy(authmess + cfblen + 1 + sfmlen + 1, clientfinalwithoutproof, cfwplen);
    *authmessage = authmess; // save the buffer into the context area for later use

    // let's compute the client signature
    // ClientSignature := HMAC(StoredKey, AuthMessage)
    unsigned char clientsign[EVP_MAX_MD_SIZE];
    unsigned int clientsignlen = 0;
    ret = HMAC_digest(auth_mech, storedkey, storedkeylen, (const unsigned char *)authmess, authmesslen, clientsign,
                      &clientsignlen);
    if (ret != SASL_OK) {
        return ret;
    }

    // final step:
    // ClientProof     := ClientKey XOR ClientSignature
    char clientproof[EVP_MAX_MD_SIZE]; // binary client proof
    unsigned int i;
    for (i = 0; i < clientsignlen; ++i) {
        clientproof[i] = clientkeyhmac[i] ^ clientsign[i];
    }

    // the final client proof must be encoded in base64
    if (lcb_base64_encode(clientproof, clientsignlen, outclientproof, outprooflen)) {
        return SASL_FAIL;
    }
    // and we are done

#else
    // nothing to do if OpenSSL is not present
    (void)auth_mech;
    (void)saltedpassword;
    (void)saltedpasslen;
    (void)clientfirstbare;
    (void)cfblen;
    (void)serverfirstmess;
    (void)sfmlen;
    (void)clientfinalwithoutproof;
    (void)cfwplen;
    (void)authmessage;
    (void)outclientproof;
    (void)outprooflen;
#endif
    return SASL_OK;
}

/**
 * Computes the Server Signature. It is computed as:
 *
 * SaltedPassword  := Hi(Normalize(password), salt, i)
 * ServerKey       := HMAC(SaltedPassword, "Server Key")
 * ServerSignature := HMAC(ServerKey, AuthMessage)
 */
cbsasl_error_t compute_server_signature(cbsasl_auth_mechanism_t auth_mech, const unsigned char *saltedpassword,
                                        unsigned int saltedpasslen, const char *authmessage, char *outserversign,
                                        unsigned int outsignlen)
{
#ifndef LCB_NO_SSL
    const char *serverkeystr = "Server Key";
    unsigned char serverkeyhmac[EVP_MAX_MD_SIZE];
    unsigned int hmaclen = 0;
    // we first compute the server key
    // ServerKey       := HMAC(SaltedPassword, "Server Key")
    cbsasl_error_t ret = HMAC_digest(auth_mech, saltedpassword, saltedpasslen, (unsigned char *)serverkeystr,
                                     strlen(serverkeystr), serverkeyhmac, &hmaclen);
    if (ret != SASL_OK) {
        return ret;
    }

    // let's compute the server signature
    // ServerSignature := HMAC(ServerKey, AuthMessage)
    unsigned char serversign[EVP_MAX_MD_SIZE];
    unsigned int serversignlen = 0;
    ret = HMAC_digest(auth_mech, serverkeyhmac, hmaclen, (unsigned char *)authmessage, strlen(authmessage), serversign,
                      &serversignlen);
    if (ret != SASL_OK) {
        return ret;
    }

    // the final client signature must be encoded in base64
    if (lcb_base64_encode((const char *)serversign, serversignlen, outserversign, outsignlen)) {
        return SASL_FAIL;
    }
    // and we are done

#else
    // nothing to do if OpenSSL is not present
    (void)auth_mech;
    (void)saltedpassword;
    (void)saltedpasslen;
    (void)authmessage;
    (void)outserversign;
    (void)outsignlen;
#endif
    return SASL_OK;
}
