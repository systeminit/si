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

#ifndef CBC_HANDLERS_H
#define CBC_HANDLERS_H
#include "config.h"
#include "common/options.h"
#include "common/histogram.h"

namespace cbc
{
#define HANDLER_DESCRIPTION(s)                                                                                         \
    const char *description() const                                                                                    \
    {                                                                                                                  \
        return s;                                                                                                      \
    }
#define HANDLER_USAGE(s)                                                                                               \
    const char *usagestr() const                                                                                       \
    {                                                                                                                  \
        return s;                                                                                                      \
    }

class Handler
{
  public:
    Handler(const char *name);
    virtual ~Handler();
    virtual const char *description() const = 0;
    virtual const char *usagestr() const
    {
        return NULL;
    }
    void execute(int argc, char **argv);

  protected:
    virtual const std::string &getLoneArg(bool required = false);
    virtual const std::string &getRequiredArg()
    {
        return getLoneArg(true);
    }
    virtual void addOptions();
    virtual void run();
    cliopts::Parser parser;
    ConnParams params;
    lcb_INSTANCE *instance;
    Histogram hg;
    std::string cmdname;
};

class GetHandler : public Handler
{
  public:
    GetHandler(const char *name = "get")
        : Handler(name), o_replica("replica"), o_exptime("expiry"), o_durability("durability"), o_scope("scope"),
          o_collection("collection")
    {

        o_scope.description("Name of the collection scope").setDefault("_default");
        o_collection.description("Name of the collection");
        o_durability.abbrev('d').description("Durability level").setDefault("none");
    }

    const char *description() const
    {
        if (isLock()) {
            return "Lock keys and retrieve them from the cluster";
        } else {
            return "Retrieve items from the cluster";
        }
    }

    DURABILITY_GETTER()

  protected:
    void addOptions();
    void run();

  private:
    cliopts::StringOption o_replica;
    cliopts::UIntOption o_exptime;
    cliopts::StringOption o_durability;
    cliopts::StringOption o_scope;
    cliopts::StringOption o_collection;
    bool isLock() const
    {
        return cmdname == "lock";
    }
};

class TouchHandler : public Handler
{
  public:
    TouchHandler(const char *name = "touch") : Handler(name), o_exptime("expiry"), o_durability("durability")
    {
        o_exptime.abbrev('e').mandatory(true);
        o_durability.abbrev('d').description("Durability level").setDefault("none");
    }
    HANDLER_DESCRIPTION("Updated expiry times for documents")

    DURABILITY_GETTER()

  protected:
    void addOptions();
    void run();

  private:
    cliopts::UIntOption o_exptime;
    cliopts::StringOption o_durability;
};

class SetHandler : public Handler
{
  public:
    SetHandler(const char *name = "create")
        : Handler(name), o_flags("flags"), o_exp("expiry"), o_add("add"), o_persist("persist-to"),
          o_replicate("replicate-to"), o_durability("durability"), o_value("value"), o_json("json"), o_mode("mode"),
          o_scope("scope"), o_collection("collection")
    {

        o_flags.abbrev('f').description("Flags for item");
        o_exp.abbrev('e').description("Expiry for item");
        o_add.abbrev('a').description("Fail if item exists").hide();
        o_durability.abbrev('d').description("Durability level").setDefault("none");
        o_persist.abbrev('p').description("Wait until item is persisted to this number of nodes");
        o_replicate.abbrev('r').description("Wait until item is replicated to this number of nodes");
        o_value.abbrev('V').description("Value to use. If unspecified, read from standard input");
        o_json.abbrev('J').description("Indicate to the server that this item is JSON");
        o_mode.abbrev('M').description("Mode to use when storing");
        o_mode.argdesc("upsert|insert|replace");
        o_mode.setDefault("upsert");
        o_scope.description("Name of the collection scope").setDefault("_default");
        o_collection.description("Name of the collection");
    }

    const char *description() const
    {
        if (hasFileList()) {
            return "Store files to the server";
        } else {
            return "Store item to the server";
        }
    }

    const char *usagestr() const
    {
        if (hasFileList()) {
            return "[OPTIONS...] FILE ...";
        } else {
            return "[OPTIONS...] KEY -V VALUE";
        }
    }

    bool hasFileList() const
    {
        return cmdname == "cp";
    }

    virtual lcb_STORE_OPERATION mode();

