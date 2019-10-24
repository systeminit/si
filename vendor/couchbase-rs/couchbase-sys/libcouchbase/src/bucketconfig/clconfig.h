/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#ifndef LCB_CLCONFIG_H
#define LCB_CLCONFIG_H

#include "hostlist.h"
#include <list>
#include <lcbio/timer-ng.h>
#include <lcbio/timer-cxx.h>

/** @file */

/**
 * @defgroup lcb-confmon Cluster Configuration Management
 *
 * @brief Monitors the retrieval and application of new cluster topology maps
 * (vBucket Configurations)
 *
 * @details
 * This module attempts to implement the 'Configuration Provider' interface
 * described at https://docs.google.com/document/d/1bSMt0Sj1uQtm0OYolQaJDJg4sASfoCEwU6_gjm1he8s/edit
 *
 * The model is fairly complex though significantly more maintainable and
 * testable than the previous model. The basic idea is as follows:
 *
 *
 * <ol>
 *
 * <li>
 * There is a _Configuration Monitor_ object (Confmon) which acts
 * as the configuration supervisor. It is responsible for returning
 * configuration objects to those entities which request it.
 * </li>
 *
 * <li>
 * There are multiple _Configuration Provider_ (Provider) objects.
 * These providers aggregate configurations from multiple sources and
 * implement a common interface to:
 *
 *  <ol>
 *  <li>Return a _quick_ configuration without fetching from network or disk
 *  (see Provider::get_cached())</i>

 *  <li>Schedule a refresh to retrieve the latest configuration from the
 *  network (see Provider::refresh())</li>
 *
 *  <li>Notify the monitor that it has received a new configuration. The
 *  monitor itself will determine whether or not to accept the new
 *  configuration by examining the configuration and determining if it is more
 *  recent than the one currently in used. See lcb_confmon_set_next()</li>
 *  </ol></li>
 *
 * <li>
 * _Configuration Info_ objects. These objects are refcounted wrappers
 * around vbucket configuration handles. They have a refcount and also an
 * integer which can be used to compare with other objects for 'freshness'.
 * See ConfigInfo
 * </li>
 *
 * <li>
 * _Configuration Listeners_. These are registered with the global supervisor
 * and are invoked whenever a new valid configuration is detected. This is
 * really only ever used during bootstrap or testing where we are explicitly
 * waiting for a configuration without having any actual commands to schedule.
 * See Listener
 * </li>
 * </ol>
 */

/**
 *@addtogroup lcb-confmon
 *@{
 */

namespace lcb
{
namespace clconfig
{

/**
 * @brief Enumeration of the various config providers available.
 * The type of methods available. These are enumerated in order of preference
 */
enum Method {
    /** File-based "configcache" provider. Implemented in bc_file.c */
    CLCONFIG_FILE,
    /** New-style config-over-memcached provider. Implemented in bc_cccp.c */
    CLCONFIG_CCCP,
    /** Old-style streaming HTTP provider. Implemented in bc_http.c */
    CLCONFIG_HTTP,
    /** Raw memcached provided */
    CLCONFIG_MCRAW,
    /** Cluster administration provider. Static config with services */
    CLCONFIG_CLADMIN,

    CLCONFIG_MAX,

    /** Ephemeral source, used for tests */
    CLCONFIG_PHONY
};

/** Event types propagated to listeners */
enum EventType {
    /** Called when a new configuration is being set in confmon */
    CLCONFIG_EVENT_GOT_NEW_CONFIG,

    /** Called when _any_ configuration is received via set_enxt */
    CLCONFIG_EVENT_GOT_ANY_CONFIG,

    /** Called when all providers have been tried */
    CLCONFIG_EVENT_PROVIDERS_CYCLED,

    /** The monitor has stopped */
    CLCONFIG_EVENT_MONITOR_STOPPED
};

/** @brief Possible confmon states */
enum State {
    /** The monitor is idle and not requesting a new configuration */
    CONFMON_S_INACTIVE = 0,

