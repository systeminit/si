#include <gtest/gtest.h>
#include <lcbht/lcbht.h>
#include <sstream>
#include <map>
#include "settings.h"

using std::string;
using namespace lcb::htparse;

class HtparseTest : public ::testing::Test
{
};

TEST_F(HtparseTest, testBasic)
{
    lcb_settings *settings = lcb_settings_new();
    // Allocate a parser
    Parser *parser = new Parser(settings);
    ASSERT_FALSE(parser == NULL);

    // Feed the parser some stuff
    unsigned state;

    string buf;
    buf = "HTTP/1.0 200 OK\r\n";

    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_EQ(0, state);

    buf = "Connec";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_EQ(0, state);
    buf = "tion: Keep-Alive\r\n";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_EQ(0, state);
    buf += "Content-Length: 5\r\n\r\n";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_EQ(Parser::S_HEADER | Parser::S_HTSTATUS, state);

    Response &resp = parser->get_cur_response();
    ASSERT_EQ(200, resp.status);

    // Add some data into the body
    buf = "H";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_EQ("H", resp.body);

    buf = "ello";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_DONE);
    ASSERT_EQ("Hello", resp.body);

    // Now find the header
    delete parser;
    lcb_settings_unref(settings);
}

TEST_F(HtparseTest, testHeaderFunctions)
{
    lcb_settings *settings = lcb_settings_new();
    Parser *parser = new Parser(settings);

    string buf = "HTTP/1.0 200 OK\r\n"
                 "Connection: keep-alive\r\n"
                 "X-Server: dummy/1.0\r\n"
                 "Content-Type: application/json\r\n"
                 "Content-Length: 0\r\n"
                 "\r\n";
    unsigned state;
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_DONE);

    Response &resp = parser->get_cur_response();
    // Now fine the header value stuff
    ASSERT_STREQ("keep-alive", resp.get_header_value("Connection"));
    ASSERT_STREQ("dummy/1.0", resp.get_header_value("X-Server"));
    ASSERT_STREQ("application/json", resp.get_header_value("Content-Type"));
    delete parser;
    lcb_settings_unref(settings);
}

TEST_F(HtparseTest, testParseErrors)
{
    lcb_settings *settings = lcb_settings_new();
    Parser *parser = new Parser(settings);

    string buf = "blahblahblah";
    unsigned state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_ERROR);
    delete parser;
    lcb_settings_unref(settings);
}

TEST_F(HtparseTest, testParseExtended)
{
    lcb_settings *settings = lcb_settings_new();
    Parser *parser = new Parser(settings);

    const char *body;
    unsigned nbody, nused;

    string buf = "HTTP/1.0 200 OK\r\n"
                 "Connection: keep-alive\r\n"
                 "Content-Length: 5\r\n";

    unsigned state;
    state = parser->parse_ex(buf.c_str(), buf.size(), &nused, &nbody, &body);
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_EQ(NULL, body);
    ASSERT_EQ(buf.size(), nused);
    ASSERT_EQ(0, nbody);

    Response &resp = parser->get_cur_response();
    buf = "\r\nHello";
    // Feed the buffer
    state = parser->parse_ex(buf.c_str(), buf.size(), &nused, &nbody, &body);
    ASSERT_EQ(0, state & Parser::S_DONE);
    ASSERT_EQ(5, nbody);
    ASSERT_FALSE(NULL == body);
    ASSERT_STREQ("Hello", body);
    ASSERT_EQ(buf.size() - 1, nused);

    size_t off = nused;

    // Parse again
    state = parser->parse_ex(buf.c_str() + off, buf.size() - off, &nused, &nbody, &body);
    ASSERT_EQ(nused, buf.size() - off);
    ASSERT_TRUE(body == NULL);
    ASSERT_EQ(0, nbody);
    ASSERT_NE(0, state & Parser::S_DONE);
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_EQ(0, resp.body.size());
    delete parser;
    lcb_settings_unref(settings);
}

TEST_F(HtparseTest, testCanKeepalive)
{
    lcb_settings *settings = lcb_settings_new();
    Parser *parser = new Parser(settings);
    string buf = "HTTP/1.0 200 OK\r\n"
                 "Content-Length: 0\r\n"
                 "\r\n";
    unsigned state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_DONE);
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_FALSE(parser->can_keepalive());

    // Use HTTP/1.1 with Connection: close
    parser->reset();
    buf = "HTTP/1.1 200 OK\r\n"
          "Content-Length: 0\r\n"
          "Connection: close\r\n"
          "\r\n";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_DONE);
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_FALSE(parser->can_keepalive());

    parser->reset();
    // Default HTTP/1.1
    buf = "HTTP/1.1 200 OK\r\n"
          "Content-Length: 0\r\n"
          "\r\n";
    state = parser->parse(buf.c_str(), buf.size());
    ASSERT_NE(0, state & Parser::S_DONE);
    ASSERT_EQ(0, state & Parser::S_ERROR);
    ASSERT_TRUE(parser->can_keepalive());

    delete parser;
    lcb_settings_unref(settings);
}
