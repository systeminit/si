
use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use si_pkg::PropSpecKind;

use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};
use telemetry::prelude::*;



use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{
    label_list::ToLabelList,
    pk,
    property_editor::schema::WidgetKind, FuncId, StandardModel, Timestamp,
};
use crate::{FuncBackendResponseType};

pub const PROP_VERSION: PropContentDiscriminants = PropContentDiscriminants::V1;

pk!(PropId);

/// An individual "field" within the tree of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Prop {
    pub id: PropId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The name of the [`Prop`].
    pub name: String,
    /// The kind of the [`Prop`].
    pub kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    pub widget_kind: WidgetKind,
    /// The configuration of the "widget".
    pub widget_options: Option<Value>,
    /// A link to external documentation for working with this specific [`Prop`].
    pub doc_link: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    pub hidden: bool,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    pub refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    pub diff_func_id: Option<FuncId>,
}

#[derive(Debug, PartialEq)]
pub struct PropGraphNode {
    id: PropId,
    content_address: ContentAddress,
    content: PropContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum PropContent {
    V1(PropContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PropContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The name of the [`Prop`].
    pub name: String,
    /// The kind of the [`Prop`].
    pub kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    pub widget_kind: WidgetKind,
    /// The configuration of the "widget".
    pub widget_options: Option<Value>,
    /// A link to external documentation for working with this specific [`Prop`].
    pub doc_link: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    pub hidden: bool,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    pub refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    pub diff_func_id: Option<FuncId>,
}

impl Prop {
    pub fn assemble(id: PropId, inner: &PropContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name.clone(),
            kind: inner.kind,
            widget_kind: inner.widget_kind,
            widget_options: inner.widget_options.clone(),
            doc_link: inner.doc_link.clone(),
            hidden: inner.hidden,
            refers_to_prop_id: inner.refers_to_prop_id,
            diff_func_id: inner.diff_func_id,
        }
    }

    pub fn id(&self) -> PropId {
        self.id
    }
}

impl From<Prop> for PropContentV1 {
    fn from(value: Prop) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name.clone(),
            kind: value.kind,
            widget_kind: value.widget_kind,
            widget_options: value.widget_options.clone(),
            doc_link: value.doc_link.clone(),
            hidden: value.hidden,
            refers_to_prop_id: value.refers_to_prop_id,
            diff_func_id: value.diff_func_id,
        }
    }
}

impl PropGraphNode {
    pub fn assemble(
        id: impl Into<PropId>,
        content_hash: ContentHash,
        content: PropContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::Prop(content_hash),
            content,
        }
    }

    pub fn id(&self) -> PropId {
        self.id
    }
}

/// This is the separator used for the "path" column. It is a vertical tab character, which should
/// not (we'll see) be able to be provided by our users in [`Prop`] names.
pub const PROP_PATH_SEPARATOR: &str = "\x0B";

/// This type should be used to manage prop paths instead of a raw string
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropPath(String);

impl PropPath {
    pub fn new<S>(parts: impl IntoIterator<Item = S>) -> Self
    where
        S: AsRef<str>,
    {
        Self(
            parts
                .into_iter()
                .map(|part| part.as_ref().to_owned())
                .collect::<Vec<String>>()
                .join(PROP_PATH_SEPARATOR),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_owned_parts(&self) -> Vec<String> {
        self.0.split(PROP_PATH_SEPARATOR).map(Into::into).collect()
    }

    pub fn join(&self, path: &PropPath) -> Self {
        Self::new([self.as_str(), path.as_str()])
    }

    pub fn with_replaced_sep(&self, sep: &str) -> String {
        self.0.to_owned().replace(PROP_PATH_SEPARATOR, sep)
    }
}

impl AsRef<str> for PropPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for PropPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<PropPath> for String {
    fn from(value: PropPath) -> Self {
        value.0
    }
}

impl From<&String> for PropPath {
    fn from(value: &String) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for PropPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// const ALL_ANCESTOR_PROPS: &str = include_str!("queries/prop/all_ancestor_props.sql");
// const FIND_ROOT_PROP_FOR_PROP: &str = include_str!("queries/prop/root_prop_for_prop.sql");
// const FIND_PROP_IN_TREE: &str = include_str!("queries/prop/find_prop_in_tree.sql");

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PropKind {
    Array,
    Boolean,
    Integer,
    Map,
    Object,
    String,
}

impl From<PropKind> for PropSpecKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::String => Self::String,
            PropKind::Integer => Self::Number,
            PropKind::Object => Self::Object,
            PropKind::Map => Self::Map,
        }
    }
}