    /** The monitor is actively requesting a configuration */
    CONFMON_S_ACTIVE = 1 << 0,

    /** The monitor is fetching a configuration, but is in a throttle state */
    CONFMON_S_ITERGRACE = 1 << 1
};

struct Provider;
struct Listener;
class ConfigInfo;

/**
 * This object contains the information needed for libcouchbase to deal with
 * when retrieving new configs.
 */
struct Confmon {
    /**
     * @brief Create a new configuration monitor.
     * This function creates a new `confmon` object which can be used to manage
     * configurations and their providers.
     *
     * @param settings pointer to LCB settings
     * @param iot pointer socket IO routines
     * @param instance LCB handle
     *
     * Once the confmon object has been created you may enable or disable various
     * providers (see lcb_confmon_set_provider_active()). Once no more providers
     * remain to be activated you should call lcb_confmon_prepare() once. Then
     * call the rest of the functions.
     */
    Confmon(lcb_settings *settings, lcbio_pTABLE iot, lcb_INSTANCE *instance);
    void destroy()
    {
        delete this;
    }
    ~Confmon();

    /**
     * Get the provider following the current provider, or NULL if this is
     * the last provider in the list.
     * @param cur The current provider.
     * @return The next provider, or NULL if no more providers remain.
     */
    Provider *next_active(Provider *cur);

    /**
     * Gets the first active provider.
     * @return the first provider, or NULL if no providers exist.
     */
    Provider *first_active();

    /**
     * Prepares the configuration monitor object for operations. This will insert
     * all the enabled providers into a list. Call this function each time a
     * provider has been enabled.
     */
    void prepare();

    /**
     * Set a given provider as being 'active'. This will activate the
     * provider as well as call #prepare() to update the list.
     * @param type The ID of the provider to activate
     * @param enabled true for activate, false for deactivate
     */
    void set_active(Method type, bool enabled);

    /**
     * @brief Request a configuration refresh
     *
     * Start traversing the list of current providers, requesting a new
     * configuration for each. This function will asynchronously loop through all
     * providers until one provides a new configuration.
     *
     * You may call #stop() to asynchronously break out of the loop.
     * If the confmon is already in a refreshing state
     * (i.e. #is_refreshing()) returns true then this function does
     * nothing.
     *
     * This function is reentrant safe and may be called at any time.
     *
     * @see lcb_confmon_add_listener()
     * @see #stop()
     * @see #is_refreshing()
     */
    void start(bool refresh = false);

    /**
     * @brief Cancel a pending configuration refresh.
     *
     * Stops the monitor. This will call Provider::pause() for each active
     * provider. Typically called before destruction or when a new configuration
     * has been found.
     *
     * This function is safe to call anywhere. If the monitor is already stopped
     * then this function does nothing.
     *
     * @see #start()
     * @see #is_refreshing()
     */
    void stop();

    /**
     * @brief Check if the monitor is waiting for a new config from a provider
     * @return true if refreshing, false if idle
     */
    bool is_refreshing() const
    {
        return (state & CONFMON_S_ACTIVE) != 0;
    }

    /**
     * Get the current configuration
     * @return The configuration
     */
    ConfigInfo *get_config() const
    {
        return config;
    }

    /**
     * Get the last error which occurred on this object
     * @return The last error
     */
    lcb_STATUS get_last_error() const
    {
        return last_error;
    }

    /**
     * @brief Get the current monitor state
     * @return a set of flags consisting of @ref State values.
     */
    int get_state() const
    {
        return state;
    }

    void stop_real();
    void do_next_provider();
    int do_set_next(ConfigInfo *, bool notify_miss);
    void invoke_listeners(EventType, ConfigInfo *);

