/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2013-2019 Couchbase, Inc.
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

/**
 * Rather than hacking together a shell script or depending on some scripting
 * language, we'll use a simple C++ application to run 'unit-tests'
 * with the appropriate settings we need.
 */

#include <libcouchbase/couchbase.h>

#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include <cassert>

#include <iostream>
#include <fstream>
#include <vector>
#include <cstring>
#include <list>
#include <sstream>

#include "config.h"
#include "mocksupport/procutil.h"
#define CLIOPTS_ENABLE_CXX
#include "contrib/cliopts/cliopts.h"
#include <libcouchbase/couchbase.h>

#define TESTS_BASE "sock-tests;nonio-tests;rdb-tests;mc-tests;"
#define PLUGIN_ENV_VAR "LCB_IOPS_NAME"
#define LCB_SRCROOT_ENV_VAR "srcdir"
#define DEFAULT_TEST_NAMES TESTS_BASE "unit-tests"

#ifdef _WIN32
const char default_plugins_string[] = "select;iocp;libuv";
#define PATHSEP "\\"
#define usleep(n) Sleep((n) / 1000)
#define setenv(key, value, ignored) SetEnvironmentVariable(key, value)
#else
#include <signal.h>
#include <unistd.h> /* usleep */
const char default_plugins_string[] = "select"
#if defined(HAVE_LIBEV3) || defined(HAVE_LIBEV4)
                                      ";libev"
#endif
#if defined(HAVE_LIBEVENT) || defined(HAVE_LIBEVENT2)
                                      ";libevent"
#endif
#ifdef HAVE_LIBUV
                                      ";libuv"
#endif
    ;
#define PATHSEP "/"
#endif

typedef std::vector< std::string > strlist;

class TestConfiguration
{

  public:
    TestConfiguration()
        : opt_debugger("debugger"), opt_plugins("plugins"), opt_jobs("jobs"), opt_srcdir("srcdir"),
          opt_bindir("testdir"), opt_interactive("interactive"), opt_verbose("verbose"), opt_cycles("repeat"),
          opt_libdir("libdir"), opt_bins("tests"), opt_realcluster("cluster"), opt_gtest_filter("gtest_filter"),
          opt_gtest_break_on_failure("gtest_break_on_failure"), opt_gtest_catch_exceptions("gtest_catch_exceptions")
    {
        opt_debugger.abbrev('d').description("Verbatim string to prepend to the binary command line");

        opt_plugins.abbrev('p')
            .description("semicolon-delimited list of plugins to test")
            .setDefault(default_plugins_string);
        opt_jobs.abbrev('j').description("Execute this many processes concurrently").setDefault(1);

        opt_srcdir.abbrev('S')
            .description("root directory of source tree (for locating mock)")
            .setDefault(getEffectiveSrcroot());

        opt_bindir.abbrev('T')
            .description("Directory where test binaries are located")
            .setDefault(getEffectiveTestdir());

        opt_interactive.abbrev('I').description(
            "Set this to true when using an interactive debugger. This unblocks stdin");

        opt_bins.abbrev('B').description("semicolon delimited list of tests to run").setDefault(DEFAULT_TEST_NAMES);

        opt_cycles.abbrev('n').description("Number of times to run the tests").setDefault(1);

        opt_libdir.abbrev('L').description("Directory where plugins are located. Useful on OS X");

        opt_realcluster.abbrev('C').description("Path to real cluster");

        opt_verbose.abbrev('v');
    }

    ~TestConfiguration() {}

    static void splitSemicolonString(const std::string &s, strlist &l)
    {
        std::string cur;

        for (const char *c = s.c_str(); *c; c++) {
            if (*c == ';') {
                l.push_back(cur);
                cur.clear();
                continue;
            }
            cur += *c;
        }

        if (!cur.empty()) {
            l.push_back(cur);
        }
    }

