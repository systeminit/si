/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2019 Couchbase, Inc.
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
#include <libcouchbase/couchbase.h>
#include <strcodecs/strcodecs.h>

class UrlEncoding : public ::testing::Test
{
};

using lcb::strcodecs::urldecode;
using lcb::strcodecs::urlencode;

TEST_F(UrlEncoding, plainTextTests)
{
    std::string input("abcdef");
    std::string exp("abcdef");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(exp, out);
}

TEST_F(UrlEncoding, plainTextWithSlashTests)
{
    std::string input("a/b/c/d/e/f/g/h/i/j");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(input, out);
}

TEST_F(UrlEncoding, plainTextWithSpaceTests)
{
    std::string out;
    std::string input("a b c d e f g");
    std::string exp("a%20b%20c%20d%20e%20f%20g");
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(exp, out);
}

TEST_F(UrlEncoding, encodedTextWithPlusAsApaceTests)
{
    std::string input("a+b+c+d+e+g+h");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(input, out);
}

TEST_F(UrlEncoding, encodedTextWithPlusAndHexAsApaceTests)
{
    std::string input("a+b%20c%20d+e+g+h");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(input, out);
}

TEST_F(UrlEncoding, mixedLegalTextTests)
{
    std::string input("a/b/c/d/e f g+32%20");
    std::string exp("a/b/c/d/e%20f%20g+32%20");
    std::string out;

    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(exp, out);
}

TEST_F(UrlEncoding, mixedIllegalEncodingTextTests)
{
    std::string input("a+ ");
    std::string out;
    ASSERT_FALSE(urlencode(input.begin(), input.end(), out));
}

TEST_F(UrlEncoding, internationalTest)
{
    std::string input("_design/beer/_view/all?startkey=\"\xc3\xb8l\"");
    std::string exp("_design/beer/_view/all?startkey=%22%C3%B8l%22");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(exp, out);
}

TEST_F(UrlEncoding, internationalEncodedTest)
{
    std::string input("_design/beer/_view/all?startkey=%22%C3%B8l%22");
    std::string exp("_design/beer/_view/all?startkey=%22%C3%B8l%22");
    std::string out;
    ASSERT_TRUE(urlencode(input.begin(), input.end(), out));
    ASSERT_EQ(exp, out);
}

TEST_F(UrlEncoding, testDecode)
{
    char obuf[4096];

    ASSERT_TRUE(urldecode("%22", obuf)) << "Single character";
    ASSERT_STREQ("\x22", obuf);

    ASSERT_TRUE(urldecode("Hello World", obuf)) << "No pct encode";
    ASSERT_STREQ("Hello World", obuf);

    ASSERT_TRUE(urldecode("Hello%20World", obuf));
    ASSERT_STREQ("Hello World", obuf);

    ASSERT_TRUE(urldecode("%2Ffoo%2Fbar%2Fbaz%2F", obuf));
    ASSERT_STREQ("/foo/bar/baz/", obuf);

    ASSERT_TRUE(urldecode("%01%02%03%04", obuf)) << "Multiple octets";
    ASSERT_STREQ("\x01\x02\x03\x04", obuf);

    ASSERT_TRUE(urldecode("%FFFF", obuf)) << "Recognize only first two hex digits";
    // Split the hex literal so we don't confuse the preprocessor
    ASSERT_STREQ("\xff"
                 "FF",
                 obuf);

    // Error tests
    ASSERT_FALSE(urldecode("%", obuf));
    ASSERT_FALSE(urldecode("%RR", obuf)) << "Invalid hex digits";
}