    /**
     * @brief Indicate that a provider has failed and advance the monitor
     *
     * Indicate that the current provider has failed to obtain a new configuration.
     * This is always called by a provider and should be invoked when the provider
     * has encountered an internal error which caused it to be unable to fetch
     * the configuration.
     *
     * Note that this function is safe to call from any provider at any time. If
     * the provider is not the current provider then it is treated as an async
     * push notification failure and ignored. This function is _not_ safe to call
     * from consumers of providers
     *
     * Once this is called, the confmon instance will either roll over to the next
     * provider or enter the inactive state depending on the configuration and
     * whether the current provider is the last provider in the list.
     *
     * @param which reference to provider, which has been failed
     * @param why error code
     */
    void provider_failed(Provider *which, lcb_STATUS why);

    /**
     * @brief Indicate that a provider has successfuly retrieved a configuration.
     *
     * Indicates that the provider has fetched a new configuration from the network
     * and that confmon should attempt to propagate it. It has similar semantics
     * to lcb_confmon_provider_failed() except that the second argument is a config
     * object rather than an error code. The second argument must not be `NULL`
     *
     * The monitor will compare the new config against the current config.
     * If the new config does not feature any changes from the current config then
     * it is ignored and the confmon instance will proceed to the next provider.
     * This is done through a direct call to provider_failed(provider, LCB_SUCCESS).
     *
     * This function should _not_ be called outside of an asynchronous provider's
     * handler.
     *
     * @param which the provider which yielded the new configuration
     * @param config the new configuration
     */
    void provider_got_config(Provider *which, ConfigInfo *config);

    /**
     * Dump information about the monitor
     * @param fp the file to which information should be written
     */
    void dump(FILE *fp);

    Provider *get_provider(Method m) const
    {
        return all_providers[m];
    }

    /**
     * @brief Register a listener to be invoked on state changes and events
     *
     * Adds a 'listener' object to be called at each configuration update. The
     * listener may co-exist with other listeners (though it should never be added
     * twice). When a new configuration is received and accept, the listener's
     * Listener::callback field will be invoked with it.
     *
     * The callback will continue to be invoked for each new configuration received
     * until remove_listener is called. Note that the listener is not allocated
     * by the confmon and its responsibility is the user's
     *
     * @param lsn the listener. The listener's contents are not copied into
     * confmon and should thus remain valid until it is removed
     */
    void add_listener(Listener *lsn);

    /**
     * @brief Unregister (and remove) a listener added via lcb_confmon_add_listener()
     * @param lsn the listener
     */
    void remove_listener(Listener *lsn);

    /**Current provider. This provider may either fail or succeed.
     * In either case unless the provider can provide us with a specific
     * config which is newer than the one we have, it will roll over to the
     * next provider. */
    Provider *cur_provider;

    /** All providers we know about. Currently this means the 'builtin' providers */
    Provider *all_providers[CLCONFIG_MAX];

    /** The current configuration pointer. This contains the most recent accepted
     * configuration */
    ConfigInfo *config;

    typedef std::list< Listener * > ListenerList;
    /**  List of listeners for events */
    ListenerList listeners;

    lcb_settings *settings;
    lcb_STATUS last_error;
    lcbio_pTABLE iot;

    /** This is the async handle for a reentrant start */
    lcb::io::Timer< Confmon, &Confmon::do_next_provider > as_start;

    /** Async handle for a reentrant stop */
    lcb::io::Timer< Confmon, &Confmon::stop_real > as_stop;

    /* CONFMON_S_* values. Used internally */
    int state;

    /** Last time the provider was stopped. As a microsecond timestamp */
    lcb_uint64_t last_stop_us;

    typedef std::list< Provider * > ProviderList;
    ProviderList active_providers;

    lcb_INSTANCE *instance;
    size_t active_provider_list_id;
};

/**
 * The base structure of a provider. This structure is intended to be
 * 'subclassed' by implementors.
 */
struct Provider {
    Provider(Confmon *, Method type_);

    /** Destroy the resources created by this provider. */
    virtual ~Provider();

    /**
     * Get the current map known to this provider. This should not perform
     * any blocking operations. Providers which use a push model may use
     * this method as an asynchronous return value for a previously-received
     * configuration.
     */
    virtual ConfigInfo *get_cached() = 0;

