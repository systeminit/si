use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt,
    fmt::{
        Display,
        Formatter,
    },
};

use json_patch::{
    Patch,
    PatchOperation,
    jsonptr::{
        Assign,
        Pointer,
    },
};
use serde_json::Value;

enum PatchTarget {
    Socket(String),
    SocketContents(String),
    Prop(String),
    PropContents((String, String)),
    Func(String),
    FuncBinding(String),
    Variant,
    VariantContent(String),
    Schema,
    SchemaContent(String),
}

impl Display for PatchTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTarget::Socket(name) => {
                let split_name = name.split("-").collect::<Vec<_>>();
                let (Some(kind), Some(name)) = (split_name.first(), split_name.get(1)) else {
                    return Err(fmt::Error);
                };

                f.write_str(format!("{kind} socket {name}").as_str())
            }
            PatchTarget::SocketContents(name) => {
                let split_name = name.split("-").collect::<Vec<_>>();
                let (Some(kind), Some(name)) = (split_name.first(), split_name.get(1)) else {
                    return Err(fmt::Error);
                };

                f.write_str(format!("contents of {kind} socket {name}").as_str())
            }
            PatchTarget::Prop(name) => f.write_str(format!("prop {name}").as_str()),
            PatchTarget::PropContents((name, target)) => {
                f.write_str(format!("contents of prop {name} at {target}").as_str())
            }
            PatchTarget::Func(name) => f.write_str(format!("function {name}").as_str()),
            PatchTarget::FuncBinding(kind) => f.write_str(format!("{kind} binding").as_str()),
            PatchTarget::Variant => f.write_str("variant"),
            PatchTarget::VariantContent(path) => {
                f.write_str(format!("variant content at {path}").as_str())
            }
            PatchTarget::Schema => f.write_str("schema"),
            PatchTarget::SchemaContent(path) => {
                f.write_str(format!("schema content at {path}").as_str())
            }
        }?;
        Ok(())
    }
}

pub fn patch_list_to_changelog(patch: Patch) -> Vec<String> {
    let mut logs = vec![];
    for operation in patch.iter() {
        let Some(target) = determine_patch_target(operation) else {
            continue;
        };

        match operation {
            PatchOperation::Add(op) => {
                // Skip null values in diff output
                if op.value.is_null() {
                    continue;
                }
                logs.push(format!(
                    "#### Added {target}:\n```\n{}\n```\n",
                    serde_json::to_string_pretty(&op.value).expect("unable to parse json")
                ));
            }
            PatchOperation::Remove(_) => logs.push(format!("Removed {target}")),
            PatchOperation::Replace(op) => {
                // Skip null values in diff output
                if op.value.is_null() {
                    continue;
                }
                logs.push(format!(
                    "#### Replaced value within {target}:\n```\n{}\n```\n",
                    serde_json::to_string_pretty(&op.value).expect("unable to parse json")
                ));
            }
            change @ (PatchOperation::Move(_)
            | PatchOperation::Copy(_)
            | PatchOperation::Test(_)) => println!("Unhandled Operation: \n{change}"),
        }
    }

    logs.sort();

    logs
}

pub struct ModificationSets {
    pub added: HashSet<String>,
    pub modified: HashSet<String>,
    pub removed: HashSet<String>,
    category: &'static str,
}

