/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2018 Couchbase, Inc.
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

#ifndef __BENCH_LEXER_H
#define __BENCH_LEXER_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum bm_TOKTYPE_ { BM_TOKEN_WORD = 0x01, BM_TOKEN_OPTION = 0x02, BM_TOKEN__MAX } bm_TOKTYPE;

typedef struct bm_TOKEN_ {
    bm_TOKTYPE type;
    union {
        struct {
            const char *ptr;
            int len;
        } word;
        struct {
            const char *key;
            int klen;
            const char *val;
            int vlen;
        } option;
    } t;
} bm_TOKEN;

const char *lex(const char *s, bm_TOKEN *tok);

#ifdef __cplusplus
}
#endif
#endif
