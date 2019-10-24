/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 * Copyright (c) <2008>, Sun Microsystems, Inc.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in the
 *       documentation and/or other materials provided with the distribution.
 *     * Neither the name of the  nor the
 *       names of its contributors may be used to endorse or promote products
 *       derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY SUN MICROSYSTEMS, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL SUN MICROSYSTEMS, INC. BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
/*
 * Summary: Constants used by to implement the binary protocol.
 *
 * Copy: See Copyright for the status of this software.
 *
 * Author: Trond Norbye <trond.norbye@sun.com>
 */

/**
 * THIS IS A SLIMMED DOWN VERSION FOR LIBCOUCHBASE!
 * It only contains constants used by the library. The header in the
 * memcached source code contains all of the commands actually used.
 */

#ifndef PROTOCOL_BINARY_H
#define PROTOCOL_BINARY_H

#if !defined HAVE_STDINT_H && defined _WIN32 && defined(_MSC_VER)
#include "win_stdint.h"
#else
#include <stdint.h>
#endif

/**
 * \addtogroup Protocol
 * @{
 */

/**
 * This file contains definitions of the constants and packet formats
 * defined in the binary specification. Please note that you _MUST_ remember
 * to convert each multibyte field to / from network byte order to / from
 * host order.
 */
#ifdef __cplusplus
extern "C" {
#endif

/**
 * Definition of the legal "magic" values used in a packet.
 * See section 3.1 Magic byte
 */
typedef enum {
    /* Request packet from client to server containing frame extras */
    PROTOCOL_BINARY_AREQ = 0x08,
    /* Response packet from server to client containing frame extras */
    PROTOCOL_BINARY_ARES = 0x18,
    /* Request packet from client to server */
    PROTOCOL_BINARY_REQ = 0x80,
    /* Response packet from server to client */
    PROTOCOL_BINARY_RES = 0x81,
    /* Request packet from server to client */
    PROTOCOL_BINARY_SREQ = 0x82,
    /* Response packet from client to server */
    PROTOCOL_BINARY_SRES = 0x83
} protocol_binary_magic;

/**
 * Definition of the valid response status numbers.
 *
 * A well written client should be "future proof" by handling new
 * error codes to be defined. Note that new error codes means that
 * the requested operation wasn't performed.
 */
typedef enum {
    /** The operation completed successfully */
    PROTOCOL_BINARY_RESPONSE_SUCCESS = 0x00,
    /** The key does not exists */
    PROTOCOL_BINARY_RESPONSE_KEY_ENOENT = 0x01,
    /** The key exists in the cluster (with another CAS value) */
    PROTOCOL_BINARY_RESPONSE_KEY_EEXISTS = 0x02,
    /** The document exceeds the maximum size */
    PROTOCOL_BINARY_RESPONSE_E2BIG = 0x03,
    /** Invalid request */
    PROTOCOL_BINARY_RESPONSE_EINVAL = 0x04,
    /** The document was not stored for some reason. This is
     * currently a "catch all" for number or error situations, and
     * should be split into multiple error codes. */
    PROTOCOL_BINARY_RESPONSE_NOT_STORED = 0x05,
    /** Non-numeric server-side value for incr or decr */
    PROTOCOL_BINARY_RESPONSE_DELTA_BADVAL = 0x06,
    /** The server is not responsible for the requested vbucket */
    PROTOCOL_BINARY_RESPONSE_NOT_MY_VBUCKET = 0x07,
    /** Not connected to a bucket */
    PROTOCOL_BINARY_RESPONSE_NO_BUCKET = 0x08,
    /** The requested resource is locked */
    PROTOCOL_BINARY_RESPONSE_LOCKED = 0x09,

    /** The authentication context is stale. You should reauthenticate*/
    PROTOCOL_BINARY_RESPONSE_AUTH_STALE = 0x1f,
    /** Authentication failure (invalid user/password combination,
     * OR an internal error in the authentication library. Could
     * be a misconfigured SASL configuration. See server logs for
     * more information.) */
    PROTOCOL_BINARY_RESPONSE_AUTH_ERROR = 0x20,
    /** Authentication OK so far, please continue */
    PROTOCOL_BINARY_RESPONSE_AUTH_CONTINUE = 0x21,
    /** The requested value is outside the legal range
     * (similar to EINVAL, but more specific) */
    PROTOCOL_BINARY_RESPONSE_ERANGE = 0x22,

    /** No access (could be opcode, value, bucket etc) */
    PROTOCOL_BINARY_RESPONSE_EACCESS = 0x24,
    /** The Couchbase cluster is currently initializing this
     * node, and the Cluster manager has not yet granted all
     * users access to the cluster. */
    PROTOCOL_BINARY_RESPONSE_NOT_INITIALIZED = 0x25,

    /** The server have no idea what this command is for */
    PROTOCOL_BINARY_RESPONSE_UNKNOWN_COMMAND = 0x81,
    /** Not enough memory */
    PROTOCOL_BINARY_RESPONSE_ENOMEM = 0x82,
    /** The server does not support this command */
    PROTOCOL_BINARY_RESPONSE_NOT_SUPPORTED = 0x83,
    /** An internal error in the server */
    PROTOCOL_BINARY_RESPONSE_EINTERNAL = 0x84,
    /** The system is currently too busy to handle the request.
     * it is _currently_ only being used by the scrubber in
     * default_engine to run a task there may only be one of
     * (subsequent requests to start it would return ebusy until
     * it's done). */
    PROTOCOL_BINARY_RESPONSE_EBUSY = 0x85,
    /** A temporary error condition occurred. Retrying the
     * operation may resolve the problem. This could be that the
     * server is in a degraded situation (like running warmup on
     * the node), the vbucket could be in an "incorrect" state, a
     * temporary failure from the underlying persistence layer,
     * etc).
     */
    PROTOCOL_BINARY_RESPONSE_ETMPFAIL = 0x86,
    /**
     * There is something wrong with the syntax of the provided
     * XATTR.
     */
    PROTOCOL_BINARY_RESPONSE_XATTR_EINVAL = 0x87,

    /**
     * Operation attempted with an unknown collection.
     */
    PROTOCOL_BINARY_RESPONSE_UNKNOWN_COLLECTION = 0x88,
    /**
     * Operation attempted and requires that the collections manifest is set.
     */
    PROTOCOL_BINARY_RESPONSE_NO_COLLECTIONS_MANIFEST = 0x89,
    /**
     * Bucket Manifest update could not be applied to vbucket(s)
     */
    PROTOCOL_BINARY_RESPONSE_CANNOT_APPLY_COLLECTIONS_MANIFEST = 0x8a,
    /**
     * Client has a collection's manifest which is from the future. This means
     * they have a uid that is greater than ours.
     */
    PROTOCOL_BINARY_RESPONSE_COLLECTIONS_MANIFEST_IS_AHEAD = 0x8b,

    /**
     * Operation attempted with an unknown scope.
     */
    PROTOCOL_BINARY_RESPONSE_UNKNOWN_SCOPE = 0x8c,

    PROTOCOL_BINARY_RESPONSE_DURABILITY_INVALID_LEVEL = 0xa0,
    PROTOCOL_BINARY_RESPONSE_DURABILITY_IMPOSSIBLE = 0xa1,
    PROTOCOL_BINARY_RESPONSE_SYNC_WRITE_IN_PROGRESS = 0xa2,
    PROTOCOL_BINARY_RESPONSE_SYNC_WRITE_AMBIGUOUS = 0xa3,

    /*
     * Sub-document specific responses.
     */

    /** The provided path does not exist in the document. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_ENOENT = 0xc0,

    /** One of path components treats a non-dictionary as a dictionary, or
     * a non-array as an array.
     * [Arithmetic operations only] The value the path points to is not
     * a number. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_MISMATCH = 0xc1,

    /** The path’s syntax was incorrect. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_EINVAL = 0xc2,

    /** The path provided is too large; either the string is too long,
     * or it contains too many components. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_E2BIG = 0xc3,

    /** The document has too many levels to parse. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_DOC_E2DEEP = 0xc4,

    /** [For mutations only] The value provided will invalidate the JSON if
     * inserted. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_VALUE_CANTINSERT = 0xc5,

    /** The existing document is not valid JSON. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_DOC_NOTJSON = 0xc6,

    /** [For arithmetic ops] The existing number is out of the valid range
     * for arithmetic ops (cannot be represented as an int64_t). */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_NUM_ERANGE = 0xc7,

    /** [For arithmetic ops] The operation would result in a number
     * outside the valid range (cannot be represented as an int64_t). */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_DELTA_ERANGE = 0xc8,

    /** [For mutations only] The requested operation requires the path to
     * not already exist, but it exists. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_PATH_EEXISTS = 0xc9,

    /** [For mutations only] Inserting the value would cause the document
     * to be too deep. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_VALUE_ETOODEEP = 0xca,

    /** [For multi-path commands only] An invalid combination of commands
     * was specified. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_INVALID_COMBO = 0xcb,

    /** [For multi-path commands only] Specified key was successfully
     * found, but one or more path operations failed. Examine the individual
     * lookup_result (MULTI_LOOKUP) / mutation_result (MULTI_MUTATION)
     * structures for details. */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_MULTI_PATH_FAILURE = 0xcc,

    /**
     * The operation completed successfully, but operated on a deleted
     * document.
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_SUCCESS_DELETED = 0xcd,

    /**
     * The combination of the subdoc flags for the xattrs doesn't make
     * any sense
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_INVALID_FLAG_COMBO = 0xce,

    /**
     * Only a single xattr key may be accessed at the same time.
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_INVALID_KEY_COMBO = 0xcf,

    /**
     * The server has no knowledge of the requested macro
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_UNKNOWN_MACRO = 0xd0,

    /**
     * The server has no knowledge of the requested virtual xattr
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_UNKNOWN_VATTR = 0xd1,

    /**
     * Virtual xattrs can't be modified
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_XATTR_CANT_MODIFY_VATTR = 0xd2,

    /**
     * [For multi-path commands only] Specified key was found as a
     * Deleted document, but one or more path operations
     * failed. Examine the individual lookup_result (MULTI_LOOKUP) /
     * mutation_result (MULTI_MUTATION) structures for details.
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_MULTI_PATH_FAILURE_DELETED = 0xd3,

    /**
     * According to the spec all xattr commands should come first,
     * followed by the commands for the document body
     */
    PROTOCOL_BINARY_RESPONSE_SUBDOC_INVALID_XATTR_ORDER = 0xd4
} protocol_binary_response_status;

/**
 * Defintion of the different command opcodes.
 * See section 3.3 Command Opcodes
 */
typedef enum {
    PROTOCOL_BINARY_CMD_GET = 0x00,
    PROTOCOL_BINARY_CMD_SET = 0x01,
    PROTOCOL_BINARY_CMD_ADD = 0x02,
    PROTOCOL_BINARY_CMD_REPLACE = 0x03,
    PROTOCOL_BINARY_CMD_DELETE = 0x04,
    PROTOCOL_BINARY_CMD_INCREMENT = 0x05,
    PROTOCOL_BINARY_CMD_DECREMENT = 0x06,
    PROTOCOL_BINARY_CMD_FLUSH = 0x08,
    PROTOCOL_BINARY_CMD_GETQ = 0x09 /* Used in tests */,
    PROTOCOL_BINARY_CMD_NOOP = 0x0a,
    PROTOCOL_BINARY_CMD_VERSION = 0x0b,
    PROTOCOL_BINARY_CMD_APPEND = 0x0e,
    PROTOCOL_BINARY_CMD_PREPEND = 0x0f,
    PROTOCOL_BINARY_CMD_STAT = 0x10,
    PROTOCOL_BINARY_CMD_VERBOSITY = 0x1b,
    PROTOCOL_BINARY_CMD_TOUCH = 0x1c,
    PROTOCOL_BINARY_CMD_GAT = 0x1d,
    PROTOCOL_BINARY_CMD_HELLO = 0x1f,

    PROTOCOL_BINARY_CMD_SASL_LIST_MECHS = 0x20,
    PROTOCOL_BINARY_CMD_SASL_AUTH = 0x21,
    PROTOCOL_BINARY_CMD_SASL_STEP = 0x22,

    PROTOCOL_BINARY_CMD_GET_REPLICA = 0x83,

    PROTOCOL_BINARY_CMD_SELECT_BUCKET = 0x89,

    PROTOCOL_BINARY_CMD_OBSERVE_SEQNO = 0x91,
    PROTOCOL_BINARY_CMD_OBSERVE = 0x92,

    PROTOCOL_BINARY_CMD_GET_LOCKED = 0x94,
    PROTOCOL_BINARY_CMD_UNLOCK_KEY = 0x95,

    PROTOCOL_BINARY_CMD_GET_CLUSTER_CONFIG = 0xb5,

    PROTOCOL_BINARY_CMD_COLLECTIONS_SET_MANIFEST = 0xb9,
    PROTOCOL_BINARY_CMD_COLLECTIONS_GET_MANIFEST = 0xba,
    PROTOCOL_BINARY_CMD_COLLECTIONS_GET_CID = 0xbb,

    /**
     * Commands for the Sub-document API.
     */

    /* Retrieval commands */
    PROTOCOL_BINARY_CMD_SUBDOC_GET = 0xc5,
    PROTOCOL_BINARY_CMD_SUBDOC_EXISTS = 0xc6,

    /* Dictionary commands */
    PROTOCOL_BINARY_CMD_SUBDOC_DICT_ADD = 0xc7,
    PROTOCOL_BINARY_CMD_SUBDOC_DICT_UPSERT = 0xc8,

    /* Generic modification commands */
    PROTOCOL_BINARY_CMD_SUBDOC_DELETE = 0xc9,
    PROTOCOL_BINARY_CMD_SUBDOC_REPLACE = 0xca,

    /* Array commands */
    PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_LAST = 0xcb,
    PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_PUSH_FIRST = 0xcc,
    PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_INSERT = 0xcd,
    PROTOCOL_BINARY_CMD_SUBDOC_ARRAY_ADD_UNIQUE = 0xce,

    /* Arithmetic commands */
    PROTOCOL_BINARY_CMD_SUBDOC_COUNTER = 0xcf,

    /* Multi-Path commands */
    PROTOCOL_BINARY_CMD_SUBDOC_MULTI_LOOKUP = 0xd0,
    PROTOCOL_BINARY_CMD_SUBDOC_MULTI_MUTATION = 0xd1,

    /* Subdoc additions for Spock: */
    PROTOCOL_BINARY_CMD_SUBDOC_GET_COUNT = 0xd2,

    /* get error code mappings */
    PROTOCOL_BINARY_CMD_GET_ERROR_MAP = 0xfe,

    /* Reserved for being able to signal invalid opcode */
    PROTOCOL_BINARY_CMD_INVALID = 0xff
} protocol_binary_command;

/**
 * Definition of the data types in the packet
 * See section 3.4 Data Types
 */
typedef enum {
    PROTOCOL_BINARY_RAW_BYTES = 0x00,
    PROTOCOL_BINARY_DATATYPE_JSON = 0x01,
    PROTOCOL_BINARY_DATATYPE_COMPRESSED = 0x02
} protocol_binary_datatypes;

/**
 * Definition of the header structure for a request packet.
 * See section 2
 */
typedef union {
    struct {
        uint8_t magic;
        uint8_t opcode;
        uint16_t keylen;
        uint8_t extlen;
        uint8_t datatype;
        uint16_t vbucket;
        uint32_t bodylen;
        uint32_t opaque;
        uint64_t cas;
    } request;
    uint8_t bytes[24];
} protocol_binary_request_header;

/**
 * Definition of the header structure for a response packet.
 * See section 2
 */
typedef union {
    struct {
        uint8_t magic;
        uint8_t opcode;
        uint16_t keylen;
        uint8_t extlen;
        uint8_t datatype;
        uint16_t status;
        uint32_t bodylen;
        uint32_t opaque;
        uint64_t cas;
    } response;
    uint8_t bytes[24];
} protocol_binary_response_header;

/**
 * Definition of a request-packet containing no extras
 */
typedef union {
    struct {
        protocol_binary_request_header header;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header)];
} protocol_binary_request_no_extras;

