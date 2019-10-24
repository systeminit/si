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

#include "internal.h"
#include <libcouchbase/metrics.h>
#include <string>
#include <vector>
#include <map>

namespace lcbmetrics {

class MetricsEntry : public lcb_SERVERMETRICS {
public:
    std::string m_hostport;
    MetricsEntry(const std::string key) : m_hostport(key) {
        memset(static_cast<lcb_SERVERMETRICS*>(this), 0, sizeof (lcb_SERVERMETRICS));
        iometrics.hostport = m_hostport.c_str();
    }

private:
    MetricsEntry();
    MetricsEntry(const MetricsEntry&);
};

class Metrics : public lcb_METRICS {
public:
    std::vector<MetricsEntry *> entries;
    std::vector<lcb_SERVERMETRICS *> raw_entries;

    Metrics() {
        memset(static_cast<lcb_METRICS*>(this), 0, sizeof (lcb_METRICS));
    }

    ~Metrics() {
        for (size_t ii = 0; ii < entries.size(); ++ii) {
            delete entries[ii];
        }
    }

    MetricsEntry *get(const char *host, const char *port, int create) {
        std::string key;
        key.append(host).append(":").append(port);
        for (size_t ii = 0; ii < entries.size(); ++ii) {
            if (entries[ii]->m_hostport == key) {
                return entries[ii];
            }
        }

        if (!create) {
            return NULL;
        }

        MetricsEntry *ent = new MetricsEntry(key);
        entries.push_back(ent);
        raw_entries.push_back(ent);
        nservers = entries.size();
        servers = (const lcb_SERVERMETRICS**) &raw_entries[0];
        return ent;
    }

    static Metrics *from(lcb_METRICS *metrics) {
        return static_cast<Metrics*>(metrics);
    }
};
}

using namespace lcbmetrics;

extern "C" {
lcb_METRICS *
lcb_metrics_new(void)
{
    return new Metrics();
}

void
lcb_metrics_destroy(lcb_METRICS *metrics)
{
    delete Metrics::from(metrics);
}

lcb_SERVERMETRICS *
lcb_metrics_getserver(lcb_METRICS *metrics, const char *h, const char *p, int c)
{
    return Metrics::from(metrics)->get(h, p, c);
}

void
lcb_metrics_dumpio(const lcb_IOMETRICS *metrics, FILE *fp)
{
    fprintf(fp, "Bytes sent: %lu\n", (unsigned long int)metrics->bytes_sent);
    fprintf(fp, "Bytes received: %lu\n", (unsigned long int)metrics->bytes_received);
    fprintf(fp, "IO Close: %lu\n", (unsigned long int)metrics->io_close);
    fprintf(fp, "IO Error: %lu\n", (unsigned long int)metrics->io_error);
}

void
lcb_metrics_dumpserver(const lcb_SERVERMETRICS *metrics, FILE *fp)
{
    lcb_metrics_dumpio(&metrics->iometrics, fp);
    fprintf(fp, "Packets queued: %lu\n", (unsigned long int)metrics->packets_queued);
    fprintf(fp, "Bytes queued: %lu\n", (unsigned long int)metrics->bytes_queued);
    fprintf(fp, "Packets sent: %lu\n", (unsigned long int)metrics->packets_sent);
    fprintf(fp, "Packets received: %lu\n", (unsigned long int)metrics->packets_read);
    fprintf(fp, "Packets errored: %lu\n", (unsigned long int)metrics->packets_errored);
    fprintf(fp, "Packets NMV: %lu\n", (unsigned long int)metrics->packets_nmv);
    fprintf(fp, "Packets timeout: %lu\n", (unsigned long int)metrics->packets_timeout);
    fprintf(fp, "Packets orphaned: %lu", (unsigned long int)metrics->packets_ownerless);
}

void
lcb_metrics_reset_pipeline_gauges(lcb_SERVERMETRICS *metrics)
{
    metrics->packets_queued = 0;
    metrics->bytes_queued = 0;
}

}