impl ToLabelList for PropKind {}

impl From<PropKind> for WidgetKind {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Checkbox,
            PropKind::String | PropKind::Integer => Self::Text,
            PropKind::Object => Self::Header,
            PropKind::Map => Self::Map,
        }
    }
}

impl From<PropKind> for FuncBackendResponseType {
    fn from(prop: PropKind) -> Self {
        match prop {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::Integer => Self::Integer,
            PropKind::Object => Self::Object,
            PropKind::Map => Self::Map,
            PropKind::String => Self::String,
        }
    }
}

// impl Prop {
//     standard_model_accessor!(name, String, PropResult);
//     standard_model_accessor!(kind, Enum(PropKind), PropResult);
//     standard_model_accessor!(widget_kind, Enum(WidgetKind), PropResult);
//     standard_model_accessor!(widget_options, Option<Value>, PropResult);
//     standard_model_accessor!(doc_link, Option<String>, PropResult);
//     standard_model_accessor!(hidden, bool, PropResult);
//     standard_model_accessor!(refers_to_prop_id, Option<Pk(PropId)>, PropResult);
//     standard_model_accessor!(diff_func_id, Option<Pk(FuncId)>, PropResult);
//     standard_model_accessor!(schema_variant_id, Pk(SchemaVariantId), PropResult);

//     pub fn path(&self) -> PropPath {
//         self.path.to_owned().into()
//     }

//     // TODO(nick): replace this table with a foreign key relationship.
//     standard_model_belongs_to!(
//         lookup_fn: parent_prop,
//         set_fn: set_parent_prop_do_not_use,
//         unset_fn: unset_parent_prop_do_not_use,
//         table: "prop_belongs_to_prop",
//         model_table: "props",
//         belongs_to_id: PropId,
//         returns: Prop,
//         result: PropResult,
//     );

//     // TODO(nick): replace this table with a foreign key relationship.
//     standard_model_has_many!(
//         lookup_fn: child_props,
//         table: "prop_belongs_to_prop",
//         model_table: "props",
//         returns: Prop,
//         result: PropResult,
//     );

//     pub async fn find_root_prop_for_prop(
//         ctx: &DalContext,
//         prop_id: PropId,
//     ) -> PropResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_ROOT_PROP_FOR_PROP,
//                 &[ctx.tenancy(), ctx.visibility(), &prop_id],
//             )
//             .await?;

//         Ok(standard_model::object_option_from_row_option::<Self>(row)?)
//     }

//     /// Returns the given [`Prop`] and all ancestor [`Props`](crate::Prop) back to the root.
//     /// Ancestor props are ordered by depth, starting from the root prop.
//     pub async fn all_ancestor_props(ctx: &DalContext, prop_id: PropId) -> PropResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 ALL_ANCESTOR_PROPS,
//                 &[ctx.tenancy(), ctx.visibility(), &prop_id],
//             )
//             .await?;
//         Ok(objects_from_rows(rows)?)
//     }

//     #[instrument(skip_all)]
//     #[async_recursion]
//     pub async fn ts_type(&self, ctx: &DalContext) -> PropResult<String> {
//         // XXX: Hack! The payload prop kind is a string but we're actually storing arbitrary json
//         // there and expect it to be a JSON object in most of our code. However, the resource_value
//         // work is likely to remove the need for this entirely
//         if self.path() == PropPath::new(["root", "resource", "payload"]) {
//             return Ok("any".to_string());
//         }

//         // Note: we should fix this by having propper enums as prop types
//         if self.path() == PropPath::new(["root", "resource", "status"]) {
//             return Ok("'ok' | 'warning' | 'error' | undefined | null".to_owned());
//         }

//         Ok(match self.kind() {
//             PropKind::Array => format!(
//                 "{}[]",
//                 self.child_props(ctx)
//                     .await?
//                     .get(0)
//                     .ok_or(PropError::ArrayMissingElementChild(self.id))?
//                     .ts_type(ctx)
//                     .await?
//             ),
//             PropKind::Boolean => "boolean".to_string(),
//             PropKind::Integer => "number".to_string(),
//             PropKind::Map => format!(
//                 "Record<string, {}>",
//                 self.child_props(ctx)
//                     .await?
//                     .get(0)
//                     .ok_or(PropError::MapMissingElementChild(self.id))?
//                     .ts_type(ctx)
//                     .await?
//             ),
//             PropKind::Object => {
//                 let mut object_type = "{\n".to_string();
//                 for child in self.child_props(ctx).await? {
//                     let name_value = serde_json::to_value(&child.name)?;
//                     let name_serialized = serde_json::to_string(&name_value)?;
//                     object_type.push_str(
//                         format!(
//                             "{}: {} | null | undefined;\n",
//                             &name_serialized,
//                             child.ts_type(ctx).await?
//                         )
//                         .as_str(),
//                     );
//                 }
//                 object_type.push('}');

