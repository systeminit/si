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

#include "internal.h"

#define LOGARGS(instance, lvl) instance->settings, "crypto", LCB_LOG_##lvl, __FILE__, __LINE__

void lcbcrypto_ref(lcbcrypto_PROVIDER *provider)
{
    provider->_refcnt++;
}

void lcbcrypto_unref(lcbcrypto_PROVIDER *provider)
{
    provider->_refcnt--;
    if (provider->_refcnt == 0 && provider->destructor) {
        provider->destructor(provider);
    }
}

void lcbcrypto_register(lcb_INSTANCE *instance, const char *name, lcbcrypto_PROVIDER *provider)
{
    if (provider->version != 1) {
        lcb_log(LOGARGS(instance, ERROR), "Unsupported version for \"%s\" crypto provider, ignoring", name);
        return;
    }
    std::map< std::string, lcbcrypto_PROVIDER * >::iterator old = instance->crypto->find(name);
    if (old != instance->crypto->end()) {
        lcbcrypto_unref(old->second);
    }
    lcbcrypto_ref(provider);
    (*instance->crypto)[name] = provider;
}

void lcbcrypto_unregister(lcb_INSTANCE *instance, const char *name)
{
    std::map< std::string, lcbcrypto_PROVIDER * >::iterator old = instance->crypto->find(name);
    if (old != instance->crypto->end()) {
        lcbcrypto_unref(old->second);
        instance->crypto->erase(old);
    }
}

static bool lcbcrypto_is_valid(lcbcrypto_PROVIDER *provider)
{
    if (!(provider && provider->_refcnt > 0)) {
        return false;
    }
    if (provider->version != 1) {
        return false;
    }
    if (provider->v.v1.sign && provider->v.v1.verify_signature == NULL) {
        return false;
    }
    return provider->v.v1.encrypt && provider->v.v1.decrypt && provider->v.v1.get_key_id;
}

#define PROVIDER_NEED_SIGN(provider) (provider)->v.v1.sign != NULL
#define PROVIDER_SIGN(provider, parts, nparts, sig, nsig)                                                              \
    (provider)->v.v1.sign((provider), (parts), (nparts), (sig), (nsig));
#define PROVIDER_VERIFY_SIGNATURE(provider, parts, nparts, sig, nsig)                                                  \
    (provider)->v.v1.verify_signature((provider), (parts), (nparts), (sig), (nsig));

#define PROVIDER_NEED_IV(provider) (provider)->v.v1.generate_iv != NULL
#define PROVIDER_GENERATE_IV(provider, iv, niv) (provider)->v.v1.generate_iv((provider), (iv), (niv))

#define PROVIDER_ENCRYPT(provider, ptext, nptext, iv, niv, ctext, nctext)                                              \
    (provider)->v.v1.encrypt((provider), (ptext), (nptext), (iv), (niv), (ctext), (nctext));
#define PROVIDER_DECRYPT(provider, ctext, nctext, iv, niv, ptext, nptext)                                              \
    (provider)->v.v1.decrypt((provider), (ctext), (nctext), (iv), (niv), (ptext), (nptext));

#define PROVIDER_GET_KEY_ID(provider) (provider)->v.v1.get_key_id((provider));

#define PROVIDER_RELEASE_BYTES(provider, bytes)                                                                        \
    if ((bytes) && (provider)->v.v1.release_bytes) {                                                                   \
        (provider)->v.v1.release_bytes((provider), (bytes));                                                           \
    }

static lcbcrypto_PROVIDER *lcb_get_provider(const lcb_st *instance, const std::string &alg)
{
    const lcb_st::lcb_ProviderMap::iterator provider_iterator = (*instance->crypto).find(alg);
    return provider_iterator != (*instance->crypto).end() ? provider_iterator->second : NULL;
}

