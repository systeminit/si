/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2015-2019 Couchbase, Inc.
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

#include "config.h"
#include <sys/types.h>
#include <libcouchbase/couchbase.h>
#include <libcouchbase/vbucket.h>
#include <iostream>
#include <list>
#include <cassert>
#include <cstdio>
#include <cstdlib>
#include <cerrno>
#include <fstream>
#include <algorithm> // random_shuffle
#include <stdexcept>
#include <sstream>
#ifndef WIN32
#include <pthread.h>
#include <unistd.h> // isatty()
#endif
#include "common/options.h"
#include "common/histogram.h"

using namespace cbc;
using namespace cliopts;
using std::cerr;
using std::endl;
using std::string;
using std::vector;

#ifndef _WIN32
static bool use_ansi_codes = true;
#else
static bool use_ansi_codes = false;
#endif

static void do_or_die(lcb_STATUS rc)
{
    if (rc != LCB_SUCCESS) {
        throw std::runtime_error(lcb_strerror_long(rc));
    }
}

class Metrics
{
  public:
    Metrics() : n_rows(0), n_queries(0), n_errors(0), last_update(time(NULL)), hg(NULL)
    {
#ifndef _WIN32
        if (pthread_mutex_init(&m_lock, NULL) != 0) {
            abort();
        }
#endif
        start_time = last_update;
    }

    size_t nerrors()
    {
        return n_errors;
    }

    void update_row(size_t n = 1)
    {
        n_rows += n;
        update_display();
    }
    void update_done(size_t n = 1)
    {
        n_queries += n;
        update_display();
    }
    void update_error(size_t n = 1)
    {
        n_errors += n;
        update_display();
    }

    void update_timings(lcb_U64 duration)
    {
        if (hg != NULL) {
            hg->record(duration);
        }
    }

#ifndef _WIN32
    bool is_tty() const
    {
        return isatty(STDOUT_FILENO);
    }
    void lock()
    {
        pthread_mutex_lock(&m_lock);
    }
    void unlock()
    {
        pthread_mutex_unlock(&m_lock);
    }
#else
    void lock() {}
    void unlock() {}
    bool is_tty() const
    {
        return false;
    }
#endif
    void prepare_screen()
    {
        if (is_tty() && use_ansi_codes) {
            printf("\n\n\n");
        }
    }

    void prepare_timings()
    {
        if (hg == NULL) {
            hg = new Histogram();
            hg->installStandalone(stdout);
        }
    }

  private:
    void update_display()
    {
        time_t now = time(NULL);
        time_t duration = now - last_update;

        if (!duration) {
            return;
        }

        last_update = now;

        const char *prefix;
        const char *final_suffix;

        // Only use "ticker" style updates if we're a TTY and we have no
        // following timings.
        if (use_ansi_codes && is_tty() && hg == NULL) {
            // Move up 3 cursors
            printf("\x1B[2A");
            prefix = "\x1B[K";
            final_suffix = "\r";
        } else {
            // Determine the total number of time
            unsigned total_duration = now - start_time;
            printf("\n"); // Clear line..
            printf("+%us\n", total_duration);
            prefix = "";
            final_suffix = "\n";
        }

        printf("%sQUERIES/SEC: %lu\n", prefix, (unsigned long)(n_queries / duration));
        printf("%sROWS/SEC:    %lu\n", prefix, (unsigned long)(n_rows / duration));
        printf("%sERRORS:      %lu%s", prefix, (unsigned long)n_errors, final_suffix);

        if (hg != NULL) {
            hg->write();
        }
        fflush(stdout);

        n_queries = 0;
        n_rows = 0;
    }

    size_t n_rows;
    size_t n_queries;
    size_t n_errors;
    time_t last_update;
    time_t start_time;
    cbc::Histogram *hg;
#ifndef _WIN32
    pthread_mutex_t m_lock;
#endif
};

Metrics GlobalMetrics;

class Configuration
{
  public:
    Configuration() : o_file("queryfile"), o_threads("num-threads"), o_errlog("error-log"), m_errlog(NULL)
    {
        o_file.mandatory(true);
        o_file.description("Path to a file containing all the queries to execute. "
                           "Each line should contain the full query body");
        o_file.abbrev('f');

        o_threads.description("Number of threads to run");
        o_threads.abbrev('t');
        o_threads.setDefault(1);

        o_errlog.description("Path to a file containing failed queries");
        o_errlog.abbrev('e');
        o_errlog.setDefault("");
    }