/**
 * Definition of a response-packet containing no extras
 */
typedef union {
    struct {
        protocol_binary_response_header header;
    } message;
    uint8_t bytes[sizeof(protocol_binary_response_header)];
} protocol_binary_response_no_extras;

/**
 * Definition of the packet returned from a successful get, getq, getk and
 * getkq.
 * See section 4
 */
typedef union {
    struct {
        protocol_binary_response_header header;
        struct {
            uint32_t flags;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_response_header) + 4];
} protocol_binary_response_get;

/* used by tests */
typedef protocol_binary_response_get protocol_binary_response_getq;

/**
 * Definition of the packet used by the delete command
 * See section 4
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        union {
            struct {
                uint8_t meta;
                uint8_t level;
                uint16_t timeout;
            } alt;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 4];
} protocol_binary_request_delete;

/**
 * Definition of the packet returned by the delete command
 * See section 4
 *
 * extlen should be either zero, or 16 if the client has enabled the
 * MUTATION_SEQNO feature, with the following format:
 *
 *   Header:           (0-23): <protocol_binary_response_header>
 *   Extras:
 *     Vbucket UUID   (24-31): 0x0000000000003039
 *     Seqno          (32-39): 0x000000000000002D
 */
typedef protocol_binary_response_no_extras protocol_binary_response_delete;

