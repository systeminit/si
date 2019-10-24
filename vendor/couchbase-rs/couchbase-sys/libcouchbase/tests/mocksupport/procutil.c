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

#include "procutil.h"
#include <stdlib.h>
#include <stdio.h>

#ifndef _WIN32
#define _XOPEN_SOURCE
#include <stdlib.h>
#include <string.h>
#include <wordexp.h>
#include <fcntl.h>  /* O_* */
#include <unistd.h> /* usleep */
#include <signal.h> /* kill */
#include <time.h>
#include <errno.h>    /* ESRCH */
#include <sys/wait.h> /* waitpid */

static char **clisplit(const char *s)
{
    wordexp_t p;
    int rv;
    char **ret;
    unsigned int ii;

    memset(&p, 0, sizeof(p));
    rv = wordexp(s, &p, WRDE_NOCMD | WRDE_SHOWERR);
    if (rv != 0) {
        return NULL;
    }
    ret = malloc(sizeof(char *) * (p.we_wordc + 1));
    if (!ret) {
        return NULL;
    }
    for (ii = 0; ii < p.we_wordc; ii++) {
        ret[ii] = strdup(p.we_wordv[ii]);
    }
    ret[ii] = NULL;
    return ret;
}

static int spawn_process_impl(child_process_t *proc)
{
    int rv;
    char **argv;

    proc->pid = fork();
    if (proc->pid < 0) {
        return -1;
    }
    if (proc->pid > 0) {
        return 0;
    }

    /** In Child */
    argv = clisplit(proc->name);
    if (!argv) {
        fprintf(stderr, "Couldn't split arguments\n");
        exit(EXIT_FAILURE);
    }
    if (proc->redirect) {
        int fd = open(proc->redirect, O_RDWR | O_CREAT | O_APPEND, 0644);
        if (fd < 0) {
            perror(proc->redirect);
            exit(EXIT_FAILURE);
        }
        if (dup2(fd, fileno(stderr)) < 0 || dup2(fd, fileno(stdout)) < 0) {
            perror("dup2");
            exit(EXIT_FAILURE);
        }
        setvbuf(stderr, NULL, _IOLBF, 0);
    }
    rv = execvp(argv[0], argv);
    if (rv < 0) {
        perror(argv[0]);
        exit(EXIT_FAILURE);
    }
    abort();
    return 0; /* make things happy */
}

void kill_process(child_process_t *process, int force)
{
    int signum = SIGTERM;
    if (-1 == kill(process->pid, signum)) {
        if (errno != ESRCH && force) {
            kill(process->pid, SIGKILL);
        }
    }
}

int wait_process(child_process_t *process, int tmosec)
{
    int ec, flags = 0;
    time_t now, tmostamp;

    if (process->exited) {
        return 0;
    }
    if (tmosec < 0 || tmosec > 0) {
        flags |= WNOHANG;
    }
    now = time(NULL);

    /** Probably better logic for this */
    if (tmosec <= 0) {
        tmostamp = 0;
    } else {
        tmostamp = now + tmosec;
    }

    do {
        pid_t pidrv = waitpid(process->pid, &ec, flags);

        if (pidrv > 0) {
            if (WIFEXITED(ec)) {
                process->status = WEXITSTATUS(ec);
                process->exited = 1;

            } else if (WIFSIGNALED(ec)) {
                process->status = WTERMSIG(ec);
                process->exited = 1;

            } else if (WIFSTOPPED(ec) || WIFCONTINUED(ec)) {
                continue;

            } else {
                fprintf(stderr,
                        "Waitpid returned pid with neither EXITED or "
                        "SIGNALLED true. Assuming something else (0x%x)\n",
                        ec);
                process->status = -1;
                process->exited = 1;
            }

        } else if (pidrv == -1 && errno == ESRCH) {
            fprintf(stderr, "Process has already terminated. waitpid(%d) == ESRCH\n", process->pid);

            process->exited = 1;
        }

        if (process->exited) {
            return 0;
        }

        if (!tmostamp) {
            break;
        }
        usleep(500);
        now = time(NULL);
    } while (now < tmostamp);

    return -1;
}

void cleanup_process(child_process_t *proc)
{
    /* nothing */
    (void)proc;
}

#else
/** Windows */
static int spawn_process_impl(child_process_t *proc)
{
    BOOL success;

    if (proc->redirect) {
        HANDLE out = NULL;
        HANDLE err = NULL;
        SECURITY_ATTRIBUTES attrs;

        memset(&attrs, 0, sizeof(attrs));
        attrs.nLength = sizeof(attrs);
        attrs.bInheritHandle = TRUE;
        out = CreateFile(proc->redirect, FILE_APPEND_DATA, FILE_SHARE_WRITE | FILE_SHARE_READ, &attrs, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
        if (out == INVALID_HANDLE_VALUE) {
            fprintf(stderr, "Couldn't open '%s'. %d\n", proc->redirect, (int)GetLastError());
            return -1;
        }
        if (!DuplicateHandle(GetCurrentProcess(), out, GetCurrentProcess(), &err, 0, TRUE, DUPLICATE_SAME_ACCESS)) {
            fprintf(stderr, "Couldn't DuplicateHandle. %d\n", (int)GetLastError());
            return -1;
        }
        proc->si.cb = sizeof(proc->si);
        proc->si.hStdError = err;
        proc->si.hStdOutput = out;
        proc->si.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
        proc->si.dwFlags = STARTF_USESTDHANDLES;
    }
    success = CreateProcess(NULL,               /* name */
                            (char *)proc->name, /* commandline */
                            NULL,               /* process attributes */
                            NULL,               /* security attributes */
                            TRUE,               /* inherit handles */
                            0,                  /* creation flags */
                            NULL,               /* environment */
                            NULL,               /* current directory */
                            &proc->si,          /* STARTUPINFO */
                            &proc->pi /* PROCESS_INFORMATION */);

    if (!success) {
        fprintf(stderr, "Couldn't spawn '%s'. [%d]\n", proc->name, (int)GetLastError());
        return -1;
    }

    return 0;
}

void kill_process(child_process_t *process, int force)
{
    if (!force) {
        return; /* nothing we can do here */
    }
    TerminateProcess(process->pi.hProcess, 0);
}

int wait_process(child_process_t *process, int tmosec)
{
    DWORD millis, result;

    if (process->exited) {
        return 0;
    }
    if (tmosec < 0) {
        millis = 0;
    } else if (tmosec == 0) {
        millis = INFINITE;
    } else {
        millis = tmosec * 1000;
    }
    result = WaitForSingleObject(process->pi.hProcess, millis);
    if (result != WAIT_OBJECT_0) {
        if (result == WAIT_FAILED) {
            fprintf(stderr, "Wait failed with code [%d]\n", (int)GetLastError());
        }
        return -1;
    }
    process->exited = 1;
    if (!GetExitCodeProcess(process->pi.hProcess, &result)) {
        fprintf(stderr, "GetExitCodeProcess: %d\n", (int)GetLastError());

    } else {
        process->status = result;
    }
    return 0;
}

void cleanup_process(child_process_t *process)
{
    CloseHandle(process->pi.hProcess);
    CloseHandle(process->pi.hThread);
    if (process->redirect) {
        CloseHandle(process->si.hStdOutput);
        CloseHandle(process->si.hStdError);
    }
}

#endif

int create_process(child_process_t *proc)
{
    int rv;

    if (proc->interactive) {
        rv = system(proc->name);
        proc->status = rv;
        proc->exited = 1;
        return 0;
    }
    proc->status = -1;
    return spawn_process_impl(proc);
}