lcb_STATUS lcbcrypto_encrypt_fields(lcb_INSTANCE *instance, lcbcrypto_CMDENCRYPT *cmd)
{
    cmd->out = NULL;
    cmd->nout = 0;

    Json::Value jdoc;
    if (!Json::Reader().parse(cmd->doc, cmd->doc + cmd->ndoc, jdoc)) {
        return LCB_EINVAL;
    }
    bool changed = false;
    std::string prefix = (cmd->prefix == NULL) ? LCBCRYPTO_DEFAULT_FIELD_PREFIX : cmd->prefix;
    for (size_t ii = 0; ii < cmd->nfields; ii++) {
        lcbcrypto_FIELDSPEC *field = cmd->fields + ii;
        lcb_STATUS rc;

        if (field->name == NULL) {
            lcb_log(LOGARGS(instance, WARN), "field name cannot be NULL");
            return LCB_EINVAL;
        }

        lcbcrypto_PROVIDER *provider = lcb_get_provider(instance, field->alg);
        if (!lcbcrypto_is_valid(provider)) {
            lcb_log(LOGARGS(instance, WARN), "Invalid crypto provider");
            return LCB_EINVAL;
        }

        if (jdoc.isMember(field->name)) {
            Json::Value encrypted;
            int ret;

            uint8_t *iv = NULL;
            char *biv = NULL;
            size_t niv = 0;
            lcb_SIZE nbiv = 0;
            if (PROVIDER_NEED_IV(provider)) {
                rc = PROVIDER_GENERATE_IV(provider, &iv, &niv);
                if (rc != LCB_SUCCESS) {
                    PROVIDER_RELEASE_BYTES(provider, iv);
                    lcb_log(LOGARGS(instance, WARN), "Unable to generate IV");
                    return rc;
                }
                ret = lcb_base64_encode2(reinterpret_cast< char * >(iv), niv, &biv, &nbiv);
                if (ret < 0) {
                    free(biv);
                    PROVIDER_RELEASE_BYTES(provider, iv);
                    lcb_log(LOGARGS(instance, WARN), "Unable to encode IV as Base64 string");
                    return LCB_EINVAL;
                }
                encrypted["iv"] = biv;
            }

            std::string contents = Json::FastWriter().write(jdoc[field->name]);
            const uint8_t *ptext = reinterpret_cast< const uint8_t * >(contents.c_str());
            uint8_t *ctext = NULL;
            size_t nptext = contents.size(), nctext = 0;
            rc = PROVIDER_ENCRYPT(provider, ptext, nptext, iv, niv, &ctext, &nctext);
            PROVIDER_RELEASE_BYTES(provider, iv);
            if (rc != LCB_SUCCESS) {
                PROVIDER_RELEASE_BYTES(provider, ctext);
                lcb_log(LOGARGS(instance, WARN), "Unable to encrypt field");
                return rc;
            }
            char *btext = NULL;
            lcb_SIZE nbtext = 0;
            ret = lcb_base64_encode2(reinterpret_cast< char * >(ctext), nctext, &btext, &nbtext);
            PROVIDER_RELEASE_BYTES(provider, ctext);
            if (ret < 0) {
                free(btext);
                lcb_log(LOGARGS(instance, WARN), "Unable to encode encrypted field as Base64 string");
                return LCB_EINVAL;
            }
            encrypted["ciphertext"] = btext;
            std::string kid = PROVIDER_GET_KEY_ID(provider);
            encrypted["kid"] = kid;

            if (PROVIDER_NEED_SIGN(provider)) {
                lcbcrypto_SIGV parts[4] = {};
                size_t nparts = 0;
                uint8_t *sig = NULL;
                size_t nsig = 0;

                parts[nparts].data = reinterpret_cast< const uint8_t * >(kid.c_str());
                parts[nparts].len = kid.size();
                nparts++;
                parts[nparts].data = reinterpret_cast< const uint8_t * >(field->alg);
                parts[nparts].len = strlen(field->alg);
                nparts++;
                if (biv) {
                    parts[nparts].data = reinterpret_cast< const uint8_t * >(biv);
                    parts[nparts].len = nbiv;
                    nparts++;
                }
                parts[nparts].data = reinterpret_cast< const uint8_t * >(btext);
                parts[nparts].len = nbtext;
                nparts++;

                rc = PROVIDER_SIGN(provider, parts, nparts, &sig, &nsig);
                if (rc != LCB_SUCCESS) {
                    PROVIDER_RELEASE_BYTES(provider, sig);
                    lcb_log(LOGARGS(instance, WARN), "Unable to sign encrypted field");
                    return rc;
                }
                char *bsig = NULL;
                lcb_SIZE nbsig = 0;
                ret = lcb_base64_encode2(reinterpret_cast< char * >(sig), nsig, &bsig, &nbsig);
                PROVIDER_RELEASE_BYTES(provider, sig);
                if (ret < 0) {
                    free(bsig);
                    lcb_log(LOGARGS(instance, WARN), "Unable to encode signature as Base64 string");
                    return LCB_EINVAL;
                }
                encrypted["sig"] = bsig;
                free(bsig);
            }
            free(biv);
            free(btext);
            encrypted["alg"] = field->alg;
            jdoc[prefix + field->name] = encrypted;
            jdoc.removeMember(field->name);
            changed = true;
        }
    }
    if (changed) {
        std::string doc = Json::FastWriter().write(jdoc);
        cmd->out = strdup(doc.c_str());
        cmd->nout = strlen(cmd->out);
    }
    return LCB_SUCCESS;
}

