//! This module contains [`Secret`], which is a reference to an underlying [`EncryptedSecret`].

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    collections::HashMap,
    fmt,
    str::FromStr,
    sync::Arc,
};

use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_crypto::{
    SymmetricCryptoError,
    SymmetricCryptoService,
    SymmetricNonce,
};
use si_data_pg::PgError;
use si_db::HistoryActor;
use si_events::{
    ContentHash,
    EncryptedSecretKey,
    Timestamp,
    encrypted_secret::EncryptedSecretKeyParseError,
    ulid::Ulid,
};
use si_hash::Hash;
use si_id::{
    PropId,
    SchemaVariantId,
};
use si_layer_cache::LayerDbError;
use sodiumoxide::crypto::{
    box_::{
        PublicKey,
        SecretKey,
    },
    sealedbox,
};
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::SensitiveContainer;

use crate::{
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    ChangeSetError,
    ComponentError,
    ComponentId,
    DalContext,
    Func,
    FuncError,
    FuncId,
    HelperError,
    KeyPair,
    KeyPairError,
    Prop,
    SchemaVariant,
    SchemaVariantError,
    TransactionsError,
    UserPk,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgument,
                AttributePrototypeArgumentError,
            },
        },
        value::AttributeValueError,
    },
    func::{
        argument::{
            FuncArgument,
            FuncArgumentError,
        },
        intrinsics::IntrinsicFunc,
    },
    implement_add_edge_to,
    key_pair::KeyPairPk,
    layer_db_types::{
        SecretContent,
        SecretContentV1,
    },
    prop::PropError,
    schema::variant::root_prop::RootPropChild,
    serde_impls::{
        base64_bytes_serde,
        nonce_serde,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        dependent_value_root::DependentValueRootError,
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        node_weight::{
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
            secret_node_weight::SecretNodeWeight,
        },
    },
};

mod algorithm;
mod definition_view;
mod event;
mod view;

pub use algorithm::{
    SecretAlgorithm,
    SecretVersion,
};
pub use definition_view::{
    SecretDefinitionView,
    SecretDefinitionViewError,
};
pub use event::{
    SecretCreatedPayload,
    SecretDeletedPayload,
    SecretUpdatedPayload,
};
pub use view::{
    SecretView,
    SecretViewError,
    SecretViewResult,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("error when decrypting encrypted secret")]
    DecryptionFailed,
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("error deserializing message: {0}")]
    DeserializeMessage(#[source] serde_json::Error),
    #[error("encrypted secret key parse error: {0}")]
    EncryptedSecretKeyParse(#[from] EncryptedSecretKeyParseError),
    #[error("encrypted secret not found for key: {0}")]
    EncryptedSecretNotFound(EncryptedSecretKey),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("func argument not found for func ({0}) and name ({1})")]
    FuncArgumentNotFound(FuncId, String),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("key pair not found for secret")]
    KeyPairNotFound,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("secret not found for key: [redacted]")]
    NotFoundForKey,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("not an error! prop id is not for a secret during MV build: {0}")]
    PropIdNotForSecret(PropId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("schema variant not secret defining: {0}")]
    SchemaVariantNotSecretDefining(SchemaVariantId),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

#[allow(missing_docs)]
pub type SecretResult<T> = Result<T, SecretError>;

pub use si_id::SecretId;

/// A reference to an [`EncryptedSecret`] with metadata.
///
/// This type does not contain any encrypted information nor any encryption metadata and is
/// therefore safe to expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Secret {
    id: SecretId,
    encrypted_secret_key: EncryptedSecretKey,
    #[serde(flatten)]
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,
    name: String,
    definition: String,
    description: Option<String>,
}

impl From<Secret> for SecretContentV1 {
    fn from(value: Secret) -> Self {
        Self {
            timestamp: value.timestamp,
            created_by: value.created_by,
            updated_by: value.updated_by,
            name: value.name,
            definition: value.definition,
            description: value.description,
        }
    }
}

