/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#ifndef CBC_HISTOGRAM_H
#define CBC_HISTOGRAM_H
#include <libcouchbase/couchbase.h>
#include <libcouchbase/utils.h>
#include <stdio.h>

namespace cbc
{

class Histogram
{
  public:
    Histogram()
    {
        hg = NULL;
        output = NULL;
    }
    void install(lcb_INSTANCE *, FILE *out = stderr);
    void installStandalone(FILE *out = stderr);
    void record(lcb_U64 duration);
    void write();
    FILE *getOutput() const
    {
        return output;
    }

  private:
    lcb_HISTOGRAM *hg;
    FILE *output;
};

} // namespace cbc

#endif
