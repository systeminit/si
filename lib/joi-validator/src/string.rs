use crate::{generic, require, rule_err, Args};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Validator {
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(flatten)]
    pub base: generic::Validator<String, Flags>,
}

impl Validator {
    pub fn validate(self, value: &Option<serde_json::Value>) -> Result<(), (String, String)> {
        self.base.validate_presence(value)?;
        match value {
            Some(serde_json::Value::String(value)) => {
                // Now that we have the string, validate it
                self.base.validate_value(value)?;
                for rule in self.rules {
                    rule.validate(value)?;
                }
                Ok(())
            }
            Some(_) => Err(rule_err("string.base", "must be a string")),
            None => Ok(()),
        }
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        let mut rule_names = self.base.rule_names();
        rule_names.push("string.base");
        rule_names.extend(self.rules.iter().map(Rule::rule_name));
        rule_names
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "name")]
enum Rule {
    // Alphanum,
    // Base64(Args<Options<Base64Options>>),
    // Case(Args<Options<CaseOptions>>),
    // CreditCard,
    // DataUri(Args<Options<DataUriOptions>>),
    // Domain(Args<Options<DomainOptions>>),
    // Email(Args<Options<EmailOptions>>),
    // Guid(Args<Options<GuidOptions>>),
    // Hex(Args<Options<HexOptions>>),
    // Hostname,
    // Ip(Args<Options<IpOptions>>),
    // IsoDate,
    // IsoDuration,
    Length(Args<LengthLimit>),
    Min(Args<LengthLimit>),
    Max(Args<LengthLimit>),
    // Normalize(Args<Normalize>),
    // Pattern(Args<Pattern>),
    // Replace(Args<Replace>),
    // Token,
    // Trim(Args<Enabled>),
    // Truncate(Args<Enabled>),
    // Uri(Args<Options<UriOptions>>),
}

impl Rule {
    fn validate(self, value: &str) -> Result<(), (String, String)> {
        match self {
            Self::Length(rule) => require(
                value.len() == rule.args.limit,
                "string.length",
                format!("length must be {} characters long", rule.args.limit),
            ),
            Self::Min(rule) => require(
                value.len() >= rule.args.limit,
                "string.min",
                format!(
                    "length must be at least {} characters long",
                    rule.args.limit
                ),
            ),
            Self::Max(rule) => require(
                value.len() <= rule.args.limit,
                "string.max",
                format!(
                    "length must be less than or equal to {} characters long",
                    rule.args.limit
                ),
            ),
        }
    }