impl Secret {
    #[allow(missing_docs)]
    pub fn assemble(secret_node_weight: SecretNodeWeight, content: SecretContentV1) -> Self {
        Self {
            id: secret_node_weight.id().into(),
            encrypted_secret_key: secret_node_weight.encrypted_secret_key().to_owned(),
            timestamp: content.timestamp,
            created_by: content.created_by,
            updated_by: content.updated_by,
            name: content.name,
            definition: content.definition,
            description: content.description,
        }
    }

    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: SecretId,
        add_fn: add_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: SecretResult,
    );

    /// Creates a new [`Secret`] with a corresponding [`EncryptedSecret`].
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        definition: impl Into<String>,
        description: Option<String>,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<Self> {
        let user = match ctx.history_actor() {
            HistoryActor::SystemInit => None,
            HistoryActor::User(user_pk) => Some(*user_pk),
        };

        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let secret_id = id.into();

        // Generate a key for the underlying encrypted secret.
        let key = Self::generate_key(ctx, secret_id).await?;

        let content = SecretContentV1 {
            timestamp: Timestamp::now(),
            created_by: user,
            updated_by: user,
            name: name.into(),
            definition: definition.into(),
            description,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(SecretContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let node_weight = NodeWeight::new_secret(id, lineage_id, key, hash);
        let secret_node_weight = node_weight.get_secret_node_weight()?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_or_replace_node(node_weight).await?;

        // Root --> Secret Category --> Secret (this)
        let secret_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Secret)
            .await?;
        Self::add_category_edge(
            ctx,
            secret_category_id,
            id.into(),
            EdgeWeightKind::new_use(),
        )
        .await?;

        let secret = Self::assemble(secret_node_weight, content);

        // After creating the secret on the graph, create an underlying encrypted secret and use
        // the key we assembled.
        EncryptedSecret::insert(ctx, key, crypted, key_pair_pk, version, algorithm).await?;

        Ok(secret)
    }

    /// Generates a key based on the [`Tenancy`](crate::Tenancy), [`SecretId`] and a newly generated
    /// [`Ulid`].
    ///
    /// A new key should be assembled anytime an [`EncryptedSecret`] is created or mutated. This
    /// method is purposefully owned by [`Secret`] to help ensure that we don't generate a key based
    /// on any encrypted contents or parameters to insert encrypted contents.
    async fn generate_key(
        ctx: &DalContext,
        secret_id: SecretId,
    ) -> SecretResult<EncryptedSecretKey> {
        let new_ulid = ctx.workspace_snapshot()?.generate_ulid().await?;

        let mut hasher = EncryptedSecretKey::hasher();
        hasher.update(&ctx.tenancy().to_bytes());
        hasher.update(secret_id.to_string().as_bytes());
        hasher.update(new_ulid.to_string().as_bytes());

        Ok(hasher.finalize())
    }

    /// Returns the [`id`](SecretId).
    pub fn id(&self) -> SecretId {
        self.id
    }

    /// Returns a reference to the name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns a reference to the definition.
    pub fn definition(&self) -> &str {
        self.definition.as_ref()
    }

    /// Returns a reference to the description.
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// Returns the key corresponding to the corresponding [`EncryptedSecret`].
    pub fn encrypted_secret_key(&self) -> EncryptedSecretKey {
        self.encrypted_secret_key
    }

    /// Returns if the secret can be decrypted in this workspace
    pub async fn can_be_decrypted(&self, ctx: &DalContext) -> SecretResult<bool> {
        let key = self.encrypted_secret_key;

        let Some(encrypted_secret) = EncryptedSecret::get_by_key(ctx, key).await? else {
            return Ok(false);
        };

        match encrypted_secret.key_pair(ctx).await {
            Ok(_) => Ok(true),
            Err(SecretError::KeyPair(KeyPairError::KeyPairNotFound(_)))
            | Err(SecretError::KeyPair(KeyPairError::UnauthorizedKeyAccess)) => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// Attach a [`Secret`] to a given [`AttributeValue`] corresponding to a
    /// "/root/secrets/\<secret\>" [`Prop`](crate::Prop).
    ///
    /// - If a [`Secret`] has already been attached to the given [`AttributeValue`], the existing
    ///   attachment will be replaced with a new one.
    /// - If no [`Secret`] is provided, then the [`AttributeValue`] will be updated to its original
    ///   state via [`AttributeValue::use_default_prototype`].
    ///
    /// This method will enqueue
    /// [`DependentValuesUpdate`](crate::job::definition::DependentValuesUpdate).
    pub async fn attach_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        secret_id: Option<SecretId>,
    ) -> SecretResult<()> {
        // First, check if the caller would like to unset the secret.
        let secret_id = match secret_id {
            Some(provided_secret_id) => provided_secret_id,
            None => {
                // We use the default prototype here rather than passing "None" to
                // "AttributeValue::update" because we neither want nor need existing prototype
                // arguments and the user cannot override the prototype for values corresponding
                // to a given "/root/secrets/<secret>".
                AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
                return Ok(());
            }
        };

        // Cache the variables we need for creating the attachment.
        let func_argument_name = "identity";
        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;

        // Create a new prototype and replace the existing one. If we are updating which secret is
        // used for the provided attribute value, this will remove the attribute prototype argument
        // that uses the previous secret. Despite having the ability to update the attribute
        // prototype argument value source, we need to re-create the prototype every time this
        // method is called. Why? We cannot be certain that the existing prototype and its arguments
        // (0 to N) will be what we need.
        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;
        AttributeValue::set_component_prototype_id(
            ctx,
            attribute_value_id,
            attribute_prototype.id(),
            None,
        )
        .await?;

        // Create an attribute prototype argument whose value source is the secret provided. Then,
        // enqueue dependent values update with the secret.
        let func_argument = FuncArgument::find_by_name_for_func(ctx, func_argument_name, func_id)
            .await?
            .ok_or(SecretError::FuncArgumentNotFound(
                func_id,
                func_argument_name.to_owned(),
            ))?;
        AttributePrototypeArgument::new(ctx, attribute_prototype.id(), func_argument.id, secret_id)
            .await?;
        ctx.add_dependent_values_and_enqueue(vec![secret_id])
            .await?;

        Ok(())
    }

    /// Gets the [`Secret`] with a given [`SecretId`]. If a [`Secret`] is not found, then return an
    /// [`error`](SecretError).
    ///
    /// _Note:_ this does not contain the encrypted or sensitive bits and is safe for external use.
    pub async fn get_by_id(ctx: &DalContext, id: SecretId) -> SecretResult<Self> {
        let (secret_node_weight, hash) =
            Self::get_node_weight_and_content_hash_or_error(ctx, id).await?;

        let content: SecretContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick): if we had a v2, then there would be migration logic here.
        let SecretContent::V1(inner) = content;

        Ok(Self::assemble(secret_node_weight, inner))
    }

    async fn get_node_weight_and_content_hash_or_error(
        ctx: &DalContext,
        id: SecretId,
    ) -> SecretResult<(SecretNodeWeight, ContentHash)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let node_weight = workspace_snapshot.get_node_weight(id).await?;

        let hash = node_weight.content_hash();
        let secret_node_weight = node_weight.get_secret_node_weight()?;
        Ok((secret_node_weight, hash))
    }

    /// Prepares the serialized payload for prototype execution for a given [`SecretId`](Secret).
    ///
    /// The corresponding [`EncryptedSecretKey`] will be converted to a string and then serialized
    /// as JSON.
    pub async fn payload_for_prototype_execution(
        ctx: &DalContext,
        secret_id: SecretId,
    ) -> SecretResult<Value> {
        let secret = Self::get_by_id(ctx, secret_id).await?;
        Ok(serde_json::to_value(
            secret.encrypted_secret_key.to_string(),
        )?)
    }

    /// Deserializes the value contained in an [`AttributeValue`] corresponding to a secret. The
    /// value will be a string, which then needs to be parsed to get the [`EncryptedSecretKey`].
    pub fn key_from_value_in_attribute_value(value: Value) -> SecretResult<EncryptedSecretKey> {
        let deserialized: String = serde_json::from_value(value)?;
        let key = EncryptedSecretKey::from_str(&deserialized)?;
        Ok(key)
    }

    /// Find all [`AttributeValues`](AttributeValue) that _directly_ depend on the [`Secret`]
    /// corresponding to the provided [`SecretId`](Secret).
    pub async fn direct_dependent_attribute_values(
        ctx: &DalContext,
        secret_id: SecretId,
    ) -> SecretResult<Vec<AttributeValueId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let attribute_prototype_argument_indices = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                secret_id,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            )
            .await?;

        let mut attribute_value_ids =
            Vec::with_capacity(attribute_prototype_argument_indices.len());
        for attribute_prototype_argument_index in attribute_prototype_argument_indices {
            let attribute_prototype_argument_node_weight = workspace_snapshot
                .get_node_weight(attribute_prototype_argument_index)
                .await?
                .get_attribute_prototype_argument_node_weight()?;
            let attribute_prototype_id = AttributePrototypeArgument::prototype_id(
                ctx,
                attribute_prototype_argument_node_weight.id().into(),
            )
            .await?;
            attribute_value_ids.extend(
                AttributePrototype::attribute_value_ids(ctx, attribute_prototype_id).await?,
            );
        }

        Ok(attribute_value_ids)
    }

    /// Assemble an object that maps all [`keys`](EncryptedSecretKey) to their corresponding
    /// [`SecretIds`](Secret).
    #[instrument(level = "debug", skip(ctx))]
    pub async fn list_ids_by_key(
        ctx: &DalContext,
    ) -> SecretResult<HashMap<EncryptedSecretKey, SecretId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let secret_category_node_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Secret)
            .await?;

        let indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                secret_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut id_by_key = HashMap::new();
        for index in indices {
            let secret_node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_secret_node_weight()?;
            id_by_key.insert(
                secret_node_weight.encrypted_secret_key().to_owned(),
                secret_node_weight.id().into(),
            );
        }

        Ok(id_by_key)
    }

    /// Get the [`SecretId`] corresponding to a given [`key`](EncryptedSecretKey).
    #[instrument(level = "debug", skip(ctx))]
    pub async fn get_id_by_key_or_error(
        ctx: &DalContext,
        key: EncryptedSecretKey,
    ) -> SecretResult<SecretId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let secret_category_node_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Secret)
            .await?;

        let indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                secret_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        for index in indices {
            let secret_node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_secret_node_weight()?;
            if secret_node_weight.encrypted_secret_key() == key {
                return Ok(secret_node_weight.id().into());
            }
        }

        Err(SecretError::NotFoundForKey)
    }

    /// Lists all [`Secrets`](Secret) in the current [`snapshot`](crate::WorkspaceSnapshot).
    #[instrument(name = "list_secrets", level = "debug", skip_all, fields(secret_count = Empty))]
    pub async fn list(ctx: &DalContext) -> SecretResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut secrets = vec![];
        let secret_category_node_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Secret)
            .await?;

        let secret_node_ids = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                secret_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let span = Span::current();
        span.record("secret_count", secret_node_ids.len());

        let mut secret_node_weights = vec![];
        let mut hashes = vec![];
        for id in secret_node_ids {
            let secret_node_weight = workspace_snapshot
                .get_node_weight(id)
                .await?
                .get_secret_node_weight()?;
            hashes.push(secret_node_weight.content_hash());
            secret_node_weights.push(secret_node_weight);
        }

        let contents: HashMap<ContentHash, SecretContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(hashes.as_slice())
            .await?;

        for secret_node_weight in secret_node_weights {
            match contents.get(&secret_node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick): if we had a v2, then there would be migration logic here.
                    let SecretContent::V1(inner) = content;

                    secrets.push(Self::assemble(secret_node_weight, inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    secret_node_weight.id(),
                ))?,
            }
        }

        Ok(secrets)
    }

    /// Updates the metadata for the [`Secret`], but not the encrypted contents.
    pub async fn update_metadata(
        self,
        ctx: &DalContext,
        name: impl Into<String>,
        description: Option<String>,
    ) -> SecretResult<Self> {
        self.modify(ctx, |s| {
            s.name = name.into();
            s.description = description;
            Ok(())
        })
        .await
    }

    /// Updates the underlying encrypted contents by generating a new key and inserting a new
    /// [`EncryptedSecret`].
    pub async fn update_encrypted_contents(
        self,
        ctx: &DalContext,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<Self> {
        // Generate a new key and insert a new encrypted secret.
        let new_key = Self::generate_key(ctx, self.id).await?;

        // NOTE(nick): we do not clean up the existing encrypted secret yet.
        EncryptedSecret::insert(ctx, new_key, crypted, key_pair_pk, version, algorithm).await?;

        // Since we are updating encrypted contents, we have a new key and need to enqueue ourselves
        // into dependent values update.
        ctx.add_dependent_values_and_enqueue(vec![self.id]).await?;

        self.modify(ctx, |s| {
            s.encrypted_secret_key = new_key;
            Ok(())
        })
        .await
    }

    /// Finds all secret prop ids for all schema variants
    #[instrument(name = "find_secret_prop_ids", level = "debug", skip_all)]
    pub async fn list_all_secret_prop_ids(ctx: &DalContext) -> SecretResult<Vec<PropId>> {
        let mut result = vec![];
        for variant_id in SchemaVariant::list_all_ids(ctx).await? {
            let secrets_prop =
                Prop::find_prop_by_path(ctx, variant_id, &RootPropChild::Secrets.prop_path())
                    .await?;
            result.extend(Prop::direct_child_prop_ids_ordered(ctx, secrets_prop.id).await?);
        }

        Ok(result)
    }

    /// Finds all secret prop ids for all schema variants
    #[instrument(name = "find_secret_prop_ids", level = "debug", skip_all)]
    pub async fn list_all_secret_prop_ids_for_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SecretResult<Vec<PropId>> {
        let secrets_prop =
            Prop::find_prop_by_path(ctx, schema_variant_id, &RootPropChild::Secrets.prop_path())
                .await?;
        Ok(Prop::direct_child_prop_ids_ordered(ctx, secrets_prop.id).await?)
    }

    /// Finds all the connected component Ids for the [`Secret`]
    #[instrument(name = "find_connected_components", level = "debug", skip_all)]
    pub async fn find_connected_components(
        self,
        ctx: &DalContext,
        prefetched_secret_props: Option<&[PropId]>,
    ) -> SecretResult<Vec<ComponentId>> {
        let secret_details = self.encrypted_secret_key().to_string();
        let mut connected_components = Vec::new();

        // This is a trick to make the borrow checker allow us to return a
        // reference to the list constructed in the list_all_secret_prop_ids
        //  call, so that we don't have to clone the variants passed in as
        // `prefetched_secret_props`.
        let secret_props;

        for &secret_prop_id in match prefetched_secret_props {
            Some(variants) => variants,
            None => {
                secret_props = Self::list_all_secret_prop_ids(ctx).await?;
                secret_props.as_slice()
            }
        } {
            let all_connected_attribute_values =
                Prop::all_attribute_values_everywhere_for_prop_id(ctx, secret_prop_id).await?;
            for connected_av in all_connected_attribute_values {
                let av = AttributeValue::get_by_id(ctx, connected_av).await?;
                if let Some(val) = av.value(ctx).await? {
                    if val == secret_details {
                        let connected_component =
                            AttributeValue::component_id(ctx, connected_av).await?;
                        connected_components.push(connected_component)
                    }
                }
            }
        }
        Ok(connected_components)
    }

    /// Deletes the secret node.
    pub async fn delete(self, ctx: &DalContext) -> SecretResult<()> {
        //TODO: Doesn't delete from layer_db or memory at this time!
        ctx.workspace_snapshot()?.remove_node_by_id(self.id).await?;

        Ok(())
    }

    async fn modify<L>(self, ctx: &DalContext, lambda: L) -> SecretResult<Self>
    where
        L: FnOnce(&mut Self) -> SecretResult<()>,
    {
        let mut secret = self;

        // NOTE(nick): I don't love the current timestamp and actor system. These likely shouldn't
        // be in the contents, but abstracted out into another service. Because of this, we have to
        // manually ensure that the actor and timestamp information is correct, regardless of what
        // the user passes in as the lambda.
        let before = SecretContentV1::from(secret.clone());
        lambda(&mut secret)?;
        if before != SecretContentV1::from(secret.clone()) {
            match ctx.history_actor() {
                HistoryActor::SystemInit => {}
                HistoryActor::User(id) => {
                    secret.updated_by = Some(*id);
                }
            }
            secret.timestamp.updated_at = Utc::now();
        }

        let (mut secret_node_weight, _) =
            Self::get_node_weight_and_content_hash_or_error(ctx, secret.id).await?;
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If the encrypted secret key has changed, we end up updating the secret node twice because
        // we always update the actor and timestamp data if anything has changed. This could be
        // optimized to do it only once.
        if secret.encrypted_secret_key() != secret_node_weight.encrypted_secret_key() {
            secret_node_weight.set_encrypted_secret_key(secret.encrypted_secret_key);

            workspace_snapshot
                .add_or_replace_node(NodeWeight::Secret(secret_node_weight.clone()))
                .await?;
        }
        let updated = SecretContentV1::from(secret.clone());

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(SecretContent::V1(updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            ctx.workspace_snapshot()?
                .update_content(secret.id.into(), hash)
                .await?;
        }

        Ok(Self::assemble(secret_node_weight, updated))
    }
}

/// The [`EncryptedSecret`] corresponding to an individual [`Secret`]. It contains sensitive, but
/// encrypted, information.
#[derive(Clone, Deserialize, Serialize)]
pub struct EncryptedSecret {
    user: Option<UserPk>,
    version: SecretVersion,
    algorithm: SecretAlgorithm,
    key_hash: Hash,
    key_pair_pk: KeyPairPk,
    // TODO(nick): confirm that base64 de/ser works as intended.
    #[serde(with = "nonce_serde")]
    nonce: SymmetricNonce,
    // TODO(nick): confirm that base64 de/ser works as intended.
    #[serde(with = "base64_bytes_serde")]
    crypted: Vec<u8>,
}

impl fmt::Debug for EncryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedSecret")
            .field("user", &self.user)
            .field("version", &self.version)
            .field("algorithm", &self.algorithm)
            .field("key_hash", &self.key_hash)
            .finish_non_exhaustive()
    }
}