impl ModificationSets {
    fn new(category: &'static str) -> ModificationSets {
        ModificationSets {
            added: Default::default(),
            modified: Default::default(),
            removed: Default::default(),
            category,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.len() == 0 && self.removed.len() == 0 && self.modified.len() == 0
    }

    pub fn into_text_summary(self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut message = self.category.to_string();

        if !self.added.is_empty() {
            message = format!("{message} ➕{}", self.added.len());
        }
        if !self.removed.is_empty() {
            message = format!("{message} ➖{}", self.removed.len());
        }
        if !self.modified.is_empty() {
            message = format!("{message} 🔀{}", self.modified.len());
        }

        Some(message)
    }
}

pub fn patch_list_to_summary(
    asset_name: impl AsRef<str>,
    patch: Patch,
) -> (Option<String>, ModificationSets) {
    let mut sockets_set = ModificationSets::new("sockets");
    let mut props_set = ModificationSets::new("props");
    let mut asset_contents_set = ModificationSets::new("other schema contents");
    let funcs_set = ModificationSets::new("funcs");

    for operation in patch.iter() {
        let Some(target) = determine_patch_target(operation) else {
            continue;
        };

        match operation {
            PatchOperation::Add(_) => match target {
                PatchTarget::Socket(name) => {
                    sockets_set.added.insert(name);
                }
                PatchTarget::SocketContents(name) => {
                    sockets_set.modified.insert(name);
                }
                PatchTarget::Prop(name) => {
                    props_set.added.insert(name);
                }
                PatchTarget::PropContents((name, _)) => {
                    props_set.modified.insert(name);
                }
                PatchTarget::Func(_) => {}
                PatchTarget::FuncBinding(func_kind) => {
                    asset_contents_set.added.insert(func_kind);
                }
                PatchTarget::Variant => {}
                PatchTarget::VariantContent(path) => {
                    asset_contents_set.added.insert(path);
                }
                PatchTarget::Schema => {}
                PatchTarget::SchemaContent(path) => {
                    asset_contents_set.added.insert(path);
                }
            },
            PatchOperation::Remove(_) => match target {
                PatchTarget::Socket(name) => {
                    sockets_set.removed.insert(name);
                }
                PatchTarget::SocketContents(name) => {
                    sockets_set.modified.insert(name);
                }
                PatchTarget::Prop(name) => {
                    props_set.removed.insert(name);
                }
                PatchTarget::PropContents((name, _)) => {
                    props_set.modified.insert(name);
                }
                PatchTarget::Func(_) => {}
                PatchTarget::FuncBinding(func_kind) => {
                    asset_contents_set.removed.insert(func_kind);
                }
                PatchTarget::Variant => {}
                PatchTarget::VariantContent(path) => {
                    asset_contents_set.removed.insert(path);
                }
                PatchTarget::Schema => {}
                PatchTarget::SchemaContent(path) => {
                    asset_contents_set.removed.insert(path);
                }
            },
            PatchOperation::Replace(_) => match target {
                PatchTarget::Socket(name) => {
                    sockets_set.modified.insert(name);
                }
                PatchTarget::SocketContents(name) => {
                    sockets_set.modified.insert(name);
                }
                PatchTarget::Prop(name) => {
                    props_set.modified.insert(name);
                }
                PatchTarget::PropContents((name, _)) => {
                    props_set.modified.insert(name);
                }
                PatchTarget::Func(_) => {}
                PatchTarget::FuncBinding(func_kind) => {
                    asset_contents_set.modified.insert(func_kind);
                }
                PatchTarget::Variant => {}
                PatchTarget::VariantContent(path) => {
                    asset_contents_set.modified.insert(path);
                }
                PatchTarget::Schema => {}
                PatchTarget::SchemaContent(path) => {
                    asset_contents_set.modified.insert(path);
                }
            },
            PatchOperation::Move(_) | PatchOperation::Copy(_) | PatchOperation::Test(_) => {}
        }
    }

    let prop_summary = props_set.into_text_summary();
    let sockets_summary = sockets_set.into_text_summary();
    let variant_contents_summary = asset_contents_set.into_text_summary();

    let summaries = vec![prop_summary, sockets_summary, variant_contents_summary]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if summaries.is_empty() {
        return (None, funcs_set);
    }

    let summaries_text = summaries.join(", ");

    let message = format!("[{}]: {summaries_text}", asset_name.as_ref());

    (Some(message), funcs_set)
}

fn determine_patch_target(operation: &PatchOperation) -> Option<PatchTarget> {
    let path = operation.path();

    let target = if path.starts_with(Pointer::from_static("/funcs")) {
        let Some(name) = path.get(1) else {
            // println!("Change directly to funcs container");
            return None;
        };

        PatchTarget::Func(name.to_string())
    } else if path.starts_with(Pointer::from_static("/schemas/0/variants/0/sockets")) {
        let Some(name) = path.get(5) else {
            // println!("Change directly to sockets container");
            return None;
        };

        // We're in a field deeper than the socket itself
        if path.get(6).is_some() {
            PatchTarget::SocketContents(name.to_string())
        } else {
            PatchTarget::Socket(name.to_string())
        }
    } else if path.starts_with(Pointer::from_static("/schemas/0/variants/0")) {
        if let Some(prop_root) = path.get(4) {
            if ["domain", "secrets", "resourceValue"].contains(&prop_root.to_string().as_str()) {
                let mut prop_name_tokens = vec![prop_root.to_string()];

                let mut found_prop_root = false;
                let mut last_one_was_entries = false;
                let mut prop_contents = vec![];

                for token in path.tokens().map(|t| t.to_string()) {
                    if !found_prop_root {
                        if token == prop_root.to_string() {
                            found_prop_root = true
                        }
                    } else if !last_one_was_entries {
                        if token == "entries" && prop_contents.is_empty() {
                            last_one_was_entries = true;
                        } else {
                            prop_contents.push(token);
                        }
                    } else {
                        last_one_was_entries = false;
                        prop_name_tokens.push(token);
                    }
                }

                let prop_name = format!("/root/{}", prop_name_tokens.join("/"));
                if !prop_contents.is_empty() {
                    PatchTarget::PropContents((prop_name, prop_contents.join("/")))
                } else {
                    PatchTarget::Prop(prop_name)
                }
            } else if [
                "actionFuncs",
                "authFuncs",
                "leafFunctions",
                "siPropFuncs",
                "managementFuncs",
                "rootPropFuncs",
            ]
            .contains(&prop_root.to_string().as_str())
            {
                PatchTarget::FuncBinding(prop_root.to_string())
            } else {
                PatchTarget::VariantContent(path.to_string())
            }
        } else {
            PatchTarget::Variant
        }
    } else if path.starts_with(Pointer::from_static("/schemas/0")) {
        if let Some(prop_root) = path.get(2) {
            PatchTarget::SchemaContent(prop_root.to_string())
        } else {
            PatchTarget::Schema
        }
    } else {
        dbg!("Unhandled patch operation", operation);
        return None;
    };

    Some(target)
}

pub fn rewrite_spec_for_diff(spec: Value) -> Value {
    let mut spec = spec.clone();

    let module_name = spec
        .pointer("/name")
        .expect("could not get module name")
        .as_str()
        .expect("could not get module name as  str")
        .to_owned();

    let variant = spec
        .pointer_mut("/schemas/0/variants/0")
        .expect("Could not find variant");

    // Make sockets a record
    let mut new_sockets = HashMap::new();
    {
        let sockets = variant.get_mut("sockets").expect("could not get sockets");

        for socket in sockets.as_array().expect("could not get sockets as array") {
            let name = socket
                .get("name")
                .expect("could not get name")
                .as_str()
                .expect("kind should be a string");
            let kind = socket
                .pointer("/data/kind")
                .expect("Could not find kind")
                .as_str()
                .expect("kind should be a string");

            let key = format!("{kind}-{name}");
            new_sockets.insert(key, socket.clone());
        }
    }

    variant
        .assign(
            Pointer::from_static("/sockets"),
            serde_json::to_value(new_sockets).expect("sockets could not parse as json"),
        )
        .expect("could not assign value");

    // Rewrite the props
    {
        fn rewrite_prop(prop: &Value) -> (String, Value) {
            let name = prop
                .get("name")
                .expect("could not get name")
                .as_str()
                .expect("kind should be a string");

            let kind = prop
                .get("kind")
                .expect("Could not find kind")
                .as_str()
                .expect("kind should be a string");

            let mut prop = prop.clone();

            if !["array", "map", "object"].contains(&kind) {
                return (name.to_string(), prop);
            };

            if kind != "object" {
                let (_, type_prop) =
                    rewrite_prop(prop.get("typeProp").expect("could not get typeProp"));
                prop.assign(
                    Pointer::from_static("/typeProp"),
                    serde_json::to_value(type_prop).expect("couldn't make new entries into json"),
                )
                .expect("could not assign value");
                return (name.to_string(), prop);
            }

            let mut new_entries = HashMap::new();

            for entry in prop
                .get("entries")
                .unwrap_or_else(|| panic!("couldn't get entries for prop: {name}"))
                .as_array()
                .unwrap_or_else(|| panic!("entries field of {name} is not an array"))
            {
                let (entry_name, entry_prop) = rewrite_prop(entry);

                new_entries.insert(entry_name, entry_prop);
            }

            prop.assign(
                Pointer::from_static("/entries"),
                serde_json::to_value(new_entries).expect("couldn't make new entries into json"),
            )
            .expect("could not assign value");

            (name.to_string(), prop)
        }

        let (_, rewritten_domain) =
            rewrite_prop(variant.get("domain").expect("could not get domain"));
        let (_, rewritten_secrets) =
            rewrite_prop(variant.get("secrets").expect("could not get secrets"));
        let (_, rewritten_resource_value) = rewrite_prop(
            variant
                .get("resourceValue")
                .expect("could not get resourceValue"),
        );

        variant
            .assign(Pointer::from_static("/domain"), rewritten_domain)
            .expect("could not assign value");
        variant
            .assign(Pointer::from_static("/secrets"), rewritten_secrets)
            .expect("could not assign value");
        variant
            .assign(
                Pointer::from_static("/resourceValue"),
                rewritten_resource_value,
            )
            .expect("could not assign value");
    }

    // Rewrite Funcs
    {
        let mut new_funcs = HashMap::new();
        let funcs = spec
            .get("funcs")
            .expect("couldn't get funcs")
            .as_array()
            .expect("funcs field is not an array");

        for func in funcs {
            let func_name = func
                .get("name")
                .expect("could not get name")
                .as_str()
                .expect("kind should be a string");

            // Ignore asset func
            if func_name == module_name {
                continue;
            }

            new_funcs.insert(func_name, func.clone());
        }
        spec.assign(
            Pointer::from_static("/funcs"),
            serde_json::to_value(new_funcs).expect("could not parse func to value"),
        )
        .expect("could not assign value");
    }

    spec
}