/**
 * Definition of the packet used by set, add and replace
 * See section 4
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        union {
            struct {
                uint32_t flags;
                uint32_t expiration;
            } norm;
            struct {
                uint8_t meta;
                uint8_t level;
                uint16_t timeout;
                uint32_t flags;
                uint32_t expiration;
            } alt;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 20];
} protocol_binary_request_set;

/**
 * Definition of the structure used by the increment and decrement
 * command.
 * See section 4
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        union {
            struct {
                uint64_t delta;
                uint64_t initial;
                uint32_t expiration;
            } norm;
            struct {
                uint8_t meta;
                uint8_t level;
                uint16_t timeout;
                uint64_t delta;
                uint64_t initial;
                uint32_t expiration;
            } alt;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 24];
} protocol_binary_request_incr;
typedef protocol_binary_request_incr protocol_binary_request_decr;

/**
 * Definition of the response from an incr or decr command
 * command.
 *
 * The result of the incr/decr is a uint64_t placed at header + extlen.
 *
 * extlen should be either zero, or 16 if the client has enabled the
 * MUTATION_SEQNO feature, with the following format:
 *
 *   Header:           (0-23): <protocol_binary_response_header>
 *   Extras:
 *     Vbucket UUID   (24-31): 0x0000000000003039
 *     Seqno          (32-39): 0x000000000000002D
 *   Value:           (40-47): ....
 *
 */
