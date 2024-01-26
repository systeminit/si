pub(crate) mod node;
mod pkg;
mod spec;

pub use pkg::*;
pub use spec::*;

#[cfg(test)]
mod tests {
    use petgraph::dot::Dot;
    use tokio::sync::Mutex;

    use crate::spec::PkgSpec;

    use super::*;

    const PACKAGE_JSON: &str = include_str!("../pkg-complex.json");
    const WORKSPACE_JSON: &str = include_str!("../pkg-workspace.json");

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
    async fn pkg_workspace_round_trip() {
        let spec: PkgSpec = serde_json::from_str(WORKSPACE_JSON).unwrap();
        let description = spec.description.to_owned();
        let pkg = SiPkg::load_from_spec(spec).expect("failed to load spec");

        let pkg_data = pkg.write_to_bytes().expect("failed to serialize pkg");

        let read_pkg = SiPkg::load_from_bytes(pkg_data).expect("failed to load pkg from bytes");
        let metadata = read_pkg.metadata().expect("Get metadata (WorkspaceBackup)");

        assert_eq!(description, metadata.description());

        assert_eq!(SiPkgKind::WorkspaceBackup, metadata.kind());

        assert_eq!(Some("head"), metadata.default_change_set());

        let change_sets = read_pkg.change_sets().expect("able to get change_sets");
        assert_eq!(2, change_sets.len());

        assert_eq!(
            "head",
            change_sets.first().expect("get first change set").name()
        );
    }

    #[tokio::test]
    async fn pkg_bytes_round_trip() {
        let spec: PkgSpec = serde_json::from_str(PACKAGE_JSON).unwrap();
        let description = spec.description.to_owned();
        let pkg = SiPkg::load_from_spec(spec).expect("failed to load spec");

        let pkg_data = pkg.write_to_bytes().expect("failed to serialize pkg");

        let read_pkg = SiPkg::load_from_bytes(pkg_data).expect("failed to load pkg from bytes");

        assert_eq!(
            description,
            read_pkg.metadata().expect("get metadata").description()
        );

        assert_eq!(
            SiPkgKind::Module,
            read_pkg.metadata().expect("get metadata for kind").kind()
        );

        let funcs = read_pkg.funcs().expect("failed to get funcs");
        assert_eq!(2, funcs.len());

        let truthy_func = funcs.first().expect("failed to get first func");
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
        assert_eq!(2, leaf_funcs.len());

        for func in leaf_funcs {
            assert!(funcs_by_unique_id.contains_key(&func.func_unique_id().to_string()));
            match func.leaf_kind() {
                LeafKind::Qualification => {
                    assert_eq!(
                        vec![LeafInputLocation::Domain, LeafInputLocation::Code],
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
            .visit_prop_tree(
                SchemaVariantSpecPropRoot::Domain,
                prop_visitor,
                None,
                &props,
            )
            .await
            .expect("able to visit prop tree");

        // k8s deployments are really complex!
        assert_eq!(123, props.lock().await.len());

        let _ = dbg!(props.lock().await);
    }
}
