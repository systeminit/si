pub(crate) mod node;
mod pkg;
mod spec;

pub use pkg::{SiPkg, SiPkgError, SiPkgMetadata, SiPkgProp, SiPkgSchema, SiPkgSchemaVariant};
pub use spec::{
    FuncBackendKind, FuncBackendResponseType, FuncSpec, PkgSpec, PkgSpecBuilder, PropSpec,
    PropSpecBuilder, PropSpecKind, SchemaSpec, SchemaSpecBuilder, SchemaVariantSpec,
    SchemaVariantSpecBuilder, SpecError,
};

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use petgraph::dot::Dot;

    use crate::spec::PkgSpec;

    use super::*;

    const PACKAGE_JSON: &str = include_str!("../pkg-complex.json");

    fn base_path() -> PathBuf {
        let base_path = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir cannot be computed"),
        )
        .join("pkg");
        fs::create_dir_all(&base_path).expect("failed to create base path");

        base_path
    }

    #[test]
    fn create_pkg() {
        let spec: PkgSpec = serde_json::from_str(PACKAGE_JSON).unwrap();
        dbg!(&spec);

        let pkg = SiPkg::load_from_spec(spec).expect("failed to load spec");

        let (graph, _root_idx) = pkg.as_petgraph();

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
        dbg!(&read_pkg);
    }
}