typedef protocol_binary_response_no_extras protocol_binary_response_incr;
typedef protocol_binary_response_no_extras protocol_binary_response_decr;

/**
 * Definition of the packet returned from a successful version command
 * See section 4
 */
typedef protocol_binary_response_no_extras protocol_binary_response_version;

/**
 * Definition of the packet used by the stats command.
 * See section 4
 */
typedef protocol_binary_request_no_extras protocol_binary_request_stats;

/**
 * Definition of the packet returned from a successful stats command
 * See section 4
 */
typedef protocol_binary_response_no_extras protocol_binary_response_stats;

/**
 * Definition of the packet used by the verbosity command
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        struct {
            uint32_t level;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 4];
} protocol_binary_request_verbosity;

/**
 * Definition of the packet returned from the verbosity command
 */
typedef protocol_binary_response_no_extras protocol_binary_response_verbosity;

/**
 * Definition of the packet used by the touch command.
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        union {
            struct {
                uint32_t expiration;
            } norm;
            struct {
                uint8_t meta;
                uint8_t level;
                uint16_t timeout;
                uint32_t expiration;
            } alt;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 4];
} protocol_binary_request_touch;

/**
 * Definition of the packet returned from the touch command
 */
typedef protocol_binary_response_no_extras protocol_binary_response_touch;

