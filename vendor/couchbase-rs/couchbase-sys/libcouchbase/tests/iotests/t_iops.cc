/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
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

#include "config.h"
#include <gtest/gtest.h>
#include <libcouchbase/couchbase.h>
#include <lcbio/lcbio.h>
#include <lcbio/iotable.h>

typedef void (*TimerCallback)(lcb_socket_t, short, void *);

class IOPS : public ::testing::Test
{
  public:
    virtual void SetUp()
    {
        lcb_STATUS err = lcb_create_io_ops(&io, NULL);
        ASSERT_EQ(err, LCB_SUCCESS);
        iot = lcbio_table_new(io);
    }

    virtual void TearDown()
    {
        lcbio_table_unref(iot);
        if (io) {
            lcb_destroy_io_ops(io);
            io = NULL;
        }
    }

    void *createTimer()
    {
        void *ret = iot->timer.create(IOT_ARG(iot));
        EXPECT_TRUE(ret != NULL);
        return ret;
    }

    void cancelTimer(void *timer)
    {
        iot->timer.cancel(IOT_ARG(iot), timer);
    }

    void scheduleTimer(void *timer, TimerCallback cb, lcb_uint32_t us, void *arg)
    {

        iot->timer.schedule(IOT_ARG(iot), timer, us, arg, cb);
    }

    void freeTimer(void *timer)
    {
        iot->timer.destroy(IOT_ARG(iot), timer);
    }

    void startLoop()
    {
        IOT_START(iot);
    }

    void stopLoop()
    {
        IOT_STOP(iot);
    }

  protected:
    lcb_io_opt_t io;
    lcbio_pTABLE iot;
};

class Continuation
{
  public:
    virtual void nextAction() = 0;
    IOPS *parent;
};

extern "C" {
static void timer_callback(lcb_socket_t, short, void *arg)
{
    reinterpret_cast< Continuation * >(arg)->nextAction();
}
}

class TimerCountdown : public Continuation
{
  public:
    int counter;
    void *timer;

    TimerCountdown(IOPS *self)
    {
        parent = self;
        counter = 1;
        timer = parent->createTimer();
    }

    virtual void nextAction()
    {
        EXPECT_TRUE(counter > 0);
        parent->cancelTimer(timer);
        counter--;
    }

    virtual ~TimerCountdown()
    {
        parent->cancelTimer(timer);
        parent->freeTimer(timer);
    }

    void reset()
    {
        parent->cancelTimer(timer);
        parent->freeTimer(timer);
        timer = parent->createTimer();
        counter = 1;
    }

  private:
    TimerCountdown(const TimerCountdown &);
};

TEST_F(IOPS, Timers)
{
    TimerCountdown cont(this);
    scheduleTimer(cont.timer, timer_callback, 0, &cont);
    startLoop();
    ASSERT_EQ(0, cont.counter);

    std::vector< TimerCountdown * > multi;

    for (int ii = 0; ii < 10; ii++) {
        TimerCountdown *cur = new TimerCountdown(this);
        multi.push_back(cur);
        scheduleTimer(cur->timer, timer_callback, ii, cur);
    }

    startLoop();
    for (unsigned int ii = 0; ii < multi.size(); ii++) {
        TimerCountdown *cur = multi[ii];
        ASSERT_EQ(0, cur->counter);
        delete cur;
    }

    // Try it again..
    cont.reset();
    multi.clear();
    for (int ii = 0; ii < 10; ii++) {
        TimerCountdown *cur = new TimerCountdown(this);
        scheduleTimer(cur->timer, timer_callback, 10000000, cur);
        multi.push_back(cur);
    }

    scheduleTimer(cont.timer, timer_callback, 0, &cont);

    for (unsigned int ii = 0; ii < multi.size(); ii++) {
        TimerCountdown *cur = multi[ii];
        cancelTimer(cur->timer);
        cur->counter = 0;
    }

    startLoop();
    for (unsigned int ii = 0; ii < multi.size(); ii++) {
        delete multi[ii];
    }
}
