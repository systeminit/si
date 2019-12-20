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

#ifndef LCB_MCSERVER_NEGOTIATE_H
#define LCB_MCSERVER_NEGOTIATE_H
#include <libcouchbase/couchbase.h>
#include <lcbio/lcbio.h>
#include <string>
#include <vector>

/**
 * @file
 * @brief SASL Negotiation routines
 *
 * @defgroup lcb-sasl Server/SASL Negotiation
 * @details
 * This module contains routines to initialize a server and authenticate it
 * against a cluster. In the future this will also be used to handle things
 * such as TLS negotiation and the HELLO command
 * @addtogroup lcb-sasl
 * @{
 */

struct lcb_settings_st;

namespace lcb
{
class SessionRequest : public lcb::io::ConnectionRequest
{
  public:
    /**
     * @brief Start negotiation on a connected socket
     *
     * This will start negotiation on the socket. Once complete (or an error has
     * taken place) the `callback` will be invoked with the result.
     *
     * @param sock A connected socket to use. Its reference count will be increased
     * @param settings A settings structure. Used for auth information as well as
     * logging
     * @param tmo Time in microseconds to wait until the negotiation is done
     * @param callback A callback to invoke when a result has been received
     * @param data User-defined pointer passed to the callback
     * @return A new handle which may be cancelled via mc_sessreq_cancel(). As with
     * other cancellable requests, once this handle is cancelled a callback will
     * not be received for it, and once the callback is received the handle may not
     * be cancelled as it will point to invalid memory.
     *
     * Once the socket has been negotiated successfuly, you may then use the
     * mc_sess_get() function to query the socket about various negotiation aspects
     *
     * @code{.c}
     * lcbio_CONNREQ creq;
     * SessionRequest *req = SessionRequest::start(sock, settings, tmo, callback, data);
     * LCBIO_CONNREQ_MKGENERIC(req, sessreq_cancel);
     * @endcode
     *
     * @see lcb::sessreq_cancel()
     * @see LCBIO_CONNREQ_MKGENERIC
     */
    static SessionRequest *start(lcbio_SOCKET *sock, lcb_settings_st *settings, uint32_t tmo,
                                 lcbio_CONNDONE_cb callback, void *data);

    /**
     * @brief Cancel a pending SASL negotiation request
     * @param handle The handle to cancel
     */
    virtual void cancel() = 0;
    virtual ~SessionRequest() {}
};
class SessionRequestImpl;

class SessionInfo : public lcbio_PROTOCTX
{
  public:
    /**
     * @brief Get an opaque handle representing the negotiated state of the socket
     * @param sock The negotiated socket
     * @return the `SASLINFO` structure if the socket is negotiated, or `NULL` if
     * the socket has not been negotiated.
     *
     * @see get_mech()
     */
    static SessionInfo *get(lcbio_SOCKET *);

    /**
     * @brief Get the mechanism employed for authentication
     * @param info pointer retrieved via mc_sasl_get()
     * @return A string indicating the mechanism used. This may be `PLAIN`,
     * `CRAM-MD5`, `SCRAM-SHA1`, `SCRAM-SHA256` or `SCRAM-SHA512` .
     */
    const std::string &get_mech() const
    {
        return mech;
    }

    /**
     * @brief Determine if a specific protocol feature is supported on the server
     * @param info info pointer returned via mc_sasl_get()
     * @param feature A feature ID
     * @return true if supported, false otherwise
     */
    bool has_feature(uint16_t feature) const;

  private:
    SessionInfo();
    friend class lcb::SessionRequestImpl;

    std::string mech;
    std::vector< uint16_t > server_features;
};

} // namespace lcb

/**@}*/

#endif