/**
 * Definition of the packet used by the GAT(Q) command.
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        union {
            struct {
                uint32_t expiration;
            } norm;
            struct {
                uint8_t meta;
                uint8_t level;
                uint16_t timeout;
                uint32_t expiration;
            } alt;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 4];
} protocol_binary_request_gat;

/**
 * Definition of the packet used by SUBDOCUMENT single-path commands.
 *
 * The path, which is always required, is in the Body, after the Key.
 *
 *   Header:                        24 @0: <protocol_binary_request_header>
 *   Extras:
 *     Sub-document flags            1 @24: <protocol_binary_subdoc_flag>
 *     Sub-document pathlen          2 @25: <variable>
 *   Body:
 *     Key                      keylen @27: <variable>
 *     Path                    pathlen @27+keylen: <variable>
 *     Value to insert/replace
 *               vallen-keylen-pathlen @27+keylen+pathlen: [variable]
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        struct {
            uint16_t pathlen;     // Length in bytes of the sub-doc path.
            uint8_t subdoc_flags; // See protocol_binary_subdoc_flag
        } extras;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 3];
} protocol_binary_request_subdocument;

/** Definition of the packet used by SUBDOCUMENT responses.
 */
typedef union {
    struct {
        protocol_binary_response_header header;
    } message;
    uint8_t bytes[sizeof(protocol_binary_response_header)];
} protocol_binary_response_subdocument;

