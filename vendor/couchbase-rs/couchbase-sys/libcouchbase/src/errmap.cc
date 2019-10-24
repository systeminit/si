#include "internal.h"
#include "errmap.h"
#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"

using namespace lcb::errmap;

ErrorMap::ErrorMap() : revision(0), version(0) {
}

static ErrorAttribute getAttribute(const std::string& s) {
    #define X(c, s_) if (s == s_) { return c; }
    LCB_XERRMAP_ATTRIBUTES(X)
    #undef X
    return INVALID_ATTRIBUTE;
}

RetrySpec *Error::getRetrySpec() const {
    return retry.specptr;
}

RetrySpec* RetrySpec::parse(const Json::Value& retryJson, std::string& emsg) {

    RetrySpec *spec = new RetrySpec();
    spec->refcount = 1;

#define FAIL_RETRY(s) \
    emsg = s; \
    delete spec; \
    return NULL;

    if (!retryJson.isObject()) {
        FAIL_RETRY("Missing retry specification");
    }

    const Json::Value& strategyJson = retryJson["strategy"];
    if (!strategyJson.isString()) {
        FAIL_RETRY("Missing `strategy`");
    }
    const char* strategy = strategyJson.asCString();
    if (!strcasecmp(strategy, "constant")) {
        spec->strategy = CONSTANT;
    } else if (!strcasecmp(strategy, "linear")) {
        spec->strategy = LINEAR;
    } else if (!strcasecmp(strategy, "exponential")) {
        spec->strategy = EXPONENTIAL;
    } else {
        FAIL_RETRY("Unknown strategy");
    }

#define GET_TIMEFLD(srcname, dstname, required) { \
    Json::Value dstname##Json = retryJson[srcname]; \
    if (dstname##Json.isNumeric()) { \
        spec->dstname = (dstname##Json).asUInt() * 1000; \
    } else if (required) { \
        FAIL_RETRY("Missing " # srcname); \
    } else { \
        spec->dstname = 0; \
    } \
}

    GET_TIMEFLD("interval", interval, true);
    GET_TIMEFLD("after", after, true);
    GET_TIMEFLD("ceil", ceil, false);
    GET_TIMEFLD("max-duration", max_duration, false);

    return spec;

#undef FAIL_RETRY
#undef GET_TIMEFLD

}

const uint32_t ErrorMap::MAX_VERSION = 1;

ErrorMap::ParseStatus
ErrorMap::parse(const char *s, size_t n, std::string& errmsg) {
    Json::Value root_nonconst;
    Json::Reader reader;
    if (!reader.parse(s, s + n, root_nonconst)) {
        errmsg = "Invalid JSON";
        return PARSE_ERROR;
    }

    const Json::Value& root = root_nonconst;
    const Json::Value& verJson = root["version"];
    if (!verJson.isNumeric()) {
        errmsg = "'version' is not a number";
        return PARSE_ERROR;
    }

    if (verJson.asUInt() > MAX_VERSION) {
        errmsg = "'version' is unreasonably high";
        return UNKNOWN_VERSION;
    }

    const Json::Value& revJson = root["revision"];
    if (!revJson.isNumeric()) {
        errmsg = "'revision' is not a number";
        return PARSE_ERROR;
    }

    if (revJson.asUInt() <= revision) {
        return NOT_UPDATED;
    }

    const Json::Value& errsJson = root["errors"];
    if (!errsJson.isObject()) {
        errmsg = "'errors' is not an object";
        return PARSE_ERROR;
    }

    Json::Value::const_iterator ii = errsJson.begin();
    for (; ii != errsJson.end(); ++ii) {
        // Key is the version in hex
        unsigned ec = 0;
        if (sscanf(ii.key().asCString(), "%x", &ec) != 1) {
            errmsg = "key " + ii.key().asString() + " is not a hex number";
            return PARSE_ERROR;
        }

        const Json::Value& errorJson = *ii;

        // Descend into the error attributes
        Error error;
        error.code = static_cast<uint16_t>(ec);

        error.shortname = errorJson["name"].asString();
        error.description = errorJson["desc"].asString();

        const Json::Value& attrs = errorJson["attrs"];
        if (!attrs.isArray()) {
            errmsg = "'attrs' is not an array";
            return PARSE_ERROR;
        }

        Json::Value::const_iterator jj = attrs.begin();
        for (; jj != attrs.end(); ++jj) {
            ErrorAttribute attr = getAttribute(jj->asString());
            if (attr == INVALID_ATTRIBUTE) {
                errmsg = "unknown attribute received";
                return UNKNOWN_VERSION;
            }
            error.attributes.insert(attr);
        }
        if (error.hasAttribute(AUTO_RETRY)) {
            const Json::Value& retryJson = errorJson["retry"];
            if (!retryJson.isObject()) {
                errmsg = "Need `retry` specification for `auto-retry` attribute";
                return PARSE_ERROR;
            }
            if ((error.retry.specptr = RetrySpec::parse(retryJson, errmsg)) == NULL) {
                return PARSE_ERROR;
            }
        }
        errors.insert(MapType::value_type(ec, error));
    }

    return UPDATED;
}

const Error& ErrorMap::getError(uint16_t code) const {
    static const Error invalid;
    MapType::const_iterator it = errors.find(code);

    if (it != errors.end()) {
        return it->second;
    } else {
        return invalid;
    }
}

ErrorMap *lcb_errmap_new() { return new ErrorMap(); }
void lcb_errmap_free(ErrorMap* m) { delete m; }
