/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2014-2019 Couchbase, Inc.
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

#include "options.h"
#include <stdexcept>
#include <iostream>
#include <sstream>
#include <fstream>
#include <ctype.h>
#include <stdio.h>
#include <time.h>

using namespace cbc;
using namespace cliopts;
using std::endl;
using std::ifstream;
using std::ofstream;
using std::string;

#ifndef _WIN32
#include <unistd.h>
#include <termios.h>
#endif

static string promptPassword(const char *prompt)
{
#ifndef _WIN32
    termios oldattr, newattr;
    tcgetattr(STDIN_FILENO, &oldattr);
    newattr = oldattr;
    newattr.c_lflag &= ~ECHO;
    tcsetattr(STDIN_FILENO, TCSANOW, &newattr);
#endif

    fprintf(stderr, "%s", prompt);
    fflush(stderr);
    // Use cin here. A bit more convenient
    string ret;
    std::cin >> ret;
#ifndef _WIN32
    fprintf(stderr, "\n");
    tcsetattr(STDIN_FILENO, TCSANOW, &oldattr);
#endif
    return ret;
}

static void makeLowerCase(string &s)
{
    for (size_t ii = 0; ii < s.size(); ++ii) {
        s[ii] = tolower(s[ii]);
    }
}

#define X(tpname, varname, longname, shortname) o_##varname(longname),
ConnParams::ConnParams() : X_OPTIONS(X) isAdmin(false)
{
// Configure the options
#undef X
#define X(tpname, varname, longname, shortname) o_##varname.abbrev(shortname);
    X_OPTIONS(X)
#undef X

    o_host.description("Hostname to connect to").setDefault("localhost");
    o_host.hide();

    o_bucket.description("Bucket to use").setDefault("default");
    o_bucket.hide();

    o_connstr.description("Connection string").setDefault("couchbase://localhost/default");
    o_user.description("Username");
    o_passwd.description("Bucket password");
    o_saslmech.description("Force SASL mechanism").argdesc("PLAIN|CRAM_MD5");
    o_timings.description("Enable command timings");
    o_timeout.description("Operation timeout");
    o_timeout.hide();
    o_transport.description("Bootstrap protocol").argdesc("HTTP|CCCP|ALL").setDefault("ALL");
    o_configcache.description("Path to cached configuration");
    o_ssl.description("Enable SSL settings").argdesc("ON|OFF|NOVERIFY").setDefault("off");
    o_certpath.description("Path to server SSL certificate");
    o_keypath.description("Path to client SSL private key");
    o_verbose.description("Set debugging output (specify multiple times for greater verbosity");
    o_dump.description("Dump verbose internal state after operations are done");
    o_compress.description("Turn on compression of outgoing data (second time to force compression)").setDefault(false);

    o_cparams.description("Additional options for connection. "
                          "Use -Dtimeout=SECONDS for KV operation timeout");
    o_cparams.argdesc("OPTION=VALUE");

    // Hide some more exotic options
    o_saslmech.hide();
    o_transport.hide();
    o_ssl.hide();
}

void ConnParams::setAdminMode()
{
    o_user.description("Administrative username").setDefault("Administrator");
    o_passwd.description("Administrative password");
    isAdmin = true;
}

void ConnParams::addToParser(Parser &parser)
{
    string errmsg;
    try {
        loadFileDefaults();
    } catch (string &exc) {
        errmsg = exc;
    } catch (const char *&exc) {
        errmsg = exc;
    }
    if (!errmsg.empty()) {
        string newmsg = "Error processing `";
        newmsg += getConfigfileName();
        newmsg += "`. ";
        newmsg += errmsg;
        throw BadArg(newmsg);
    }

#define X(tp, varname, longname, shortname) parser.addOption(o_##varname);
    X_OPTIONS(X)
#undef X
}

string ConnParams::getUserHome()
{
    string ret;
#if _WIN32
    const char *v = getenv("APPDATA");
    if (v) {
        ret = v;
        ret += "\\";
        ret += CBC_WIN32_APPDIR;
        ret += "\\";
    }
#else
    const char *home = getenv("HOME");
    if (home) {
        ret = home;
        ret += "/";
    }
#endif
    return ret;
}

string ConnParams::getConfigfileName()
{
    const char *override = getenv("CBC_CONFIG");
    if (override && *override) {
        return override;
    }

    return getUserHome() + CBC_CONFIG_FILENAME;
}