    /**
     * Request a new configuration. This will be called by the manager when
     * the cached configuration (i.e. 'get_cached') is deemed invalid. Thus
     * this function should unconditionally try to schedule getting the
     * newest configuration it can. When the configuration has been received,
     * the provider may call provider_success or provider_failed.
     *
     * @note
     * The PROVIDER is responsible for terminating its own
     * process. In other words there is no safeguard within confmon itself
     * against a provider taking an excessively long time; therefore a provider
     * should implement a timeout mechanism of its choice to promptly deliver
     * a success or failure.
     */
    virtual lcb_STATUS refresh() = 0;

    /**
     * Callback invoked to the provider to indicate that it should cease
     * performing any "Active" configuration changes. Note that this is only
     * a hint and a provider may perform its own hooking based on this. In any
     * event receiving this callback is indicating that the provider will not
     * be needed again in quite some time. How long this "time" is can range
     * between 0 seconds and several minutes depending on how a user has
     * configured the client.
     *
     * @return true if actually paused
     */
    virtual bool pause()
    {
        return false;
    }

    /**
     * Called when a new configuration has been received.
     *
     * @param config the current configuration.
     * Note that this should only update the server list and do nothing
     * else.
     */
    virtual void config_updated(lcbvb_CONFIG *config)
    {
        (void)config;
    }

    /**
     * Retrieve the list of nodes from this provider, if applicable
     *
     * @return A list of nodes, or NULL if the provider does not have a list
     */
    virtual const lcb::Hostlist *get_nodes() const
    {
        return NULL;
    }

    /**
     * Call to change the configured nodes of this provider.
     *
     * @param l The list of nodes to apply
     */
    virtual void configure_nodes(const lcb::Hostlist &l)
    {
        (void)l;
    }

    /**
     * Dump state information. This callback is optional
     *
     * @param f the file to write to
     */
    virtual void dump(FILE *f) const
    {
        (void)f;
    }

    void enable()
    {
        enabled = 1;
    }

    virtual void enable(void *)
    {
        lcb_assert("Must be implemented in subclass if used" && 0);
    }

    /** The type of provider */
    const Method type;

    /** Whether this provider has been disabled/enabled explicitly by a user */
    bool enabled;

    /** The parent manager object */
    Confmon *parent;

    lcb_settings &settings() const
    {
        return *parent->settings;
    }
};

Provider *new_cccp_provider(Confmon *);
Provider *new_file_provider(Confmon *);
Provider *new_http_provider(Confmon *);
Provider *new_mcraw_provider(Confmon *);
Provider *new_cladmin_provider(Confmon *);

/** @brief refcounted object encapsulating a vbucket config */
class ConfigInfo
{
  public:
    /**
     * Creates a new configuration wrapper object containing the vbucket config
     * pointed to by 'config'. Its initial refcount will be set to 1.
     *
     * @param vbc a newly parsed configuration
     * @param origin the type of provider from which the config originated.
     * @return a new ConfigInfo object. This should be incref()'d/decref()'d
     * as needed.
     */
    static ConfigInfo *create(lcbvb_CONFIG *vbc, Method origin)
    {
        return new ConfigInfo(vbc, origin);
    }
    /**
     * @brief Compares two info structures and determine which one is newer
     *
     * This function returns an integer less than
     * zero, zero or greater than zero if the first argument is considered older
     * than, equal to, or later than the second argument.
     *
     * @param config anoother config
     * @see lcbvb_get_revision
     * @see ConfigInfo::cmpclock
     */
    int compare(const ConfigInfo &config);

    /**
     * @brief Increment the refcount on a config object
     */
    void incref()
    {
        refcount++;
    }

    /**
     * @brief Decrement the refcount on a config object.
     * Decrement the refcount. If the internal refcount reaches 0 then the internal
     * members (including the vbucket config handle itself) will be freed.
     */
    void decref()
    {
        if (!--refcount) {
            delete this;
        }
    }

    operator lcbvb_CONFIG *() const
    {
        return vbc;
    }

    Method get_origin() const
    {
        return origin;
    }

    /** Actual configuration */
    lcbvb_CONFIG *vbc;

