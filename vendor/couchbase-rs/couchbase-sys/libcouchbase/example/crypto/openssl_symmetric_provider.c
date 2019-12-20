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

/**
 * This is an example of using crypto API of libcouchbase. The implementation should not be considered as production
 * ready, because it uses hardcoded keys, insecure memory allocation, copying and comparison. Consult documentation of
 * your crypto library on how to properly work with keys and buffers.
 */

#include <stdlib.h>
#include <string.h>
#include "openssl_symmetric_provider.h"

#include <openssl/ssl.h>
#include <openssl/conf.h>
#include <openssl/evp.h>
#include <openssl/err.h>

static void osp_free(lcbcrypto_PROVIDER *provider)
{
    free(provider);
}

static void osp_release_bytes(lcbcrypto_PROVIDER *provider, void *bytes)
{
    free(bytes);
    (void)provider;
}

static const char *osp_get_key_id(lcbcrypto_PROVIDER *provider)
{
    return common_aes256_key_id;
}

static lcb_STATUS osp_generate_iv(struct lcbcrypto_PROVIDER *provider, uint8_t **iv, size_t *iv_len)
{
    *iv_len = AES256_IV_SIZE;
    *iv = malloc(*iv_len);
    memcpy(*iv, common_aes256_iv, *iv_len);

    (void)provider;
    return LCB_SUCCESS;
}

static lcb_STATUS osp_sign(struct lcbcrypto_PROVIDER *provider, const lcbcrypto_SIGV *inputs, size_t inputs_num,
                           uint8_t **sig, size_t *sig_len)
{
    const EVP_MD *md;
    uint8_t out[EVP_MAX_MD_SIZE];
    size_t out_len = EVP_MAX_MD_SIZE;
    int rc, key_len = strlen((const char *)common_hmac_sha256_key);
    EVP_MD_CTX *ctx = NULL;
    EVP_PKEY *key = NULL;
    size_t ii;

    md = EVP_get_digestbyname("SHA256");
    if (md == NULL) {
        return LCB_EINVAL;
    }

    key = EVP_PKEY_new_mac_key(EVP_PKEY_HMAC, NULL, common_hmac_sha256_key, key_len);
    if (key == NULL) {
        return LCB_EINVAL;
    }

    ctx = EVP_MD_CTX_new();
    if (ctx == NULL) {
        EVP_PKEY_free(key);
        return LCB_EINVAL;
    }
    rc = EVP_DigestSignInit(ctx, NULL, md, NULL, key);
    if (rc != 1) {
        EVP_PKEY_free(key);
        return LCB_EINVAL;
    }

    for (ii = 0; ii < inputs_num; ii++) {
        rc = EVP_DigestSignUpdate(ctx, inputs[ii].data, inputs[ii].len);
        if (rc != 1) {
            EVP_PKEY_free(key);
            EVP_MD_CTX_destroy(ctx);
            return LCB_EINVAL;
        }
    }
    rc = EVP_DigestSignFinal(ctx, out, &out_len);
    if (rc != 1 || out_len == 0) {
        EVP_PKEY_free(key);
        EVP_MD_CTX_destroy(ctx);
        return LCB_EINVAL;
    }
    *sig = malloc(out_len);
    memcpy(*sig, out, out_len);
    *sig_len = out_len;

    return LCB_SUCCESS;
}

static lcb_STATUS osp_verify_signature(struct lcbcrypto_PROVIDER *provider, const lcbcrypto_SIGV *inputs,
                                       size_t inputs_num, uint8_t *sig, size_t sig_len)
{
    const EVP_MD *md;
    uint8_t actual[EVP_MAX_MD_SIZE];
    size_t actual_len = EVP_MAX_MD_SIZE;
    int rc, key_len = strlen((const char *)common_hmac_sha256_key);
    EVP_MD_CTX *ctx = NULL;
    EVP_PKEY *key = NULL;
    size_t ii;

    md = EVP_get_digestbyname("SHA256");
    if (md == NULL) {
        return LCB_EINVAL;
    }
    key = EVP_PKEY_new_mac_key(EVP_PKEY_HMAC, NULL, common_hmac_sha256_key, key_len);
    if (key == NULL) {
        return LCB_EINVAL;
    }

    ctx = EVP_MD_CTX_new();
    if (ctx == NULL) {
        EVP_PKEY_free(key);
        return LCB_EINVAL;
    }
    rc = EVP_DigestSignInit(ctx, NULL, md, NULL, key);
    if (rc != 1) {
        EVP_PKEY_free(key);
        return LCB_EINVAL;
    }

    for (ii = 0; ii < inputs_num; ii++) {
        rc = EVP_DigestSignUpdate(ctx, inputs[ii].data, inputs[ii].len);
        if (rc != 1) {
            EVP_PKEY_free(key);
            EVP_MD_CTX_destroy(ctx);
            return LCB_EINVAL;
        }
    }
    rc = EVP_DigestSignFinal(ctx, actual, &actual_len);
    if (rc != 1 || actual_len == 0) {
        EVP_PKEY_free(key);
        EVP_MD_CTX_destroy(ctx);
        return LCB_EINVAL;
    }

    if (memcmp(actual, sig, sig_len < actual_len ? sig_len : actual_len) == 0) {
        return LCB_SUCCESS;
    }
    return LCB_EINVAL;
}

