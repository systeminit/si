/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#ifndef LCB_CRYPTO_H
#define LCB_CRYPTO_H

/**
 * @file
 * Field encryption
 *
 * @uncommitted
 */

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @ingroup lcb-public-api
 * @defgroup lcb-crypto-api Encryption
 * @brief Register crypto-providers and working with encrypted fields of the documents.
 * @details
 * These routines contain functionality to define and hook crypto providers, as well as
 * functions which should be used for portable (cross SDK) encoding of encrypted fields.
 */

/**
 * @addtogroup lcb-crypto-api
 * @{
 */

/**
 * IOV-style structure for signing functions of crypto-provider.
 *
 * @committed
 */
typedef struct lcbcrypto_SIGV {
    const uint8_t *data; /**< pointer to data */
    size_t len;          /**< length of the data in bytes */
} lcbcrypto_SIGV;

struct lcbcrypto_PROVIDER;
/**
 * Crypto-provider interface.
 *
 * See full example in @ref example/crypto/openssl_symmetric_provider.c
 *
 * @see lcbcrypto_register
 * @see lcbcrypto_unregister
 *
 * @committed
 */
typedef struct lcbcrypto_PROVIDER {
    uint16_t version;                                        /**< version of the structure, current value is 1 */
    int16_t _refcnt;                                         /**< reference counter */
    uint64_t flags;                                          /**< provider-specific flags */
    void *cookie;                                            /**< opaque pointer (e.g. pointer to wrapper instance) */
    void (*destructor)(struct lcbcrypto_PROVIDER *provider); /**< destructor function, or NULL */
    union {
        struct {
            /** function to use when the library wants to deallocate memory, returned by provider */
            void (*release_bytes)(struct lcbcrypto_PROVIDER *provider, void *bytes);
            /** initialization vector (IV) generator */
            lcb_STATUS (*generate_iv)(struct lcbcrypto_PROVIDER *provider, uint8_t **iv, size_t *iv_len);
            /** generate cryptographic signature for the data */
            lcb_STATUS (*sign)(struct lcbcrypto_PROVIDER *provider, const lcbcrypto_SIGV *inputs, size_t input_num,
                               uint8_t **sig, size_t *sig_len);
            /** verify signature of the data */
            lcb_STATUS (*verify_signature)(struct lcbcrypto_PROVIDER *provider, const lcbcrypto_SIGV *inputs,
                                           size_t input_num, uint8_t *sig, size_t sig_len);
            /** encrypt data */
            lcb_STATUS (*encrypt)(struct lcbcrypto_PROVIDER *provider, const uint8_t *input, size_t input_len,
                                  const uint8_t *iv, size_t iv_len, uint8_t **output, size_t *output_len);
            /** decrypt data */
            lcb_STATUS (*decrypt)(struct lcbcrypto_PROVIDER *provider, const uint8_t *input, size_t input_len,
                                  const uint8_t *iv, size_t iv_len, uint8_t **output, size_t *output_len);
            /** returns key identifier, associated with the crypto-provider */
            const char *(*get_key_id)(struct lcbcrypto_PROVIDER *provider);
        } v1;
    } v;
} lcbcrypto_PROVIDER;

/**
 * Structure for JSON field specification for encrypt/decrypt API.
 *
 * @see lcbcrypto_encrypt_fields
 * @see lcbcrypto_decrypt_fields
 *
 * @committed
 */
typedef struct lcbcrypto_FIELDSPEC {
    const char *name; /**< field name (NUL-terminated) */
    const char *alg;  /**< crypto provider alias (NUL-terminated) */
    LCB_DEPRECATED2(const char *kid,
                    "Do not use kid field. Encryption keys have to be part of the provider implementation");
} lcbcrypto_FIELDSPEC;

/**
 * Command to encrypt JSON fields.
 *
 * @see lcbcrypto_encrypt_fields
 * @committed
 */
typedef struct lcbcrypto_CMDENCRYPT {
    uint16_t version;   /**< version of the structure, currently valid value is 0 */
    const char *prefix; /**< prefix to encrypted field. When NULL, it will use @ref LCBCRYPTO_DEFAULT_FIELD_PREFIX */
    const char *doc;    /**< pointer to the input JSON document */
    size_t ndoc;        /**< size of the input JSON document */
    char *out;   /**< pointer to output JSON document. When no changes were applied, this field will be set to NULL */
    size_t nout; /**< size of the output JSON document */
    lcbcrypto_FIELDSPEC *fields; /**< list of field specs */
    size_t nfields;              /**< number of field specs */
} lcbcrypto_CMDENCRYPT;

/**
 * Command to decrypt JSON fields.
 *
 * @see lcbcrypto_decrypt_fields
 * @committed
 */
typedef struct lcbcrypto_CMDDECRYPT {
    uint16_t version;   /**< version of the structure, currently valid value is 0 */
    const char *prefix; /**< prefix to encrypted field. When NULL, it will use @ref LCBCRYPTO_DEFAULT_FIELD_PREFIX */
    const char *doc;    /**< pointer to the input JSON document */
    size_t ndoc;        /**< size of the input JSON document */
    char *out;   /**< pointer to output JSON document. When no changes were applied, this field will be set to NULL */
    size_t nout; /**< size of the output JSON document */
    lcbcrypto_FIELDSPEC *fields; /**< list of field specs */
    size_t nfields;              /**< number of field specs */
} lcbcrypto_CMDDECRYPT;