/**
 * Definition of the request packets used by SUBDOCUMENT multi-path commands.
 *
 * Multi-path sub-document commands differ from single-path in that they
 * encode a series of multiple paths to operate on (from a single key).
 * There are two multi-path commands - MULTI_LOOKUP and MULTI_MUTATION.
 * - MULTI_LOOKUP consists of variable number of subdoc lookup commands
 *                (SUBDOC_GET or SUBDOC_EXISTS).
 * - MULTI_MUTATION consists of a variable number of subdoc mutation
 *                  commands (i.e. all subdoc commands apart from
 *                  SUBDOC_{GET,EXISTS}).
 *
 * Each path to be operated on is specified by an Operation Spec, which are
 * contained in the body. This defines the opcode, path, and value
 * (for mutations).
 *
 * A maximum of MULTI_MAX_PATHS paths (operations) can be encoded in a
 * single multi-path command.
 *
 *  SUBDOC_MULTI_LOOKUP:
 *    Header:                24 @0:  <protocol_binary_request_header>
 *    Extras:                 0 @24: no extras
 *    Body:         <variable>  @24:
 *        Key            keylen @24: <variable>
 *        1..MULTI_MAX_PATHS [Lookup Operation Spec]
 *
 *        Lookup Operation Spec:
 *                            1 @0 : Opcode
 *                            1 @1 : Flags
 *                            2 @2 : Path Length
 *                      pathlen @4 : Path
 */
static const int PROTOCOL_BINARY_SUBDOC_MULTI_MAX_PATHS = 16;

typedef struct {
    uint8_t opcode;
    uint8_t flags;
    uint16_t pathlen;
    /* uint8_t path[pathlen] */
} protocol_binary_subdoc_multi_lookup_spec;

typedef protocol_binary_request_no_extras protocol_binary_request_subdocument_multi_lookup;

/*
 *
 * SUBDOC_MULTI_MUTATION
 *    Header:                24 @0:  <protocol_binary_request_header>
 *    Extras:                 0 @24:
 *    Body:           variable  @24:
 *        Key            keylen @24: <variable>
 *        1..MULTI_MAX_PATHS [Mutation Operation Spec]
 *
 *        Mutation Operation Spec:
 *                            1 @0         : Opcode
 *                            1 @1         : Flags
 *                            2 @2         : Path Length
 *                            4 @4         : Value Length
 *                      pathlen @8         : Path
 *                       vallen @8+pathlen : Value
 */
typedef struct {
    uint8_t opcode;
    uint8_t flags;
    uint16_t pathlen;
    uint32_t valuelen;
    /* uint8_t path[pathlen] */
    /* uint8_t value[valuelen]  */
} protocol_binary_subdoc_multi_mutation_spec;

typedef protocol_binary_request_no_extras protocol_binary_request_subdocument_multi_mutation;

/**
 * Definition of the response packets used by SUBDOCUMENT multi-path
 * commands.
 *
 * SUBDOC_MULTI_LOOKUP - Body consists of a series of lookup_result structs,
 *                       one per lookup_spec in the request.
 *
 * Lookup Result:
 *                            2 @0 : status
 *                            4 @2 : resultlen
 *                    resultlen @6 : result
 */
typedef struct {
    protocol_binary_request_header header;
    /* Variable-length 1..PROTOCOL_BINARY_SUBDOC_MULTI_MAX_PATHS */
    protocol_binary_subdoc_multi_lookup_spec body[1];
} protocol_binary_response_subdoc_multi_lookup;

/**
 * SUBDOC_MULTI_MUTATION - Body is either empty (if all mutations
 *                         successful), or contains the sub-code and
 *                         index of the first failed mutation spec..
 * Mutation Result (failure):
 *                   2 @0 : Status code of first spec which failed.
 *                   1 @2 : 0-based index of the first spec which failed.
 */