    DURABILITY_GETTER()

  protected:
    void run();
    void addOptions();
    void storeItem(const std::string &key, const char *value, size_t nvalue);
    void storeItem(const std::string &key, FILE *input);

  private:
    cliopts::UIntOption o_flags;
    cliopts::UIntOption o_exp;
    cliopts::BoolOption o_add;
    cliopts::IntOption o_persist;
    cliopts::IntOption o_replicate;
    cliopts::StringOption o_durability;
    cliopts::StringOption o_value;
    cliopts::BoolOption o_json;
    cliopts::StringOption o_mode;
    cliopts::StringOption o_scope;
    cliopts::StringOption o_collection;
    std::map< std::string, lcb_cas_t > items;
};

class HashHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Get mapping information for keys")
    HANDLER_USAGE("KEY ... [OPTIONS ...]")
    HashHandler() : Handler("hash") {}

  protected:
    void run();
};

class ObserveHandler : public Handler
{
  public:
    ObserveHandler() : Handler("observe") {}
    HANDLER_DESCRIPTION("Obtain persistence and replication status for keys")
    HANDLER_USAGE("KEY ... ")
  protected:
    void run();
};

class ObserveSeqnoHandler : public Handler
{
  public:
    ObserveSeqnoHandler() : Handler("observe-seqno") {}

    HANDLER_DESCRIPTION("Request information about a particular vBucket UUID")
    HANDLER_USAGE("UUID")

  protected:
    void run();
};

class ExistsHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Check if keys exist on server")
    HANDLER_USAGE("KEY [OPTIONS ...]")

    ExistsHandler() : Handler("exists"), o_scope("scope"), o_collection("collection")
    {
        o_scope.description("Name of the collection scope").setDefault("_default");
        o_collection.description("Name of the collection");
    }

  protected:
    void run();

    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_scope);
        parser.addOption(o_collection);
    }

  private:
    cliopts::StringOption o_scope;
    cliopts::StringOption o_collection;
};

class UnlockHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Unlock keys")
    HANDLER_USAGE("KEY CAS [OPTIONS ...]")
    UnlockHandler() : Handler("unlock") {}

  protected:
    void run();
};

class VersionHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Display information about libcouchbase")
    VersionHandler() : Handler("version") {}
    void run();
    void addOptions() {}
};

class RemoveHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Remove items from the cluster")
    HANDLER_USAGE("KEY ... [OPTIONS ...]")
    RemoveHandler() : Handler("rm"), o_durability("durability")
    {
        o_durability.abbrev('d').description("Durability level").setDefault("none");
    }

    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_durability);
    }

    DURABILITY_GETTER()
  protected:
    void run();

    cliopts::StringOption o_durability;
};

class StatsHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Retrieve cluster statistics")
    HANDLER_USAGE("[STATS_KEY ...] [OPTIONS ...]")
    StatsHandler() : Handler("stats"), o_keystats("keystats")
    {
        o_keystats.description("Keys are document IDs. retrieve information about them");
    }

  protected:
    void run();
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_keystats);
    }

  private:
    cliopts::BoolOption o_keystats;
};

class WatchHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Aggregate and display server statistics")
    HANDLER_USAGE("[KEYS ....] [OPTIONS ...]")
    WatchHandler() : Handler("watch"), o_interval("interval")
    {
        o_interval.abbrev('n').description("Update interval in seconds").setDefault(1);
    }

  protected:
    void run();
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_interval);
    }

  private:
    cliopts::UIntOption o_interval;
};

class VerbosityHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Modify the memcached logging level")
    HANDLER_USAGE("<detail|debug|info|warning> [OPTIONS ...]")
    VerbosityHandler() : Handler("verbosity") {}

  protected:
    void run();
};

class McVersionHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Query server versions using the memcached command")
    HANDLER_USAGE("[OPTIONS ...]")
    McVersionHandler() : Handler("mcversion") {}

  protected:
    void run();
};

class KeygenHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Output a list of keys that equally distribute amongst every vbucket")
    HANDLER_USAGE("[OPTIONS ...]")
    KeygenHandler() : Handler("keygen"), o_keys_per_vbucket("keys-per-vbucket")
    {
        o_keys_per_vbucket.setDefault(1).description("number of keys to generate per vbucket");
    }

  protected:
    void run();
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_keys_per_vbucket);
    }

  private:
    cliopts::UIntOption o_keys_per_vbucket;
};

class PingHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Reach all services on every node and measure response time")
    HANDLER_USAGE("[OPTIONS ...]")
    PingHandler() : Handler("ping"), o_details("details")
    {
        o_details.description("Render extra details about status of the services");
    }

  protected:
    void run();
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_details);
    }

  private:
    cliopts::BoolOption o_details;
};

class ArithmeticHandler : public Handler
{
  public:
    HANDLER_USAGE("KEY ... [OPTIONS ...]")

    ArithmeticHandler(const char *name)
        : Handler(name), o_initial("initial"), o_delta("delta"), o_expiry("expiry"), o_durability("durability")
    {

        o_initial.description("Initial value if item does not exist");
        o_delta.setDefault(1);
        o_expiry.abbrev('e').description("Expiration time for key");
        o_durability.abbrev('d').description("Durability level").setDefault("none");
    }

    DURABILITY_GETTER()

  protected:
    cliopts::ULongLongOption o_initial;
    cliopts::ULongLongOption o_delta;
    cliopts::UIntOption o_expiry;
    cliopts::StringOption o_durability;
    void run();
    virtual bool shouldInvert() const = 0;
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_initial);
        parser.addOption(o_delta);
        parser.addOption(o_expiry);
        parser.addOption(o_durability);
    }
};

class IncrHandler : public ArithmeticHandler
{
  public:
    HANDLER_DESCRIPTION("Increment a counter")
    IncrHandler() : ArithmeticHandler("incr")
    {
        o_delta.description("Amount to increment by");
    }

  protected:
    bool shouldInvert() const
    {
        return false;
    }
};

class DecrHandler : public ArithmeticHandler
{
  public:
    HANDLER_DESCRIPTION("Decrement a counter")
    DecrHandler() : ArithmeticHandler("decr")
    {
        o_delta.description("Amount to decrement by");
    }

  protected:
    bool shouldInvert() const
    {
        return true;
    }
};

class ViewsHandler : public Handler
{
  public:
    ViewsHandler() : Handler("view"), o_incdocs("with-docs"), o_params("params") {}

    HANDLER_DESCRIPTION("Query a view")
    HANDLER_USAGE("DESIGN/VIEW")

  protected:
    void run();
    void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_incdocs);
        parser.addOption(o_params);
    }

  private:
    cliopts::BoolOption o_incdocs;
    cliopts::StringOption o_params;
};

class N1qlHandler : public Handler
{
  public:
    N1qlHandler() : Handler("query"), o_args("qarg"), o_opts("qopt"), o_prepare("prepare"), o_analytics("analytics") {}
    HANDLER_DESCRIPTION("Execute a N1QL/Analytics Query")
    HANDLER_USAGE("QUERY [--qarg PARAM1=VALUE1 --qopt PARAM2=VALUE2]")

  protected:
    void run();

    void addOptions()
    {
        Handler::addOptions();
        o_args.description("Specify values for placeholders (can be specified multiple times");
        o_args.abbrev('A');
        o_args.argdesc("PLACEHOLDER_PARAM=PLACEHOLDER_VALUE");

        o_opts.description("Additional query options");
        o_opts.abbrev('Q');

        o_prepare.description("Prepare query before issuing");
        o_analytics.description("Perform query to analytics service");

        parser.addOption(o_args);
        parser.addOption(o_opts);
        parser.addOption(o_prepare);
        parser.addOption(o_analytics);
    }

  private:
    cliopts::ListOption o_args;
    cliopts::ListOption o_opts;
    cliopts::BoolOption o_prepare;
    cliopts::BoolOption o_analytics;
};

class HttpReceiver
{
  public:
    HttpReceiver() : statusInvoked(false) {}
    virtual ~HttpReceiver() {}
    void maybeInvokeStatus(const lcb_RESPHTTP *);
    void install(lcb_INSTANCE *);
    virtual void handleStatus(lcb_STATUS, int) {}
    virtual void onDone() {}
    virtual void onChunk(const char *data, size_t size)
    {
        resbuf.append(data, size);
    }
    bool statusInvoked;
    std::string resbuf;
    std::map< std::string, std::string > headers;
};

