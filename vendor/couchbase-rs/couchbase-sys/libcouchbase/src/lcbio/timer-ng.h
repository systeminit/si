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

#ifndef LCBIO_TIMER_H
#define LCBIO_TIMER_H

#ifdef __cplusplus
extern "C" {
#endif /** __cplusplus */

/**
 * @file
 * @brief Timer Routines
 *
 * This file contains the timer routines. This provides a simpler interface
 * than the one provided via the `lcb_timer_*` functions.
 */

/**
 * @ingroup lcbio
 * @defgroup lcbio-timers Timer Routines
 *
 * @details
 *
 * The timer routines here allow for an asynchronous event to be scheduled
 * within a given amount of time, or "immediately". The basic idea is that
 * these allow "Safe" invocation of routines without worrying about reentrancy
 * issues.
 *
 * There is natually a performance hit in using these functions (since this needs
 * to operate with the event loop) so this shouldn't be used for normal steady
 * state operations (such as memcached operations).
 *
 * A timer may be created via lcbio_timer_new(). The timer's initial state
 * is _unarmed_, meaning it will not be invoked until one of the scheduling
 * routines are invoked.
 *
 * When a timer is armed, its callback (passed lcbio_timer_new()) will be
 * invoked with the argument provided to lcbio_timer_new() as well.
 *
 * To schedule a timer, use the lcbio_timer_rearm() to unconditionally schedule
 * an event within a certain timeframe, or lcbio_async_signal() to invoke the
 * timer as soon as possible, once the event loop regains control.
 *
 * To cancel an armed timer (that is, to ensure the event is _not_ called), use
 * the lcbio_timer_disarm() function or the lcbio_async_cancel() function (which
 * itself is just an alias).
 *
 * Timers are not persistent, meaning that once they are fired they will enter
 * an inactive state.
 *
 * @addtogroup lcbio-timers
 * @{
 */

/** @private */
typedef enum {
    LCBIO_TIMER_S_ENTERED = 0x01,
    LCBIO_TIMER_S_DESTROYED = 0x02,
    LCBIO_TIMER_S_ARMED = 0x04
} lcbio_TIMERSTATE;

/**
 * @brief Timer callback
 * @see lcb_timer_new()
 */
typedef void (*lcbio_TIMER_cb)(void *);

typedef struct lcbio_TIMER {
    void *event;
    void *data;
    lcbio_TIMER_cb callback;
    uint32_t usec_;
    lcbio_TIMERSTATE state;
    lcbio_pTABLE io;
} lcbio_TIMER, lcbio_ASYNC;

/**
 * @brief Creates a new timer object.
 *
 * The newly created timer will be in an _unarmed_ state, but may
 * may be activated with lcbio_timer_rearm()
 *
 * @param iot
 * @param data
 * @param callback
 * @return A new timer object. Destroy with lcbio_timer_destroy()
 */
lcbio_TIMER *lcbio_timer_new(lcbio_pTABLE iot, void *data, lcbio_TIMER_cb callback);

/**
 * @brief Release the memory allocated by the timers
 * @param tm the timer to free
 */
void lcbio_timer_destroy(lcbio_TIMER *tm);

/**
 * @brief Schedule the timer invocation
 * @param timer The timer
 * @param usec The number of microseconds (from now) in which the callback
 * should be invoked
 */
void lcbio_timer_rearm(lcbio_TIMER *timer, uint32_t usec);

/**
 * @brief Cancel a pending invocation
 * @param timer The timer
 * If no pending invocation is present, this does nothing
 */
void lcbio_timer_disarm(lcbio_TIMER *timer);

/**
 * @brief Schedule an asynchronous call
 * @param timer The timer to schedule
 *
 * This function is equivalent to calling
 * @code{.c}
 * lcbio_timer_rearm(timer, 0);
 * @endcode
 */
void lcbio_async_signal(lcbio_TIMER *timer);

/**
 * @brief alias for lcbio_timer_disarm()
 * @param timer
 */
void lcbio_async_cancel(lcbio_TIMER *timer);

/**
 * @brief Check if timer is armed
 * @param timer the timer to inspect
 * @return nonzero if armed, zero if unarmed.
 */
#define lcbio_timer_armed(timer) ((timer)->state & LCBIO_TIMER_S_ARMED)

/**
 * Get the callback that is to be invoked for the timer
 * @param timer the timer to query
 * @return the current callback
 * @see lcbio_timer_set_target()
 */
#define lcbio_timer_get_target(timer) (timer)->callback

/**
 * Change the target callback for the timer
 * @param timer the timer to modify
 * @param tgt the target callback to set.
 */
#define lcbio_timer_set_target(timer, tgt) (timer)->callback = tgt

void lcbio_timer_dump(lcbio_TIMER *timer, FILE *fp);

/**@}*/

#ifdef __cplusplus
}
#endif /** __cplusplus */
#endif /* LCBIO_TIMER_H */
