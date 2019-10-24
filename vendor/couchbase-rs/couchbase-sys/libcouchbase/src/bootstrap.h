/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#ifndef LCB_BOOTSTRAP_H
#define LCB_BOOTSTRAP_H

/**@file
 * Core bootstrap/cluster configuration routines */

/**@defgroup lcb_bootstrap Bootstrap Routines
 * @addtogroup lcb_bootstrap
 * @{
 */

#ifdef __cplusplus
#include "bucketconfig/clconfig.h"
#include <lcbio/timer-cxx.h>

namespace lcb
{
/**
 * Structure containing the bootstrap state for the instance.
 *
 * Derived from Listener,
 * used to react when a new configuration is received. This
 * is used for both requested configurations (i.e. an explicit call to
 * lcb_bootstrap_common()) as well as unsolicited updates such as
 * HTTP streaming configurations or Not-My-Vbucket "Carrier" updates.
 */
class Bootstrap : lcb::clconfig::Listener
{
  public:
    Bootstrap(lcb_INSTANCE *);
    ~Bootstrap();
    lcb_STATUS bootstrap(unsigned options);

    hrtime_t get_last_refresh() const
    {
        return last_refresh;
    }
    void reset_last_refresh()
    {
        last_refresh = 0;
    }
    size_t get_errcounter() const
    {
        return errcounter;
    }

    /**
     * Try to start/stop background polling depending on whether we're able to.
     */
    void check_bgpoll();

  private:
    // Override
    void clconfig_lsn(lcb::clconfig::EventType e, lcb::clconfig::ConfigInfo *i);

    inline void config_callback(lcb::clconfig::EventType, lcb::clconfig::ConfigInfo *);
    inline void initial_error(lcb_STATUS, const char *);
    void timer_dispatch();
    void bgpoll();

    lcb_INSTANCE *parent;

    /**Timer used for initial bootstrap as an interval timer, and for subsequent
     * updates as an asynchronous event (to allow safe updates and avoid
     * reentrancy issues)
     */
    lcb::io::Timer< Bootstrap, &Bootstrap::timer_dispatch > tm;

    /**Timer used for periodic polling of config */
    lcb::io::Timer< Bootstrap, &Bootstrap::bgpoll > tmpoll;

    /**
     * Timestamp indicating the most recent configuration activity. This
     * timestamp is used to control throttling, such that the @ref
     * LCB_CNTL_CONFDELAY_THRESH setting is applied as an offset to this
     * timestamp (accounting for ns-to-us conversion). This flag is set whenever
     *
     * * A new configuration is received (solicited or unsolicited)
     * * A request for a new configuration is made, and the request has not
     *   been throttled
     */
    hrtime_t last_refresh;

    /**
     * Counter incremented each time a request is based to lcb_bootstrap_common()
     * with the @ref LCB_BS_REFRESH_INCRERR flag, and where the request itself
     * had been throttled. This increments the internal error counter and when
     * the counter reaches a threshold higher than @ref LCB_CNTL_CONFERRTHRESH
     * a new configuration is requested.
     * This counter is cleared whenever a new configuration arrives.
     */
    unsigned errcounter;

    enum State {
        /** Initial 'blank' state */
        S_INITIAL_PRE = 0,
        /** We got something after our initial callback */
        S_INITIAL_TRIGGERED,
        /** Have received at least one valid configuration */
        S_BOOTSTRAPPED
    };
    State state;
};

/**
 * These flags control the bootstrap refreshing mode that will take place
 * when lcb_bootstrap_common() is invoked. These options may be OR'd with
 * each other (with the exception of ::LCB_BS_REFRESH_ALWAYS).
 */
enum BootstrapOptions {
    /** Always fetch a new configuration. No throttling checks are performed */
    BS_REFRESH_ALWAYS = 0x00,
    /** Special mode used to fetch the first configuration */
    BS_REFRESH_INITIAL = 0x02,

    /** Make the request for a new configuration subject to throttling
     * limitations. Currently this will be subject to the interval specified
     * in the @ref LCB_CNTL_CONFDELAY_THRESH setting and the @ref
     * LCB_CNTL_CONFERRTHRESH setting. If the refresh has been throttled
     * the lcb_confmon_is_refreshing() function will return false */
    BS_REFRESH_THROTTLE = 0x04,

    /** To be used in conjunction with ::LCB_BS_REFRESH_THROTTLE, this will
     * increment the error counter in case the current refresh is throttled,
     * such that when the error counter reaches the threshold, the throttle
     * limitations will expire and a new refresh will take place */
    BS_REFRESH_INCRERR = 0x08,

    BS_REFRESH_OPEN_BUCKET = 0x10
};

void lcb_bootstrap_destroy(lcb_INSTANCE *instance);

/**@}*/

} // namespace lcb
#endif // __cplusplus
#endif /* LCB_BOOTSTRAP_H */
