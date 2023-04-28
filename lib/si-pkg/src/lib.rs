pub(crate) mod node;
mod pkg;
mod spec;

pub use pkg::{
    SiPkg, SiPkgAction, SiPkgAttrFuncInput, SiPkgAttrFuncInputView, SiPkgError, SiPkgFunc,
    SiPkgLeafFunction, SiPkgMetadata, SiPkgProp, SiPkgSchema, SiPkgSchemaVariant, SiPkgSocket,
    SiPkgValidation, SiPkgWorkflow,
};
pub use spec::{
    ActionSpec, ActionSpecBuilder, ActionSpecKind, AttrFuncInputSpec, AttrFuncInputSpecKind,
    FuncArgumentKind, FuncArgumentSpec, FuncArgumentSpecBuilder, FuncSpec, FuncSpecBackendKind,
    FuncSpecBackendResponseType, FuncUniqueId, LeafFunctionSpec, LeafFunctionSpecBuilder,
    LeafInputLocation, LeafKind, PkgSpec, PkgSpecBuilder, PropSpec, PropSpecBuilder, PropSpecKind,
    SchemaSpec, SchemaSpecBuilder, SchemaVariantSpec, SchemaVariantSpecBuilder,
    SchemaVariantSpecComponentType, SocketSpec, SocketSpecArity, SocketSpecKind, SpecError,
    ValidationSpec, ValidationSpecKind, WorkflowSpec, WorkflowSpecBuilder,
};

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use petgraph::dot::Dot;

    use crate::spec::PkgSpec;

    use super::*;

    use tokio::sync::Mutex;

    const PACKAGE_JSON: &str = include_str!("../pkg-complex.json");

    fn base_path() -> PathBuf {
        let base_path = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir cannot be computed"),
        )
        .join("pkg");
        fs::create_dir_all(&base_path).expect("failed to create base path");

        base_path
    }

    pub async fn prop_visitor(
        prop: SiPkgProp<'_>,
        _parent_id: Option<()>,
        context: &Mutex<Vec<String>>,
    ) -> Result<Option<()>, SiPkgError> {
        context.lock().await.push(prop.name().to_string());

        Ok(None)
    }

    #[tokio::test]
    async fn create_pkg() {
        let spec: PkgSpec = serde_json::from_str(PACKAGE_JSON).unwrap();
        dbg!(&spec);

        let pkg = SiPkg::load_from_spec(spec).expect("failed to load spec");

        let (graph, _root_idx) = pkg.as_petgraph();

        let funcs = pkg.funcs().expect("failed to get funcs");
        assert_eq!(2, funcs.len());

        // println!("{}", serde_json::to_string_pretty(&graph).unwrap());

        println!("\n---- snip ----\n{:?}\n---- snip ----", Dot::new(graph));
    }

    #[tokio::test]
    async fn pkg_fs_round_trip() {
        let spec: PkgSpec = serde_json::from_str(PACKAGE_JSON).unwrap();
        let pkg = SiPkg::load_from_spec(spec).expect("failed to load spec");

        pkg.write_to_dir(base_path())
            .await
            .expect("failed to write pkg to dir");

        let read_pkg = SiPkg::load_from_dir(base_path())
            .await
            .expect("failed to load pkg from dir");

        let funcs = read_pkg.funcs().expect("failed to get funcs");
        assert_eq!(2, funcs.len());

        let truthy_func = funcs.get(0).expect("failed to get first func");
        assert_eq!("si:truthy", truthy_func.name());
        let args = truthy_func.arguments().expect("failed to get arguments");
        assert_eq!(6, args.len());
        let arg3 = args.get(2).expect("arg3 exists");
        assert_eq!("map_value", arg3.name());
        assert_eq!(FuncArgumentKind::Map, arg3.kind());
        assert_eq!(Some(&FuncArgumentKind::Object), arg3.element_kind());

        let falsey_func = funcs.get(1).expect("failed to get second func");
        assert_eq!("si:falsey", falsey_func.name());

        let variant = read_pkg
            .schemas()
            .expect("get schema")
            .pop()
            .expect("has schema")
            .variants()
            .expect("get variants")
            .pop()
            .expect("has a variant");

        let funcs_by_unique_id = read_pkg
            .funcs_by_unique_id()
            .expect("cannot get funcs by unique id");

        let leaf_funcs = variant.leaf_functions().expect("get leaf funcs");
        assert_eq!(3, leaf_funcs.len());

        for func in leaf_funcs {
            assert!(funcs_by_unique_id.contains_key(&func.func_unique_id()));
            match func.leaf_kind() {
                LeafKind::Qualification => {
                    assert_eq!(
                        vec![LeafInputLocation::Domain, LeafInputLocation::Code],
                        func.inputs()
                    )
                }
                LeafKind::Confirmation => {
                    assert_eq!(
                        vec![LeafInputLocation::Resource, LeafInputLocation::DeletedAt],
                        func.inputs()
                    )
                }
                LeafKind::CodeGeneration => {
                    assert_eq!(vec![LeafInputLocation::Domain], func.inputs())
                }
            }
        }

        // Ensure we get the props
        let props: Mutex<Vec<String>> = Mutex::new(Vec::new());
        variant
            .visit_prop_tree(prop_visitor, None, &props)
            .await
            .expect("able to visit prop tree");

        // k8s deployments are really complex!
        assert_eq!(123, props.lock().await.len());

        let _ = dbg!(props.lock().await);
    }
}