static lcb_STATUS osp_encrypt(struct lcbcrypto_PROVIDER *provider, const uint8_t *input, size_t input_len,
                              const uint8_t *iv, size_t iv_len, uint8_t **output, size_t *output_len)
{
    EVP_CIPHER_CTX *ctx;
    const EVP_CIPHER *cipher;
    int rc, len, block_len, out_len;
    uint8_t *out;

    if (iv_len != 16) {
        return LCB_EINVAL;
    }

    ctx = EVP_CIPHER_CTX_new();
    if (!ctx) {
        return LCB_EINVAL;
    }
    cipher = EVP_aes_256_cbc();
    rc = EVP_EncryptInit_ex(ctx, cipher, NULL, common_aes256_key, iv);
    if (rc != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    block_len = EVP_CIPHER_block_size(cipher);
    out = calloc(input_len + block_len - 1, sizeof(uint8_t));
    rc = EVP_EncryptUpdate(ctx, out, &len, input, input_len);
    if (rc != 1) {
        free(out);
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    out_len = len;
    rc = EVP_EncryptFinal_ex(ctx, out + len, &len);
    if (rc != 1) {
        free(out);
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    out_len += len;
    EVP_CIPHER_CTX_free(ctx);
    *output = out;
    *output_len = out_len;
    return LCB_SUCCESS;
}

static lcb_STATUS osp_decrypt(struct lcbcrypto_PROVIDER *provider, const uint8_t *input, size_t input_len,
                              const uint8_t *iv, size_t iv_len, uint8_t **output, size_t *output_len)
{
    EVP_CIPHER_CTX *ctx;
    const EVP_CIPHER *cipher;
    int rc, len, out_len;
    uint8_t *out;

    if (iv_len != 16) {
        return LCB_EINVAL;
    }

    ctx = EVP_CIPHER_CTX_new();
    if (!ctx) {
        return LCB_EINVAL;
    }
    cipher = EVP_aes_256_cbc();
    rc = EVP_DecryptInit_ex(ctx, cipher, NULL, common_aes256_key, iv);
    if (rc != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    out = calloc(input_len, sizeof(uint8_t));
    rc = EVP_DecryptUpdate(ctx, out, &len, input, input_len);
    if (rc != 1) {
        free(out);
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    out_len = len;
    rc = EVP_DecryptFinal_ex(ctx, out + len, &len);
    if (rc != 1) {
        free(out);
        EVP_CIPHER_CTX_free(ctx);
        return LCB_EINVAL;
    }
    out_len += len;
    EVP_CIPHER_CTX_free(ctx);
    *output = out;
    *output_len = out_len;
    return LCB_SUCCESS;
}

lcbcrypto_PROVIDER *osp_create()
{
    lcbcrypto_PROVIDER *provider = calloc(1, sizeof(lcbcrypto_PROVIDER));
    provider->version = 1;
    provider->destructor = osp_free;
    provider->v.v1.release_bytes = osp_release_bytes;
    provider->v.v1.generate_iv = osp_generate_iv;
    provider->v.v1.sign = osp_sign;
    provider->v.v1.verify_signature = osp_verify_signature;
    provider->v.v1.encrypt = osp_encrypt;
    provider->v.v1.decrypt = osp_decrypt;
    provider->v.v1.get_key_id = osp_get_key_id;
    return provider;
}

void osp_initialize()
{
    SSL_library_init();
    SSL_load_error_strings();
    EVP_add_cipher(EVP_aes_256_cbc());
}
