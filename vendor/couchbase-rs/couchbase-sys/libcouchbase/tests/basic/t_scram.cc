/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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
#include "config.h"
#include <gtest/gtest.h>

#include "contrib/cbsasl/src/scram-sha/scram_utils.h"

class ScramTest : public ::testing::Test
{
};

/**
 * Tests the parse_server_challenge function.
 */
TEST_F(ScramTest, ParseValidServerChallenge)
{
    const char *serverin = "r=CCCCSSSS,s=xxxx,i=4096";
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_OK, ret);
    EXPECT_EQ(8, noncelength);
    EXPECT_EQ("CCCCSSSS", std::string(nonce, noncelength));
    EXPECT_EQ(4, saltlength);
    EXPECT_EQ("xxxx", std::string(salt, saltlength));
    EXPECT_EQ(4096, itcount);
}

TEST_F(ScramTest, ParseInvalidServerChallenge_WithWrongAttribute)
{
    const char *serverin = "r=CCCCSSSS,t=xxxx,i=4096"; // 't' is not a valid attribute
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_BADPARAM, ret);
}

TEST_F(ScramTest, ParseInvalidServerChallenge_WithMissingAttributeType)
{
    const char *serverin = "r=CCCCSSSS,xxxx,i=4096"; // no "s="
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_BADPARAM, ret);
}

TEST_F(ScramTest, ParseInvalidServerChallenge_WithVoidField)
{
    const char *serverin = ",s=xxxx,i=4096";
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_BADPARAM, ret);
}

TEST_F(ScramTest, ParseInvalidServerChallenge_WithInvalidIterationCount)
{
    const char *serverin = "r=CCCCSSSS,s=xxxx,i=123456789012345"; // value too big for an integer
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_BADPARAM, ret);
}

TEST_F(ScramTest, ParseInvalidServerChallenge_WithDuplicateAttribute)
{
    const char *serverin = "r=CCCCSSSS,r=CCCCSSSS,s=xxxx,i=4096"; // "r" field appearing twice
    const char *nonce = NULL;
    unsigned int noncelength = 0;
    const char *salt = NULL;
    unsigned int saltlength = 0;
    unsigned int itcount = 0;

    cbsasl_error_t ret =
        parse_server_challenge(serverin, strlen(serverin), &nonce, &noncelength, &salt, &saltlength, &itcount);

    ASSERT_EQ(SASL_BADPARAM, ret);
}

// the following tests are valid only if OpenSSL is linked to the library
#ifndef LCB_NO_SSL
TEST_F(ScramTest, GenerateSaltedPasswordWithSHA512)
{
    // here we check that the generate_salted_password function returns the expected output
    // for a predefined password/salt/iteration count combination
    union {
        cbsasl_secret_t secret;
        char buffer[30];
    } u_auth;
    unsigned char outbuffer[CBSASL_SHA512_DIGEST_SIZE];
    unsigned int outlen;
    const char *salt = "c2FsdA=="; // "salt" in base64
    memcpy(u_auth.secret.data, "password", 8);
    u_auth.secret.len = 8; // strlen("password")
    // expected output was generated using the following Python algorithm:
    // import hashlib, binascii
    // dk = hashlib.pbkdf2_hmac('sha512', b'password', b'salt', 1000)
    // print binascii.hexlify(dk)
    const char *expectedoutput = "\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31"
                                 "\xbd\x5e\xe4\x32\xee\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a"
                                 "\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef\xc4\x00\x7d\x52\x98"
                                 "\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8"
                                 "\xd8\x34\xcf\xec";
    // warning: the expected output contains a binary zero, so don't use strlen()

    cbsasl_error_t ret = generate_salted_password(SASL_AUTH_MECH_SCRAM_SHA512, &u_auth.secret, salt, strlen(salt), 1000,
                                                  outbuffer, &outlen);
    ASSERT_EQ(SASL_OK, ret);
    EXPECT_EQ(CBSASL_SHA512_DIGEST_SIZE, outlen);
    EXPECT_EQ(std::string(expectedoutput, CBSASL_SHA512_DIGEST_SIZE),
              std::string((const char *)outbuffer, CBSASL_SHA512_DIGEST_SIZE));
}

