use std::collections::HashSet;

use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    IntoEnumIterator,
};
use url::Url;

use super::{
    ActionFuncSpec,
    LeafFunctionSpec,
    ManagementFuncSpec,
    PropSpec,
    PropSpecData,
    PropSpecWidgetKind,
    RootPropFuncSpec,
    SiPropFuncSpec,
    SocketSpec,
    SocketSpecData,
    SpecError,
};
use crate::{
    InputMismatchTruth,
    MergeSkip,
    PropSpecKind,
    SocketSpecKind,
    spec::authentication_func::AuthenticationFuncSpec,
};

#[remain::sorted]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub enum SchemaVariantSpecComponentType {
    #[serde(alias = "AggregationFrame")]
    #[strum(serialize = "AggregationFrame", serialize = "aggregationFrame")]
    AggregationFrame,
    #[default]
    #[serde(alias = "Component")]
    #[strum(serialize = "Component", serialize = "component")]
    Component,
    #[serde(alias = "ConfigurationFrameDown")]
    #[strum(
        serialize = "ConfigurationFrameDown",
        serialize = "configurationFrameDown",
        serialize = "ConfigurationFrame",
        serialize = "configurationFrame"
    )] // this was called ConfigurationFrame so we need to keep compatibility
    ConfigurationFrameDown,
    #[strum(serialize = "ConfigurationFrameUp", serialize = "configurationFrameUp")]
    ConfigurationFrameUp,
}

#[remain::sorted]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
pub enum SchemaVariantSpecPropRoot {
    Code,
    Domain,
    Qualification,
    Resource,
    ResourceValue,
    SecretDefinition,
    Secrets,
}

