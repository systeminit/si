pub mod content_hash;
pub mod encrypted_secret;
pub mod merkle_tree_hash;
pub mod ulid;
pub mod workspace_snapshot_address;
pub mod xxhash_type;

mod actor;
mod cas;
mod func_execution;
mod func_run;
mod func_run_log;
mod tenancy;
mod web_event;

pub use crate::{
    actor::Actor,
    actor::UserPk,
    cas::CasValue,
    content_hash::ContentHash,
    encrypted_secret::EncryptedSecretKey,
    func_execution::*,
    func_run::{
        ActionId, ActionKind, ActionPrototypeId, ActionResultState, AttributeValueId, ComponentId,
        FuncBackendKind, FuncBackendResponseType, FuncKind, FuncRun, FuncRunBuilder,
        FuncRunBuilderError, FuncRunId, FuncRunState, FuncRunValue,
    },
    func_run_log::{FuncRunLog, FuncRunLogId, OutputLine},
    tenancy::ChangeSetId,
    tenancy::Tenancy,
    tenancy::WorkspacePk,
    web_event::WebEvent,
    workspace_snapshot_address::WorkspaceSnapshotAddress,
};

#[macro_export]
macro_rules! id {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        $(#[$($attrss)*])*
        #[allow(missing_docs)]
        #[derive(
            Eq,
            PartialEq,
            PartialOrd,
            Ord,
            Copy,
            Clone,
            Hash,
            Default,
            derive_more::From,
            derive_more::Into,
            derive_more::Display,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(::ulid::Ulid);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0.to_string()).finish()
            }
        }

        impl $name {
            /// Generates a new key which is virtually guaranteed to be unique.
            pub fn generate() -> Self {
                Self(::ulid::Ulid::new())
            }

            pub fn new() -> Self {
                Self::generate()
            }

            pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ::ulid::ULID_LEN]) -> &'buf mut str {
                self.0.array_to_str(buf)
            }

            pub fn array_to_str_buf() -> [u8; ::ulid::ULID_LEN] {
                [0; ::ulid::ULID_LEN]
            }

            /// Converts type into inner [`Ulid`](::ulid::Ulid).
            pub fn into_inner(self) -> ::ulid::Ulid {
                self.0
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                ulid::Ulid::from(id.0).into()
            }
        }

        impl<'a> From<&'a $name> for ulid::Ulid {
            fn from(id: &'a $name) -> Self {
                id.0
            }
        }

        impl<'a> From<&'a $name> for $name {
            fn from(id: &'a $name) -> Self {
                *id
            }
        }

        impl std::str::FromStr for $name {
            type Err = ::ulid::DecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(::ulid::Ulid::from_string(s)?))
            }
        }

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let id: String = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(::ulid::Ulid::from_string(&id)?))
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                ty == &postgres_types::Type::BPCHAR
                    || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::BPCHAR)
            }
        }

        impl postgres_types::ToSql for $name {
            fn to_sql(
                &self,
                ty: &postgres_types::Type,
                out: &mut postgres_types::private::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                postgres_types::ToSql::to_sql(&self.0.to_string(), ty, out)
            }

            fn accepts(ty: &postgres_types::Type) -> bool
            where
                Self: Sized,
            {
                ty == &postgres_types::Type::BPCHAR
                    || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::BPCHAR)
            }

            fn to_sql_checked(
                &self,
                ty: &postgres_types::Type,
                out: &mut postgres_types::private::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
                postgres_types::ToSql::to_sql(&self.0.to_string(), ty, out)
            }
        }
    };
}
