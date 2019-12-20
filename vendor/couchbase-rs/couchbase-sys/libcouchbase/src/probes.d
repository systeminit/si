/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2012-2017 Couchbase, Inc.
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

provider libcouchbase
{
    probe get_begin(void *,       /* lcb_INSTANCE* */
                    uint32_t,     /* opaque */
                    uint16_t,     /* vbucket */
                    uint8_t,      /* opcode (get, gat, getl) */
                    const char *, /* key */
                    size_t,       /* nkey */
                    uint32_t);    /* expiration */
    probe get_end(void *,         /* lcb_INSTANCE* */
                  uint32_t,       /* opaque */
                  uint8_t,        /* opcode (get, gat, getl) */
                  uint64_t,       /* latency, ns */
                  uint16_t,       /* return code (from libcouchbase) */
                  const char *,   /* key */
                  size_t,         /* nkey */
                  const char *,   /* bytes */
                  size_t,         /* nbytes */
                  uint32_t,       /* flags */
                  uint64_t,       /* cas */
                  uint8_t);       /* datatype */

    probe unlock_begin(void *,       /* lcb_INSTANCE* */
                       uint32_t,     /* opaque */
                       uint16_t,     /* vbucket */
                       uint8_t,      /* opcode */
                       const char *, /* key */
                       size_t);      /* nkey */
    probe unlock_end(void *,         /* lcb_INSTANCE* */
                     uint32_t,       /* opaque */
                     uint8_t,        /* opcode */
                     uint64_t,       /* latency, ns */
                     uint16_t,       /* return code (from libcouchbase) */
                     const char *,   /* key */
                     size_t);        /* nkey */

    probe store_begin(void *,       /* lcb_INSTANCE* */
                      uint32_t,     /* opaque */
                      uint16_t,     /* vbucket */
                      uint8_t,      /* opcode (set, add, replace, append, prepend) */
                      const char *, /* key */
                      size_t,       /* nkey */
                      const char *, /* bytes */
                      size_t,       /* nbytes */
                      uint32_t,     /* flags */
                      uint64_t,     /* cas */
                      uint8_t,      /* datatype */
                      uint32_t);    /* expiration */
    probe store_end(void *,         /* lcb_INSTANCE* */
                    uint32_t,       /* opaque */
                    uint8_t,        /* opcode (set, add, replace, append, prepend) */
                    uint64_t,       /* latency, ns */
                    uint16_t,       /* return code (from libcouchbase) */
                    const char *,   /* key */
                    size_t,         /* nkey */
                    uint64_t);      /* cas */

    probe arithmetic_begin(void *,       /* lcb_INSTANCE* */
                           uint32_t,     /* opaque */
                           uint16_t,     /* vbucket */
                           uint8_t,      /* opcode (increment, decrement) */
                           const char *, /* key */
                           size_t,       /* nkey */
                           uint64_t,     /* delta */
                           uint64_t,     /* initial */
                           uint32_t);    /* expiration */
    probe arithmetic_end(void *,         /* lcb_INSTANCE* */
                         uint32_t,       /* opaque */
                         uint8_t,        /* opcode (increment, decrement) */
                         uint64_t,       /* latency, ns */
                         uint16_t,       /* return code (from libcouchbase) */
                         const char *,   /* key */
                         size_t,         /* nkey */
                         uint64_t,       /* value */
                         uint64_t);      /* cas */

    probe touch_begin(void *,       /* lcb_INSTANCE* */
                      uint32_t,     /* opaque */
                      uint16_t,     /* vbucket */
                      uint8_t,      /* opcode */
                      const char *, /* key */
                      size_t,       /* nkey */
                      uint32_t);    /* expiration */
    probe touch_end(void *,         /* lcb_INSTANCE* */
                    uint32_t,       /* opaque */
                    uint8_t,        /* opcode */
                    uint64_t,       /* latency, ns */
                    uint16_t,       /* return code (from libcouchbase) */
                    const char *,   /* key */
                    size_t,         /* nkey */
                    uint64_t);      /* cas */

    probe remove_begin(void *,       /* lcb_INSTANCE* */
                       uint32_t,     /* opaque */
                       uint16_t,     /* vbucket */
                       uint8_t,      /* opcode */
                       const char *, /* key */
                       size_t);      /* nkey */
    probe remove_end(void *,         /* lcb_INSTANCE* */
                     uint32_t,       /* opaque */
                     uint8_t,        /* opcode */
                     uint64_t,       /* latency, ns */
                     uint16_t,       /* return code (from libcouchbase) */
                     const char *,   /* key */
                     size_t,         /* nkey */
                     uint64_t);      /* cas */

    /*
     * OBSERVE_BEGIN probe intended to be parsed in the handler.
     * the bytes argument is a blob with nbytes length:
     *
     * +---------+---------+------------+----
     * | 16 bits | 16 bits | nkey bytes | ...
     * +---------+---------+------------+----
     * | vbucket |   nkey  |    key     | ...
     * +---------+---------+------------+----
     */
    probe observe_begin(void *,          /* lcb_INSTANCE* */
                        uint32_t,        /* opaque */
                        uint8_t,         /* opcode */
                        const char *,    /* bytes */
                        size_t);         /* nbytes */
    probe observe_progress(void *,       /* lcb_INSTANCE* */
                           uint32_t,     /* opaque */
                           uint8_t,      /* opcode */
                           uint64_t,     /* latency, ns */
                           uint16_t,     /* return code (from libcouchbase) */
                           const char *, /* key */
                           size_t,       /* nkey */
                           uint64_t,     /* cas */
                           uint8_t,      /* observe status: FOUND = 0x00, PERSISTED = 0x01, NOT_FOUND = 0x80 */
                           uint8_t,      /* master (zero if from replica) */
                           uint32_t,     /* ttp, time to persist */
                           uint32_t);    /* ttr, time to replicate */
    probe observe_end(void *,            /* lcb_INSTANCE* */
                      uint32_t,          /* opaque */
                      uint8_t,           /* opcode */
                      uint64_t,          /* latency, ns */
                      uint16_t);         /* return code (from libcouchbase) */

    probe http_begin(void *,        /* lcb_INSTANCE* */
                     void *,        /* pointer to request */
                     uint16_t,      /* type */
                     uint16_t,      /* method */
                     const char *,  /* url */
                     const char *,  /* host */
                     const char *); /* port */
    probe http_end(void *,          /* lcb */
                   void *,          /* pointer to request */
                   uint16_t,        /* type */
                   uint16_t,        /* method */
                   const char *,    /* url */
                   const char *,    /* host */
                   const char *,    /* port */
                   uint16_t,        /* return code (from libcouchbase) */
                   uint16_t,        /* HTTP status code or zero */
                   uint64_t);       /* latency, ns */

    probe new_config(void *,         /* lcb_INSTANCE* */
                     int32_t,        /* revid */
                     const char*,    /* bucket name */
                     const char*,    /* bucket UUID */
                     const void*     /* pointer to config */
                    );

    probe exists_begin(void *,       /* lcb_INSTANCE* */
                       uint32_t,     /* opaque */
                       uint16_t,     /* vbucket */
                       uint8_t,      /* opcode */
                       const char *, /* key */
                       size_t);      /* nkey */
    probe exists_end(void *,         /* lcb_INSTANCE* */
                     uint32_t,       /* opaque */
                     uint8_t,        /* opcode */
                     uint64_t,       /* latency, ns */
                     uint16_t,       /* return code (from libcouchbase) */
                     const char *,   /* key */
                     size_t,         /* nkey */
                     uint64_t);      /* cas */
};
