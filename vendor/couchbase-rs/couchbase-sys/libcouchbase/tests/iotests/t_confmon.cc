/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
#define LCB_BOOTSTRAP_DEFINE_STRUCT

#include "iotests.h"
#include "config.h"
#include "internal.h"
#include "bucketconfig/clconfig.h"
#include <lcbio/iotable.h>
#include <set>

using namespace lcb::clconfig;

class ConfmonTest : public ::testing::Test
{
    void SetUp()
    {
        MockEnvironment::Reset();
    }
};

struct evstop_listener : Listener {
    lcbio_pTABLE io;
    int called;

    void clconfig_lsn(EventType event, ConfigInfo *)
    {
        if (event != CLCONFIG_EVENT_GOT_NEW_CONFIG) {
            return;
        }
        called = 1;
        IOT_STOP(io);
    }

    evstop_listener() : Listener(), io(NULL), called(0) {}
};

extern "C" {
static void listen_callback1(Listener *lsn, EventType event, ConfigInfo *info) {}
}

TEST_F(ConfmonTest, testBasic)
{
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_INSTANCE *instance;
    MockEnvironment::getInstance()->createConnection(hw, &instance);

    Confmon *mon = new Confmon(instance->settings, instance->iotable, instance);
    Provider *http = mon->get_provider(CLCONFIG_HTTP);
    http->enable();
    http->configure_nodes(*instance->ht_nodes);

    mon->prepare();

    EXPECT_EQ(NULL, mon->get_config());
    mon->start();
    mon->start(); // Twice!
    mon->stop();
    mon->stop();

    // Try to find a provider..
    Provider *provider = mon->get_provider(CLCONFIG_HTTP);
    ASSERT_NE(0, provider->enabled);

    evstop_listener listener;
    listener.io = instance->iotable;
    mon->add_listener(&listener);
    mon->start();
    IOT_START(instance->iotable);
    ASSERT_NE(0, listener.called);
    delete mon;
}

struct listener2 : Listener {
    int call_count;
    lcbio_pTABLE io;
    Method last_source;
    std::set< EventType > expected_events;

    void reset()
    {
        call_count = 0;
        last_source = CLCONFIG_PHONY;
        expected_events.clear();
    }

    listener2() : Listener()
    {
        io = NULL;
        reset();
    }

    void clconfig_lsn(EventType event, ConfigInfo *info)
    {
        if (event == CLCONFIG_EVENT_MONITOR_STOPPED) {
            IOT_START(io);
            return;
        }

        if (!expected_events.empty()) {
            if (expected_events.end() == expected_events.find(event)) {
                return;
            }
        }

        call_count++;
        last_source = info->get_origin();
        IOT_STOP(io);
    }
};

static void runConfmonTest(lcbio_pTABLE io, Confmon *mon)
{
    IOT_START(io);
}

TEST_F(ConfmonTest, testCycle)
{
    HandleWrap hw;
    lcb_INSTANCE *instance;
    lcb_create_st cropts;
    MockEnvironment *mock = MockEnvironment::getInstance();

    if (mock->isRealCluster()) {
        return;
    }

    mock->createConnection(hw, &instance);
    instance->settings->bc_http_stream_time = 100000;
    instance->memd_sockpool->get_options().tmoidle = 100000;

    Confmon *mon = new Confmon(instance->settings, instance->iotable, instance);

    struct listener2 lsn;
    lsn.io = instance->iotable;
    lsn.reset();
    mon->add_listener(&lsn);

    mock->makeConnectParams(cropts, NULL);
    Provider *cccp = mon->get_provider(CLCONFIG_CCCP);
    Provider *http = mon->get_provider(CLCONFIG_HTTP);

    lcb::Hostlist hl;
    hl.add(cropts.v.v2.mchosts, 11210);
    cccp->enable(instance);
    cccp->configure_nodes(hl);

    http->enable();
    http->configure_nodes(*instance->ht_nodes);

    mon->prepare();
    mon->start();
    lsn.expected_events.insert(CLCONFIG_EVENT_GOT_NEW_CONFIG);
    runConfmonTest(lsn.io, mon);

    // Ensure CCCP is functioning properly and we're called only once.
    ASSERT_EQ(1, lsn.call_count);
    ASSERT_EQ(CLCONFIG_CCCP, lsn.last_source);

    mon->start();
    lsn.reset();
    lsn.expected_events.insert(CLCONFIG_EVENT_GOT_ANY_CONFIG);
    runConfmonTest(lsn.io, mon);
    ASSERT_EQ(1, lsn.call_count);
    ASSERT_EQ(CLCONFIG_CCCP, lsn.last_source);

    mock->setCCCP(false);
    mock->failoverNode(5);
    lsn.reset();
    mon->start();
    lsn.expected_events.insert(CLCONFIG_EVENT_GOT_ANY_CONFIG);
    lsn.expected_events.insert(CLCONFIG_EVENT_GOT_NEW_CONFIG);
    runConfmonTest(lsn.io, mon);
    ASSERT_EQ(CLCONFIG_HTTP, lsn.last_source);
    ASSERT_EQ(1, lsn.call_count);
    delete mon;
}

TEST_F(ConfmonTest, testBootstrapMethods)
{
    lcb_INSTANCE *instance;
    HandleWrap hw;
    MockEnvironment::getInstance()->createConnection(hw, &instance);
    lcb_STATUS err = lcb_connect(instance);
    ASSERT_EQ(LCB_SUCCESS, err);

    // Try the various bootstrap times
    lcb::Bootstrap *bs = instance->bs_state;
    hrtime_t last = bs->get_last_refresh(), cur = 0;

    // Reset it for the time being
    bs->reset_last_refresh();
    instance->confmon->stop();

    // Refreshing now should work
    instance->bootstrap(lcb::BS_REFRESH_THROTTLE);
    ASSERT_TRUE(instance->confmon->is_refreshing());

    cur = bs->get_last_refresh();
    ASSERT_GT(cur, 0);
    ASSERT_EQ(0, bs->get_errcounter());
    last = cur;

    // Stop it, so the state is reset
    instance->confmon->stop();
    ASSERT_FALSE(instance->confmon->is_refreshing());

    instance->bootstrap(lcb::BS_REFRESH_THROTTLE | lcb::BS_REFRESH_INCRERR);
    ASSERT_EQ(last, bs->get_last_refresh());
    ASSERT_EQ(1, bs->get_errcounter());

    // Ensure that a throttled-without-incr doesn't actually incr
    instance->bootstrap(lcb::BS_REFRESH_THROTTLE);
    ASSERT_EQ(1, bs->get_errcounter());

    // No refresh yet
    ASSERT_FALSE(instance->confmon->is_refreshing());

    instance->bootstrap(lcb::BS_REFRESH_ALWAYS);
    ASSERT_TRUE(instance->confmon->is_refreshing());
    instance->confmon->stop();
}
