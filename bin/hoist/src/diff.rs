use json_patch::jsonptr::{Assign, Pointer};
use json_patch::{Patch, PatchOperation};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

enum PatchTarget {
    Socket(String),
    Prop(String),
    PropContents((String, String)),
    Func(String),
    Variant,
}

impl Display for PatchTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchTarget::Socket(name) => {
                let split_name = name.split("-").collect::<Vec<_>>();
                let (Some(kind), Some(name)) = (split_name.first(), split_name.get(1)) else {
                    return Err(fmt::Error);
                };

                f.write_str(format!("{} socket {}", kind, name).as_str())
            }
            PatchTarget::Prop(name) => f.write_str(format!("prop {}", name).as_str()),
            PatchTarget::PropContents((name, target)) => {
                f.write_str(format!("prop contents {} at {}", name, target).as_str())
            }
            PatchTarget::Func(name) => f.write_str(format!("function {}", name).as_str()),
            PatchTarget::Variant => f.write_str("variant"),
        }?;
        Ok(())
    }
}

pub fn patch_list_to_changelog(patch: Patch) -> Vec<String> {
    let mut logs = vec![];
    for operation in patch.iter() {
        let path = operation.path();

        let target = if path.starts_with(Pointer::from_static("/funcs")) {
            let Some(name) = path.get(1) else {
                println!("Change directly to funcs object");
                continue;
            };

            PatchTarget::Func(name.to_string())
        } else if path.starts_with(Pointer::from_static("/schemas/0/variants/0/sockets")) {
            let Some(name) = path.get(5) else {
                println!("Change directly to sockets object");
                continue;
            };

            PatchTarget::Socket(name.to_string())
        } else if path.starts_with(Pointer::from_static("/schemas/0/variants/0")) {
            let Some(prop_root) = path.get(4) else {
                println!("Change directly to variant object");
                continue;
            };

            if ["domain", "secret", "resourceValue"].contains(&prop_root.to_string().as_str()) {
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

                if prop_name_tokens.len() == 1 {
                    println!("unhandled change");
                    continue;
                }

                let prop_name = format!("/root/{}", prop_name_tokens.join("/"));
                if !prop_contents.is_empty() {
                    PatchTarget::PropContents((prop_name, prop_contents.join("/")))
                } else {
                    PatchTarget::Prop(prop_name)
                }
            } else {
                PatchTarget::Variant
            }
        } else {
            panic!("Unhandled patch operation")
        };

        match operation {
            PatchOperation::Add(op) => {
                logs.push(format!(
                    "Added {target}:\n{}",
                    serde_json::to_string_pretty(&op.value).expect("unable to parse json")
                ));
            }
            PatchOperation::Remove(_) => logs.push(format!("Removed {target}")),
            PatchOperation::Replace(op) => {
                logs.push(format!(
                    "Replaced value within {target}:\n{}",
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

pub fn rewrite_spec_for_diff(spec: Value) -> Value {
    let mut spec = spec.clone();

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
            let name = func
                .get("name")
                .expect("could not get name")
                .as_str()
                .expect("kind should be a string");

            new_funcs.insert(name, func.clone());
        }
        spec.assign(
            Pointer::from_static("/funcs"),
            serde_json::to_value(new_funcs).expect("could not parse func to value"),
        )
        .expect("could not assign value");
    }

    spec
}
