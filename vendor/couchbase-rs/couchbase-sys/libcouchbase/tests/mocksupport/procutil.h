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

#ifndef LCB_PROCUTIL_H
#define LCB_PROCUTIL_H

#ifdef _WIN32
#include <windows.h>
#include <process.h>
#else
#include <sys/types.h>
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    /** Full command line to launch */
    const char *name;

    /** Name of file to which output should be redirected. Optional */
    const char *redirect;

    /** Whether this should be a foreground process (uses 'system') */
    int interactive;

    /** Exit status */
    int status;

    /** Whether the application exited*/
    int exited;

    /** PLATFORM-SPECIFIC */
#ifdef _WIN32
    STARTUPINFO si;
    PROCESS_INFORMATION pi;
#else
    pid_t pid;
#endif
} child_process_t;

/**
 * Create a new process.
 * Returns 0 on success and nonzero on failure.
 */
int create_process(child_process_t *process);

/**
 * Try to kill the process. If 'force' is specified, the process is killed
 * using more "forceful" action
 */
void kill_process(child_process_t *process, int force);

/**
 * Wait until a process has terminated.
 * If tmosec is negative, it polls without blocking,
 * if it is 0, it waits forever. If it is
 * positive, it will wait for that many seconds, polling intermittently.
 *
 * Returns 0 if the process exited. nonzero on timeout
 */
int wait_process(child_process_t *process, int tmosec);

/**
 * Cleans up any resources opened while creating the process.
 */
void cleanup_process(child_process_t *process);

#ifdef __cplusplus
}
#endif

#endif
