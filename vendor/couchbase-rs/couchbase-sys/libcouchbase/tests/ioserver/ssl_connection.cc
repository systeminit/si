#include "ioserver.h"

#ifndef LCB_NO_SSL
#include <openssl/ssl.h>

using namespace LCBTest;
using std::vector;

class SslSocket : public SockFD
{
  public:
    SslSocket(SockFD *innner);

    ~SslSocket();

    // Receive data via SSL
    size_t send(const void *, size_t, int);

    // Send data via SSL
    ssize_t recv(void *, size_t, int);

    // Close the SSL context first
    void close();

    // Return the real FD
    int getFD() const
    {
        return sfd->getFD();
    }

  private:
    SSL *ssl;
    SSL_CTX *ctx;
    SockFD *sfd;
    bool ok;
};

SockFD *TestServer::sslSocketFactory(int fd)
{
    return new SslSocket(new SockFD(fd));
}

extern "C" {
static void log_callback(const SSL *ssl, int where, int ret)
{
    const char *retstr = "";
    int should_log = 0;

    if (where & SSL_CB_ALERT) {
        should_log = 1;
    }
    if (where == SSL_CB_HANDSHAKE_START || where == SSL_CB_HANDSHAKE_DONE) {
        should_log = 1;
    }
    if ((where & SSL_CB_EXIT) && ret == 0) {
        should_log = 1;
    }

    if (!should_log) {
        return;
    }

    retstr = SSL_alert_type_string(ret);
    fprintf(stderr, "SslSocket: ST(0x%x). %s. R(0x%x)%s\n", where, SSL_state_string_long(ssl), ret, retstr);

    if (where == SSL_CB_HANDSHAKE_DONE) {
        fprintf(stderr, "SslSocket: Using SSL version %s. Cipher=%s\n", SSL_get_version(ssl), SSL_get_cipher_name(ssl));
    }
}
}

// http://stackoverflow.com/questions/256405/programmatically-create-x509-certificate-using-openssl
// http://www.opensource.apple.com/source/OpenSSL/OpenSSL-22/openssl/demos/x509/mkcert.c
// Note we deviate from the examples by directly setting the certificate.

static void genCertificate(SSL_CTX *ctx)
{
    EVP_PKEY *pkey = EVP_PKEY_new();
    X509 *x509 = X509_new();
    RSA *rsa = RSA_generate_key(2048, RSA_F4, NULL, NULL);

    EVP_PKEY_assign_RSA(pkey, rsa);
    ASN1_INTEGER_set(X509_get_serialNumber(x509), 1);
    X509_gmtime_adj(X509_get_notBefore(x509), 0);
    X509_gmtime_adj(X509_get_notAfter(x509), 31536000L);
    X509_set_pubkey(x509, pkey);

    X509_NAME *name = X509_get_subject_name(x509);
    X509_NAME_add_entry_by_txt(name, "C", MBSTRING_ASC, (unsigned char *)"CA", -1, -1, 0);
    X509_NAME_add_entry_by_txt(name, "O", MBSTRING_ASC, (unsigned char *)"MyCompany Inc.", -1, -1, 0);
    X509_NAME_add_entry_by_txt(name, "CN", MBSTRING_ASC, (unsigned char *)"localhost", -1, -1, 0);
    X509_set_issuer_name(x509, name);
    X509_sign(x509, pkey, EVP_sha1());

    SSL_CTX_use_PrivateKey(ctx, pkey);
    SSL_CTX_use_certificate(ctx, x509);
    X509_free(x509);
    EVP_PKEY_free(pkey);
}

SslSocket::SslSocket(SockFD *inner) : SockFD(inner->getFD())
{
    sfd = inner;
    ctx = SSL_CTX_new(SSLv23_server_method());
    assert(ctx != NULL);

    SSL_CTX_set_info_callback(ctx, log_callback);
    genCertificate(ctx);
    SSL_CTX_set_mode(ctx, SSL_MODE_ACCEPT_MOVING_WRITE_BUFFER);
    SSL_CTX_set_options(ctx, SSL_OP_NO_SSLv2 | SSL_OP_NO_SSLv3);
    SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL);
    SSL_CTX_load_verify_locations(ctx, NULL, NULL);

    ssl = SSL_new(ctx);
    assert(ssl != NULL);
    SSL_set_accept_state(ssl);
    SSL_set_fd(ssl, sfd->getFD());
}

size_t SslSocket::send(const void *buf, size_t n, int)
{
    return SSL_write(ssl, buf, n);
}

ssize_t SslSocket::recv(void *buf, size_t n, int)
{
    return SSL_read(ssl, buf, n);
}

void SslSocket::close()
{
    SSL_shutdown(ssl);
    sfd->close();
}

SslSocket::~SslSocket()
{
    SSL_free(ssl);
    SSL_CTX_free(ctx);
    delete sfd;
}

#else
namespace
{
void dummySym() {}
} // namespace
#endif