typedef union {
    struct {
        protocol_binary_response_header header;
    } message;
    uint8_t bytes[sizeof(protocol_binary_response_header)];
} protocol_binary_response_subdoc_multi_mutation;

/**
 * Definition of hello's features.
 */
typedef enum {
    PROTOCOL_BINARY_FEATURE_INVALID = 0x01,
    PROTOCOL_BINARY_FEATURE_TLS = 0x2,
    PROTOCOL_BINARY_FEATURE_TCPNODELAY = 0x03,
    PROTOCOL_BINARY_FEATURE_MUTATION_SEQNO = 0x04,
    PROTOCOL_BINARY_FEATURE_TCPDELAY = 0x05,
    PROTOCOL_BINARY_FEATURE_XATTR = 0x06,
    PROTOCOL_BINARY_FEATURE_XERROR = 0x07,
    PROTOCOL_BINARY_FEATURE_SELECT_BUCKET = 0x08,
    PROTOCOL_BINARY_FEATURE_INVALID2 = 0x09,
    PROTOCOL_BINARY_FEATURE_SNAPPY = 0x0a,
    PROTOCOL_BINARY_FEATURE_JSON = 0x0b,
    PROTOCOL_BINARY_FEATURE_DUPLEX = 0x0c,
    PROTOCOL_BINARY_FEATURE_CLUSTERMAP_CHANGE_NOTIFICATION = 0x0d,
    PROTOCOL_BINARY_FEATURE_UNORDERED_EXECUTION = 0x0e,
    PROTOCOL_BINARY_FEATURE_TRACING = 0x0f,
    PROTOCOL_BINARY_FEATURE_ALT_REQUEST_SUPPORT = 0x10,
    PROTOCOL_BINARY_FEATURE_SYNC_REPLICATION = 0x11,
    PROTOCOL_BINARY_FEATURE_COLLECTIONS = 0x12
} protocol_binary_hello_features;

#define MEMCACHED_FIRST_HELLO_FEATURE 0x01
#define MEMCACHED_TOTAL_HELLO_FEATURES 15

// clang-format off
#define protocol_feature_2_text(a) \
    (a == PROTOCOL_BINARY_FEATURE_INVALID) ? "Invalid" : \
    (a == PROTOCOL_BINARY_FEATURE_TLS) ? "TLS" : \
    (a == PROTOCOL_BINARY_FEATURE_TCPNODELAY) ? "TCP nodelay" : \
    (a == PROTOCOL_BINARY_FEATURE_MUTATION_SEQNO) ? "Mutation seqno" : \
    (a == PROTOCOL_BINARY_FEATURE_TCPDELAY) ? "TCP delay" : \
    (a == PROTOCOL_BINARY_FEATURE_XATTR) ? "XATTR" : \
    (a == PROTOCOL_BINARY_FEATURE_XERROR) ? "XERROR": \
    (a == PROTOCOL_BINARY_FEATURE_SELECT_BUCKET) ? "Select bucket": \
    (a == PROTOCOL_BINARY_FEATURE_INVALID2) ? "Invalid2": \
    (a == PROTOCOL_BINARY_FEATURE_SNAPPY) ? "Snappy": \
    (a == PROTOCOL_BINARY_FEATURE_JSON) ? "JSON": \
    (a == PROTOCOL_BINARY_FEATURE_DUPLEX) ? "Duplex": \
    (a == PROTOCOL_BINARY_FEATURE_CLUSTERMAP_CHANGE_NOTIFICATION) ? "Clustermap change notification": \
    (a == PROTOCOL_BINARY_FEATURE_UNORDERED_EXECUTION) ? "Unordered execution": \
    (a == PROTOCOL_BINARY_FEATURE_TRACING) ? "Tracing": \
    (a == PROTOCOL_BINARY_FEATURE_ALT_REQUEST_SUPPORT) ? "Alt request support": \
    (a == PROTOCOL_BINARY_FEATURE_SYNC_REPLICATION) ? "Synchronous Replication": \
    (a == PROTOCOL_BINARY_FEATURE_COLLECTIONS) ? "Collections": \
    "Unknown"
// clang-format on

