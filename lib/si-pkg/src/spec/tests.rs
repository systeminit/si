use super::*;

#[test]
fn test_leaf_function_spec_has_unique_id_some() {
    let spec = LeafFunctionSpec {
        func_unique_id: "func123".to_string(),
        unique_id: Some("unique456".to_string()),
        leaf_kind: LeafKind::CodeGeneration,
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), Some("unique456"));
}

#[test]
fn test_leaf_function_spec_has_unique_id_none() {
    let spec = LeafFunctionSpec {
        func_unique_id: "func123".to_string(),
        unique_id: None,
        leaf_kind: LeafKind::CodeGeneration,
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), None);
}

#[test]
fn test_action_func_spec_has_unique_id_some() {
    let spec = ActionFuncSpec {
        func_unique_id: "action123".to_string(),
        unique_id: Some("unique789".to_string()),
        name: Some("Test Action".to_string()),
        kind: ActionFuncSpecKind::Create,
        deleted: false,
    };
    assert_eq!(spec.unique_id(), Some("unique789"));
}

#[test]
fn test_action_func_spec_has_unique_id_none() {
    let spec = ActionFuncSpec {
        func_unique_id: "action123".to_string(),
        unique_id: None,
        name: Some("Test Action".to_string()),
        kind: ActionFuncSpecKind::Create,
        deleted: false,
    };
    assert_eq!(spec.unique_id(), None);
}

#[test]
fn test_authentication_func_spec_has_unique_id_some() {
    let spec = AuthenticationFuncSpec {
        func_unique_id: "auth123".to_string(),
        unique_id: Some("unique456".to_string()),
        name: Some("Test Auth".to_string()),
        deleted: false,
    };
    assert_eq!(spec.unique_id(), Some("unique456"));
}

#[test]
fn test_authentication_func_spec_has_unique_id_none() {
    let spec = AuthenticationFuncSpec {
        func_unique_id: "auth123".to_string(),
        unique_id: None,
        name: Some("Test Auth".to_string()),
        deleted: false,
    };
    assert_eq!(spec.unique_id(), None);
}

#[test]
fn test_si_prop_func_spec_has_unique_id_some() {
    let spec = SiPropFuncSpec {
        kind: SiPropFuncSpecKind::Name,
        func_unique_id: "si_prop123".to_string(),
        unique_id: Some("unique789".to_string()),
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), Some("unique789"));
}

#[test]
fn test_si_prop_func_spec_has_unique_id_none() {
    let spec = SiPropFuncSpec {
        kind: SiPropFuncSpecKind::Name,
        func_unique_id: "si_prop123".to_string(),
        unique_id: None,
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), None);
}

#[test]
fn test_root_prop_func_spec_has_unique_id_some() {
    use crate::SchemaVariantSpecPropRoot;

    let spec = RootPropFuncSpec {
        prop: SchemaVariantSpecPropRoot::Domain,
        func_unique_id: "root_prop123".to_string(),
        unique_id: Some("unique456".to_string()),
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), Some("unique456"));
}

#[test]
fn test_root_prop_func_spec_has_unique_id_none() {
    use crate::SchemaVariantSpecPropRoot;

    let spec = RootPropFuncSpec {
        prop: SchemaVariantSpecPropRoot::Domain,
        func_unique_id: "root_prop123".to_string(),
        unique_id: None,
        deleted: false,
        inputs: vec![],
    };
    assert_eq!(spec.unique_id(), None);
}

#[test]
fn test_management_func_spec_always_has_unique_id() {
    let spec = ManagementFuncSpec {
        func_unique_id: "mgmt789".to_string(),
        name: "test".to_string(),
        description: None,
    };
    // Management functions always return Some since they use func_unique_id
    assert_eq!(spec.unique_id(), Some("mgmt789"));
}

#[test]
fn test_management_func_spec_with_description() {
    let spec = ManagementFuncSpec {
        func_unique_id: "mgmt456".to_string(),
        name: "test management".to_string(),
        description: Some("Test description".to_string()),
    };
    assert_eq!(spec.unique_id(), Some("mgmt456"));
}