    ~Configuration()
    {
        if (m_errlog != NULL) {
            delete m_errlog;
            m_errlog = NULL;
        }
    }
    void addToParser(Parser &parser)
    {
        parser.addOption(o_file);
        parser.addOption(o_threads);
        parser.addOption(o_errlog);
        m_params.addToParser(parser);
    }

    void processOptions()
    {
        std::ifstream ifs(o_file.const_result().c_str());
        if (!ifs.is_open()) {
            int ec_save = errno;
            string errstr(o_file.const_result());
            errstr += ": ";
            errstr += strerror(ec_save);
            throw std::runtime_error(errstr);
        }

        string curline;
        while (ifs.good()) {
            std::getline(ifs, curline);
            if (!curline.empty()) {
                m_queries.push_back(curline);
            }
        }
        std::cerr << "Loaded " << m_queries.size() << " queries "
                  << "from \"" << o_file.const_result() << "\"" << std::endl;
        if (m_params.useTimings()) {
            GlobalMetrics.prepare_timings();
        }

        if (o_errlog.passed()) {
            m_errlog = new std::ofstream(o_errlog.const_result().c_str());
            if (!m_errlog->is_open()) {
                int ec_save = errno;
                string errstr(o_file.const_result());
                errstr += ": ";
                errstr += strerror(ec_save);
                throw std::runtime_error(errstr);
            }
        }
    }

    void set_cropts(lcb_create_st &opts)
    {
        m_params.fillCropts(opts);
    }
    const vector< string > &queries() const
    {
        return m_queries;
    }
    size_t nthreads()
    {
        return o_threads.result();
    }
    std::ofstream *errlog()
    {
        return m_errlog;
    }

  private:
    vector< string > m_queries;
    StringOption o_file;
    UIntOption o_threads;
    ConnParams m_params;
    StringOption o_errlog;
    std::ofstream *m_errlog;
};

extern "C" {
static void n1qlcb(lcb_INSTANCE *, int, const lcb_RESPN1QL *resp);
}
extern "C" {
static void *pthrfunc(void *);
}

class ThreadContext;
struct QueryContext {
    lcb_U64 begin;
    bool received;      // whether any row was received
    ThreadContext *ctx; // Parent

    QueryContext(ThreadContext *tctx) : begin(lcb_nstime()), received(false), ctx(tctx) {}
};

class ThreadContext
{
  public:
    void run()
    {
        while (!m_cancelled) {
            vector< string >::const_iterator ii = m_queries.begin();
            for (; ii != m_queries.end(); ++ii) {
                run_one_query(*ii);
            }
        }
    }

#ifndef _WIN32
    void start()
    {
        assert(m_thr == NULL);
        m_thr = new pthread_t;
        int rc = pthread_create(m_thr, NULL, pthrfunc, this);
        if (rc != 0) {
            throw std::runtime_error(strerror(rc));
        }
    }

    void join()
    {
        assert(m_thr != NULL);
        void *arg = NULL;
        pthread_join(*m_thr, &arg);
    }

    ~ThreadContext()
    {
        if (m_thr != NULL) {
            join();
            delete m_thr;
            m_thr = NULL;
        }
    }
#else
    void start()
    {
        run();
    }
    void join() {}
#endif

    void handle_response(const lcb_RESPN1QL *resp, QueryContext *ctx)
    {
        if (!ctx->received) {
            lcb_U64 duration = lcb_nstime() - ctx->begin;
            m_metrics->lock();
            m_metrics->update_timings(duration);
            m_metrics->unlock();

            ctx->received = true;
        }

        if (lcb_respn1ql_is_final(resp)) {
            lcb_STATUS rc = lcb_respn1ql_status(resp);
            if (rc != LCB_SUCCESS) {
                if (m_errlog != NULL) {
                    const char *p;
                    size_t n;

                    lcb_cmdn1ql_payload(m_cmd, &p, &n);
                    std::stringstream ss;
                    ss.write(p, n);
                    ss << endl;
                    lcb_respn1ql_row(resp, &p, &n);
                    ss.write(p, n);
                    log_error(rc, ss.str().c_str(), ss.str().size());
                } else {
                    log_error(rc, NULL, 0);
                }
            }
        } else {
            last_nrow++;
        }
    }

    ThreadContext(lcb_INSTANCE *instance, const vector< string > &initial_queries, std::ofstream *errlog)
        : m_instance(instance), last_nerr(0), last_nrow(0), m_metrics(&GlobalMetrics), m_cancelled(false), m_thr(NULL),
          m_errlog(errlog)
    {
        lcb_cmdn1ql_reset(m_cmd);
        lcb_cmdn1ql_callback(m_cmd, n1qlcb);

        // Shuffle the list
        m_queries = initial_queries;
        std::random_shuffle(m_queries.begin(), m_queries.end());
    }