    bool parseOptions(int argc, char **argv)
    {
        std::stringstream ss;
        cliopts::Parser parser("check-all");

        parser.addOption(opt_debugger);
        parser.addOption(opt_plugins);
        parser.addOption(opt_jobs);
        parser.addOption(opt_srcdir);
        parser.addOption(opt_bindir);
        parser.addOption(opt_interactive);
        parser.addOption(opt_verbose);
        parser.addOption(opt_cycles);
        parser.addOption(opt_libdir);
        parser.addOption(opt_bins);
        parser.addOption(opt_realcluster);
        parser.addOption(opt_gtest_filter);
        parser.addOption(opt_gtest_break_on_failure);
        parser.addOption(opt_gtest_catch_exceptions);

        if (!parser.parse(argc, argv, false)) {
            return false;
        }

        using std::string;
        using std::vector;

        const vector< string > &args = parser.getRestArgs();
        for (size_t ii = 0; ii < args.size(); ii++) {
            ss << args[ii] << " ";
        }

        if (!opt_gtest_filter.result().empty()) {
            ss << " "
               << "--gtest_filter=" << opt_gtest_filter.result();
        }
        if (opt_gtest_break_on_failure.passed()) {
            ss << " "
               << "--gtest_break_on_failure=1";
        }
        if (opt_gtest_catch_exceptions.passed()) {
            ss << " "
               << "--gtest_catch_exceptions=1";
        }

        binOptions = ss.str();
        srcroot = opt_srcdir.result();
        testdir = opt_bindir.result();
        debugger = opt_debugger.result();
        libDir = opt_libdir.result();
        realClusterEnv = opt_realcluster.result();

        // Verbosity
        isVerbose = opt_verbose.result();

        // isInteractive
        isInteractive = opt_interactive.result();

        // Jobs
        maxJobs = opt_jobs.result();
        maxCycles = opt_cycles.result();
        setJobsFromEnvironment();

        // Plugin list:
        splitSemicolonString(opt_plugins.result(), plugins);

        // Test names:
        splitSemicolonString(opt_bins.result(), testnames);

        // Set the library dir, if not set
        if (libDir.empty()) {
            libDir = testdir + "/../" + "lib";
        }
        return true;
    }

    // Sets up the command line, appending any debugger info and paths
    std::string setupCommandline(std::string &name)
    {
        std::stringstream ss;
        std::string ret;

        if (!debugger.empty()) {
            ss << debugger << " ";
        }

        ss << testdir << PATHSEP << name;

        if (!binOptions.empty()) {
            ss << " " << binOptions;
        }

        return ss.str();
    }

    // Options passed to the binary itself
    std::string binOptions;
    std::string srcroot;
    std::string testdir;
    std::string debugger;
    std::string libDir;
    std::string realClusterEnv;

    strlist plugins;
    strlist testnames;

    bool isVerbose;
    bool isInteractive;
    int maxJobs;
    int maxCycles;
    int getVerbosityLevel()
    {
        return opt_verbose.numSpecified();
    }

  private:
    cliopts::StringOption opt_debugger;
    cliopts::StringOption opt_plugins;
    cliopts::UIntOption opt_jobs;
    cliopts::StringOption opt_srcdir;
    cliopts::StringOption opt_bindir;
    cliopts::BoolOption opt_interactive;
    cliopts::BoolOption opt_verbose;
    cliopts::IntOption opt_cycles;
    cliopts::StringOption opt_libdir;
    cliopts::StringOption opt_bins;
    cliopts::StringOption opt_realcluster;
    cliopts::StringOption opt_gtest_filter;
    cliopts::BoolOption opt_gtest_break_on_failure;
    cliopts::BoolOption opt_gtest_catch_exceptions;

    void setJobsFromEnvironment()
    {
        char *tmp = getenv("MAKEFLAGS");

        if (tmp == NULL || *tmp == '\0') {
            return;
        }

        if (strstr(tmp, "-j")) {
            maxJobs = 32;

        } else {
            maxJobs = 1;
        }
    }

    std::string getEffectiveSrcroot()
    {
        const char *tmp = getenv(LCB_SRCROOT_ENV_VAR);
        if (tmp && *tmp) {
            return tmp;
        }

        return getDefaultSrcroot();
    }

    std::string getEffectiveTestdir()
    {
        const char *tmp = getenv("outdir");
        if (tmp && *tmp) {
            return tmp;
        }
        return getDefaultTestdir();
    }

#ifndef _WIN32
    // Evaluated *before*
    std::string getDefaultSrcroot()
    {
        return ".";
    }

