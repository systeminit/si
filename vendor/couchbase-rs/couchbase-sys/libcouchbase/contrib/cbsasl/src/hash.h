#ifndef SRC_HASH_H_
#define SRC_HASH_H_ 1
#include "config.h"

#ifdef __cplusplus
extern "C" {
#endif

    uint32_t cbsasl_hash(const void *key, size_t length, const uint32_t initval);

#ifdef __cplusplus
}
#endif

#endif  /* SRC_HASH_H_ */
