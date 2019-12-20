/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2017-2019 Couchbase, Inc.
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

#ifndef CBC_OPTIONS_H
#define CBC_OPTIONS_H

#define CLIOPTS_ENABLE_CXX 1
#include <libcouchbase/couchbase.h>
#include <exception>
#include <stdexcept>
#include <sstream>
#include "contrib/cliopts/cliopts.h"

#define CBC_CONFIG_FILENAME ".cbcrc"
#define CBC_WIN32_APPDIR "Couchbase CBC Utility"

#define DURABILITY_GETTER()                                                                                            \
    lcb_DURABILITY_LEVEL durability()                                                                                  \
    {                                                                                                                  \
        if (o_durability.passed()) {                                                                                   \
            std::string s = o_durability.const_result();                                                               \
            if (s == "none") {                                                                                         \
                return LCB_DURABILITYLEVEL_NONE;                                                                       \
            } else if (s == "majority") {                                                                              \
                return LCB_DURABILITYLEVEL_MAJORITY;                                                                   \
            } else if (s == "majority_and_persist_on_master") {                                                        \
                return LCB_DURABILITYLEVEL_MAJORITY_AND_PERSIST_ON_MASTER;                                             \
            } else if (s == "persist_to_majority") {                                                                   \
                return LCB_DURABILITYLEVEL_PERSIST_TO_MAJORITY;                                                        \
            } else {                                                                                                   \
                throw BadArg(                                                                                          \
                    std::string("Invalid durability level \"") + s +                                                   \
                    "\". Allowed values: \"majority\", \"majority_and_persist_on_master\", \"persist_to_majority\"."); \
            }                                                                                                          \
        }                                                                                                              \
        return LCB_DURABILITYLEVEL_NONE;                                                                               \
    }

namespace cbc
{

#define X_OPTIONS(X)                                                                                                   \
    X(String, host, "host", 'h')                                                                                       \
    X(String, bucket, "bucket", 'b')                                                                                   \
    X(String, passwd, "password", 'P')                                                                                 \
    X(String, user, "username", 'u')                                                                                   \
    X(String, transport, "bootstrap-protocol", 'C')                                                                    \
    X(String, configcache, "config-cache", 'Z')                                                                        \
    X(String, saslmech, "force-sasl-mech", 'S')                                                                        \
    X(String, connstr, "spec", 'U')                                                                                    \
    X(String, ssl, "ssl", '\0')                                                                                        \
    X(String, truststorepath, "truststorepath", '\0')                                                                  \
    X(String, certpath, "certpath", '\0')                                                                              \
    X(String, keypath, "keypath", '\0')                                                                                \
    X(UInt, timeout, "timeout", '\0')                                                                                  \
    X(Bool, timings, "timings", 'T')                                                                                   \
    X(Bool, verbose, "verbose", 'v')                                                                                   \
    X(Bool, dump, "dump", '\0')                                                                                        \
    X(Bool, compress, "compress", 'y')                                                                                 \
    X(List, cparams, "cparam", 'D')

class LcbError : public std::runtime_error
{
  private:
    static std::string format_err(lcb_STATUS err, std::string msg)
    {
        std::stringstream ss;
        if (!msg.empty()) {
            ss << msg << ". ";
        }
        ss << "libcouchbase error: " << lcb_strerror_long(err);
        return ss.str();
    }

  public:
    lcb_STATUS rc;
    LcbError(lcb_STATUS code, std::string msg = "") : std::runtime_error(format_err(code, msg)) {}
};

class BadArg : public std::runtime_error
{
  public:
    BadArg(std::string w) : std::runtime_error(w) {}
};

class ConnParams
{
  public:
    ConnParams();
    void fillCropts(lcb_create_st &);
    void addToParser(cliopts::Parser &parser);
    lcb_STATUS doCtls(lcb_INSTANCE *instance);
    bool useTimings()
    {
        return o_timings.result();
    }
    int numTimings()
    {
        return o_timings.numSpecified();
    }
    cliopts::BoolOption &getTimings()
    {
        return o_timings;
    }
    void setAdminMode();
    bool shouldDump()
    {
        return o_dump.result();
    }
    void writeConfig(const std::string &dest = getConfigfileName());
    static std::string getUserHome();
    static std::string getConfigfileName();

  private:
#define X(tp, varname, longdesc, shortdesc) cliopts::tp##Option o_##varname;

    X_OPTIONS(X)
#undef X
    std::string connstr;
    std::string passwd;
    bool isAdmin;
    bool loadFileDefaults();
};

} // namespace cbc

#endif
