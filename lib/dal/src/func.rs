use std::string::FromUtf8Error;

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::CycloneValueEncryptError;

use crate::func::argument::FuncArgumentError;
use crate::{
    generate_unique_id, impl_standard_model, pk, standard_model, standard_model_accessor,
    standard_model_accessor_ro, ChangeSetPk, DalContext, FuncBinding, HistoryEventError,
    SecretError, StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError,
    Visibility, WorkspacePk, WsEvent, WsEventResult, WsPayload,
};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

pub mod argument;
pub mod backend;
pub mod before;
pub mod binding;
pub mod binding_return_value;
pub mod execution;
pub mod identity;
pub mod intrinsics;

pub fn is_intrinsic(name: &str) -> bool {
    intrinsics::IntrinsicFunc::iter().any(|intrinsic| intrinsic.name() == name)
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("cyclone value encrypt error: {0}")]
    CycloneValueEncrypt(#[from] CycloneValueEncryptError),
    #[error("error decoding code_base64: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("utf8 encoding error: {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(String),
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    /// Could not find [`FuncArgument`](crate::FuncArgument) corresponding to the identity [`Func`].
    #[error("identity func argument not found")]
    IdentityFuncArgumentNotFound,
    /// Could not find the identity [`Func`].
    #[error("identity func not found")]
    IdentityFuncNotFound,
    #[error("intrinsic parse error: {0} is not an intrinsic")]
    IntrinsicParse(String),
    #[error("intrinsic spec creation error {0}")]
    IntrinsicSpecCreation(String),
    #[error("Function missing expected code: {0}")]
    MissingCode(FuncId),
    #[error("Function missing expected handler: {0}")]
    MissingHandler(FuncId),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("could not find func by id: {0}")]
    NotFound(FuncId),
    #[error("could not find func by name: {0}")]
    NotFoundByName(String),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    /// When attempting to find the identity [`Func`], there were too many [`Funcs`](Func) returned.
    #[error("too many funcs found when looking for identity func")]
    TooManyFuncsFoundForIdentity,
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type FuncResult<T> = Result<T, FuncError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncMetadataView {
    pub display_name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

pk!(FuncPk);
pk!(FuncId);

/// A `Func` is the declaration of the existence of a function. It has a name,
/// and corresponds to a given function backend (and its associated return types).
///
/// `handler` is the name of the entry point into the code in `code_base64`.
/// For example, if we had a code block of
/// `function myValidator(actual, expected) { return true; }` in `code_base64`,
/// the `handler` value should be `myValidator`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pk: FuncPk,
    id: FuncId,
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    hidden: bool,
    builtin: bool,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    handler: Option<String>,
    code_base64: Option<String>,
    code_sha256: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Func,
    pk: FuncPk,
    id: FuncId,
    table_name: "funcs",
    history_event_label_base: "function",
    history_event_message_name: "Function"
}

