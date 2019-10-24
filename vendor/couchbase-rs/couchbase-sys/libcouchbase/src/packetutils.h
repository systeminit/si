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

#ifndef LCB_PACKETUTILS_H
#define LCB_PACKETUTILS_H

#include "config.h"

#include <libcouchbase/couchbase.h>
#include <memcached/protocol_binary.h>
#include "rdb/rope.h"

#ifndef __cplusplus
typedef struct packet_info_st packet_info;
#else
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include <math.h>
namespace lcb
{
class Server;

/**
 * Response packet informational structure.
 *
 * This contains information regarding the response packet which is used by
 * the response processors.
 */
class MemcachedResponse
{
  public:
    MemcachedResponse() : payload(NULL), bufh(NULL)
    {
        // Bodyless. Members are initialized via load!
    }

    MemcachedResponse(protocol_binary_command cmd, uint32_t opaque_, protocol_binary_response_status code)
        : res(), payload(NULL), bufh(NULL)
    {
        res.response.opcode = cmd;
        res.response.opaque = opaque_;
        res.response.status = htons(code);
    }
    /**
     * Read from an 'IOR' structure to parse the packet information. This will
     * always load a full packet.
     *
     * @param ior the rope structure to read from
     * @param[out] required how much total bytes must remain in the buffer for the
     *  parse to complete.
     *
     * @return false if more data is needed, true otherwise
     */
    bool load(rdb_IOROPE *ior, unsigned *required)
    {
        unsigned total = rdb_get_nused(ior);
        unsigned wanted = sizeof(res.bytes);

        if (total < wanted) {
            *required = wanted;
            return false;
        }

        rdb_copyread(ior, res.bytes, sizeof(res.bytes));
        if (!bodylen()) {
            rdb_consumed(ior, sizeof(res.bytes));
            return true;
        }

        wanted += bodylen();
        if (total < wanted) {
            *required = wanted;
            return false;
        }

        rdb_consumed(ior, sizeof(res.bytes));
        payload = rdb_get_consolidated(ior, bodylen());
        return true;
    }

    template < typename T > bool load(T ctx, unsigned *required)
    {
        return load(&ctx->ior, required);
    }

    void release(rdb_IOROPE *ior)
    {
        if (!bodylen()) {
            return;
        }
        rdb_consumed(ior, bodylen());
    }

    template < typename T > void release(T ctx)
    {
        release(&ctx->ior);
    }

    /**
     * Gets the command for the packet
     */
    uint8_t opcode() const
    {
        return res.response.opcode;
    }

    /**
     * Gets the CAS for the packet
     */
    uint64_t cas() const
    {
        return lcb_ntohll(res.response.cas);
    }

    /**
     * Gets the 'datatype' field for the packet.
     */
    uint8_t datatype() const
    {
        return res.response.datatype;
    }

#define FRAMING_EXTRAS_TRACING 0x00
#if defined(_MSC_VER)
#define __lcb_round(d) ((d) > 0.0) ? ((d) + 0.5) : ((d)-0.5)
#else
#define __lcb_round(d) round(d)
#endif

    uint64_t duration() const
    {
        if (ffextlen() == 0) {
            return 0;
        }

        const char *end, *ptr;
        ptr = ffext();
        end = ptr + ffextlen();

        for (; ptr < end;) {
            uint8_t control = *ptr;
            uint8_t id = control & 0xF0;
            uint8_t len = control & 0x0F;
            ptr++;
            if (id == FRAMING_EXTRAS_TRACING && len == sizeof(uint16_t)) {
                uint16_t encoded;
                memcpy(&encoded, ptr, sizeof(uint16_t));
                encoded = ntohs(encoded);
                return (uint64_t)__lcb_round(pow(encoded, 1.74) / 2);
            }
            ptr += len;
        }
        return 0;
    }

#undef __lcb_round

    /**
     * Gets a pointer starting at the packet's flexible framing ext field
     */
    const char *ffext() const
    {
        return body< const char * >();
    }

    /**
     * Gets a pointer starting at the packet's ext field.
     */
    const char *ext() const
    {
        return body< const char * >() + ffextlen();
    }

    /**
     * Gets a pointer starting at the packet's key field. Only use if NKEY is 0
     */
    const char *key() const
    {
        return body< const char * >() + extlen() + ffextlen();
    }

    /**
     * Gets a pointer starting at the packet's value field. Only use if NVALUE is 0
     */
    const char *value() const
    {
        return body< const char * >() + keylen() + extlen() + ffextlen();
    }

    /**
     * Gets the size of the packet value. The value is the part of the payload
     * which is after the key (if applicable) and extras (if applicable).
     */
    uint32_t vallen() const
    {
        return bodylen() - (keylen() + extlen() + ffextlen());
    }

    /**
     * Gets the status of the packet
     */
    uint16_t status() const
    {
        return ntohs(res.response.status);
    }

    /**
     * Gets the payload
     */
    template < typename T > const T body() const
    {
        return reinterpret_cast< const T >(payload);
    }