TEST_F(ScramTest, GenerateSaltedPasswordWithSHA256)
{
    // here we check that the generate_salted_password function returns the expected output
    // for a predefined password/salt/iteration count combination
    union {
        cbsasl_secret_t secret;
        char buffer[30];
    } u_auth;
    unsigned char outbuffer[CBSASL_SHA256_DIGEST_SIZE];
    unsigned int outlen;
    const char *salt = "c2FsdA=="; // "salt" in base64
    memcpy(u_auth.secret.data, "password", 8);
    u_auth.secret.len = 8; // strlen("password")
    // expected output was generated using the following Python algorithm:
    // import hashlib, binascii
    // dk = hashlib.pbkdf2_hmac('sha256', b'password', b'salt', 1000)
    // print binascii.hexlify(dk)
    const char *expectedoutput = "\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d"
                                 "\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7"
                                 "\xdc\xb3";

    cbsasl_error_t ret = generate_salted_password(SASL_AUTH_MECH_SCRAM_SHA256, &u_auth.secret, salt, strlen(salt), 1000,
                                                  outbuffer, &outlen);
    ASSERT_EQ(SASL_OK, ret);
    EXPECT_EQ(CBSASL_SHA256_DIGEST_SIZE, outlen);
    EXPECT_EQ(std::string(expectedoutput, CBSASL_SHA256_DIGEST_SIZE),
              std::string((const char *)outbuffer, CBSASL_SHA256_DIGEST_SIZE));
}

TEST_F(ScramTest, GenerateSaltedPasswordWithSHA1)
{
    // here we check that the generate_salted_password function returns the expected output
    // for a predefined password/salt/iteration count combination
    union {
        cbsasl_secret_t secret;
        char buffer[30];
    } u_auth;
    unsigned char outbuffer[CBSASL_SHA1_DIGEST_SIZE];
    unsigned int outlen;
    const char *salt = "c2FsdA=="; // "salt" in base64
    memcpy(u_auth.secret.data, "password", 8);
    u_auth.secret.len = 8; // strlen("password")
    // expected output was generated using the following Python algorithm:
    // import hashlib, binascii
    // dk = hashlib.pbkdf2_hmac('sha1', b'password', b'salt', 1000)
    // print binascii.hexlify(dk)
    const char *expectedoutput = "\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03"
                                 "\x4f\xed\x48\xd0\x3f";

    cbsasl_error_t ret = generate_salted_password(SASL_AUTH_MECH_SCRAM_SHA1, &u_auth.secret, salt, strlen(salt), 1000,
                                                  outbuffer, &outlen);
    ASSERT_EQ(SASL_OK, ret);
    EXPECT_EQ(CBSASL_SHA1_DIGEST_SIZE, outlen);
    EXPECT_EQ(std::string(expectedoutput, CBSASL_SHA1_DIGEST_SIZE),
              std::string((const char *)outbuffer, CBSASL_SHA1_DIGEST_SIZE));
}