class HttpBaseHandler : public Handler, public HttpReceiver
{
  public:
    HttpBaseHandler(const char *name) : Handler(name), o_method("method")
    {

        o_method.setDefault("GET").abbrev('X').description("HTTP Method to use");
    }

  protected:
    void run();
    virtual std::string getURI() = 0;
    virtual const std::string &getBody();
    virtual std::string getContentType()
    {
        return "";
    }
    virtual bool isAdmin() const
    {
        return false;
    }
    virtual lcb_HTTP_METHOD getMethod();
    virtual void handleStatus(lcb_STATUS err, int code);
    virtual void addOptions()
    {
        if (isAdmin()) {
            params.setAdminMode();
        }
        Handler::addOptions();
        parser.addOption(o_method);
    }
    cliopts::StringOption o_method;

  private:
    std::string body_cached;
};

class AdminHandler : public HttpBaseHandler
{
  public:
    HANDLER_DESCRIPTION("Invoke an administrative REST API")
    HANDLER_USAGE("PATH ... [OPTIONS ...]")
    AdminHandler(const char *name = "admin") : HttpBaseHandler(name) {}

  protected:
    virtual void run();
    virtual std::string getURI();
    virtual bool isAdmin() const
    {
        return true;
    }
};

class RbacHandler : public AdminHandler
{
  public:
    HANDLER_USAGE("[OPTIONS ...]")
    RbacHandler(const char *name) : AdminHandler(name), o_raw('r', "raw")
    {
        o_raw.description("Do not reformat output from server (display JSON response)");
    }

  protected:
    virtual void run();
    virtual void format() = 0;
    virtual void addOptions()
    {
        AdminHandler::addOptions();
        parser.addOption(o_raw);
    }

  private:
    cliopts::BoolOption o_raw;
};

class RoleListHandler : public RbacHandler
{
  public:
    HANDLER_DESCRIPTION("List roles")
    RoleListHandler() : RbacHandler("role-list") {}

  protected:
    virtual void format();
    virtual void addOptions()
    {
        RbacHandler::addOptions();
    }
    std::string getURI()
    {
        return "/settings/rbac/roles";
    }
    const std::string &getBody()
    {
        static std::string e;
        return e;
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_GET;
    }
};

class UserListHandler : public RbacHandler
{
  public:
    HANDLER_DESCRIPTION("List users")
    UserListHandler() : RbacHandler("user-list") {}

  protected:
    virtual void format();
    virtual void addOptions()
    {
        RbacHandler::addOptions();
    }
    std::string getURI()
    {
        return "/settings/rbac/users";
    }
    const std::string &getBody()
    {
        static std::string e;
        return e;
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_GET;
    }
};

class UserDeleteHandler : public AdminHandler
{
  public:
    HANDLER_DESCRIPTION("Delete a user")
    HANDLER_USAGE("NAME [OPTIONS ...]")
    UserDeleteHandler() : AdminHandler("user-delete"), o_domain("domain")
    {
        o_domain.description("The domain, where user account defined {local,external}").setDefault("local");
    }

  protected:
    virtual void addOptions()
    {
        AdminHandler::addOptions();
        parser.addOption(o_domain);
    }
    void run()
    {
        name = getRequiredArg();
        domain = o_domain.result();
        if (domain != "local" && domain != "external") {
            throw BadArg("Unrecognized domain type");
        }
        AdminHandler::run();
    }
    std::string getURI()
    {
        return std::string("/settings/rbac/users/") + domain + "/" + name;
    }
    const std::string &getBody()
    {
        static std::string e;
        return e;
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_DELETE;
    }

  private:
    cliopts::StringOption o_domain;
    std::string name;
    std::string domain;
};

class UserUpsertHandler : public AdminHandler
{
  public:
    HANDLER_DESCRIPTION("Create or update a user")
    HANDLER_USAGE("NAME [OPTIONS ...]")
    UserUpsertHandler()
        : AdminHandler("user-upsert"), o_domain("domain"), o_full_name("full-name"), o_password("user-password"),
          o_roles("role")
    {
        o_domain.description("The domain, where user account defined {local,external}").setDefault("local");
        o_full_name.description("The user's fullname");
        o_roles.description("The role associated with user (can be specified multiple times if needed)");
        o_password.description("The password for the user");
    }

