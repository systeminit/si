/**
 * This file loosely based on the one from the UV project, whose copyright
 * notice reads below
 */

/* Copyright Joyent, Inc. and other Node contributors. All rights reserved.
 *
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to
 * deal in the Software without restriction, including without limitation the
 * rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE.
 */

#ifndef LCB_IOCP_DEFS_H
#define LCB_IOCP_DEFS_H

#include <winsock2.h>
#include <mswsock.h>
#include <ws2tcpip.h>
#include <windows.h>

/*
 * Guids and typedefs for winsock extension functions
 * Mingw32 doesn't have these :-(
 */

#ifndef WSAID_ACCEPTEX
# define WSAID_ACCEPTEX                                                       \
         {0xb5367df1, 0xcbac, 0x11cf,                                         \
         {0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92}}

# define WSAID_CONNECTEX                                                      \
         {0x25a207b9, 0xddf3, 0x4660,                                         \
         {0x8e, 0xe9, 0x76, 0xe5, 0x8c, 0x74, 0x06, 0x3e}}

# define WSAID_GETACCEPTEXSOCKADDRS                                           \
         {0xb5367df2, 0xcbac, 0x11cf,                                         \
         {0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92}}

# define WSAID_DISCONNECTEX                                                   \
         {0x7fda2e11, 0x8630, 0x436f,                                         \
         {0xa0, 0x31, 0xf5, 0x36, 0xa6, 0xee, 0xc1, 0x57}}

# define WSAID_TRANSMITFILE                                                   \
         {0xb5367df0, 0xcbac, 0x11cf,                                         \
         {0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92}}

typedef BOOL PASCAL(*LPFN_ACCEPTEX)(SOCKET sListenSocket,
                                    SOCKET sAcceptSocket,
                                    PVOID lpOutputBuffer,
                                    DWORD dwReceiveDataLength,
                                    DWORD dwLocalAddressLength,
                                    DWORD dwRemoteAddressLength,
                                    LPDWORD lpdwBytesReceived,
                                    LPOVERLAPPED lpOverlapped);

typedef BOOL PASCAL(*LPFN_CONNECTEX)(SOCKET s,
                                     const struct sockaddr *name,
                                     int namelen,
                                     PVOID lpSendBuffer,
                                     DWORD dwSendDataLength,
                                     LPDWORD lpdwBytesSent,
                                     LPOVERLAPPED lpOverlapped);

typedef void PASCAL(*LPFN_GETACCEPTEXSOCKADDRS)(PVOID lpOutputBuffer,
                                                DWORD dwReceiveDataLength,
                                                DWORD dwLocalAddressLength,
                                                DWORD dwRemoteAddressLength,
                                                LPSOCKADDR *LocalSockaddr,
                                                LPINT LocalSockaddrLength,
                                                LPSOCKADDR *RemoteSockaddr,
                                                LPINT RemoteSockaddrLength);

typedef BOOL PASCAL(*LPFN_DISCONNECTEX)(SOCKET hSocket,
                                        LPOVERLAPPED lpOverlapped,
                                        DWORD dwFlags,
                                        DWORD reserved);

typedef BOOL PASCAL(*LPFN_TRANSMITFILE)(SOCKET hSocket,
                                        HANDLE hFile,
                                        DWORD nNumberOfBytesToWrite,
                                        DWORD nNumberOfBytesPerSend,
                                        LPOVERLAPPED lpOverlapped,
                                        LPTRANSMIT_FILE_BUFFERS lpTransmitBuffers,
                                        DWORD dwFlags);

typedef PVOID RTL_SRWLOCK;
typedef RTL_SRWLOCK SRWLOCK, *PSRWLOCK;
#endif

typedef int (WSAAPI *LPFN_WSARECV)(SOCKET socket,
                                   LPWSABUF buffers,
                                   DWORD buffer_count,
                                   LPDWORD bytes,
                                   LPDWORD flags,
                                   LPWSAOVERLAPPED overlapped,
                                   LPWSAOVERLAPPED_COMPLETION_ROUTINE completion_routine);

typedef int (WSAAPI *LPFN_WSARECVFROM)(SOCKET socket,
                                       LPWSABUF buffers,
                                       DWORD buffer_count,
                                       LPDWORD bytes,
                                       LPDWORD flags,
                                       struct sockaddr *addr,
                                       LPINT addr_len,
                                       LPWSAOVERLAPPED overlapped,
                                       LPWSAOVERLAPPED_COMPLETION_ROUTINE completion_routine);

typedef BOOL (WINAPI *sGetQueuedCompletionStatusEx)(HANDLE CompletionPort,
                                                    LPOVERLAPPED_ENTRY lpCompletionPortEntries,
                                                    ULONG ulCount,
                                                    PULONG ulNumEntriesRemoved,
                                                    DWORD dwMilliseconds,
                                                    BOOL fAlertable);

typedef BOOL (WINAPI *sCancelIoEx)(HANDLE hFile,
                                   LPOVERLAPPED lpOverlapped);

#endif