impl EncryptedSecret {
    /// This _private_ method is used by [`Secret`] during its creation or when the user wishes
    /// to "change" its corresponding encrypted contents.
    async fn insert(
        ctx: &DalContext,
        key: EncryptedSecretKey,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<()> {
        let user = match ctx.history_actor() {
            HistoryActor::SystemInit => None,
            HistoryActor::User(user_pk) => Some(*user_pk),
        };

        let (double_crypted, nonce, key_hash) = ctx.symmetric_crypto_service().encrypt(crypted);

        let value = Self {
            user,
            key_pair_pk,
            nonce,
            key_hash: key_hash.to_owned(),
            crypted: double_crypted,
            version,
            algorithm,
        };

        ctx.layer_db().encrypted_secret().write(
            key,
            Arc::new(value),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        Ok(())
    }

    /// Gets the [`EncryptedSecret`] with a given key.
    ///
    /// _Warning:_ sensitive, but encrypted, contents will be returned.
    pub async fn get_by_key(
        ctx: &DalContext,
        key: EncryptedSecretKey,
    ) -> SecretResult<Option<Self>> {
        Ok(ctx.layer_db().encrypted_secret().try_read_as(&key).await?)
    }

    /// Decrypts the encrypted secret with its associated [`KeyPair`] and returns a
    /// [`DecryptedSecret`].
    pub async fn decrypt(self, ctx: &DalContext) -> SecretResult<DecryptedSecret> {
        let key_pair = self.key_pair(ctx).await?;

        self.into_decrypted(
            key_pair.public_key(),
            key_pair.secret_key(),
            ctx.symmetric_crypto_service(),
        )
    }

    fn into_decrypted(
        self,
        pkey: &PublicKey,
        skey: &SecretKey,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> SecretResult<DecryptedSecret> {
        // Explicitly match on (version, algorithm) tuple to ensure that any new
        // versions/algorithms will trigger a compilation failure
        match (self.version, self.algorithm) {
            (SecretVersion::V1, SecretAlgorithm::Sealedbox) => {
                let symmetric_decrypted =
                    symmetric_crypto_service.decrypt(&self.crypted, &self.nonce, &self.key_hash)?;

                let message = serde_json::from_slice(
                    &sealedbox::open(&symmetric_decrypted, pkey, skey)
                        .map_err(|_| SecretError::DecryptionFailed)?,
                )
                .map_err(SecretError::DeserializeMessage)?;

                Ok(DecryptedSecret { message })
            }
        }
    }

    /// Gets the [`KeyPair`] corresponding to the [`KeyPairPk`] on the [`EncryptedSecret`].
    pub async fn key_pair(&self, ctx: &DalContext) -> SecretResult<KeyPair> {
        Ok(KeyPair::get_by_pk(ctx, self.key_pair_pk).await?)
    }
}

/// This type corresponds to a secret that has been decrypted. It is returned by calling
/// [`EncryptedSecret::decrypt`], which contains the raw decrypted message, and without the
/// encrypted payload and other metadata. It is not persist-able and is only intended to be used
/// internally when passing secrets through to "cyclone".
///
/// _Note:_ we're being a bit careful here as to which traits are derived in an effort to minimize
/// leaking sensitive data.
#[derive(Serialize)]
pub struct DecryptedSecret {
    message: Value,
}

impl DecryptedSecret {
    pub(crate) fn message(&self) -> SensitiveContainer<Value> {
        self.message.clone().into()
    }
}

impl fmt::Debug for DecryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecryptedSecret").finish_non_exhaustive()
    }
}

impl From<AttributeValueError> for SecretError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for SecretError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for SecretError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for SecretError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for SecretError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for SecretError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for SecretError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for SecretError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::crypto::box_;