    std::string getDefaultTestdir()
    {
        return (srcroot + PATHSEP) + "tests";
    }

#else
    std::string getSelfDirname()
    {
        DWORD result;
        char pathbuf[4096] = {0};
        result = GetModuleFileName(NULL, pathbuf, sizeof(pathbuf));
        assert(result > 0);
        assert(result < sizeof(pathbuf));

        for (DWORD ii = result; ii; ii--) {
            if (pathbuf[ii] == '\\') {
                break;
            }
            pathbuf[ii] = '\0';
        }
        return pathbuf;
    }
    // For windows, we reside in the same directory as the binaries
    std::string getDefaultSrcroot()
    {
        std::string dir = getSelfDirname();
        std::stringstream ss;
        ss << dir;

        int components_max;

#ifdef _MSC_VER
        // Visual Studio projects are usually something like:
        // $ROOT\VS\10.0\bin\Debug
        // (1)..\bin, (2)..\10.0, (3)..\VS, (4)..\$ROOT
        components_max = 4;
#else
        // For MinGW, it's something like $ROOT\$BUILD\bin; so
        // (1) ..\BUILD, (2) ..\ROOT
        components_max = 2;
#endif

        for (int ii = 0; ii < components_max; ii++) {
            ss << PATHSEP << "..";
        }

        return ss.str();
    }

    std::string getDefaultTestdir()
    {
        return getSelfDirname();
    }
#endif
};

static void setPluginEnvironment(std::string &name)
{
    const char *v = NULL;
    if (name != "default") {
        v = name.c_str();
    }

    setenv(PLUGIN_ENV_VAR, v, 1);

    fprintf(stderr, "%s=%s ... ", PLUGIN_ENV_VAR, name.c_str());
    struct lcb_cntl_iops_info_st ioi;
    memset(&ioi, 0, sizeof(ioi));

    lcb_STATUS err = lcb_cntl(NULL, LCB_CNTL_GET, LCB_CNTL_IOPS_DEFAULT_TYPES, &ioi);
    if (err != LCB_SUCCESS) {
        fprintf(stderr, "LCB Error 0x%x\n", err);
    } else {
        fprintf(stderr, "Plugin ID: 0x%x\n", ioi.v.v0.effective);
    }
}

static void setLinkerEnvironment(std::string &path)
{
#ifdef _WIN32
    if (false) {
        return;
    }
#endif

    if (path.empty()) {
        return;
    }
#if __APPLE__
    const char *varname = "DYLD_LIBRARY_PATH";
#else
    const char *varname = "LD_LIBRARY_PATH";
#endif

    const char *existing = getenv(varname);
    std::string newenv;
    if (existing) {
        newenv += existing;
        newenv += ":";
    }
    newenv += path;
    fprintf(stderr, "export %s=%s\n", varname, newenv.c_str());
    setenv(varname, newenv.c_str(), 1);
}

struct Process {
    child_process_t proc_;
    std::string commandline;
    std::string logfileName;
    std::string pluginName;
    std::string testName;
    bool exitedOk;
    bool verbose;

    Process(std::string &plugin, std::string &name, std::string &cmd, TestConfiguration &config)
    {
        this->pluginName = plugin;
        this->testName = name;
        this->commandline = cmd;
        this->verbose = config.isVerbose;
        proc_.interactive = config.isInteractive;
        this->logfileName = "check-all-" + pluginName + "-" + testName + ".log";
    }

    void writeLog(const char *msg)
    {
        std::ofstream out(logfileName.c_str(), std::ios::app);
        out << msg << std::endl;
        out.close();
    }

    void setupPointers()
    {
        memset(&proc_, 0, sizeof(proc_));

        proc_.name = commandline.c_str();

        if (!verbose) {
            proc_.redirect = logfileName.c_str();
        }
    }
};

class TestScheduler
{
  public:
    TestScheduler(unsigned int lim) : limit(lim) {}

    typedef std::list< Process * > proclist;
    std::vector< Process > _all;

    proclist executing;
    proclist scheduled;
    proclist completed;

    unsigned int limit;

    void schedule(Process proc)
    {
        _all.push_back(proc);
    }

