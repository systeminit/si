//! Create a [`SchemaVariant`](crate::SchemaVariant) with a [`Prop`](crate::Prop) tree via a
//! "definition" struct, usually provided via a "definition" file.

use async_recursion::async_recursion;
use serde::Deserialize;
use std::collections::HashMap;

use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{DalContext, Prop, PropId, PropKind, RootProp, SchemaId, SchemaVariant, StandardModel};

/// A cache of [`PropIds`](crate::Prop) where the _key_ is a tuple corresponding to the
/// [`Prop`](crate::Prop) name and the _parent_ [`PropId`](crate::Prop) who's child is the
/// [`PropId`](crate::Prop) in the _value_ of the entry.
///
/// It is recommended to start with the [`RootProp`](crate::RootProp) in order to descend into the
/// cache.
#[derive(Debug, Clone)]
pub struct PropCache(HashMap<(String, PropId), PropId>);

impl PropCache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Attempts to retrieve the [`PropId`](crate::Prop) value for a given [`Prop`](crate::Prop)
    /// name and parent [`PropId`](crate::Prop) key tuple. An error is returned if nothing is found.
    pub fn get(
        &self,
        prop_name: impl AsRef<str>,
        parent_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        // NOTE(nick): the string handling could probably be better here.
        let prop_name = prop_name.as_ref().to_string();
        let prop_id = *self.0.get(&(prop_name.clone(), parent_prop_id)).ok_or(
            SchemaVariantError::PropNotFoundInCache(prop_name, parent_prop_id),
        )?;
        Ok(prop_id)
    }

    /// Insert the [`PropId`](crate::Prop) into [`self`](Self). The returned `option` from the
    /// underlying method is ignored.
    pub fn insert(&mut self, key: (String, PropId), value: PropId) {
        self.0.insert(key, value);
    }
}

impl Default for PropCache {
    fn default() -> Self {
        Self::new()
    }
}

/// The definition for a [`SchemaVariant`](crate::SchemaVariant)'s [`Prop`](crate::Prop) tree (and
/// more in the future).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinition {
    /// The name of the [`SchemaVariant`](crate::SchemaVariant). This is _not_ the name of the
    /// [`Schema`](crate::Schema) and will default to "v0" if empty.
    name: Option<String>,
    /// A map of documentation links to reference. To reference links (values) specify the key via
    /// the "doc_link_ref" field for a [`PropDefinition`].
    doc_links: HashMap<String, String>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/domain".
    #[serde(default)]
    props: Vec<PropDefinition>,
}

/// The definition for a [`Prop`](crate::Prop) in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropDefinition {
    /// The name of the [`Prop`](crate::Prop) to be created.
    name: String,
    /// The [`kind`](crate::PropKind) of the [`Prop`](crate::Prop) to be created.
    kind: PropKind,
    /// An optional reference to a documentation link in the "doc_links" field for the
    /// [`SchemaVariantDefinition`] for the [`Prop`](crate::Prop) to be created.
    doc_link_ref: Option<String>,
    /// An optional documentation link for the [`Prop`](crate::Prop) to be created.
    doc_link: Option<String>,
    /// If our [`kind`](crate::PropKind) is [`Object`](crate::PropKind::Object), specify the
    /// child definition(s).
    #[serde(default)]
    children: Vec<PropDefinition>,
    /// If our [`kind`](crate::PropKind) is [`Array`](crate::PropKind::Array), specify the entry
    /// definition.
    entry: Option<Box<PropDefinition>>,
}

impl SchemaVariant {
    /// Create a [`SchemaVariant`] like [`usual`](Self::new()), but use the
    /// [`SchemaVariantDefinition`] to create a [`Prop`](crate::Prop) tree as well with a
    /// [`cache`](PropCache).
    pub async fn new_with_definition(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_definition: SchemaVariantDefinition,
    ) -> SchemaVariantResult<(Self, RootProp, PropCache)> {
        let name = match schema_variant_definition.name {
            Some(name) => name,
            None => "v0".to_string(),
        };
        let (schema_variant, root_prop) = Self::new(ctx, schema_id, name).await?;

        // NOTE(nick): allow users to use a definition without props... just in case, I guess.
        let mut prop_cache = PropCache::new();
        for prop_definition in schema_variant_definition.props {
            Self::walk_definition(
                ctx,
                &mut prop_cache,
                prop_definition,
                root_prop.domain_prop_id,
                &schema_variant_definition.doc_links,
            )
            .await?;
        }
        Ok((schema_variant, root_prop, prop_cache))
    }

    /// A recursive walk of [`PropDefinition`] that populates the [`cache`](PropCache) as each
    /// [`Prop`](crate::Prop) is created.
    #[async_recursion]
    async fn walk_definition(
        ctx: &DalContext,
        prop_cache: &mut PropCache,
        definition: PropDefinition,
        parent_prop_id: PropId,
        doc_links: &HashMap<String, String>,
    ) -> SchemaVariantResult<()> {
        // Start by creating the prop and setting the parent. We cache the id for later.
        let mut prop = Prop::new(ctx, definition.name.clone(), definition.kind, None).await?;
        prop.set_parent_prop(ctx, parent_prop_id).await?;
        let prop_id = *prop.id();

        // Always cache the prop that was created.
        prop_cache.insert((prop.name().to_string(), parent_prop_id), prop_id);

        // Either use the doc link or the doc link ref. Do not use both.
        match (definition.doc_link.is_some(), definition.doc_link_ref) {
            (true, Some(_)) => {
                return Err(SchemaVariantError::MultipleDocLinksProvided(
                    definition.name.clone(),
                ));
            }
            (true, None) => prop.set_doc_link(ctx, definition.doc_link).await?,
            (false, Some(doc_link_ref)) => match doc_links.get(&doc_link_ref) {
                Some(link) => prop.set_doc_link(ctx, Some(link)).await?,
                None => return Err(SchemaVariantError::LinkNotFoundForDocLinkRef(doc_link_ref)),
            },
            (false, None) => {}
        }

        // Determine if we need to descend and check the "entry" and "children" fields accordingly.
        match definition.kind {
            PropKind::Object => {
                if definition.entry.is_some() {
                    return Err(SchemaVariantError::FoundEntryForObject(
                        definition.name.clone(),
                    ));
                }
                if definition.children.is_empty() {
                    return Err(SchemaVariantError::MissingChildrenForObject(
                        definition.name.clone(),
                    ));
                }
                for child in definition.children {
                    Self::walk_definition(ctx, prop_cache, child, prop_id, doc_links).await?;
                }
            }
            PropKind::Array => match definition.entry {
                Some(entry) => {
                    if !definition.children.is_empty() {
                        return Err(SchemaVariantError::FoundChildrenForArray(
                            definition.name.clone(),
                        ));
                    }
                    Self::walk_definition(ctx, prop_cache, *entry, prop_id, doc_links).await?;
                }
                None => {
                    return Err(SchemaVariantError::MissingEntryForArray(
                        definition.name.clone(),
                    ));
                }
            },
            PropKind::Map => todo!("maps not yet implemented simply because nick didn't need them yet and didn't want an untested solution"),
            _ => match (definition.entry.is_none(), definition.children.is_empty()) {
                (false, false) => {
                    return Err(SchemaVariantError::FoundChildrenAndEntryForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (false, true) => {
                    return Err(SchemaVariantError::FoundEntryForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (true, false) => {
                    return Err(SchemaVariantError::FoundChildrenForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (true, true) => {}
            },
        }

        Ok(())
    }
}