static void stripWhitespacePadding(string &s)
{
    while (s.empty() == false && isspace((int)s[0])) {
        s.erase(0, 1);
    }
    while (s.empty() == false && isspace((int)s[s.size() - 1])) {
        s.erase(s.size() - 1, 1);
    }
}

bool ConnParams::loadFileDefaults()
{
    // Figure out the home directory
    ifstream f(getConfigfileName().c_str());
    if (!f.good()) {
        return false;
    }

    string curline;
    while ((std::getline(f, curline).good()) && !f.eof()) {
        string key, value;
        size_t pos;

        stripWhitespacePadding(curline);
        if (curline.empty() || curline[0] == '#') {
            continue;
        }

        pos = curline.find('=');
        if (pos == string::npos || pos == curline.size() - 1) {
            throw BadArg("Configuration file must be formatted as key-value pairs. Check " + getConfigfileName());
        }

        key = curline.substr(0, pos);
        value = curline.substr(pos + 1);
        stripWhitespacePadding(key);
        stripWhitespacePadding(value);
        if (key.empty() || value.empty()) {
            throw BadArg("Key and value cannot be empty. Check " + getConfigfileName());
        }

        if (key == "uri") {
            // URI isn't really supported anymore, but try to be compatible
            o_host.setDefault(value).setPassed();
        } else if (key == "user") {
            o_user.setDefault(value).setPassed();
        } else if (key == "password") {
            o_passwd.setDefault(value).setPassed();
        } else if (key == "bucket") {
            o_bucket.setDefault(value).setPassed();
        } else if (key == "timeout") {
            unsigned ival = 0;
            if (!sscanf(value.c_str(), "%u", &ival)) {
                throw BadArg("Invalid formatting for timeout. Check " + getConfigfileName());
            }
            o_timeout.setDefault(ival).setPassed();
        } else if (key == "connstr") {
            o_connstr.setDefault(value).setPassed();
        } else if (key == "certpath") {
            o_certpath.setDefault(value).setPassed();
        } else if (key == "keypath") {
            o_keypath.setDefault(value).setPassed();
        } else if (key == "ssl") {
            o_ssl.setDefault(value).setPassed();
        } else {
            throw BadArg(string("Unrecognized key: ") + key + ". Check " + getConfigfileName());
        }
    }
    return true;
}

static void writeOption(ofstream &f, StringOption &opt, const string &key)
{
    if (!opt.passed()) {
        return;
    }
    f << key << '=' << opt.const_result() << endl;
}

void ConnParams::writeConfig(const string &s)
{
    // Figure out the intermediate directories
    ofstream f;
    try {
        f.exceptions(std::ios::failbit | std::ios::badbit);
        f.open(s.c_str());
    } catch (std::exception &ex) {
        throw std::runtime_error("Couldn't open " + s + " " + ex.what());
    }

    time_t now = time(NULL);
    const char *timestr = ctime(&now);
    f << "# Generated by cbc at " << string(timestr) << endl;

    if (!connstr.empty()) {
        // Contains bucket, user
        f << "connstr=" << connstr << endl;
    }
    writeOption(f, o_user, "user");
    writeOption(f, o_passwd, "password");
    writeOption(f, o_ssl, "ssl");
    writeOption(f, o_truststorepath, "truststorepath");
    writeOption(f, o_certpath, "certpath");
    writeOption(f, o_keypath, "keypath");

    if (o_timeout.passed()) {
        f << "timeout=" << std::dec << o_timeout.result() << endl;
    }

    f.flush();
    f.close();
}