    bool runAll()
    {
        proclist::iterator iter;
        scheduleAll();

        while (!(executing.empty() && scheduled.empty())) {
            while ((!scheduled.empty()) && executing.size() < limit) {
                Process *proc = scheduled.front();
                scheduled.pop_front();
                invokeScheduled(proc);
            }

            // Wait for them to complete
            proclist to_remove_e;
            for (iter = executing.begin(); iter != executing.end(); iter++) {
                Process *cur = *iter;
                int rv = wait_process(&cur->proc_, -1);

                if (rv == 0) {
                    char msg[2048];
                    cur->exitedOk = cur->proc_.status == 0;
                    snprintf(msg, 2048, "REAP [%s] '%s' .. %s", cur->pluginName.c_str(), cur->commandline.c_str(),
                             cur->exitedOk ? "OK" : "FAIL");
                    cur->writeLog(msg);
                    fprintf(stderr, "%s\n", msg);
                    cleanup_process(&cur->proc_);
                    to_remove_e.push_back(cur);
                }
            }

            for (iter = to_remove_e.begin(); iter != to_remove_e.end(); iter++) {
                executing.remove(*iter);
                completed.push_back(*iter);
            }

            usleep(5000);
        }

        for (iter = completed.begin(); iter != completed.end(); iter++) {
            if (!(*iter)->exitedOk) {
                return false;
            }
        }

        return true;
    }

  private:
    void scheduleAll()
    {
        for (unsigned int ii = 0; ii < _all.size(); ii++) {
            Process *p = &_all[ii];
            scheduled.push_back(p);
        }
    }
    void invokeScheduled(Process *proc)
    {
        proc->setupPointers();
        setPluginEnvironment(proc->pluginName);
        char msg[2048];
        snprintf(msg, 2048, "START [%s] '%s'", proc->pluginName.c_str(), proc->commandline.c_str());
        proc->writeLog(msg);
        fprintf(stderr, "%s\n", msg);

        int rv = create_process(&proc->proc_);
        if (rv < 0) {
            snprintf(msg, 2048, "FAIL couldn't invoke [%s] '%s'", proc->pluginName.c_str(), proc->commandline.c_str());
            proc->writeLog(msg);
            fprintf(stderr, "%s\n", msg);
            proc->exitedOk = false;
            completed.push_back(proc);
            return;
        } else {
            executing.push_back(proc);
        }
    }
};

static bool runSingleCycle(TestConfiguration &config)
{
    TestScheduler scheduler(config.maxJobs);
    setLinkerEnvironment(config.libDir);
    for (strlist::iterator iter = config.plugins.begin(); iter != config.plugins.end(); iter++) {

        fprintf(stderr, "Testing with plugin '%s'\n", iter->c_str());
#ifdef __linux__
        {
            char buf[1024] = {0};
            sprintf(buf, "ldd %s/libcouchbase_%s.so", config.libDir.c_str(), iter->c_str());
            fprintf(stderr, "%s\n", buf);
            system(buf);
        }
#endif
        for (strlist::iterator iterbins = config.testnames.begin(); iterbins != config.testnames.end(); iterbins++) {

            std::string cmdline = config.setupCommandline(*iterbins);
            fprintf(stderr, "Command line '%s'\n", cmdline.c_str());
            scheduler.schedule(Process(*iter, *iterbins, cmdline, config));
        }
    }

    return scheduler.runAll();
}

int main(int argc, char **argv)
{
#ifndef _WIN32
    signal(SIGPIPE, SIG_IGN);
#endif

    TestConfiguration config;
    if (!config.parseOptions(argc, argv)) {
        exit(EXIT_FAILURE);
    }

    // Set the environment for 'srcdir'
    std::stringstream ss;
    fprintf(stderr, "export %s=%s\n", LCB_SRCROOT_ENV_VAR, config.srcroot.c_str());
    setenv(LCB_SRCROOT_ENV_VAR, config.srcroot.c_str(), 1);
    fprintf(stderr, "export LCB_VERBOSE_TESTS=1\n");
    setenv("LCB_VERBOSE_TESTS", "1", 1);

    char loglevel_s[4096] = {0};
    if (config.getVerbosityLevel() > 0) {
        sprintf(loglevel_s, "%d", config.getVerbosityLevel());
        setenv("LCB_LOGLEVEL", loglevel_s, 0);
        fprintf(stderr, "export LCB_LOGLEVEL=%s\n", loglevel_s);
    }

    if (!config.realClusterEnv.empty()) {
        // format the string
        setenv("LCB_TEST_CLUSTER_CONF", config.realClusterEnv.c_str(), 0);
        fprintf(stderr, "export LCB_TEST_CLUSTER_CONF=%s\n", config.realClusterEnv.c_str());
    }

    for (int ii = 0; ii < config.maxCycles; ii++) {
        if (!runSingleCycle(config)) {
            return EXIT_FAILURE;
        }
    }
    return EXIT_SUCCESS;
}