    /**
     * Map a command 'subclass' so that its body field starts at the payload. Note
     * that the return value is actually an ephemeral pointer starting 24 bytes
     * _before_ the actual memory block, so only use the non-header part.
     */
    const char *ephemeral_start() const
    {
        return body< const char * >() - 24;
    }

    /**
     * Gets the size of the _total_ non-header part of the packet. This data is
     * also featured inside the payload field itself.
     */
    uint32_t bodylen() const
    {
        return ntohl(res.response.bodylen);
    }

    /**
     * Gets the key size, if included in the packet.
     */
    uint16_t keylen() const
    {
        if (res.response.magic == PROTOCOL_BINARY_ARES) {
            return (res.response.keylen >> 8) & 0xff;
        } else {
            return ntohs(res.response.keylen);
        }
    }

    /**
     * Gets the length of the 'extras' in the body
     */
    uint8_t extlen() const
    {
        return (res.response.extlen);
    }

    /**
     * Gets flexible framing extras length
     */
    uint8_t ffextlen() const
    {
        if (res.response.magic == PROTOCOL_BINARY_ARES) {
            return res.response.keylen & 0xff;
        } else {
            return 0;
        }
    }

    /**
     * Gets the raw unconverted 'opaque' 32 bit field
     */
    uint32_t opaque() const
    {
        return (res.response.opaque);
    }

    size_t hdrsize() const
    {
        return sizeof(res.bytes);
    }

    uint8_t *hdrbytes()
    {
        return res.bytes;
    }

    void *bufseg() const
    {
        return bufh;
    }

    static lcb_STATUS parse_enhanced_error(const char *value, lcb_SIZE nvalue, char **err_ref, char **err_ctx)
    {
        if (value == NULL || nvalue == 0) {
            return LCB_EINVAL;
        }
        Json::Value jval;
        if (!Json::Reader().parse(value, value + nvalue, jval)) {
            return LCB_EINVAL;
        }
        if (jval.empty()) {
            return LCB_EINVAL;
        }
        Json::Value jerr = jval["error"];
        if (jerr.empty()) {
            return LCB_EINVAL;
        }
        std::string emsg;
        if (!jerr["ref"].empty()) {
            *err_ref = strdup(jerr["ref"].asString().c_str());
        }
        if (!jerr["context"].empty()) {
            *err_ctx = strdup(jerr["context"].asString().c_str());
        }
        return LCB_SUCCESS;
    }

  protected:
    /** The response header */
    protocol_binary_response_header res;
    /** The payload of the response. This should only be used if there is a body */
    void *payload;
    /** Segment for payload */
    void *bufh;

    friend class lcb::Server;
};

#define PACKET_REQUEST(pkt) ((protocol_binary_request_header *)&(pkt)->res)

#define PACKET_REQ_VBID(pkt) (ntohs(PACKET_REQUEST(pkt)->request.vbucket))

class MemcachedRequest
{
  public:
    /**
     * Declare the extras, key, and value size for the packet
     * @param extlen Length of extras
     * @param keylen Length of key
     * @param valuelen Length of value (i.e. minus extras and key)
     */
    void sizes(uint8_t extlen, uint16_t keylen, uint32_t valuelen)
    {
        hdr.request.bodylen = htonl(extlen + keylen + valuelen);
        hdr.request.keylen = htons(keylen);
        hdr.request.extlen = extlen;
    }

    void vbucket(uint16_t vb)
    {
        hdr.request.vbucket = htons(vb);
    }

    void opaque(uint32_t opaque_)
    {
        hdr.request.opaque = opaque_;
    }

    uint32_t opaque() const
    {
        return hdr.request.opaque;
    }

    uint8_t opcode() const
    {
        return hdr.request.opcode;
    }

    MemcachedRequest(uint8_t opcode_)
    {
        assign(opcode_);
    }

    MemcachedRequest(uint8_t opcode_, uint32_t opaque_)
    {
        assign(opcode_);
        hdr.request.opaque = opaque_;
    }

    MemcachedRequest(const void *buf)
    {
        memcpy(hdr.bytes, buf, sizeof hdr.bytes);
    }

    const void *data() const
    {
        return hdr.bytes;
    }
    size_t size() const
    {
        return sizeof hdr.bytes;
    }

  private:
    protocol_binary_request_header hdr;

    void assign(uint8_t opcode_)
    {
        hdr.request.opcode = opcode_;
        hdr.request.magic = PROTOCOL_BINARY_REQ;
        hdr.request.datatype = PROTOCOL_BINARY_RAW_BYTES;
        hdr.request.cas = 0;
        hdr.request.vbucket = 0;
        hdr.request.opaque = 0;
        hdr.request.bodylen = 0;
        hdr.request.extlen = 0;
        hdr.request.keylen = 0;
        hdr.request.opaque = 0;
    }
};
} // namespace lcb
typedef lcb::MemcachedResponse packet_info;
#endif
#endif