void ConnParams::fillCropts(lcb_create_st &cropts)
{
    memset(&cropts, 0, sizeof(lcb_create_st));
    passwd = o_passwd.result();
    if (passwd == "-") {
        passwd = promptPassword("Bucket password: ");
    }

    if (o_connstr.passed()) {
        if (o_host.passed() || o_bucket.passed()) {
            throw BadArg("Use of the deprecated "
                         "-h/--host or -b/--bucket options with -U is "
                         "not allowed!");
        }
        connstr = o_connstr.const_result();
        if (connstr.find('?') == string::npos) {
            connstr += '?';
        } else if (connstr[connstr.size() - 1] != '&') {
            connstr += '&';
        }
    } else {
        string host = o_host.result();
        string bucket = o_bucket.result();

        for (size_t ii = 0; ii < host.size(); ++ii) {
            if (host[ii] == ';') {
                host[ii] = ',';
            }
        }

        if (o_host.passed() || o_bucket.passed()) {
            fprintf(stderr, "CBC: WARNING\n");
            fprintf(stderr, "  The -h/--host and -b/--bucket options are deprecated. Use connection string instead\n");
            fprintf(stderr, "  e.g. -U couchbase://%s/%s\n", host.c_str(), o_bucket.const_result().c_str());
        }

        connstr = "http://";
        connstr += host;
        connstr += "/";
        connstr += bucket;
        connstr += "?";
    }

    if (connstr.find("8091") != string::npos) {
        fprintf(stderr, "CBC: WARNING\n");
        fprintf(stderr, "  Specifying the default port (8091) has no effect\n");
    }

    if (o_truststorepath.passed()) {
        connstr += "truststorepath=";
        connstr += o_truststorepath.result();
        connstr += '&';
    }
    if (o_certpath.passed()) {
        connstr += "certpath=";
        connstr += o_certpath.result();
        connstr += '&';
    }
    if (o_keypath.passed()) {
        connstr += "keypath=";
        connstr += o_keypath.result();
        connstr += '&';
    }
    if (o_ssl.passed()) {
        connstr += "ssl=";
        connstr += o_ssl.result();
        connstr += '&';
    }
    if (o_transport.passed()) {
        connstr += "bootstrap_on=";
        string tmp = o_transport.result();
        makeLowerCase(tmp);
        connstr += tmp;
        connstr += '&';
    }
    if (o_timeout.passed()) {
        std::cerr << "Warning: --timeout option is deprecated. Use -Dtimeout=SECONDS" << std::endl;
        std::cerr << "         --timeout will be interpreted as SECONDS" << std::endl;
        connstr += "operation_timeout=";
        std::stringstream ss;
        ss << std::dec << o_timeout.result();
        connstr += ss.str();
        connstr += '&';
    }
    if (o_configcache.passed()) {
        connstr += "config_cache=";
        connstr += o_configcache.result();
        connstr += '&';
    }
    if (o_user.passed()) {
        connstr += "username=";
        connstr += o_user.const_result();
        connstr += '&';
    }

    const std::vector< std::string > &extras = o_cparams.const_result();
    for (size_t ii = 0; ii < extras.size(); ii++) {
        connstr += extras[ii];
        connstr += '&';
    }

    int vLevel = 1;
    if (o_verbose.passed()) {
        vLevel += o_verbose.numSpecified();
        std::stringstream ss;
        ss << std::dec << vLevel;
        connstr += "console_log_level=";
        connstr += ss.str();
        connstr += '&';
    }

    cropts.version = 3;
    cropts.v.v3.io = NULL;
    cropts.v.v3.username = NULL;
    cropts.v.v3.passwd = passwd.c_str();
    cropts.v.v3.connstr = connstr.c_str();
    if (isAdmin) {
        cropts.v.v3.type = LCB_TYPE_CLUSTER;
    } else {
        cropts.v.v3.type = LCB_TYPE_BUCKET;
    }
}

template < typename T > void doPctl(lcb_INSTANCE *instance, int cmd, T arg)
{
    lcb_STATUS err;
    err = lcb_cntl(instance, LCB_CNTL_SET, cmd, (void *)arg);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
}

template < typename T > void doSctl(lcb_INSTANCE *instance, int cmd, T arg)
{
    doPctl< T * >(instance, cmd, &arg);
}

void doStringCtl(lcb_INSTANCE *instance, const char *s, const char *val)
{
    lcb_STATUS err;
    err = lcb_cntl_string(instance, s, val);
    if (err != LCB_SUCCESS) {
        throw LcbError(err);
    }
}

lcb_STATUS ConnParams::doCtls(lcb_INSTANCE *instance)
{
    try {
        if (o_saslmech.passed()) {
            doPctl< const char * >(instance, LCB_CNTL_FORCE_SASL_MECH, o_saslmech.result().c_str());
        }

        // Set the detailed error codes option
        doSctl< int >(instance, LCB_CNTL_DETAILED_ERRCODES, 1);

        if (!o_connstr.passed() || o_connstr.result().find("compression=") == std::string::npos) {
            int opts = LCB_COMPRESS_IN;
            if (o_compress.passed()) {
                opts |= LCB_COMPRESS_OUT;
                if (o_compress.numSpecified() > 1) {
                    opts |= LCB_COMPRESS_FORCE;
                }
            }
            doPctl(instance, LCB_CNTL_COMPRESSION_OPTS, &opts);
        }
    } catch (lcb_STATUS &err) {
        return err;
    }
    return LCB_SUCCESS;
}