lcb_STATUS lcbcrypto_decrypt_fields(lcb_INSTANCE *instance, lcbcrypto_CMDDECRYPT *cmd)
{
    cmd->out = NULL;
    cmd->nout = 0;

    Json::Value jdoc;
    if (!Json::Reader().parse(cmd->doc, cmd->doc + cmd->ndoc, jdoc)) {
        return LCB_EINVAL;
    }

    if (!jdoc.isObject()) {
        return LCB_EINVAL;
    }

    bool changed = false;
    std::string prefix = (cmd->prefix == NULL) ? LCBCRYPTO_DEFAULT_FIELD_PREFIX : cmd->prefix;

    for (size_t ii = 0; ii < cmd->nfields; ii++) {
        lcbcrypto_FIELDSPEC *field = cmd->fields + ii;

        if (field->name == NULL) {
            lcb_log(LOGARGS(instance, WARN), "field name cannot be NULL");
            return LCB_EINVAL;
        }
        lcbcrypto_PROVIDER *provider = lcb_get_provider(instance, field->alg);
        if (!lcbcrypto_is_valid(provider)) {
            lcb_log(LOGARGS(instance, WARN), "Invalid crypto provider");
            return LCB_EINVAL;
        }

        std::string name = prefix + field->name;
        if (!jdoc.isMember(name)) {
            continue;
        }
        Json::Value &encrypted = jdoc[name];
        if (!encrypted.isObject()) {
            lcb_log(LOGARGS(instance, WARN), "Expected encrypted field to be an JSON object");
            return LCB_EINVAL;
        }

        Json::Value &jkid = encrypted["kid"];
        if (!jkid.isString()) {
            lcb_log(LOGARGS(instance, WARN), "Expected \"kid\" to be a JSON string");
            return LCB_EINVAL;
        }
        const std::string &kid = jkid.asString();

        Json::Value &jalg = encrypted["alg"];
        if (!jalg.isString()) {
            lcb_log(LOGARGS(instance, WARN), "Expected provider alias \"alg\" to be a JSON string");
            return LCB_EINVAL;
        }
        const std::string &alg = jalg.asString();

        Json::Value &jiv = encrypted["iv"];
        const char *biv = NULL;
        size_t nbiv = 0;
        if (jiv.isString()) {
            biv = jiv.asCString();
            nbiv = strlen(biv);
        }

        int ret;
        lcb_STATUS rc;

        Json::Value &jctext = encrypted["ciphertext"];
        if (!jctext.isString()) {
            lcb_log(LOGARGS(instance, WARN), "Expected encrypted field \"ciphertext\" to be a JSON string");
            return LCB_EINVAL;
        }
        const std::string &btext = jctext.asString();

        if (PROVIDER_NEED_SIGN(provider)) {
            Json::Value &jsig = encrypted["sig"];
            if (!jsig.isString()) {
                lcb_log(LOGARGS(instance, WARN), "Expected signature field \"sig\" to be a JSON string");
                return LCB_EINVAL;
            }
            uint8_t *sig = NULL;
            lcb_SIZE nsig = 0;
            const std::string &bsig = jsig.asString();
            ret = lcb_base64_decode2(bsig.c_str(), bsig.size(), reinterpret_cast< char ** >(&sig), &nsig);
            if (ret < 0) {
                PROVIDER_RELEASE_BYTES(provider, sig);
                lcb_log(LOGARGS(instance, WARN), "Unable to decode signature as Base64 string");
                return LCB_EINVAL;
            }

            lcbcrypto_SIGV parts[4] = {};
            size_t nparts = 0;

            parts[nparts].data = reinterpret_cast< const uint8_t * >(kid.c_str());
            parts[nparts].len = kid.size();
            nparts++;
            parts[nparts].data = reinterpret_cast< const uint8_t * >(alg.c_str());
            parts[nparts].len = alg.size();
            nparts++;
            if (biv) {
                parts[nparts].data = reinterpret_cast< const uint8_t * >(biv);
                parts[nparts].len = nbiv;
                nparts++;
            }
            parts[nparts].data = reinterpret_cast< const uint8_t * >(btext.c_str());
            parts[nparts].len = btext.size();
            nparts++;

            rc = PROVIDER_VERIFY_SIGNATURE(provider, parts, nparts, sig, nsig);
            free(sig);
            if (rc != LCB_SUCCESS) {
                lcb_log(LOGARGS(instance, WARN), "Signature verification for encrypted field \"ciphertext\" failed");
                return rc;
            }
        }

        uint8_t *ctext = NULL;
        lcb_SIZE nctext = 0;
        ret = lcb_base64_decode2(btext.c_str(), btext.size(), reinterpret_cast< char ** >(&ctext), &nctext);
        if (ret < 0) {
            lcb_log(LOGARGS(instance, WARN), "Unable to decode encrypted field \"ciphertext\" as Base64 string");
            return LCB_EINVAL;
        }

        uint8_t *iv = NULL;
        lcb_SIZE niv = 0;
        if (biv) {
            ret = lcb_base64_decode2(biv, nbiv, reinterpret_cast< char ** >(&iv), &niv);
            if (ret < 0) {
                free(ctext);
                lcb_log(LOGARGS(instance, WARN), "Unable to decode IV field \"iv\" as Base64 string");
                return LCB_EINVAL;
            }
        }

        uint8_t *ptext = NULL;
        size_t nptext = 0;
        rc = PROVIDER_DECRYPT(provider, ctext, nctext, iv, niv, &ptext, &nptext);
        free(ctext);
        if (rc != LCB_SUCCESS) {
            PROVIDER_RELEASE_BYTES(provider, ptext);
            lcb_log(LOGARGS(instance, WARN), "Unable to decrypt encrypted field");
            return rc;
        }
        Json::Value frag;
        char *json = reinterpret_cast< char * >(ptext);
        bool valid_json = Json::Reader().parse(json, json + nptext, frag);
        PROVIDER_RELEASE_BYTES(provider, ptext);
        if (!valid_json) {
            lcb_log(LOGARGS(instance, WARN), "Result of decryption is not valid JSON");
            return LCB_EINVAL;
        }
        jdoc[name.substr(prefix.size())] = frag;
        jdoc.removeMember(name);
        changed = true;
    }
    if (changed) {
        std::string doc = Json::FastWriter().write(jdoc);
        cmd->out = strdup(doc.c_str());
        cmd->nout = strlen(cmd->out);
    }
    return LCB_SUCCESS;
}

lcb_STATUS lcbcrypto_encrypt_document(lcb_INSTANCE *, lcbcrypto_CMDENCRYPT *)
{
    return LCB_NOT_SUPPORTED;
}

lcb_STATUS lcbcrypto_decrypt_document(lcb_INSTANCE *, lcbcrypto_CMDDECRYPT *)
{
    return LCB_NOT_SUPPORTED;
}