  protected:
    virtual void addOptions()
    {
        AdminHandler::addOptions();
        parser.addOption(o_domain);
        parser.addOption(o_full_name);
        parser.addOption(o_roles);
        parser.addOption(o_password);
    }
    virtual void run();
    std::string getURI()
    {
        return std::string("/settings/rbac/users/") + domain + "/" + name;
    }
    const std::string &getBody()
    {
        return body;
    }
    std::string getContentType()
    {
        return "application/x-www-form-urlencoded";
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_PUT;
    }

  private:
    cliopts::StringOption o_domain;
    cliopts::StringOption o_full_name;
    cliopts::StringOption o_password;
    cliopts::ListOption o_roles;
    std::string name;
    std::string domain;
    std::string body;
};

class BucketCreateHandler : public AdminHandler
{
  public:
    HANDLER_DESCRIPTION("Create a bucket")
    HANDLER_USAGE("NAME [OPTIONS ...]")
    BucketCreateHandler()
        : AdminHandler("bucket-create"), o_btype("bucket-type"), o_ramquota("ram-quota"), o_bpass("bucket-password"),
          o_replicas("num-replicas"), o_proxyport("moxi-port"), isMemcached(false)
    {
        o_btype.description("Bucket type {couchbase,memcached}").setDefault("couchbase");
        o_ramquota.description("RAM Quota for bucket (MB)").setDefault(100);
        o_bpass.description("Bucket password");
        o_replicas.description("Number of replicas for bucket").setDefault(1);
        o_proxyport.description("[Compatibility] memcached listening port");
    }

  protected:
    virtual void run();
    virtual void addOptions()
    {
        AdminHandler::addOptions();
        parser.addOption(o_btype);
        parser.addOption(o_ramquota);
        parser.addOption(o_bpass);
        parser.addOption(o_replicas);
        parser.addOption(o_proxyport);
    }

    std::string getURI()
    {
        return "/pools/default/buckets";
    }
    const std::string &getBody()
    {
        return body_s;
    }
    std::string getContentType()
    {
        return "application/x-www-form-urlencoded";
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_POST;
    }

  private:
    cliopts::StringOption o_btype;
    cliopts::UIntOption o_ramquota;
    cliopts::StringOption o_bpass;
    cliopts::UIntOption o_replicas;
    cliopts::UIntOption o_proxyport;
    std::string body_s;
    bool isMemcached;
};

class BucketDeleteHandler : public AdminHandler
{
  public:
    HANDLER_DESCRIPTION("Delete a bucket")
    HANDLER_USAGE("NAME [OPTIONS ...]")
    BucketDeleteHandler() : AdminHandler("bucket-delete") {}

  protected:
    void run()
    {
        bname = getRequiredArg();
        AdminHandler::run();
    }
    std::string getURI()
    {
        return std::string("/pools/default/buckets/") + bname;
    }
    lcb_HTTP_METHOD getMethod()
    {
        return LCB_HTTP_METHOD_DELETE;
    }
    const std::string &getBody()
    {
        static std::string e;
        return e;
    }

  private:
    std::string bname;
};

class BucketFlushHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Flush a bucket")
    HANDLER_USAGE("[COMMON OPTIONS ...]")
    BucketFlushHandler() : Handler("bucket-flush") {}

  protected:
    void run();
};

class ConnstrHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Parse a connection string and provide info on its components")
    HANDLER_USAGE("CONNSTR")
    ConnstrHandler() : Handler("connstr") {}

  protected:
    void handleOptions() {}
    void run();
};

class WriteConfigHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Write the configuration file based on arguments passed")
    WriteConfigHandler() : Handler("write-config") {}

  protected:
    void run();
};

class CollectionGetManifestHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Get collection manifest")
    HANDLER_USAGE("[OPTIONS ...]")
    CollectionGetManifestHandler() : Handler("collection-manifest") {}

  protected:
    void run();
};

class CollectionGetCIDHandler : public Handler
{
  public:
    HANDLER_DESCRIPTION("Get collection ID by name")
    HANDLER_USAGE("[OPTIONS ...] COLLECTION-NAME...")
    CollectionGetCIDHandler() : Handler("collection-id"), o_scope("scope")
    {
        o_scope.description("Scope name").setDefault("_default");
    }

  protected:
    virtual void addOptions()
    {
        Handler::addOptions();
        parser.addOption(o_scope);
    }
    void run();

  private:
    cliopts::StringOption o_scope;
};

} // namespace cbc
#endif