impl SchemaVariantSpecPropRoot {
    pub fn path_parts(&self) -> &'static [&'static str] {
        match self {
            Self::Domain => &["root", "domain"],
            Self::ResourceValue => &["root", "resource_value"],
            Self::Resource => &["root", "resource"],
            Self::SecretDefinition => &["root", "secret_definition"],
            Self::Secrets => &["root", "secrets"],
            Self::Code => &["root", "code"],
            Self::Qualification => &["root", "qualification"],
        }
    }

    pub fn maybe_from_str(leaf_name: &str) -> Option<SchemaVariantSpecPropRoot> {
        match leaf_name {
            "domain" => Some(SchemaVariantSpecPropRoot::Domain),
            "resource" => Some(SchemaVariantSpecPropRoot::Resource),
            "resource_value" => Some(SchemaVariantSpecPropRoot::ResourceValue),
            "secret_definition" => Some(SchemaVariantSpecPropRoot::SecretDefinition),
            "secrets" => Some(SchemaVariantSpecPropRoot::Secrets),
            "qualification" => Some(SchemaVariantSpecPropRoot::Qualification),
            "code" => Some(SchemaVariantSpecPropRoot::Code),
            _ => None,
        }
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpecData {
    #[builder(setter(into))]
    pub version: String,
    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,
    #[builder(setter(into, strip_option), default)]
    pub color: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub display_name: Option<String>,
    #[builder(setter(into), default)]
    pub component_type: SchemaVariantSpecComponentType,
    #[builder(setter(into))]
    pub func_unique_id: String,
    #[builder(setter(into), default)]
    pub description: Option<String>,
}

impl SchemaVariantSpecData {
    pub fn builder() -> SchemaVariantSpecDataBuilder {
        SchemaVariantSpecDataBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.func_unique_id = "".to_string();
        self.version = "".to_string();
    }
}

impl SchemaVariantSpecDataBuilder {
    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpec {
    #[builder(setter(into))]
    pub version: String,

    #[builder(setter(into, strip_option), default)]
    pub data: Option<SchemaVariantSpecData>,

    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub unique_id: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub is_builtin: bool,

    #[builder(setter(each(name = "action_func"), into), default)]
    pub action_funcs: Vec<ActionFuncSpec>,

    #[builder(setter(each(name = "auth_func"), into), default)]
    pub auth_funcs: Vec<AuthenticationFuncSpec>,

    #[builder(setter(each(name = "leaf_function"), into), default)]
    pub leaf_functions: Vec<LeafFunctionSpec>,

    #[builder(setter(each(name = "socket"), into), default)]
    pub sockets: Vec<SocketSpec>,

    #[builder(setter(each(name = "si_prop_func"), into), default)]
    pub si_prop_funcs: Vec<SiPropFuncSpec>,

    #[builder(setter(each(name = "management_func"), into), default)]
    pub management_funcs: Vec<ManagementFuncSpec>,

    #[builder(private, default = "Self::default_domain()")]
    pub domain: PropSpec,

    #[builder(private, default = "Self::default_secrets()")]
    pub secrets: PropSpec,

    #[builder(private, default)]
    pub secret_definition: Option<PropSpec>,

    #[builder(private, default = "Self::default_resource_value()")]
    pub resource_value: PropSpec,

    #[builder(setter(each(name = "root_prop_func"), into), default)]
    #[serde(default)]
    pub root_prop_funcs: Vec<RootPropFuncSpec>,
}

impl SchemaVariantSpec {
    pub fn builder() -> SchemaVariantSpecBuilder {
        SchemaVariantSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.version = String::new();
        self.unique_id = None;

        if let Some(ref mut data) = self.data {
            data.anonymize();
        }

        self.action_funcs.iter_mut().for_each(|f| f.anonymize());
        self.auth_funcs.iter_mut().for_each(|f| f.anonymize());
        self.leaf_functions.iter_mut().for_each(|f| f.anonymize());
        self.management_funcs.iter_mut().for_each(|f| f.anonymize());

        self.sockets.iter_mut().for_each(|f| f.anonymize());

        self.domain.anonymize();
        self.secrets.anonymize();
        if let Some(ref mut secret_def) = self.secret_definition {
            secret_def.anonymize()
        };
        self.resource_value.anonymize();
    }

    // This is only used when merging prototypes. If the structure of
    // resource/code/qualification changes, these have to be updated
    fn get_root_prop_for_merge(&self, root: SchemaVariantSpecPropRoot) -> Option<PropSpec> {
        // On the expects: A panic here means we used the builder wrong, and
        // should occur during tests
        let resource_spec = PropSpec::builder()
            .name("resource")
            .kind(PropSpecKind::Object)
            .entry(
                PropSpec::builder()
                    .kind(PropSpecKind::String)
                    .name("last_synced")
                    .build()
                    .expect("should build"),
            )
            .entry(
                PropSpec::builder()
                    .kind(PropSpecKind::String)
                    .name("status")
                    .build()
                    .expect("should build"),
            )
            .entry(
                PropSpec::builder()
                    .kind(PropSpecKind::Json)
                    .name("payload")
                    .build()
                    .expect("should build"),
            )
            .entry(
                PropSpec::builder()
                    .kind(PropSpecKind::Json)
                    .name("message")
                    .build()
                    .expect("should build"),
            )
            .build()
            .expect("should build");

        let qualification_spec = PropSpec::builder()
            .name("qualification")
            .kind(PropSpecKind::Map)
            .type_prop(
                PropSpec::builder()
                    .kind(PropSpecKind::Object)
                    .name("qualificationItem")
                    .entry(
                        PropSpec::builder()
                            .name("result")
                            .kind(PropSpecKind::String)
                            .build()
                            .expect("should build"),
                    )
                    .entry(
                        PropSpec::builder()
                            .name("message")
                            .kind(PropSpecKind::String)
                            .build()
                            .expect("should build"),
                    )
                    .build()
                    .expect("build prop"),
            )
            .build()
            .expect("should build");

        let code_spec = PropSpec::builder()
            .name("code")
            .kind(PropSpecKind::Map)
            .type_prop(
                PropSpec::builder()
                    .kind(PropSpecKind::Object)
                    .name("codeItem")
                    .entry(
                        PropSpec::builder()
                            .name("code")
                            .kind(PropSpecKind::String)
                            .build()
                            .expect("should build"),
                    )
                    .entry(
                        PropSpec::builder()
                            .name("format")
                            .kind(PropSpecKind::String)
                            .build()
                            .expect("should build"),
                    )
                    .build()
                    .expect("build prop"),
            )
            .build()
            .expect("should build");

        match root {
            SchemaVariantSpecPropRoot::Domain => Some(self.domain.to_owned()),
            SchemaVariantSpecPropRoot::ResourceValue => Some(self.resource_value.to_owned()),
            SchemaVariantSpecPropRoot::SecretDefinition => self.secret_definition.to_owned(),
            SchemaVariantSpecPropRoot::Secrets => Some(self.secrets.to_owned()),
            SchemaVariantSpecPropRoot::Resource => Some(resource_spec),
            SchemaVariantSpecPropRoot::Code => Some(code_spec),
            SchemaVariantSpecPropRoot::Qualification => Some(qualification_spec),
        }
    }

    fn input_sockets(&self) -> Vec<String> {
        self.sockets
            .iter()
            .filter_map(|socket_spec| match socket_spec.kind() {
                Some(SocketSpecKind::Input) => Some(socket_spec.name.to_owned()),
                _ => None,
            })
            .collect()
    }

    fn output_sockets(&self) -> Vec<String> {
        self.sockets
            .iter()
            .filter_map(|socket_spec| match socket_spec.kind() {
                Some(SocketSpecKind::Output) => Some(socket_spec.name.to_owned()),
                _ => None,
            })
            .collect()
    }

    fn make_fake_root_prop(&self) -> PropSpec {
        let mut root = PropSpec::builder();
        root.kind(PropSpecKind::Object).name("root");
        for root_prop_kind in SchemaVariantSpecPropRoot::iter() {
            if let Some(prop_spec) = self.get_root_prop_for_merge(root_prop_kind) {
                root.entry(prop_spec.clone());
            }
        }

        root.build().expect("failure to build is a logic error")
    }

    // This should be used after props are merged from other, so that the root
    // prop on self has all the currently existing props
    fn merge_sockets_from(
        &self,
        self_root: &PropSpec,
        other_spec: &Self,
        input_sockets: &[String],
        output_sockets: &[String],
    ) -> (Vec<SocketSpec>, Vec<MergeSkip>) {
        let mut merged_sockets = vec![];
        let mut merge_skips = vec![];

        let self_prop_map = self_root.build_prop_spec_index_map();

        for this_socket in &self.sockets {
            // Does this socket exist in other? If it doesn't, push it onto the merged sockets list
            // and continue.
            let Some(other_socket) = other_spec
                .sockets
                .iter()
                .find(|sock| sock.name == this_socket.name && sock.kind() == this_socket.kind())
            else {
                merged_sockets.push(this_socket.to_owned());
                continue;
            };

            let other_socket_maybe_func_unique_id = other_socket
                .data
                .as_ref()
                .and_then(|data| data.func_unique_id.to_owned());

            if let Some(other_socket_func_unique_id) = other_socket_maybe_func_unique_id {
                let new_merge_skips = PropSpec::get_input_mismatches(
                    &this_socket.name,
                    InputMismatchTruth::PropSpecMap(&self_prop_map),
                    other_socket.inputs.as_slice(),
                    &other_socket_func_unique_id,
                    input_sockets,
                    output_sockets,
                );
                // If there is nothing to skip, we still need to include the data from the newer this_socket
                // and copy it to the other_socket so we don't lose any changes here
                if new_merge_skips.is_empty() {
                    if let (Some(this_socket_data), Some(other_socket_data)) =
                        (this_socket.data.as_ref(), other_socket.data.as_ref())
                    {
                        // keep other_socket's func_unique_id, but everything else from the newer this_socket
                        let new_data = Some(SocketSpecData {
                            func_unique_id: other_socket_data.func_unique_id.clone(),
                            ..this_socket_data.clone()
                        });

                        merged_sockets.push(SocketSpec {
                            data: new_data,
                            ..other_socket.clone()
                        });
                    } else {
                        merged_sockets.push(other_socket.to_owned());
                    }
                } else {
                    merge_skips.extend(new_merge_skips.into_iter());
                    merged_sockets.push(this_socket.to_owned());
                }
            } else {
                merged_sockets.push(this_socket.to_owned());
            }
        }

        (merged_sockets, merge_skips)
    }

    /// Makes a best effort to merge the prototypes of `other_spec` into `self`,
    /// producing a new `SchemaVariantSpec`. The prop tree for `self` is taken as
    /// the source of truth. Props present in `other_spec` but not present in
    /// `self` are considered removed, and the types from `self` take priority.
    /// This works by walking the prop tree in self and trying to find the
    /// equivalent prop in other. If an equivalent prop in other is found, the
    /// attribute functions for that prop are copied over to self. Then, all
    /// other functions are copied over, if they have matching props (or
    /// sockets) in self.
    pub fn merge_prototypes_from(&self, other_spec: &Self) -> (Self, Vec<MergeSkip>) {
        let mut schema_variant_builder = SchemaVariantSpec::builder();
        schema_variant_builder.version(&self.version);
        schema_variant_builder.data = Some(self.data.clone());

        // These are the sockets as defined by the new asset (just their names)
        let self_input_sockets = self.input_sockets();
        let self_output_sockets = self.output_sockets();

        // The inputs to these prototypes will always be available so we just copy
        schema_variant_builder.action_funcs = Some(other_spec.action_funcs.clone());
        schema_variant_builder.auth_funcs = Some(other_spec.auth_funcs.clone());
        schema_variant_builder.leaf_functions = Some(other_spec.leaf_functions.clone());
        schema_variant_builder.management_funcs = Some(other_spec.management_funcs.clone());

        // These are fake root props that include all the "root prop children"
        // (domain, resource_value, etc) as entries
        let self_root_prop = self.make_fake_root_prop();
        let other_root_prop = other_spec.make_fake_root_prop();

        let (new_root, mut merge_skips) =
            self_root_prop.merge_with(&other_root_prop, &self_input_sockets, &self_output_sockets);

        // The prop spec is now merged, so we can use it for socket merges
        let (new_sockets, socket_merge_skips) = self.merge_sockets_from(
            &new_root,
            other_spec,
            &self_input_sockets,
            &self_output_sockets,
        );

        schema_variant_builder.replace_roots(new_root);
        schema_variant_builder.sockets(new_sockets);

        let missing_props: HashSet<String> = merge_skips
            .iter()
            .filter_map(|skip| match skip {
                MergeSkip::PropMissing(path) => Some(path.to_owned()),
                _ => None,
            })
            .collect();

        merge_skips.extend(socket_merge_skips);

        for si_prop_func in &other_spec.si_prop_funcs {
            let path = PropSpec::make_path(&si_prop_func.kind.prop_path(), None);
            let si_prop_skips = PropSpec::get_input_mismatches(
                &path,
                crate::InputMismatchTruth::MissingPropSet(&missing_props),
                &si_prop_func.inputs,
                &si_prop_func.func_unique_id,
                &self_input_sockets,
                &self_output_sockets,
            );

            if si_prop_skips.is_empty() {
                schema_variant_builder.si_prop_func(si_prop_func.to_owned());
            } else {
                merge_skips.extend(si_prop_skips);
            }
        }

        for root_prop_func in &other_spec.root_prop_funcs {
            let path = PropSpec::make_path(root_prop_func.prop.path_parts(), None);
            let root_prop_skips = PropSpec::get_input_mismatches(
                &path,
                crate::InputMismatchTruth::MissingPropSet(&missing_props),
                &root_prop_func.inputs,
                &root_prop_func.func_unique_id,
                &self_input_sockets,
                &self_output_sockets,
            );

            if root_prop_skips.is_empty() {
                schema_variant_builder.root_prop_func(root_prop_func.to_owned());
            } else {
                merge_skips.extend(root_prop_skips);
            }
        }

        merge_skips.extend(
            other_spec
                .input_sockets()
                .iter()
                .filter_map(|input_socket| {
                    if !self_input_sockets.contains(input_socket) {
                        Some(MergeSkip::InputSocketMissing {
                            socket_name: input_socket.to_owned(),
                        })
                    } else {
                        None
                    }
                }),
        );

        merge_skips.extend(
            other_spec
                .output_sockets()
                .iter()
                .filter_map(|output_socket| {
                    if !self_output_sockets.contains(output_socket) {
                        Some(MergeSkip::OutputSocketMissing {
                            socket_name: output_socket.to_owned(),
                        })
                    } else {
                        None
                    }
                }),
        );

        (
            schema_variant_builder.build().expect("should build"),
            merge_skips,
        )
    }
}

impl SchemaVariantSpecBuilder {
    // XXX: these need to take in a unique_id
    fn default_domain() -> PropSpec {
        PropSpec::Object {
            name: "domain".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "domain".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
                ui_optionals: Default::default(),
            }),
            entries: vec![],
        }
    }

    fn default_secrets() -> PropSpec {
        PropSpec::Object {
            name: "secrets".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "secrets".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
                ui_optionals: Default::default(),
            }),
            entries: vec![],
        }
    }

    fn default_secret_definition() -> Option<PropSpec> {
        Some(PropSpec::Object {
            name: "secret_definition".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "secret_definition".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
                ui_optionals: Default::default(),
            }),
            entries: vec![],
        })
    }

    fn default_resource_value() -> PropSpec {
        PropSpec::Object {
            name: "resource_value".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "resource_value".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
                ui_optionals: Default::default(),
            }),
            entries: vec![],
        }
    }

    pub fn domain_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::Domain, item)
    }

    pub fn secret_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::Secrets, item)
    }

    pub fn secret_definition_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::SecretDefinition, item)
    }

    pub fn resource_value_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::ResourceValue, item)
    }

    pub fn replace_roots(&mut self, new_root: PropSpec) -> &mut Self {
        for child in new_root.direct_children() {
            match SchemaVariantSpecPropRoot::maybe_from_str(child.name()) {
                Some(SchemaVariantSpecPropRoot::Domain) => {
                    self.domain = Some(child.to_owned());
                }
                Some(SchemaVariantSpecPropRoot::ResourceValue) => {
                    self.resource_value = Some(child.to_owned());
                }
                Some(SchemaVariantSpecPropRoot::SecretDefinition) => {
                    self.secret_definition = Some(Some(child.to_owned()));
                }
                Some(SchemaVariantSpecPropRoot::Secrets) => {
                    self.secrets = Some(child.to_owned());
                }
                Some(SchemaVariantSpecPropRoot::Resource)
                | Some(SchemaVariantSpecPropRoot::Code)
                | Some(SchemaVariantSpecPropRoot::Qualification)
                | None => {}
            }
        }

        self
    }

    #[allow(unused_mut)]
    pub fn prop(
        &mut self,
        root: SchemaVariantSpecPropRoot,
        item: impl Into<PropSpec>,
    ) -> &mut Self {
        let converted: PropSpec = item.into();

        let maybe_root = match root {
            SchemaVariantSpecPropRoot::Domain => {
                Some(self.domain.get_or_insert_with(Self::default_domain))
            }
            SchemaVariantSpecPropRoot::ResourceValue => Some(
                self.resource_value
                    .get_or_insert_with(Self::default_resource_value),
            ),
            SchemaVariantSpecPropRoot::SecretDefinition => Some(
                self.secret_definition
                    .get_or_insert_with(Self::default_secret_definition)
                    .as_mut()
                    .expect("secret_definition was created with Some(...)"),
            ),
            SchemaVariantSpecPropRoot::Secrets => {
                Some(self.secrets.get_or_insert_with(Self::default_secrets))
            }
            SchemaVariantSpecPropRoot::Resource
            | SchemaVariantSpecPropRoot::Code
            | SchemaVariantSpecPropRoot::Qualification => None,
        };

        if let Some(converted_root) = maybe_root {
            match converted_root {
                PropSpec::Object { entries, .. } => entries.push(converted),
                invalid => unreachable!(
                    "{:?} prop under root should be Object but was found to be: {:?}",
                    root, invalid
                ),
            }
        }

        self
    }

    #[allow(unused_mut)]
    pub fn try_prop<I>(
        &mut self,
        root: SchemaVariantSpecPropRoot,
        item: I,
    ) -> Result<&mut Self, I::Error>
    where
        I: TryInto<PropSpec>,
    {
        let converted: PropSpec = item.try_into()?;
        Ok(self.prop(root, converted))
    }

    #[allow(unused_mut)]
    pub fn props(&mut self, value: Vec<PropSpec>) -> &mut Self {
        match self.domain.get_or_insert_with(Self::default_domain) {
            PropSpec::Object { entries, .. } => *entries = value,
            invalid => unreachable!(
                "domain prop is an object but was found to be: {:?}",
                invalid
            ),
        };
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActionFuncSpecKind,
        AttrFuncInputSpec,
        LeafInputLocation,
        LeafKind,
    };

    #[test]
    fn test_schema_variant_merge() {
        let mercedes_dantes_beloved_path =
            PropSpec::make_path(&["root", "domain", "mercedes", "dantes_beloved"], None);
        let si_name_path = PropSpec::make_path(&["root", "si", "name"], None);

        let sv_1 = SchemaVariantSpec::builder()
            .version("v0")
            .unique_id("monte_cristo_sv_1")
            .data(
                SchemaVariantSpecData::builder()
                    .version("v0")
                    .color("#ffff00")
                    .func_unique_id("cristo_1")
                    .build()
                    .expect("build variant spec data"),
            )
            .domain_prop(
                PropSpec::builder()
                    .name("edmond_dantes")
                    .kind(PropSpecKind::String)
                    .func_unique_id("set_edmond_dantes")
                    .input(
                        AttrFuncInputSpec::builder()
                            .kind(crate::AttrFuncInputSpecKind::Prop)
                            .name("beloved")
                            .prop_path(mercedes_dantes_beloved_path.clone())
                            .build()
                            .expect("func input"),
                    )
                    .build()
                    .expect("build prop"),
            )
            .domain_prop(
                PropSpec::builder()
                    .name("mercedes")
                    .kind(PropSpecKind::Object)
                    .entry(
                        PropSpec::builder()
                            .name("dantes_beloved")
                            .kind(PropSpecKind::Boolean)
                            .func_unique_id("set_dantes_beloved")
                            .input(
                                AttrFuncInputSpec::builder()
                                    .kind(crate::AttrFuncInputSpecKind::Prop)
                                    .name("si_name")
                                    .prop_path(si_name_path.clone())
                                    .build()
                                    .expect("beloved input from si name"),
                            )
                            .build()
                            .expect("build prop"),
                    )
                    .entry(
                        PropSpec::builder()
                            .name("mother_of")
                            .kind(PropSpecKind::Object)
                            .entry(
                                PropSpec::builder()
                                    .name("albert_de_morcef")
                                    .kind(PropSpecKind::Number)
                                    .build()
                                    .expect("brop puild"),
                            )
                            .build()
                            .expect("prop build"),
                    )
                    .build()
                    .expect("build prop"),
            )
            .action_func(
                ActionFuncSpec::builder()
                    .kind(ActionFuncSpecKind::Create)
                    .func_unique_id("create_monte_cristo")
                    .build()
                    .expect("action func spec"),
            )
            .action_func(
                ActionFuncSpec::builder()
                    .kind(ActionFuncSpecKind::Refresh)
                    .func_unique_id("refresh_monte_cristo")
                    .build()
                    .expect("action func spec"),
            )
            .leaf_function(
                LeafFunctionSpec::builder()
                    .func_unique_id("qualify_cristo")
                    .leaf_kind(LeafKind::Qualification)
                    .inputs(vec![LeafInputLocation::Domain])
                    .build()
                    .expect("build leaf func"),
            )
            .build()
            .expect("build sv");

        let sv_2 = SchemaVariantSpec::builder()
            .version("v1")
            .unique_id("monte_cristo_sv_1")
            .data(
                SchemaVariantSpecData::builder()
                    .version("v0")
                    .color("#ffff00")
                    .func_unique_id("cristo_2")
                    .build()
                    .expect("build variant spec data"),
            )
            .domain_prop(
                PropSpec::builder()
                    .name("edmond_dantes")
                    .kind(PropSpecKind::String)
                    .build()
                    .expect("build prop"),
            )
            .domain_prop(
                PropSpec::builder()
                    .name("mercedes")
                    .kind(PropSpecKind::Object)
                    .entry(
                        PropSpec::builder()
                            .name("dantes_beloved")
                            .kind(PropSpecKind::Boolean)
                            .build()
                            .expect("build prop"),
                    )
                    .build()
                    .expect("build prop"),
            )
            .build()
            .expect("build sv");

        let (merged_sv, skips) = sv_2.merge_prototypes_from(&sv_1);

        assert_eq!(
            &[
                MergeSkip::PropMissing(PropSpec::make_path(
                    &["root", "domain", "mercedes", "mother_of"],
                    None
                )),
                MergeSkip::PropMissing(PropSpec::make_path(
                    &[
                        "root",
                        "domain",
                        "mercedes",
                        "mother_of",
                        "albert_de_morcef"
                    ],
                    None
                ))
            ],
            skips.as_slice()
        );

        assert_eq!(2, merged_sv.action_funcs.len());
        assert_eq!(1, merged_sv.leaf_functions.len());

        let edmond_dantes_path = PropSpec::make_path(&["domain", "edmond_dantes"], None);

        let dantes_prop_spec = merged_sv
            .domain
            .build_prop_spec_index_map()
            .get(&edmond_dantes_path)
            .expect("should exist in the map")
            .0;

        assert_eq!(Some("set_edmond_dantes"), dantes_prop_spec.func_unique_id());
        assert_eq!(
            Some(mercedes_dantes_beloved_path.as_str()),
            dantes_prop_spec
                .inputs()
                .expect("has inputs")
                .iter()
                .next()
                .expect("has an input")
                .prop_path()
        );

        let mercedes_path_under_domain =
            PropSpec::make_path(&["domain", "mercedes", "dantes_beloved"], None);
        let beloved_spec = merged_sv
            .domain
            .build_prop_spec_index_map()
            .get(&mercedes_path_under_domain)
            .expect("should exist in the map")
            .0;

        assert_eq!(Some("set_dantes_beloved"), beloved_spec.func_unique_id());
        assert_eq!(
            Some(si_name_path.as_str()),
            beloved_spec
                .inputs()
                .expect("has inputs")
                .iter()
                .next()
                .expect("has an input")
                .prop_path()
        );
    }
}
