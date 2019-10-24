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

#include <unistd.h>
#include "lexer.h"

const char *lex(const char *s, bm_TOKEN *tok)
{
    const char *m, *k, *ke, *v, *ve;
    /*!stags:re2c format = 'const char *@@;'; */
loop:
    /*!re2c
        re2c:define:YYCTYPE = char;
        re2c:define:YYCURSOR = s;
        re2c:define:YYMARKER = m;
        re2c:yyfill:enable = 0;

        end    = "\x00";
        sp     = [ \t\n\r];
        eq     = "=";
        wsp    = sp*;
        char   = [^=] \ end;
        ochar  = char \ sp;
        pchar  = ochar \ [/];
        str    = ["] (char \ ["] | [\]["])* ["];
        opt    = ochar+;
        word   = ochar* | str;

        [-]{1,2} @k opt @ke eq? @v word @ve {
            tok->type = BM_TOKEN_OPTION;
            tok->t.option.key = k;
            tok->t.option.klen = (int)(ke - k);
            tok->t.option.val = v;
            tok->t.option.vlen = (int)(ve - v);
            return s;
        }
        @k word @ke {
            tok->type = BM_TOKEN_WORD;
            tok->t.word.ptr = k;
            tok->t.word.len = (int)(ke - k);
            return s;
        }
        end { return NULL; }
        wsp { goto loop; }
    */
    return NULL;
}