/**
 * The HELLO command is used by the client and the server to agree
 * upon the set of features the other end supports. It is initiated
 * by the client by sending its agent string and the list of features
 * it would like to use. The server will then reply with the list
 * of the requested features it supports.
 *
 * ex:
 * Client ->  HELLO [myclient 2.0] datatype, tls
 * Server ->  HELLO SUCCESS datatype
 *
 * In this example the server responds that it allows the client to
 * use the datatype extension, but not the tls extension.
 */

/**
 * Definition of the packet requested by hello cmd.
 * Key: This is a client-specific identifier (not really used by
 *      the server, except for logging the HELLO and may therefore
 *      be used to identify the client at a later time)
 * Body: Contains all features supported by client. Each feature is
 *       specified as an uint16_t in network byte order.
 */
typedef protocol_binary_request_no_extras protocol_binary_request_hello;

/**
 * Definition of the packet returned by hello cmd.
 * Body: Contains all features requested by the client that the
 *       server agrees to ssupport. Each feature is
 *       specified as an uint16_t in network byte order.
 */
typedef protocol_binary_response_no_extras protocol_binary_response_hello;

/**
 * The message format for getLocked engine API
 */
typedef protocol_binary_request_gat protocol_binary_request_getl;

/**
 * Message format for CMD_GET_CONFIG
 */
typedef protocol_binary_request_no_extras protocol_binary_request_get_cluster_config;

#define OBS_STATE_NOT_PERSISTED 0x00
#define OBS_STATE_PERSISTED 0x01
#define OBS_STATE_NOT_FOUND 0x80
#define OBS_STATE_LOGICAL_DEL 0x81

/**
 * The PROTOCOL_BINARY_CMD_OBSERVE_SEQNO command is used by the
 * client to retrieve information about the vbucket in order to
 * find out if a particular mutation has been persisted or
 * replicated at the server side. In order to do so, the client
 * would pass the vbucket uuid of the vbucket that it wishes to
 * observe to the serve.  The response would contain the last
 * persisted sequence number and the latest sequence number in the
 * vbucket. For example, if a client sends a request to observe
 * the vbucket 0 with uuid 12345 and if the response contains the
 * values <58, 65> and then the client can infer that sequence
 * number 56 has been persisted, 60 has only been replicated and
 * not been persisted yet and 68 has not been replicated yet.
 */

/**
 * Definition of the request packet for the observe_seqno command.
 *
 * Header: Contains the vbucket id of the vbucket that the client
 *         wants to observe.
 *
 * Body: Contains the vbucket uuid of the vbucket that the client
 *       wants to observe. The vbucket uuid is of type uint64_t.
 *
 */
typedef union {
    struct {
        protocol_binary_request_header header;
        struct {
            uint64_t uuid;
        } body;
    } message;
    uint8_t bytes[sizeof(protocol_binary_request_header) + 8];
} protocol_binary_request_observe_seqno;

/**
 * Definition of the response packet for the observe_seqno command.
 * Body: Contains a tuple of the form
 *       <format_type, vbucket id, vbucket uuid, last_persisted_seqno, current_seqno>
 *
 *       - format_type is of type uint8_t and it describes whether
 *         the vbucket has failed over or not. 1 indicates a hard
 *         failover, 0 indicates otherwise.
 *       - vbucket id is of type uint16_t and it is the identifier for
 *         the vbucket.
 *       - vbucket uuid is of type uint64_t and it represents a UUID for
 *          the vbucket.
 *       - last_persisted_seqno is of type uint64_t and it is the
 *         last sequence number that was persisted for this
 *         vbucket.
 *       - current_seqno is of the type uint64_t and it is the
 *         sequence number of the latest mutation in the vbucket.
 *
 *       In the case of a hard failover, the tuple is of the form
 *       <format_type, vbucket id, vbucket uuid, last_persisted_seqno, current_seqno,
 *       old vbucket uuid, last_received_seqno>
 *
 *       - old vbucket uuid is of type uint64_t and it is the
 *         vbucket UUID of the vbucket prior to the hard failover.
 *
 *       - last_received_seqno is of type uint64_t and it is the
 *         last received sequence number in the old vbucket uuid.
 *
 *       The other fields are the same as that mentioned in the normal case.
 */
typedef protocol_binary_response_no_extras protocol_binary_response_observe_seqno;

/**
 * @}
 */
#ifdef __cplusplus
}
#endif
#endif /* PROTOCOL_BINARY_H */
