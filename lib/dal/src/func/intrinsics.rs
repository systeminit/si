use si_pkg::{
    FuncArgumentKind, FuncArgumentSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType,
    PkgSpec,
};

use super::{FuncError, FuncResult};
use chrono::DateTime;
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator};

#[remain::sorted]
#[derive(AsRefStr, Display, EnumIter, EnumString, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicFunc {
    Identity,
    SetArray,
    SetBoolean,
    SetInteger,
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
        builder.created_at(
            DateTime::parse_from_rfc2822("Wed, 24 May 2023 00:00:00 PST")
                .expect("able to parse default datetime"),
        );
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
        builder.handler("");
        builder.code_plaintext("");

        match self {
            Self::Identity => {
                builder.backend_kind(FuncSpecBackendKind::Identity);
                builder.response_type(FuncSpecBackendResponseType::Identity);
                builder.argument(
                    FuncArgumentSpec::builder()
                        .name("identity")
                        .kind(FuncArgumentKind::Any)
                        .build()
                        .map_err(|e| FuncError::IntrinsicSpecCreation(e.to_string()))?,
                );
            }
            Self::SetArray => {
                builder.backend_kind(FuncSpecBackendKind::Array);
                builder.response_type(FuncSpecBackendResponseType::Array);
            }
            Self::SetBoolean => {
                builder.backend_kind(FuncSpecBackendKind::Boolean);
                builder.response_type(FuncSpecBackendResponseType::Boolean);
            }
            Self::SetInteger => {
                builder.backend_kind(FuncSpecBackendKind::Integer);
                builder.response_type(FuncSpecBackendResponseType::Integer);
            }
            Self::SetMap => {
                builder.backend_kind(FuncSpecBackendKind::Map);
                builder.response_type(FuncSpecBackendResponseType::Map);
            }
            Self::SetObject => {
                builder.backend_kind(FuncSpecBackendKind::Object);
                builder.response_type(FuncSpecBackendResponseType::Object);
            }
            Self::SetString => {
                builder.backend_kind(FuncSpecBackendKind::String);
                builder.response_type(FuncSpecBackendResponseType::String);
            }
            Self::Unset => {
                builder.backend_kind(FuncSpecBackendKind::Unset);
                builder.response_type(FuncSpecBackendResponseType::Unset);
            }
            Self::Validation => {
                builder.backend_kind(FuncSpecBackendKind::Validation);
                builder.response_type(FuncSpecBackendResponseType::Validation);
            }
        };

        builder
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
            Self::SetString => "si:setString",
            Self::Unset => "si:unset",
            Self::Validation => "si:validation",
        }
    }
}
