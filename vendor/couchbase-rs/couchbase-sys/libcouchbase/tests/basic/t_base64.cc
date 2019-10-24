/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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
#include <gtest/gtest.h>

extern "C" {
extern int lcb_base64_encode(const char *src, size_t nsrc, char *dst, size_t ndst);
extern int lcb_base64_decode(const char *src, size_t nsrc, char *dst, size_t ndst);

extern int lcb_base64_encode2(const char *src, size_t nsrc, char **dst, size_t *ndst);
extern int lcb_base64_decode2(const char *src, size_t nsrc, char **dst, size_t *ndst);
}

class Base64 : public ::testing::Test
{
  protected:
    void validate(const char *src, const char *result)
    {
        char dest[1024];
        ASSERT_GE(lcb_base64_encode(src, strlen(src), dest, sizeof(dest)), 0);
        EXPECT_STREQ(result, dest);
        memset(dest, 0, sizeof(dest));
        ASSERT_GE(lcb_base64_decode(result, strlen(result), dest, sizeof(dest)), 0);
        EXPECT_STREQ(src, dest);

        char *tmp = NULL;
        size_t ntmp = 0;
        ASSERT_GE(lcb_base64_encode2(src, strlen(src), &tmp, &ntmp), 0);
        EXPECT_STREQ(result, tmp);
        free(tmp);
        tmp = NULL;
        ntmp = 0;
        ASSERT_GE(lcb_base64_decode2(result, strlen(result), &tmp, &ntmp), 0);
        EXPECT_STREQ(src, tmp);
        free(tmp);
    }
};

TEST_F(Base64, testRFC4648)
{
    validate("", "");
    validate("f", "Zg==");
    validate("fo", "Zm8=");
    validate("foo", "Zm9v");
    validate("foob", "Zm9vYg==");
    validate("fooba", "Zm9vYmE=");
    validate("foobar", "Zm9vYmFy");
}

TEST_F(Base64, testWikipediaExample)
{
    /* Examples from http://en.wikipedia.org/wiki/Base64 */
    validate("Man is distinguished, not only by his reason, but by this singular "
             "passion from other animals, which is a lust of the mind, that by a "
             "perseverance of delight in the continued and indefatigable generation"
             " of knowledge, exceeds the short vehemence of any carnal pleasure.",
             "TWFuIGlzIGRpc3Rpbmd1aXNoZWQsIG5vdCBvbmx5IGJ5IGhpcyByZWFzb24sIGJ1dCBieSB0aGlz"
             "IHNpbmd1bGFyIHBhc3Npb24gZnJvbSBvdGhlciBhbmltYWxzLCB3aGljaCBpcyBhIGx1c3Qgb2Yg"
             "dGhlIG1pbmQsIHRoYXQgYnkgYSBwZXJzZXZlcmFuY2Ugb2YgZGVsaWdodCBpbiB0aGUgY29udGlu"
             "dWVkIGFuZCBpbmRlZmF0aWdhYmxlIGdlbmVyYXRpb24gb2Yga25vd2xlZGdlLCBleGNlZWRzIHRo"
             "ZSBzaG9ydCB2ZWhlbWVuY2Ugb2YgYW55IGNhcm5hbCBwbGVhc3VyZS4=");
    validate("pleasure.", "cGxlYXN1cmUu");
    validate("leasure.", "bGVhc3VyZS4=");
    validate("easure.", "ZWFzdXJlLg==");
    validate("asure.", "YXN1cmUu");
    validate("sure.", "c3VyZS4=");
}

TEST_F(Base64, testStuff)
{
    // Dummy test data. It looks like the "base64" command line
    // utility from gnu coreutils adds the "\n" to the encoded data...
    validate("Administrator:password", "QWRtaW5pc3RyYXRvcjpwYXNzd29yZA==");
    validate("@", "QA==");
    validate("@\n", "QAo=");
    validate("@@", "QEA=");
    validate("@@\n", "QEAK");
    validate("@@@", "QEBA");
    validate("@@@\n", "QEBACg==");
    validate("@@@@", "QEBAQA==");
    validate("@@@@\n", "QEBAQAo=");
    validate("blahblah:bla@@h", "YmxhaGJsYWg6YmxhQEBo");
    validate("blahblah:bla@@h\n", "YmxhaGJsYWg6YmxhQEBoCg==");
}

TEST_F(Base64, testTooSmallOutputBuffer)
{
    const char *plain = "foobar";
    const char *base64 = "Zm9vYmFy";

    char dest[2];

    ASSERT_EQ(lcb_base64_encode(plain, strlen(plain), dest, sizeof(dest)), -1);
    ASSERT_EQ(lcb_base64_decode(base64, strlen(base64), dest, sizeof(dest)), -1);
}
