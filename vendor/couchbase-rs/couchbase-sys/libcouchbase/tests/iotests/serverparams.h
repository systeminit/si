/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#ifndef TESTS_SERVER_PARAMS_H
#define TESTS_SERVER_PARAMS_H 1

#include "config.h"
#include <string>
#include <string.h>
#include <libcouchbase/couchbase.h>

class ServerParams
{
  public:
    ServerParams() {}
    ServerParams(const char *h, const char *b, const char *u, const char *p)
    {
        loadParam(host, h);
        loadParam(bucket, b);
        loadParam(user, u);
        loadParam(pass, p);
    }

    void makeConnectParams(lcb_create_st &crst, lcb_io_opt_t io)
    {
        memset(&crst, 0, sizeof(crst));
        if (mcNodes.empty()) {
            crst.version = 3;
            crst.v.v3.connstr = host.c_str();
            crst.v.v3.username = user.c_str();
            crst.v.v3.passwd = pass.c_str();
            crst.v.v3.io = io;
        } else {
            crst.version = 2;
            crst.v.v2.host = host.c_str();
            crst.v.v2.bucket = bucket.c_str();
            crst.v.v2.user = user.c_str();
            crst.v.v2.passwd = pass.c_str();
            crst.v.v2.io = io;
            crst.v.v2.mchosts = mcNodes.c_str();
        }
    }

    void setMcPorts(const std::vector< int > &portlist)
    {
        std::stringstream ss;
        std::vector< int >::const_iterator ii = portlist.begin();
        for (; ii != portlist.end(); ii++) {
            ss << "localhost";
            ss << ":";
            ss << std::dec << *ii;
            ss << ";";
        }
        mcNodes = ss.str();
    }

  protected:
    std::string host;
    std::string user;
    std::string pass;
    std::string bucket;
    std::string mcNodes;

  private:
    void loadParam(std::string &d, const char *s)
    {
        if (s) {
            d.assign(s);
        }
    }
};

#endif