  private:
    void log_error(lcb_STATUS err, const char *info, size_t ninfo)
    {
        size_t erridx;
        m_metrics->lock();
        m_metrics->update_error();
        erridx = m_metrics->nerrors();
        m_metrics->unlock();

        if (m_errlog != NULL) {
            std::stringstream ss;
            ss << "[" << erridx << "] " << lcb_strerror_short(err) << endl;
            if (ninfo) {
                ss.write(info, ninfo);
                ss << endl;
            }
            *m_errlog << ss.str();
            m_errlog->flush();
        }
    }

    void run_one_query(const string &txt)
    {
        // Reset counters
        last_nrow = 0;
        last_nerr = 0;

        lcb_cmdn1ql_query(m_cmd, txt.c_str(), txt.size());

        // Set up our context
        QueryContext qctx(this);

        lcb_STATUS rc = lcb_n1ql(m_instance, &qctx, m_cmd);
        if (rc != LCB_SUCCESS) {
            log_error(rc, txt.c_str(), txt.size());
        } else {
            lcb_wait(m_instance);
            m_metrics->lock();
            m_metrics->update_row(last_nrow);
            m_metrics->update_done(1);
            m_metrics->unlock();
        }
    }

    lcb_INSTANCE *m_instance;
    vector< string > m_queries;
    size_t last_nerr;
    size_t last_nrow;
    lcb_CMDN1QL *m_cmd;
    Metrics *m_metrics;
    volatile bool m_cancelled;
#ifndef _WIN32
    pthread_t *m_thr;
#else
    void *m_thr;
#endif
    std::ofstream *m_errlog;
};

static void n1qlcb(lcb_INSTANCE *, int, const lcb_RESPN1QL *resp)
{
    QueryContext *qctx;
    lcb_respn1ql_cookie(resp, (void **)&qctx);
    qctx->ctx->handle_response(resp, qctx);
}

static void *pthrfunc(void *arg)
{
    reinterpret_cast< ThreadContext * >(arg)->run();
    return NULL;
}

static bool instance_has_n1ql(lcb_INSTANCE *instance)
{
    // Check that the instance supports N1QL
    lcbvb_CONFIG *vbc;
    do_or_die(lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_VBCONFIG, &vbc));

    int sslmode = 0;
    lcbvb_SVCMODE svcmode = LCBVB_SVCMODE_PLAIN;

    do_or_die(lcb_cntl(instance, LCB_CNTL_GET, LCB_CNTL_SSL_MODE, &sslmode));

    if (sslmode & LCB_SSL_ENABLED) {
        svcmode = LCBVB_SVCMODE_SSL;
    }

    int hix = lcbvb_get_randhost(vbc, LCBVB_SVCTYPE_N1QL, svcmode);
    return hix > -1;
}

static void real_main(int argc, char **argv)
{
    Configuration config;
    Parser parser;
    config.addToParser(parser);
    parser.parse(argc, argv);

    vector< ThreadContext * > threads;
    vector< lcb_INSTANCE * > instances;

    lcb_create_st cropts = {0};
    config.set_cropts(cropts);
    config.processOptions();

    for (size_t ii = 0; ii < config.nthreads(); ii++) {
        lcb_INSTANCE *instance;
        do_or_die(lcb_create(&instance, &cropts));
        do_or_die(lcb_connect(instance));
        lcb_wait(instance);
        do_or_die(lcb_get_bootstrap_status(instance));

        if (ii == 0 && !instance_has_n1ql(instance)) {
            throw std::runtime_error("Cluster does not support N1QL!");
        }

        ThreadContext *cx = new ThreadContext(instance, config.queries(), config.errlog());
        threads.push_back(cx);
        instances.push_back(instance);
    }

    GlobalMetrics.prepare_screen();

    for (size_t ii = 0; ii < threads.size(); ++ii) {
        threads[ii]->start();
    }
    for (size_t ii = 0; ii < threads.size(); ++ii) {
        threads[ii]->join();
    }
    for (size_t ii = 0; ii < instances.size(); ++ii) {
        lcb_destroy(instances[ii]);
    }
}

int main(int argc, char **argv)
{
    try {
        real_main(argc, argv);
        return 0;
    } catch (std::exception &exc) {
        cerr << exc.what() << endl;
        exit(EXIT_FAILURE);
    }
}
