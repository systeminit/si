pub(crate) mod node;
mod pkg;
mod spec;

pub use pkg::{
    SiPkg, SiPkgError, SiPkgFunc, SiPkgMetadata, SiPkgProp, SiPkgSchema, SiPkgSchemaVariant,
};
pub use spec::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, PkgSpec, PkgSpecBuilder, PropSpec,
    PropSpecBuilder, PropSpecKind, QualificationSpec, QualificationSpecBuilder, SchemaSpec,
    SchemaSpecBuilder, SchemaVariantSpec, SchemaVariantSpecBuilder, SpecError,
};

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf, sync::Arc};

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
        context: &Arc<Mutex<Vec<String>>>,
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
        let falsey_func = funcs.get(1).expect("failed to get second func");
        assert_eq!("si:falsey", falsey_func.name());
        dbg!(falsey_func.unique_id());

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

        dbg!(&funcs_by_unique_id);

        let qualifications = variant.qualifications().await.expect("get quals");
        assert_eq!(2, qualifications.len());

        for qual in qualifications {
            assert!(funcs_by_unique_id.contains_key(&qual.func_unique_id()));
        }

        // Ensure we get the props
        let props: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        variant
            .visit_prop_tree(prop_visitor, None, &props.clone())
            .await
            .expect("able to visit prop tree");

        // k8s deployments are really complex!
        assert_eq!(123, props.lock().await.len());
    }
}