/**
 * Register crypto-provider for specified alias.
 *
 * See full example in @ref example/crypto/openssl_symmetric_provider.c
 *
 * @param instance the handle
 * @param name provider alias, this will be recorded in JSON.
 * @param provider implementation of the crypto-provider
 *
 * @par Register provider as "AES-256-HMAC-SHA256".
 * @code{.c}
 * lcbcrypto_PROVIDER *provider = calloc(1, sizeof(lcbcrypto_PROVIDER));
 * provider->version = 1;
 * provider->destructor = osp_free;
 * provider->v.v1.release_bytes = osp_release_bytes;
 * provider->v.v1.generate_iv = osp_generate_iv;
 * provider->v.v1.sign = osp_sign;
 * provider->v.v1.verify_signature = osp_verify_signature;
 * provider->v.v1.encrypt = osp_encrypt;
 * provider->v.v1.decrypt = osp_decrypt;
 * provider->v.v1.get_key_id = osp_get_key_id;
 * lcbcrypto_register(instance, "AES-256-HMAC-SHA256", provider);
 * @endcode
 */
LIBCOUCHBASE_API void lcbcrypto_register(lcb_INSTANCE *instance, const char *name, lcbcrypto_PROVIDER *provider);

/**
 * Unregister crypto-provider for specified alias.
 *
 * See full example in @ref example/crypto/openssl_symmetric_provider.c
 *
 * @param instance the handle
 * @param name provider alias.
 */
LIBCOUCHBASE_API void lcbcrypto_unregister(lcb_INSTANCE *instance, const char *name);

/**
 * Increment reference counter for crypto-provider.
 *
 * @param provider provider instance
 */
LIBCOUCHBASE_API void lcbcrypto_ref(lcbcrypto_PROVIDER *provider);

/**
 * Decrement reference counter for crypto-provider.
 *
 * It calls destructor once counter reaches zero. The provider instance should not be used after calling this function.
 *
 * @param provider provider instance
 */
LIBCOUCHBASE_API void lcbcrypto_unref(lcbcrypto_PROVIDER *provider);

/**
 * Default prefix for encrypted JSON fields.
 */
#define LCBCRYPTO_DEFAULT_FIELD_PREFIX "__crypt_"

/**
 * Encrypt all specified fields in the JSON encoded object.
 *
 * The function will remove original content of the field, and rename it using @ref LCBCRYPTO_DEFAULT_FIELD_PREFIX, or
 * custom prefix, specified in the command.
 *
 * See full example in @ref example/crypto/openssl_symmetric_encrypt.c
 *
 * @param instance the handle
 * @param cmd the command structure
 * @return LCB_SUCCESS if successful, an error code otherwise
 *
 * @par Encrypt field "message" in the document using provider registered as "AES-256-HMAC-SHA256"
 * @code{.c}
 * lcbcrypto_CMDENCRYPT cmd = {};
 * lcbcrypto_FIELDSPEC field = {};
 * lcb_STATUS err;
 *
 * cmd.version = 0;
 * cmd.prefix = NULL;
 * cmd.doc = "{\"message\":\"hello world\"}";
 * cmd.ndoc = strlen(cmd.doc);
 * cmd.nfields = 1;
 * cmd.fields = &field;
 * field.name = "message";
 * field.alg = "AES-256-HMAC-SHA256";
 *
 * err = lcbcrypto_encrypt_fields(instance, &cmd);
 * @endcode
 *
 * @committed
 */
LIBCOUCHBASE_API lcb_STATUS lcbcrypto_encrypt_fields(lcb_INSTANCE *instance, lcbcrypto_CMDENCRYPT *cmd);

/**
 * Decrypt all specified fields in the JSON encoded object.
 *
 * The function will remove original content of the field, and rename it using @ref LCBCRYPTO_DEFAULT_FIELD_PREFIX, or
 * custom prefix, specified in the command.
 *
 * See full example in @ref example/crypto/openssl_symmetric_decrypt.c
 *
 * @param instance the handle
 * @param cmd the command structure
 * @return LCB_SUCCESS if successful, an error code otherwise
 *
 * @par Decrypt field "message" in the document using provider registered as "AES-256-HMAC-SHA256"
 * @code{.c}
 * lcbcrypto_CMDDECRYPT cmd = {};
 * lcbcrypto_FIELDSPEC field = {};
 * lcb_STATUS err;
 *
 * cmd.version = 0;
 * cmd.prefix = NULL;
 * cmd.doc = "{\"__crypt_message\":{" \
 *               "\"alg\":\"AES-256-HMAC-SHA256\"," \
 *               "\"ciphertext\":\"gYuyEhf6S0AiMGZJZZV35Q==\"," \
 *               "\"iv\":\"ZedmvjWy0lIrLn6OmQmNqQ==\"," \
 *               "\"kid\":\"mykeyid\"," \
 *               "\"sig\":\"FgleInW3Iia04XqLbm5Hd3qVoa77Ocs7g2x4pOutEtY=\"}" \
 *           "}";
 * cmd.ndoc = strlen(cmd.doc);
 * cmd.nfields = 1;
 * cmd.fields = &field;
 * field.name = "message";
 * field.alg = "AES-256-HMAC-SHA256";
 *
 * err = lcbcrypto_decrypt_fields(instance, &cmd);
 * @endcode
 *
 * @committed
 */
LIBCOUCHBASE_API lcb_STATUS lcbcrypto_decrypt_fields(lcb_INSTANCE *instance, lcbcrypto_CMDDECRYPT *cmd);
/**@}*/

#ifdef __cplusplus
}
#endif
#endif /* LCB_CRYPTO_H */