    fn rule_name(&self) -> &'static str {
        match self {
            Self::Length(_) => "string.length",
            Self::Min(_) => "string.min",
            Self::Max(_) => "string.max",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct LengthLimit {
    limit: usize,
    // encoding: Option<Encoding>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Flags {
    // insensitive: Option<bool>,
}

// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct Base64Options {
//     pub padding_required: Option<bool>,
//     pub url_safe: Option<bool>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct CaseOptions {
//     pub direction: CaseDirection,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum CaseDirection {
//     Lower,
//     Upper,
// }

// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct DataUriOptions {
//     pub padding_required: Option<bool>,
// }

// // allowFullyQualified - if true, domains ending with a . character are permitted. Defaults to false.
// // allowUnicode - if true, Unicode characters are permitted. Defaults to true.
// // minDomainSegments - number of segments required for the domain. Defaults to 2.
// // maxDomainSegments - maximum number of allowed domain segments. Default to no limit.
// // tlds - options for TLD (top level domain) validation. By default, the TLD must be a valid name listed on the IANA registry. To disable validation, set tlds to false. To customize how TLDs are validated, set one of these:
// // allow - one of:
// // true to use the IANA list of registered TLDs. This is the default value.
// // false to allow any TLD not listed in the deny list, if present.
// // a Set or array of the allowed TLDs. Cannot be used together with deny.
// // deny - one of:
// // a Set or array of the forbidden TLDs. Cannot be used together with a custom allow list.
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct DomainOptions {
//     pub allow_fully_qualified: Option<bool>,
//     pub allow_unicode: Option<bool>,
//     pub min_domain_segments: Option<u64>,
//     pub max_domain_segments: Option<u64>,
//     pub tlds: Option<DomainTldsOptions>,
// }

// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct DomainTldsOptions {
//     pub allow: DomainTldsAllow,
//     pub deny: Option<Vec<String>>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// #[serde(untagged)]
// pub enum DomainTldsAllow {
//     All(bool),
//     Some(Vec<String>),
// }

// impl Default for DomainTldsAllow {
//     fn default() -> Self {
//         DomainTldsAllow::All(true)
//     }
// }

// // options - optional settings:
// // allowFullyQualified - if true, domains ending with a . character are permitted. Defaults to false.
// // allowUnicode - if true, Unicode characters are permitted. Defaults to true.
// // ignoreLength - if true, ignore invalid email length errors. Defaults to false.
// // minDomainSegments - number of segments required for the domain. The default setting excludes single segment domains such as example@io which is a valid email but very uncommon. Defaults to 2.
// // maxDomainSegments - maximum number of allowed domain segments. Default to no limit.
// // multiple - if true, allows multiple email addresses in a single string, separated by , or the separator characters. Defaults to false.
// // separator - when multiple is true, overrides the default , separator. String can be a single character or multiple separator characters. Defaults to ','.
// // tlds - options for TLD (top level domain) validation. By default, the TLD must be a valid name listed on the IANA registry. To disable validation, set tlds to false. To customize how TLDs are validated, set one of these:
// // allow - one of:
// // true to use the IANA list of registered TLDs. This is the default value.
// // false to allow any TLD not listed in the deny list, if present.
// // a Set or array of the allowed TLDs. Cannot be used together with deny.
// // deny - one of:
// // a Set or array of the forbidden TLDs. Cannot be used together with a custom allow list.
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct EmailOptions {
//     pub allow_fully_qualified: Option<bool>,
//     pub allow_unicode: Option<bool>,
//     pub ignore_length: Option<bool>,
//     pub min_domain_segments: Option<u64>,
//     pub max_domain_segments: Option<u64>,
//     pub multiple: Option<bool>,
//     pub separator: Option<String>,
//     pub tlds: Option<DomainTldsOptions>,
// }

// // Requires the string value to be a valid GUID.

// // options - optional settings:
// // version - specifies one or more acceptable versions. Can be an Array or String with the following values: uuidv1, uuidv2, uuidv3, uuidv4, uuidv5, uuidv6, uuidv7 or uuidv8. If no version is specified then it is assumed to be a generic guid which will not validate the version or variant of the guid and just check for general structure format.
// // separator - defines the allowed or required GUID separator where:
// // true - a separator is required, can be either : or -.
// // false - separator is not allowed.
// // '-' - a dash separator is required.
// // ':' - a colon separator is required.
// // defaults to optional : or - separator.
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct GuidOptions {
//     version: Option<OneOrMany<GuidVersion>>,
//     separator: Option<GuidSeparator>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// enum GuidSeparator {
//     #[serde(rename = "-")]
//     Dash,
//     #[serde(rename = ":")]
//     Colon,
//     Any(Option<bool>),
// }

// impl Default for GuidSeparator {
//     fn default() -> Self {
//         GuidSeparator::Any(None)
//     }
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// enum GuidVersion {
//     Uuidv1,
//     Uuidv2,
//     Uuidv3,
//     Uuidv4,
//     Uuidv5,
//     Uuidv6,
//     Uuidv7,
//     Uuidv8,
// }

// // Requires the string value to be a valid hexadecimal string.

// // options - optional settings:
// // byteAligned - Boolean specifying whether you want to check that the hexadecimal string is byte aligned. If convert is true, a 0 will be added in front of the string in case it needs to be aligned. Defaults to false.
// // prefix - Boolean or optional. When true, the string will be considered valid if prefixed with 0x or 0X. When false, the prefix is forbidden. When optional, the string will be considered valid if prefixed or not prefixed at all. Defaults to false.
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct HexOptions {
//     byte_aligned: Option<bool>,
//     prefix: Option<HexPrefix>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// enum HexPrefix {
//     Required(bool),
//     #[serde(rename = "optional")]
//     Optional,
// }

// impl Default for HexPrefix {
//     fn default() -> Self {
//         HexPrefix::Required(false)
//     }
// }

// // options - optional settings:
// // version - One or more IP address versions to validate against. Valid values: ipv4, ipv6, ipvfuture
// // cidr - Used to determine if a CIDR is allowed or not. Valid values: optional, required, forbidden
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct IpOptions {
//     version: Option<OneOrMany<IpVersion>>,
//     cidr: Option<IpCidr>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// enum IpVersion {
//     Ipv4,
//     Ipv6,
//     Ipvfuture,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// enum IpCidr {
//     Optional,
//     Required,
//     Forbidden,
// }

// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Normalize {
//     form: NormalizedForm,
// }

// #[derive(Debug, Clone, Deserialize, Default)]
// // We deliberately do not rename_all
// enum NormalizedForm {
//     #[default]
//     NFC,
//     NFD,
//     NFKC,
//     NFKD,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Replace {
//     pattern: Pattern,
//     replacement: String,
// }

// // options - optional settings:
// // scheme - Specifies one or more acceptable Schemes, should only include the scheme name. Can be an Array or String (strings are automatically escaped for use in a Regular Expression).
// // allowRelative - Allow relative URIs. Defaults to false.
// // relativeOnly - Restrict only relative URIs. Defaults to false.
// // allowQuerySquareBrackets - Allows unencoded square brackets inside the query string. This is NOT RFC 3986 compliant but query strings like abc[]=123&abc[]=456 are very common these days. Defaults to false.
// // domain - Validate the domain component using the options specified in string.domain().
// // encodeUri - When convert is true, if the validation fails, attempts to encode the URI using encodeURI before validating it again. This allows to provide, for example, unicode URIs, and have it encoded for you. Defaults to false.
// #[derive(Debug, Clone, Deserialize, Default)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct UriOptions {
//     scheme: Option<OneOrMany<String>>,
//     allow_relative: Option<bool>,
//     relative_only: Option<bool>,
//     allow_query_square_brackets: Option<bool>,
//     domain: Option<DomainOptions>,
//     encode_uri: Option<bool>,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Enabled {
//     enabled: bool,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// enum Encoding {
//     Utf8,
//     // ???
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Pattern {
//     regex: String,
//     name: Option<String>,
//     invert: Option<bool>,
// }
