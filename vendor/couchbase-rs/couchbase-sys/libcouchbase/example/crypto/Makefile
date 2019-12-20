LDFLAGS=-lcouchbase -lm
CFLAGS=-g

OPENSSL_LDFLAGS=$(shell pkg-config --libs openssl) ${LDFLAGS}
OPENSSL_CFLAGS=$(shell pkg-config --cflags openssl) ${CFLAGS}

all: openssl_symmetric_encrypt openssl_symmetric_decrypt

openssl_symmetric_encrypt: openssl_symmetric_encrypt.c openssl_symmetric_provider.c common_provider.c
	${CC} ${OPENSSL_CFLAGS} ${OPENSSL_LDFLAGS} -o $@ $^

openssl_symmetric_decrypt: openssl_symmetric_decrypt.c openssl_symmetric_provider.c common_provider.c
	${CC} ${OPENSSL_CFLAGS} ${OPENSSL_LDFLAGS} -o $@ $^