  private:
    ConfigInfo(lcbvb_CONFIG *vbc, Method origin);

    ~ConfigInfo();

    /** Comparative clock with which to compare */
    uint64_t cmpclock;

    /** Reference counter */
    unsigned int refcount;

    /** Origin provider type which produced this config */
    Method origin;
};

/**
 * @brief Listener for events
 * One or more listeners may be installed into the confmon which will have
 * a callback invoked on significant vbucket events. See clconfig_event_t
 * for a variety of events the listener can know.
 */
struct Listener {
    virtual ~Listener() {}

    /** Linked list node */
    lcb_list_t ll;

    /**
     * Callback invoked for significant events
     *
     * @param event the event which took place
     * @param config the configuration associated with the event. Note that
     * `config` may also be NULL
     */
    virtual void clconfig_lsn(EventType event, ConfigInfo *config) = 0;
};

/* Method-specific setup methods.. */

/**
 * @name File Provider-specific APIs
 * @{
 */

/**
 * Sets the input/output filename for the file provider. This also enables
 * the file provider.
 * @param p the provider
 * @param f the filename (if NULL, a temporary filename will be created)
 * @param ro whether the client will never modify the file
 * @return true on success, false on failure.
 */
bool file_set_filename(Provider *p, const char *f, bool ro);

/**
 * Retrieve the filename for the provider
 * @param p The provider of type CLCONFIG_FILE
 * @return the current filename being used.
 */
const char *file_get_filename(Provider *p);
void file_set_readonly(Provider *p, bool val);
/**@}*/

/**
 * @name HTTP Provider-specific APIs
 * @{
 */

/**
 * Get the socket representing the current REST connection to the cluster
 * (if applicable)
 * @param p The provider of type CLCONFIG_HTTP
 * @return
 */
const lcbio_SOCKET *http_get_conn(const Provider *p);

static inline const lcbio_SOCKET *http_get_conn(Confmon *c)
{
    return http_get_conn(c->get_provider(CLCONFIG_HTTP));
}

/**
 * Get the hostname for the current REST connection to the cluster
 * @param p The provider of type CLCONFIG_HTTP
 * @return
 */
const lcb_host_t *http_get_host(const Provider *p);
static inline const lcb_host_t *http_get_host(Confmon *c)
{
    return http_get_host(c->get_provider(CLCONFIG_HTTP));
}
/**@}*/

/**
 * @name CCCP Provider-specific APIs
 * @{
 */

/**
 * Note, to initialize the CCCP provider, you should use
 * cccp->enable(instance);
 */

/**
 * @brief Notify the CCCP provider about a new configuration from a
 * `NOT_MY_VBUCKET` response
 *
 * This should be called by the packet handler when a configuration has been
 * received as a payload to a response with the error of `NOT_MY_VBUCKET`.
 *
 * @param provider The CCCP provider
 * @param host The hostname (without the port) on which the packet was received
 * @param data The configuration JSON blob
 * @return LCB_SUCCESS, or an error code if the configuration could not be
 * set
 */
lcb_STATUS cccp_update(Provider *provider, const char *host, const char *data);

/**
 * @brief Notify the CCCP provider about a configuration received from a
 * `CMD_GET_CLUSTER_CONFIG` response.
 *
 * @param cookie The cookie object attached to the packet
 * @param err The error code for the reply
 * @param bytes The payload pointer
 * @param nbytes Size of payload
 * @param origin Host object from which the packet was received
 */
void cccp_update(const void *cookie, lcb_STATUS err, const void *bytes, size_t nbytes, const lcb_host_t *origin);

/**
 * @brief record status of SELECT_BUCKET command
 * @param cookie_
 * @param err
 */
void select_status(const void *cookie_, lcb_STATUS err);

/**@}*/

/**@name Raw Memcached (MCRAW) Provider-specific APIs
 * @{*/
/**@}*/
/**@}*/

} // namespace clconfig
} // namespace lcb
#endif /* LCB_CLCONFIG_H */
