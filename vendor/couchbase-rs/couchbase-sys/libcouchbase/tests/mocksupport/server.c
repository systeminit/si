/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2011-2019 Couchbase, Inc.
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
#include "server.h"
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <ctype.h>

#ifndef _WIN32
#define DIRSEP "/"
#include <pthread.h>
#include <sys/wait.h>
#include <arpa/inet.h>
#include <netinet/tcp.h>
#include <netinet/in.h>
#include <signal.h>
#ifdef linux
#undef ntohs
#undef ntohl
#undef htons
#undef htonl
#endif
#else
#define DIRSEP "\\"
#include <io.h> /* for access() */
#endif          /* ! _WIN32 */

static int create_monitor(struct test_server_info *info)
{
    struct addrinfo hints, *next, *ai;
    int error;

    memset(&hints, 0, sizeof(hints));
    hints.ai_flags = AI_PASSIVE;
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    info->sock = -1;
    error = getaddrinfo(NULL, "0", &hints, &ai);
    if (error != 0) {
#ifdef _WIN32
        if (0) {
#else
        if (error != EAI_SYSTEM) {
#endif
            fprintf(stderr, "getaddrinfo failed: %s\n", gai_strerror(error));
        } else {
            perror("getaddrinfo failed:");
        }
        return 0;
    }

    for (next = ai; next; next = next->ai_next) {
        int flags = 1;
        socklen_t len;

        if ((info->sock = socket(next->ai_family, next->ai_socktype, next->ai_protocol)) == -1) {
            continue;
        }

        setsockopt(info->sock, SOL_SOCKET, SO_REUSEADDR, (void *)&flags, sizeof(flags));

        if (bind(info->sock, next->ai_addr, next->ai_addrlen) == -1) {
            closesocket(info->sock);
            info->sock = -1;
            continue;
        } else if (listen(info->sock, 10) == -1) {
            closesocket(info->sock);
            info->sock = -1;
            continue;
        }

        /* Ok, I've got a working socket :) */
        len = sizeof(info->storage);
        if (getsockname(info->sock, (struct sockaddr *)&info->storage, &len) == -1) {
            closesocket(info->sock);
            info->sock = -1;
            continue;
        }
        if (next->ai_addr->sa_family == AF_INET) {
            info->port = ntohs((*(struct sockaddr_in *)&info->storage).sin_port);
        } else {
            info->port = ntohs((*(struct sockaddr_in6 *)&info->storage).sin6_port);
        }
    }

    freeaddrinfo(ai);
    return info->sock != -1;
}

static void wait_for_server(const char *port)
{
    struct addrinfo hints, *next, *ai;
    int sock = -1;
    int error;

    memset(&hints, 0, sizeof(hints));
    hints.ai_flags = AI_PASSIVE;
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    error = getaddrinfo("localhost", port, &hints, &ai);
    if (error != 0) {
#ifndef _WIN32
        if (error != EAI_SYSTEM) {
#else
        if (0) {
#endif
            fprintf(stderr, "getaddrinfo failed: %s\n", gai_strerror(error));
        } else {
            perror("getaddrinfo failed:");
        }
        abort();
    }

    while (1) {
        for (next = ai; next; next = next->ai_next) {
            if ((sock = socket(next->ai_family, next->ai_socktype, next->ai_protocol)) == -1) {
                continue;
            }

            if (connect(sock, next->ai_addr, next->ai_addrlen) == 0) {
                closesocket(sock);
                freeaddrinfo(ai);
                return;
            }

            closesocket(sock);
        }
        usleep(250);
    }
}

/**
 * Parse server parameters from environment;
 * format is host,bucket,username,password
 */
static int parse_server_conf(struct test_server_info *info, const char *param)
{
    char *strings[10] = {NULL};
    int curix = 0;
    char *param_copy = strdup(param);
    param = param_copy;

    while (*param && curix < 10) {
        const char *curfld;
        char *curval;
        size_t diff;

        for (curfld = param; *param && *param != ','; param++)
            ;
        diff = (param - curfld);
        curval = calloc(1, diff + 1);
        curval[diff] = '\0';
        memcpy(curval, curfld, diff);
        strings[curix++] = curval;
        if (*param == ',') {
            param++;
        }
    }

    info->http = strings[0];
    info->bucket = strings[1];
    info->username = strings[2];
    info->password = strings[3];

    free(param_copy);

    if (!info->http) {
        fprintf(stderr, "Must have node entry point for real cluster test\n");
        return 0;
    }
    return 1;
}

static int start_mock_process(struct test_server_info *info, char **argv)
{
    char argbuf[4096] = {0};
    char **arg;
    for (arg = argv; *arg; arg++) {
        strcat(argbuf, *arg);
        strcat(argbuf, " ");
    }

    memset(&info->process, 0, sizeof(info->process));
    info->process.name = argbuf;

    return create_process(&info->process);
}

static void kill_mock_process(struct test_server_info *info)
{
    kill_process(&info->process, 1);
    wait_process(&info->process, 1);
    cleanup_process(&info->process);
}

#ifndef _WIN32
#define WRAPPER_BASE "start_mock.sh"
#else
#define WRAPPER_BASE "start_mock.bat"
#endif

static void negotiate_mock_connection(struct test_server_info *info)
{
    char buffer[1024];
    lcb_ssize_t offset;
    lcb_ssize_t nr;
    int ii;

    /* wait until the server connects */
    for (ii = 0; ii < 10; ii++) {
        info->client = accept(info->sock, NULL, NULL);
        if (info->client == -1) {
            /* running this in gdb on OS X, I got an EINTR a few times */
            if (errno == EINTR) {
                fprintf(stderr, "start_mock_server: Sleeping 1 second on EINTR\n");
                sleep(1);
            } else {
                perror("start_mock_server");
                abort();
            }
        } else {
            break;
        }
    }

    lcb_assert(info->client != -1);
    /* Get the port number of the http server */
    offset = snprintf(buffer, sizeof(buffer), "localhost:");
    nr = recv(info->client, buffer + offset, sizeof(buffer) - (size_t)offset - 1, 0);
    lcb_assert(nr > 0);
    buffer[nr + offset] = '\0';
    info->http = strdup(buffer);
    wait_for_server(buffer + offset);
}

static int start_mock_server(struct test_server_info *info, char **cmdline)
{

    char wrapper[1024];
    char monitor[1024];
    char *argv[1024];
#ifdef _WIN32
    int access_mode = 00;
#else
    int access_mode = X_OK;
#endif
    const char *srcdir = getenv("srcdir");

    if (srcdir == NULL) {
        srcdir = ".";
    }

    snprintf(wrapper, sizeof(wrapper), "%s" DIRSEP "tests" DIRSEP WRAPPER_BASE, srcdir);

    if (access(wrapper, access_mode) == -1) {
        fprintf(stderr, "Failed to locate \"%s\": %s\n", wrapper, strerror(errno));
        return 0;
    }

    if (!create_monitor(info)) {
        return 0;
    }

    {
        int arg = 0;
        argv[arg++] = (char *)wrapper;
        sprintf(monitor, "--harakiri-monitor=localhost:%d", info->port);
        argv[arg++] = monitor;

        if (cmdline != NULL) {
            int ii = 0;
            while (cmdline[ii] != NULL && arg < 1022) {
                argv[arg++] = cmdline[ii++];
            }
        }
        argv[arg++] = NULL;
    }

    start_mock_process(info, argv);
    negotiate_mock_connection(info);
    sleep(1); /* give it a bit time to initialize itself */
    return 1;
}

const void *start_test_server(char **cmdline)
{
    const char *clconf = getenv(LCB_TEST_REALCLUSTER_ENV);
    int server_ok = 0;
    struct test_server_info *info = calloc(1, sizeof(*info));

#ifdef _WIN32
    /** Winsock boilerplate */
    {
        WSADATA wsaData;
        if (WSAStartup(MAKEWORD(2, 0), &wsaData) != 0) {
            fprintf(stderr, "WSAStartup failed. Abort\n");
            abort();
        }
    }
#endif

    if (info == NULL) {
        return NULL;
    }

    if (clconf) {
        server_ok = parse_server_conf(info, clconf);
        info->is_mock = 0;
    } else {
        server_ok = start_mock_server(info, cmdline);
        info->is_mock = 1;
    }

    if (!server_ok) {
        fprintf(stderr, "Couldn't setup server!\n");
        abort();
    }

    return info;
}

void shutdown_mock_server(const void *handle)
{
    struct test_server_info *info = (void *)handle;
    if (info != NULL) {
        free(info->http);
        free(info->bucket);
        free(info->username);
        free(info->password);
        if (info->is_mock) {
            closesocket(info->client);
            closesocket(info->sock);
            kill_mock_process(info);
        }
        free((void *)handle);
    }
}

const char *get_mock_http_server(const void *handle)
{
    struct test_server_info *info = (void *)handle;
    return info->http;
}

void get_mock_std_creds(const void *handle, const char **userp, const char **passp)
{
    const struct test_server_info *info = handle;
    if (info->is_mock) {
        *userp = NULL;
        *passp = NULL;
    } else {
        *userp = info->username;
        *passp = info->password;
    }
}

int is_using_real_cluster(void)
{
    return getenv(LCB_TEST_REALCLUSTER_ENV) != NULL;
}