impl Func {
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
    ) -> FuncResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &name,
                    &backend_kind.as_ref(),
                    &backend_response_type.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Creates a new [`Func`] from [`self`](Func). All relevant fields are duplicated, but rows
    /// existing on relationship tables (e.g. "belongs_to" or "many_to_many") are not.
    pub async fn duplicate(&self, ctx: &DalContext) -> FuncResult<Self> {
        // Generate a unique name and make sure it's not in use
        let mut new_unique_name;
        loop {
            new_unique_name = format!("{}{}", self.name(), generate_unique_id(4));
            if Self::find_by_name(ctx, &new_unique_name).await?.is_none() {
                break;
            };
        }

        let mut new_func = Self::new(
            ctx,
            new_unique_name,
            *self.backend_kind(),
            *self.backend_response_type(),
        )
        .await?;

        // Duplicate all fields on the func that do not come in through the constructor.
        new_func.set_display_name(ctx, self.display_name()).await?;
        new_func.set_description(ctx, self.description()).await?;
        new_func.set_link(ctx, self.link()).await?;
        new_func.set_hidden(ctx, self.hidden).await?;
        new_func.set_builtin(ctx, self.builtin).await?;
        new_func.set_handler(ctx, self.handler()).await?;
        new_func.set_code_base64(ctx, self.code_base64()).await?;

        Ok(new_func)
    }

    #[allow(clippy::result_large_err)]
    pub fn code_plaintext(&self) -> FuncResult<Option<String>> {
        Ok(match self.code_base64() {
            Some(base64_code) => Some(String::from_utf8(
                general_purpose::STANDARD_NO_PAD.decode(base64_code)?,
            )?),
            None => None,
        })
    }

    pub async fn is_builtin(&self, ctx: &DalContext) -> FuncResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT id FROM funcs WHERE id = $1 and tenancy_workspace_pk = $2 LIMIT 1",
                &[self.id(), &WorkspacePk::NONE],
            )
            .await?;

        Ok(row.is_some())
    }

    pub async fn set_code_plaintext(
        &mut self,
        ctx: &DalContext,
        code: Option<&'_ str>,
    ) -> FuncResult<()> {
        self.set_code_base64(
            ctx,
            code.as_ref()
                .map(|code| general_purpose::STANDARD_NO_PAD.encode(code)),
        )
        .await
    }

    pub fn metadata_view(&self) -> FuncMetadataView {
        FuncMetadataView {
            display_name: self.display_name().unwrap_or_else(|| self.name()).into(),
            description: self.description().map(Into::into),
            link: self.description().map(Into::into),
        }
    }

    pub async fn for_binding(ctx: &DalContext, func_binding: &FuncBinding) -> FuncResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT row_to_json(funcs.*) AS object
                FROM funcs_v1($1, $2) AS funcs
                INNER JOIN func_binding_belongs_to_func_v1($1, $2) AS func_binding_belongs_to_func
                    ON funcs.id = func_binding_belongs_to_func.belongs_to_id
                WHERE func_binding_belongs_to_func.object_id = $3",
                &[ctx.tenancy(), ctx.visibility(), func_binding.id()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn find_by_name(ctx: &DalContext, name: &str) -> FuncResult<Option<Self>> {
        Ok(Self::find_by_attr(ctx, "name", &name).await?.pop())
    }

    /// Returns `true` if this function is one handled internally by the `dal`, `false` if the
    /// function is one that will be executed by `veritech`
    pub fn is_intrinsic(&self) -> bool {
        is_intrinsic(self.name())
    }

    standard_model_accessor!(name, String, FuncResult);
    standard_model_accessor!(display_name, Option<String>, FuncResult);
    standard_model_accessor!(description, Option<String>, FuncResult);
    standard_model_accessor!(link, Option<String>, FuncResult);
    standard_model_accessor!(hidden, bool, FuncResult);
    standard_model_accessor!(builtin, bool, FuncResult);
    standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncResult);
    standard_model_accessor!(
        backend_response_type,
        Enum(FuncBackendResponseType),
        FuncResult
    );
    standard_model_accessor!(handler, Option<String>, FuncResult);
    standard_model_accessor!(code_base64, Option<String>, FuncResult);
    standard_model_accessor_ro!(code_sha256, String);
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncCreatedPayload {
    func_id: FuncId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncDeletedPayload {
    func_id: FuncId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncRevertedPayload {
    func_id: FuncId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncSavedPayload {
    func_id: FuncId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn func_created(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncCreated(FuncCreatedPayload {
                func_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn func_deleted(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncDeleted(FuncDeletedPayload {
                func_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn func_reverted(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncReverted(FuncRevertedPayload {
                func_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn func_saved(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncSaved(FuncSavedPayload {
                func_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }
}

// Variants don't map 1:1 onto FuncBackendKind, since some JsAttribute functions
// are a special case (Qualification, CodeGeneration etc)
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum FuncVariant {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Qualification,
    Reconciliation,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Reconciliation => FuncBackendKind::JsReconciliation,
            FuncVariant::Action => FuncBackendKind::JsAction,
            FuncVariant::Validation => FuncBackendKind::JsValidation,
            FuncVariant::Attribute | FuncVariant::CodeGeneration | FuncVariant::Qualification => {
                FuncBackendKind::JsAttribute
            }
            FuncVariant::Authentication => FuncBackendKind::JsAuthentication,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind(), func.backend_response_type()) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsReconciliation, _) => Ok(FuncVariant::Reconciliation),
            (FuncBackendKind::JsAction, _) => Ok(FuncVariant::Action),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            (FuncBackendKind::JsAuthentication, _) => Ok(FuncVariant::Authentication),
            (FuncBackendKind::Array, _)
            | (FuncBackendKind::Boolean, _)
            | (FuncBackendKind::Diff, _)
            | (FuncBackendKind::Identity, _)
            | (FuncBackendKind::Integer, _)
            | (FuncBackendKind::JsSchemaVariantDefinition, _)
            | (FuncBackendKind::Map, _)
            | (FuncBackendKind::Object, _)
            | (FuncBackendKind::String, _)
            | (FuncBackendKind::Unset, _)
            | (FuncBackendKind::Validation, _) => {
                Err(FuncError::FuncCannotBeTurnedIntoVariant(*func.id()))
            }
        }
    }
}