//                 object_type
//             }
//             PropKind::String => "string".to_string(),
//         })
//     }

//     /// Assembles the "json_pointer" representing the full "path" to a [`Prop`] based on its
//     /// lineage.
//     ///
//     /// For examples, if a [`Prop`] named "poop" had a parent named "domain" and a grandparent named
//     /// "root", then the "json_pointer" would be "/root/domain/poop".
//     pub async fn json_pointer(&self, ctx: &DalContext) -> PropResult<String> {
//         // NOTE(nick,zack): if this ends up getting used frequently to manage paths corresponding
//         // to attribute (and/or property editor) values, then we should consider strongly typing
//         // "json_pointer".
//         Ok([
//             "/".to_string(),
//             Prop::all_ancestor_props(ctx, *self.id())
//                 .await?
//                 .iter()
//                 .map(|prop| prop.name().to_string())
//                 .collect::<Vec<String>>()
//                 .join("/"),
//         ]
//         .join(""))
//     }

//     /// Finds a prop by a path made up of prop names separated by
//     /// [`PROP_PATH_SEPARATOR`](crate::prop::PROP_PATH_SEPARATOR) for each depth level
//     pub async fn find_prop_by_path(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         path: &PropPath,
//     ) -> PropResult<Self> {
//         Self::find_prop_by_path_opt(ctx, schema_variant_id, path)
//             .await?
//             .ok_or(PropError::NotFoundAtPath(
//                 path.to_string(),
//                 *ctx.visibility(),
//             ))
//     }

//     /// Finds a prop by a path made up of prop names separated by
//     /// [`PROP_PATH_SEPARATOR`](crate::prop::PROP_PATH_SEPARATOR) for each depth level
//     pub async fn find_prop_by_path_opt(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         path: &PropPath,
//     ) -> PropResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_PROP_IN_TREE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &schema_variant_id,
//                     &path.as_str(),
//                 ],
//             )
//             .await?;

//         Ok(object_option_from_row_option(row)?)
//     }

//     pub async fn set_default_value<T: Serialize>(
//         &self,
//         ctx: &DalContext,
//         value: T,
//     ) -> PropResult<()> {
//         let value = serde_json::to_value(value)?;
//         match self.kind() {
//             PropKind::String | PropKind::Boolean | PropKind::Integer => {
//                 let attribute_read_context = AttributeReadContext::default_with_prop(self.id);
//                 let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
//                     .await?
//                     .ok_or(AttributeValueError::NotFoundForReadContext(
//                         attribute_read_context,
//                     ))?;
//                 let parent_attribute_value = attribute_value
//                     .parent_attribute_value(ctx)
//                     .await?
//                     .ok_or_else(|| AttributeValueError::ParentNotFound(*attribute_value.id()))?;

//                 // Ensure the parent project is an object. Technically, we should ensure that every
//                 // prop in entire lineage is of kind object, but this should (hopefully) suffice
//                 // for now. Ideally, this would be handled in a query.
//                 let parent_prop = Prop::get_by_id(ctx, &parent_attribute_value.context.prop_id())
//                     .await?
//                     .ok_or_else(|| {
//                         PropError::NotFound(
//                             parent_attribute_value.context.prop_id(),
//                             *ctx.visibility(),
//                         )
//                     })?;
//                 if parent_prop.kind() != &PropKind::Object {
//                     return Err(PropError::ParentPropIsNotObjectForPropWithDefaultValue(
//                         *parent_prop.kind(),
//                     ));
//                 }

//                 let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
//                 AttributeValue::update_for_context(
//                     ctx,
//                     *attribute_value.id(),
//                     Some(*parent_attribute_value.id()),
//                     context,
//                     Some(value),
//                     None,
//                 )
//                 .await?;
//                 Ok(())
//             }
//             _ => Err(PropError::SetDefaultForNonScalar(*self.kind())),
//         }
//     }

//     pub async fn set_default_diff(&mut self, ctx: &DalContext) -> PropResult<()> {
//         let func = Func::find_by_attr(ctx, "name", &"si:diff")
//             .await?
//             .pop()
//             .ok_or(PropError::DefaultDiffFunctionNotFound)?;
//         self.set_diff_func_id(ctx, Some(*func.id())).await
//     }
// }
