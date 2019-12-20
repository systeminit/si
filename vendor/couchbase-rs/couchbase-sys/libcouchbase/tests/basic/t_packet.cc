/** for ntohl/htonl */
#ifndef _WIN32
#include <netinet/in.h>
#else
#include "winsock2.h"
#endif

#include <libcouchbase/couchbase.h>
#include "config.h"
#include <gtest/gtest.h>
#include "packetutils.h"

class Packet : public ::testing::Test
{
};

class Pkt
{
  public:
    Pkt() : pkt(NULL), len(0) {}

    void getq(const std::string &value, lcb_uint32_t opaque, lcb_uint16_t status = 0, lcb_cas_t cas = 0,
              lcb_uint32_t flags = 0)
    {
        protocol_binary_response_getq msg;
        protocol_binary_response_header *hdr = &msg.message.header;
        memset(&msg, 0, sizeof(msg));

        hdr->response.magic = PROTOCOL_BINARY_RES;
        hdr->response.opaque = opaque;
        hdr->response.status = htons(status);
        hdr->response.opcode = PROTOCOL_BINARY_CMD_GETQ;
        hdr->response.cas = lcb_htonll(cas);
        hdr->response.bodylen = htonl((lcb_uint32_t)value.size() + 4);
        hdr->response.extlen = 4;
        msg.message.body.flags = htonl(flags);

        // Pack the response
        clear();
        len = sizeof(msg.bytes) + value.size();
        pkt = new char[len];

        EXPECT_TRUE(pkt != NULL);

        memcpy(pkt, msg.bytes, sizeof(msg.bytes));

        memcpy((char *)pkt + sizeof(msg.bytes), value.c_str(), (unsigned long)value.size());
    }

    void get(const std::string &key, const std::string &value, lcb_uint32_t opaque, lcb_uint16_t status = 0,
             lcb_cas_t cas = 0, lcb_uint32_t flags = 0)
    {
        protocol_binary_response_getq msg;
        protocol_binary_response_header *hdr = &msg.message.header;
        hdr->response.magic = PROTOCOL_BINARY_RES;
        hdr->response.opaque = opaque;
        hdr->response.cas = lcb_htonll(cas);
        hdr->response.opcode = PROTOCOL_BINARY_CMD_GET;
        hdr->response.keylen = htons((lcb_uint16_t)key.size());
        hdr->response.extlen = 4;
        hdr->response.bodylen = htonl(key.size() + value.size() + 4);
        hdr->response.status = htons(status);
        msg.message.body.flags = flags;

        clear();
        len = sizeof(msg.bytes) + value.size() + key.size();
        pkt = new char[len];
        char *ptr = pkt;

        memcpy(ptr, msg.bytes, sizeof(msg.bytes));
        ptr += sizeof(msg.bytes);
        memcpy(ptr, key.c_str(), (unsigned long)key.size());
        ptr += key.size();
        memcpy(ptr, value.c_str(), (unsigned long)value.size());
    }

    void rbWrite(rdb_IOROPE *ior)
    {
        rdb_copywrite(ior, pkt, len);
    }

    void rbWriteHeader(rdb_IOROPE *ior)
    {
        rdb_copywrite(ior, pkt, 24);
    }

    void rbWriteBody(rdb_IOROPE *ior)
    {
        rdb_copywrite(ior, pkt + 24, len - 24);
    }

    void writeGenericHeader(unsigned long bodylen, rdb_IOROPE *ior)
    {
        protocol_binary_response_header hdr;
        memset(&hdr, 0, sizeof(hdr));
        hdr.response.opcode = 0;
        hdr.response.bodylen = htonl(bodylen);
        rdb_copywrite(ior, hdr.bytes, sizeof(hdr.bytes));
    }

    ~Pkt()
    {
        clear();
    }

    void clear()
    {
        if (pkt != NULL) {
            delete[] pkt;
        }
        pkt = NULL;
        len = 0;
    }

    size_t size()
    {
        return len;
    }

  private:
    char *pkt;
    size_t len;
    Pkt(Pkt &);
};

TEST_F(Packet, testParseBasic)
{
    std::string value = "foo";
    rdb_IOROPE ior;
    rdb_init(&ior, rdb_libcalloc_new());

    Pkt pkt;
    pkt.getq(value, 0);
    pkt.rbWrite(&ior);

    lcb::MemcachedResponse pi;
    memset(&pi, 0, sizeof(pi));
    unsigned wanted;
    ASSERT_TRUE(pi.load(&ior, &wanted));

    ASSERT_EQ(0, pi.status());
    ASSERT_EQ(PROTOCOL_BINARY_CMD_GETQ, pi.opcode());
    ASSERT_EQ(0, pi.opaque());
    ASSERT_EQ(7, pi.bodylen());
    ASSERT_EQ(3, pi.vallen());
    ASSERT_EQ(0, pi.keylen());
    ASSERT_EQ(4, pi.extlen());
    ASSERT_EQ(pi.bodylen(), rdb_get_nused(&ior));
    ASSERT_EQ(0, strncmp(value.c_str(), pi.value(), 3));

    pi.release(&ior);
    ASSERT_EQ(0, rdb_get_nused(&ior));
    rdb_cleanup(&ior);
}

TEST_F(Packet, testParsePartial)
{
    rdb_IOROPE ior;
    Pkt pkt;
    rdb_init(&ior, rdb_libcalloc_new());

    std::string value;
    value.insert(0, 1024, '*');

    lcb::MemcachedResponse pi;

    // Test where we're missing just one byte
    pkt.writeGenericHeader(10, &ior);
    unsigned wanted;
    ASSERT_FALSE(pi.load(&ior, &wanted));

    for (int ii = 0; ii < 9; ii++) {
        char c = 'O';
        rdb_copywrite(&ior, &c, 1);
        ASSERT_FALSE(pi.load(&ior, &wanted));
    }
    char tmp = 'O';
    rdb_copywrite(&ior, &tmp, 1);
    ASSERT_TRUE(pi.load(&ior, &wanted));
    pi.release(&ior);
    rdb_cleanup(&ior);
}

TEST_F(Packet, testKeys)
{
    rdb_IOROPE ior;
    rdb_init(&ior, rdb_libcalloc_new());
    std::string key = "a simple key";
    std::string value = "a simple value";
    Pkt pkt;
    pkt.get(key, value, 1000, PROTOCOL_BINARY_RESPONSE_ETMPFAIL, 0xdeadbeef, 50);
    pkt.rbWrite(&ior);

    lcb::MemcachedResponse pi;
    unsigned wanted;
    ASSERT_TRUE(pi.load(&ior, &wanted));

    ASSERT_EQ(key.size(), pi.keylen());
    ASSERT_EQ(0, memcmp(key.c_str(), pi.key(), pi.keylen()));
    ASSERT_EQ(value.size(), pi.vallen());
    ASSERT_EQ(0, memcmp(value.c_str(), pi.value(), pi.vallen()));
    ASSERT_EQ(0xdeadbeef, pi.cas());
    ASSERT_EQ(PROTOCOL_BINARY_RESPONSE_ETMPFAIL, pi.status());
    ASSERT_EQ(PROTOCOL_BINARY_CMD_GET, pi.opcode());
    ASSERT_EQ(4, pi.extlen());
    ASSERT_EQ(4 + key.size() + value.size(), pi.bodylen());
    ASSERT_NE(pi.body< const char * >(), pi.value());
    ASSERT_EQ(4 + key.size(), pi.value() - pi.body< const char * >());

    pi.release(&ior);
    rdb_cleanup(&ior);
}