TEST_F(ScramTest, ComputeClientProof_SHA512)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA512
    const unsigned char *saltedpassword =
        (const unsigned char *)"\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31"
                               "\xbd\x5e\xe4\x32\xee\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a"
                               "\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef\xc4\x00\x7d\x52\x98"
                               "\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8"
                               "\xd8\x34\xcf\xec";
    const char *clientfirstbare = "n=foo,r=001122334455667788";
    const char *serverfirstmess = "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000";
    const char *clientfinalwithoutproof = "c=biws,r=00112233445566778899aabbccddeeff";
    char *authmessage = NULL;
    char outclientproof[(CBSASL_SHA512_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret =
        compute_client_proof(SASL_AUTH_MECH_SCRAM_SHA512, saltedpassword, CBSASL_SHA512_DIGEST_SIZE, clientfirstbare,
                             strlen(clientfirstbare), serverfirstmess, strlen(serverfirstmess), clientfinalwithoutproof,
                             strlen(clientfinalwithoutproof), &authmessage, outclientproof, sizeof(outclientproof));

    EXPECT_EQ(SASL_OK, ret);
    // expected authentication message: concatenation of clientfirstbare, serverfirstmess and
    // clientfinalwithoutproof (with commas)
    const char *expectedauth = "n=foo,r=001122334455667788,"
                               "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                               "c=biws,r=00112233445566778899aabbccddeeff";

    EXPECT_TRUE(authmessage != NULL);
    EXPECT_STREQ(expectedauth, authmessage);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31\xbd\x5e\xe4\x32\xee'\
    //                  '\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef'\
    //                   '\xc4\x00\x7d\x52\x98\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8\xd8\x34\xcf\xec'
    // clientkey = hmac.new(saltedpassword, 'Client Key', hashlib.sha512).digest()
    // storedkey = hashlib.sha512(clientkey).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // clientsign = hmac.new(storedkey, authmess, hashlib.sha512).digest()
    // clientproof = ''.join(chr(ord(cs) ^ ord(ck)) for cs,ck in zip(clientsign, clientkey))
    // print base64.b64encode(clientproof)
    EXPECT_STREQ("dbXLc1MsNIdWj1AgSHRi/6E0OhWG2j6MwLKHR+UyVotT3G7VgYPlkQjwaewpH7v5BMXgkIqKRP/IUEbNA0M40w==",
                 outclientproof);

    free(authmessage);
}

TEST_F(ScramTest, ComputeClientProof_SHA256)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA256
    const unsigned char *saltedpassword =
        (const unsigned char *)"\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d"
                               "\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7"
                               "\xdc\xb3";
    const char *clientfirstbare = "n=foo,r=001122334455667788";
    const char *serverfirstmess = "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000";
    const char *clientfinalwithoutproof = "c=biws,r=00112233445566778899aabbccddeeff";
    char *authmessage = NULL;
    char outclientproof[(CBSASL_SHA256_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret =
        compute_client_proof(SASL_AUTH_MECH_SCRAM_SHA256, saltedpassword, CBSASL_SHA256_DIGEST_SIZE, clientfirstbare,
                             strlen(clientfirstbare), serverfirstmess, strlen(serverfirstmess), clientfinalwithoutproof,
                             strlen(clientfinalwithoutproof), &authmessage, outclientproof, sizeof(outclientproof));

    EXPECT_EQ(SASL_OK, ret);
    // expected authentication message: concatenation of clientfirstbare, serverfirstmess and
    // clientfinalwithoutproof (with commas)
    const char *expectedauth = "n=foo,r=001122334455667788,"
                               "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                               "c=biws,r=00112233445566778899aabbccddeeff";

    EXPECT_TRUE(authmessage != NULL);
    EXPECT_STREQ(expectedauth, authmessage);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d'\
    //                  '\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7'\
    //                  '\xdc\xb3'
    // clientkey = hmac.new(saltedpassword, 'Client Key', hashlib.sha256).digest()
    // storedkey = hashlib.sha256(clientkey).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // clientsign = hmac.new(storedkey, authmess, hashlib.sha256).digest()
    // clientproof = ''.join(chr(ord(cs) ^ ord(ck)) for cs,ck in zip(clientsign, clientkey))
    // print base64.b64encode(clientproof)
    EXPECT_STREQ("V2VMc1luh0OKg7VgRO2Wt7BoBUaW8ZxUhNav2RUbAHc=", outclientproof);

    free(authmessage);
}

TEST_F(ScramTest, ComputeClientProof_SHA1)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA1
    const unsigned char *saltedpassword =
        (const unsigned char *)"\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03"
                               "\x4f\xed\x48\xd0\x3f";
    const char *clientfirstbare = "n=foo,r=001122334455667788";
    const char *serverfirstmess = "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000";
    const char *clientfinalwithoutproof = "c=biws,r=00112233445566778899aabbccddeeff";
    char *authmessage = NULL;
    char outclientproof[(CBSASL_SHA1_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret =
        compute_client_proof(SASL_AUTH_MECH_SCRAM_SHA1, saltedpassword, CBSASL_SHA1_DIGEST_SIZE, clientfirstbare,
                             strlen(clientfirstbare), serverfirstmess, strlen(serverfirstmess), clientfinalwithoutproof,
                             strlen(clientfinalwithoutproof), &authmessage, outclientproof, sizeof(outclientproof));

    EXPECT_EQ(SASL_OK, ret);
    // expected authentication message: concatenation of clientfirstbare, serverfirstmess and
    // clientfinalwithoutproof (with commas)
    const char *expectedauth = "n=foo,r=001122334455667788,"
                               "r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                               "c=biws,r=00112233445566778899aabbccddeeff";

    EXPECT_TRUE(authmessage != NULL);
    EXPECT_STREQ(expectedauth, authmessage);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03'\
    //                  '\x4f\xed\x48\xd0\x3f'
    // clientkey = hmac.new(saltedpassword, 'Client Key', hashlib.sha1).digest()
    // storedkey = hashlib.sha1(clientkey).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // clientsign = hmac.new(storedkey, authmess, hashlib.sha1).digest()
    // clientproof = ''.join(chr(ord(cs) ^ ord(ck)) for cs,ck in zip(clientsign, clientkey))
    // print base64.b64encode(clientproof)
    EXPECT_STREQ("Iu9QH+CO2nAtVwmJaQe55UzlBEQ=", outclientproof);

    free(authmessage);
}

TEST_F(ScramTest, ComputeServerSignature_SHA512)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA512
    const unsigned char *saltedpassword =
        (const unsigned char *)"\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31"
                               "\xbd\x5e\xe4\x32\xee\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a"
                               "\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef\xc4\x00\x7d\x52\x98"
                               "\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8"
                               "\xd8\x34\xcf\xec";
    const char *authmessage = "n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                              "c=biws,r=00112233445566778899aabbccddeeff";
    char outserversign[(CBSASL_SHA512_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret =
        compute_server_signature(SASL_AUTH_MECH_SCRAM_SHA512, saltedpassword, CBSASL_SHA512_DIGEST_SIZE, authmessage,
                                 outserversign, sizeof(outserversign));

    EXPECT_EQ(SASL_OK, ret);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31\xbd\x5e\xe4\x32\xee'\
    //                 '\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef'\
    //                  '\xc4\x00\x7d\x52\x98\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8\xd8\x34\xcf\xec'
    // serverkey = hmac.new(saltedpassword, 'Server Key', hashlib.sha512).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // serversign = hmac.new(serverkey, authmess, hashlib.sha512).digest()
    // print base64.b64encode(serversign)
    EXPECT_STREQ("qonE7dZI6HvlX7nzSxbwmXBnr8xbw1pLhcwGFfnh+q1kqT+VoIood7EReeGXSog9Q9UNxqYKITudfYvSxJCQzg==",
                 outserversign);
}

TEST_F(ScramTest, ComputeServerSignature_SHA256)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA256
    const unsigned char *saltedpassword =
        (const unsigned char *)"\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d"
                               "\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7"
                               "\xdc\xb3";
    const char *authmessage = "n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                              "c=biws,r=00112233445566778899aabbccddeeff";
    char outserversign[(CBSASL_SHA256_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret =
        compute_server_signature(SASL_AUTH_MECH_SCRAM_SHA256, saltedpassword, CBSASL_SHA256_DIGEST_SIZE, authmessage,
                                 outserversign, sizeof(outserversign));

    EXPECT_EQ(SASL_OK, ret);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d'\
    //                  '\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7'\
    //                  '\xdc\xb3'
    // serverkey = hmac.new(saltedpassword, 'Server Key', hashlib.sha256).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // serversign = hmac.new(serverkey, authmess, hashlib.sha256).digest()
    // print base64.b64encode(serversign)
    EXPECT_STREQ("iPG9IiKPBI9165j9aGfbGM9FwHsANnspy5pMGJUbaS8=", outserversign);
}

TEST_F(ScramTest, ComputeServerSignature_SHA1)
{
    // we use the salted password computed in GenerateSaltedPasswordWithSHA1
    const unsigned char *saltedpassword =
        (const unsigned char *)"\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03"
                               "\x4f\xed\x48\xd0\x3f";
    const char *authmessage = "n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                              "c=biws,r=00112233445566778899aabbccddeeff";
    char outserversign[(CBSASL_SHA1_DIGEST_SIZE / 3 + 1) * 4 + 1];

    cbsasl_error_t ret = compute_server_signature(SASL_AUTH_MECH_SCRAM_SHA1, saltedpassword, CBSASL_SHA1_DIGEST_SIZE,
                                                  authmessage, outserversign, sizeof(outserversign));

    EXPECT_EQ(SASL_OK, ret);

    // expected client proof
    // Here is how to generate the same value in Python 2.7:
    // import hmac, hashlib, base64
    // saltedpassword = '\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03'\
    //                  '\x4f\xed\x48\xd0\x3f'
    // serverkey = hmac.new(saltedpassword, 'Server Key', hashlib.sha1).digest()
    // authmess='n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,c=biws,r=00112233445566778899aabbccddeeff'
    // serversign = hmac.new(serverkey, authmess, hashlib.sha1).digest()
    // print base64.b64encode(serversign)
    EXPECT_STREQ("WfiXP3zx55r8GXP1n2Bz/FVk/hk=", outserversign);
}

TEST_F(ScramTest, FinalServerCheck_SHA512)
{
    // Testing cbsasl_client_check
    cbsasl_conn_t ctx;
    ctx.client = 1;
    ctx.c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA512;
    // for computing the server signature, we only need the salted password and
    // the authentication message
    ctx.c.client.saltedpassword = (unsigned char *)"\xaf\xe6\xc5\x53\x07\x85\xb6\xcc\x6b\x1c\x64\x53\x38\x47\x31"
                                                   "\xbd\x5e\xe4\x32\xee\x54\x9f\xd4\x2f\xb6\x69\x57\x79\xad\x8a"
                                                   "\x1c\x5b\xf5\x9d\xe6\x9c\x48\xf7\x74\xef\xc4\x00\x7d\x52\x98"
                                                   "\xf9\x03\x3c\x02\x41\xd5\xab\x69\x30\x5e\x7b\x64\xec\xee\xb8"
                                                   "\xd8\x34\xcf\xec";
    ctx.c.client.saltedpasslen = CBSASL_SHA512_DIGEST_SIZE;
    ctx.c.client.auth_message =
        (char *)"n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                "c=biws,r=00112233445566778899aabbccddeeff";
    const char *invalid_sign =
        "v=USEHlpFIwMJu0ytKPTmXPTXzZag82/F9qkAT2gB0gVaA00RTdQeSgCPhMVWcmvp7dhscVnkE/ZYLbDpMaeMp6g==";
    EXPECT_EQ(SASL_FAIL, cbsasl_client_check(&ctx, invalid_sign, strlen(invalid_sign)));

    const char *valid_sign =
        "v=qonE7dZI6HvlX7nzSxbwmXBnr8xbw1pLhcwGFfnh+q1kqT+VoIood7EReeGXSog9Q9UNxqYKITudfYvSxJCQzg==";
    EXPECT_EQ(SASL_OK, cbsasl_client_check(&ctx, valid_sign, strlen(valid_sign)));
}

TEST_F(ScramTest, FinalServerCheck_SHA256)
{
    // Testing cbsasl_client_check
    cbsasl_conn_t ctx;
    ctx.client = 1;
    ctx.c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA256;
    // for computing the server signature, we only need the salted password and
    // the authentication message
    ctx.c.client.saltedpassword = (unsigned char *)"\x63\x2c\x28\x12\xe4\x6d\x46\x04\x10\x2b\xa7\x61\x8e\x9d\x6d"
                                                   "\x7d\x2f\x81\x28\xf6\x26\x6b\x4a\x03\x26\x4d\x2a\x04\x60\xb7"
                                                   "\xdc\xb3";
    ctx.c.client.saltedpasslen = CBSASL_SHA256_DIGEST_SIZE;
    ctx.c.client.auth_message =
        (char *)"n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                "c=biws,r=00112233445566778899aabbccddeeff";

    const char *invalid_sign =
        "v=USEHlpFIwMJu0ytKPTmXPTXzZag82/F9qkAT2gB0gVaA00RTdQeSgCPhMVWcmvp7dhscVnkE/ZYLbDpMaeMp6g==";
    EXPECT_EQ(SASL_FAIL, cbsasl_client_check(&ctx, invalid_sign, strlen(invalid_sign)));

    const char *valid_sign = "v=iPG9IiKPBI9165j9aGfbGM9FwHsANnspy5pMGJUbaS8=";
    EXPECT_EQ(SASL_OK, cbsasl_client_check(&ctx, valid_sign, strlen(valid_sign)));
}

TEST_F(ScramTest, FinalServerCheck_SHA1)
{
    // Testing cbsasl_client_check
    cbsasl_conn_t ctx;
    ctx.client = 1;
    ctx.c.client.auth_mech = SASL_AUTH_MECH_SCRAM_SHA1;
    // for computing the server signature, we only need the salted password and
    // the authentication message
    ctx.c.client.saltedpassword = (unsigned char *)"\x6e\x88\xbe\x8b\xad\x7e\xae\x9d\x9e\x10\xaa\x06\x12\x24\x03"
                                                   "\x4f\xed\x48\xd0\x3f";
    ctx.c.client.saltedpasslen = CBSASL_SHA1_DIGEST_SIZE;
    ctx.c.client.auth_message =
        (char *)"n=foo,r=001122334455667788,r=00112233445566778899aabbccddeeff,s=c2FsdA==,i=1000,"
                "c=biws,r=00112233445566778899aabbccddeeff";

    const char *invalid_sign =
        "v=USEHlpFIwMJu0ytKPTmXPTXzZag82/F9qkAT2gB0gVaA00RTdQeSgCPhMVWcmvp7dhscVnkE/ZYLbDpMaeMp6g==";
    EXPECT_EQ(SASL_FAIL, cbsasl_client_check(&ctx, invalid_sign, strlen(invalid_sign)));

    const char *valid_sign = "v=WfiXP3zx55r8GXP1n2Bz/FVk/hk=";
    EXPECT_EQ(SASL_OK, cbsasl_client_check(&ctx, valid_sign, strlen(valid_sign)));
}

#endif // LCB_NO_SSL
