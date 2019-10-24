#ifndef PKTMAKER_H
#define PKTMAKER_H

#include <memcached/protocol_binary.h>
namespace PacketMaker
{

class Packet
{
  public:
    Packet()
    {
        memset(&hdr_s, 0, sizeof(hdr_s));
        req = &hdr_s;
        res = (protocol_binary_response_header *)&hdr_s;
    }

    uint8_t magic() const
    {
        return req->request.magic;
    }
    void magic(uint8_t mg)
    {
        req->request.magic = mg;
    }

    uint8_t op() const
    {
        return req->request.opcode;
    }
    void op(uint8_t cc)
    {
        req->request.opcode = cc;
    }

    uint8_t extlen() const
    {
        return req->request.extlen;
    }

    void opaque(uint32_t seq)
    {
        req->request.opaque = seq;
    }
    uint32_t opaque() const
    {
        return req->request.opaque;
    }

    const char *keyptr() const
    {
        return &body[extlen()];
    }

    std::string key() const
    {
        uint16_t len = keylen();
        if (!len) {
            return std::string("");
        }
        return std::string(keyptr(), len);
    }

    uint16_t keylen() const
    {
        return ntohs(req->request.keylen);
    }

    void serialize(std::vector< char > &ret)
    {
        req->request.bodylen = htonl(body.size());
        ret.insert(ret.end(), hdr_s.bytes, hdr_s.bytes + sizeof(hdr_s.bytes));
        ret.insert(ret.end(), body.begin(), body.end());
    }

    void load(std::vector< char > &buf)
    {
        memcpy(&hdr_s.bytes, &buf[0], sizeof(hdr_s.bytes));
        body.assign(&buf[24], &buf[0] + buf.size());
    }

    void setKey(const char *kbuf, uint16_t len)
    {
        body.insert(body.end(), kbuf, kbuf + len);
        req->request.keylen = htons((uint16_t)len);
    }

    void setValue(const char *val, uint32_t len)
    {
        body.insert(body.end(), val, val + len);
    }

  protected:
    protocol_binary_request_header *req;
    protocol_binary_response_header *res;

    void addExtra(char *extbuf, uint8_t nbuf)
    {
        body.insert(body.begin(), extbuf, extbuf + nbuf);
        req->request.extlen += nbuf;
    }

  private:
    protocol_binary_request_header hdr_s;
    std::vector< char > body;
    Packet(Packet &);
};

class StorageRequest : public Packet
{
  public:
    StorageRequest(const std::string &key, const std::string &val) : Packet()
    {
        setKey(key.c_str(), (uint16_t)key.size());
        setValue(val.c_str(), (uint32_t)val.size());
    }
};

class GetRequest : public Packet
{
  public:
    GetRequest(std::string &key) : Packet()
    {
        setKey(key.c_str(), (uint16_t)key.size());
    }
};

class Response : public Packet
{
  public:
    Response(const Packet &request, uint16_t status = 0) : Packet()
    {
        res->response.status = htons(status);
        opaque(request.opaque());
    }
};

} // namespace PacketMaker

#endif
