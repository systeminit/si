/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2010-2019 Couchbase, Inc.
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
#ifndef LIBCOUCHBASE_TEST_SERVER_H
#define LIBCOUCHBASE_TEST_SERVER_H 1
#define LCB_TEST_REALCLUSTER_ENV "LCB_TEST_CLUSTER_CONF"

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#define in_port_t USHORT
#ifndef usleep
#define usleep(us) Sleep((us) / 1000)
#endif
#ifndef sleep
#define sleep(s) Sleep((s)*1000)
#endif

#include <winsock2.h>
#else
#define closesocket close
#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#endif
#include <libcouchbase/couchbase.h>
#include "procutil.h"

struct test_server_info {
    child_process_t process;
    char *http;
    char *bucket;
    char *username;
    char *password;
    in_port_t port;
    struct sockaddr_storage storage;
    int sock;
    int client;
    int is_mock;
};

const void *start_test_server(char **cmdline);
const char *get_mock_http_server(const void *);
void get_mock_std_creds(const void *handle, const char **userp, const char **passp);
int is_using_real_cluster(void);

void shutdown_mock_server(const void *);

struct lcb_io_opt_st *get_test_io_opts(void);
void setup_test_timeout_handler(void);

#ifdef __cplusplus
}
#endif

#endif
