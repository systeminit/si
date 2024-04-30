use chrono::DateTime;
use si_pkg::{
    FuncArgumentKind, FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType,
    FuncSpecData, PkgSpec,
};
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator};

use crate::func::{FuncError, FuncResult};
use crate::PropKind;

#[remain::sorted]
#[derive(AsRefStr, Display, EnumIter, EnumString, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicFunc {
    Identity,
    SetArray,
    SetBoolean,
    SetInteger,
    SetJson,
    SetMap,
    SetObject,
    SetString,
    Unset,
    Validation,
}

impl IntrinsicFunc {
    pub fn pkg_spec() -> FuncResult<PkgSpec> {
        let mut builder = PkgSpec::builder();
        builder.name("si-intrinsic-funcs");
        builder.version("2023-05-24");
        builder.created_at(DateTime::parse_from_rfc2822(
            "Wed, 24 May 2023 00:00:00 PST",
        )?);
        builder.created_by("System Initiative");
        for instrinsic in IntrinsicFunc::iter() {
            builder.func(instrinsic.to_spec()?);
        }

        builder
            .build()
            .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))
    }

    pub fn to_spec(&self) -> FuncResult<FuncSpec> {
        let mut builder = FuncSpec::builder();
        builder.name(self.name());

        let mut data_builder = FuncSpecData::builder();
        data_builder
            .name(self.name())
            .handler("")
            .code_plaintext("");

        // These magic unique ids are here to keep them consistent with the intrinsic ids in the
        // existing builtin packages (chicken/egg problem here a bit)
        match self {
            Self::Identity => {
                builder
                    .unique_id("c6938e12287ab65f8ba8234559178413f2e2c02c44ea08384ed6687a36ec4f50");
                data_builder.backend_kind(FuncSpecBackendKind::Identity);
                data_builder.response_type(FuncSpecBackendResponseType::Identity);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("identity")
                        .kind(FuncArgumentKind::Any)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetArray => {
                builder
                    .unique_id("51049a590fb64860f159972012ac2657c629479a244d6bcc4b1b73ba4b29f87f");
                data_builder.backend_kind(FuncSpecBackendKind::Array);
                data_builder.response_type(FuncSpecBackendResponseType::Array);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Array)
                        .element_kind(FuncArgumentKind::Any)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetBoolean => {
                builder
                    .unique_id("577a7deea25cfad0d4b2dd1e1f3d96b86b8b1578605137b8c4128d644c86964b");
                data_builder.backend_kind(FuncSpecBackendKind::Boolean);
                data_builder.response_type(FuncSpecBackendResponseType::Boolean);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Boolean)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetInteger => {
                builder
                    .unique_id("7d384b237852f20b8dec2fbd2e644ffc6bde901d7dc937bd77f50a0d57e642a9");
                data_builder.backend_kind(FuncSpecBackendKind::Integer);
                data_builder.response_type(FuncSpecBackendResponseType::Integer);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Integer)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetMap => {
                builder
                    .unique_id("dea5084fbf6e7fe8328ac725852b96f4b5869b14d0fe9dd63a285fa876772496");
                data_builder.backend_kind(FuncSpecBackendKind::Map);
                data_builder.response_type(FuncSpecBackendResponseType::Map);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Map)
                        .element_kind(FuncArgumentKind::Any)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetJson => {
                builder
                    .unique_id("c48ahif4739799f3ab84bcb88495f93b27b47c31a341f8005a60ca39308909fd");
                data_builder.backend_kind(FuncSpecBackendKind::Json);
                data_builder.response_type(FuncSpecBackendResponseType::Json);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Json)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetObject => {
                builder
                    .unique_id("cb9bf94739799f3a8b84bcb88495f93b27b47c31a341f8005a60ca39308909fd");
                data_builder.backend_kind(FuncSpecBackendKind::Object);
                data_builder.response_type(FuncSpecBackendResponseType::Object);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Object)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetString => {
                builder
                    .unique_id("bbe86d1a2b92c3e34b72a407cca424878d3466d29ca60e56a251a52a0840bfbd");
                data_builder.backend_kind(FuncSpecBackendKind::String);
                data_builder.response_type(FuncSpecBackendResponseType::String);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::String)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::Unset => {
                builder
                    .unique_id("8143ff98fbe8954bb3ab89ee521335d45ba9a42b7b79289eff53b503c4392c37");
                data_builder.backend_kind(FuncSpecBackendKind::Unset);
                data_builder.response_type(FuncSpecBackendResponseType::Unset);
            }
            Self::Validation => {
                builder
                    .unique_id("039ff70bc7922338978ab52a39156992b7d8e3390f0ef7e99d5b6ffd43141d8a");
                data_builder.backend_kind(FuncSpecBackendKind::Validation);
                data_builder.response_type(FuncSpecBackendResponseType::Validation);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("value")
                        .kind(FuncArgumentKind::Any)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
        };

        let data = data_builder
            .build()
            .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?;

        builder
            .data(data)
            .build()
            .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Identity => "si:identity",
            Self::SetArray => "si:setArray",
            Self::SetBoolean => "si:setBoolean",
            Self::SetInteger => "si:setInteger",
            Self::SetMap => "si:setMap",
            Self::SetObject => "si:setObject",
            Self::SetJson => "si:setJson",
            Self::SetString => "si:setString",
            Self::Unset => "si:unset",
            Self::Validation => "si:validation",
        }
    }

    pub fn maybe_from_str(s: impl AsRef<str>) -> Option<Self> {
        Some(match s.as_ref() {
            "si:identity" => Self::Identity,
            "si:setArray" => Self::SetArray,
            "si:setBoolean" => Self::SetBoolean,
            "si:setInteger" => Self::SetInteger,
            "si:setMap" => Self::SetMap,
            "si:setObject" => Self::SetObject,
            "si:setJson" => Self::SetJson,
            "si:setString" => Self::SetString,
            "si:unset" => Self::Unset,
            "si:validation" => Self::Validation,
            _ => {
                return None;
            }
        })
    }
}

impl From<PropKind> for IntrinsicFunc {
    fn from(value: PropKind) -> Self {
        match value {
            PropKind::Array => IntrinsicFunc::SetArray,
            PropKind::Boolean => IntrinsicFunc::SetBoolean,
            PropKind::Integer => IntrinsicFunc::SetInteger,
            PropKind::Json => IntrinsicFunc::SetJson,
            PropKind::Map => IntrinsicFunc::SetMap,
            PropKind::Object => IntrinsicFunc::SetObject,
            PropKind::String => IntrinsicFunc::SetString,
        }
    }
}