    use super::*;

    fn encrypted_secret(
        crypted: impl Into<Vec<u8>>,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> EncryptedSecret {
        let crypted = crypted.into();
        let (double_crypted, nonce, key_hash) = symmetric_crypto_service.encrypt(crypted.as_ref());

        EncryptedSecret {
            user: None,
            key_pair_pk: KeyPairPk::NONE,
            nonce,
            key_hash: *key_hash,
            crypted: double_crypted,
            version: Default::default(),
            algorithm: Default::default(),
        }
    }

    fn crypt<T>(value: &T, pkey: &PublicKey) -> Vec<u8>
    where
        T: ?Sized + Serialize,
    {
        sealedbox::seal(
            &serde_json::to_vec(value).expect("failed to serialize value"),
            pkey,
        )
    }

    #[test]
    fn into_decrypted() {
        sodiumoxide::init().expect("crypto failed to init");
        let (pkey, skey) = box_::gen_keypair();

        let message =
            serde_json::json!({"username": "The Cadillac Three", "password": "Slow Rollin"});
        let crypted = crypt(&message, &pkey);

        let service = SymmetricCryptoService::new(SymmetricCryptoService::generate_key(), vec![]);

        let encrypted = encrypted_secret(crypted, &service);
        let decrypted = encrypted
            .into_decrypted(&pkey, &skey, &service)
            .expect("could not decrypt secret");

        assert_eq!(message, decrypted.message);
    }
}
