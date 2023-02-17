pub mod schema {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Schema {
        pub domain: Prop,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    pub enum Prop {
        #[serde(rename_all = "camelCase")]
        String { name: String },
        #[serde(rename_all = "camelCase")]
        Number { name: String },
        #[serde(rename_all = "camelCase")]
        Boolean { name: String },
        #[serde(rename_all = "camelCase")]
        Map { name: String, type_prop: Box<Prop> },
        #[serde(rename_all = "camelCase")]
        Array { name: String, type_prop: Box<Prop> },
        #[serde(rename_all = "camelCase")]
        Object { name: String, entries: Vec<Prop> },
    }

    pub mod node;

    #[cfg(test)]
    mod tests {
        use std::{env, fs, path::PathBuf};

        use object_tree::{ObjectTree, TreeFileSystemReader, TreeFileSystemWriter};
        use petgraph::dot::Dot;

        use crate::schema::node::PropNode;

        use super::*;

        const SCHEMA_JSON: &str = include_str!("../schema-complex.json");

        fn base_path() -> PathBuf {
            let base_path = PathBuf::from(
                env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir cannot be computed"),
            )
            .join("pkg");
            fs::create_dir_all(&base_path).expect("failed to create base path");

            base_path
        }

        #[test]
        fn create_prop_tree() {
            let schema: Schema = serde_json::from_str(SCHEMA_JSON).unwrap();
            dbg!(&schema);

            let tree =
                ObjectTree::create_from_root(schema.domain.into()).expect("failed to hash tree");
            // dbg!(&hashed_tree);

            let (graph, _root_idx) = tree.as_petgraph();

            // println!("{}", serde_json::to_string_pretty(&graph).unwrap());

            println!("\n---- snip ----\n{:?}\n---- snip ----", Dot::new(graph));
        }

        #[tokio::test]
        async fn prop_tree_fs_round_trip() {
            let schema: Schema = serde_json::from_str(SCHEMA_JSON).unwrap();
            let tree =
                ObjectTree::create_from_root(schema.domain.into()).expect("failed to hash tree");
            dbg!(&tree);

            TreeFileSystemWriter::physical(base_path())
                .write(&tree)
                .await
                .expect("failed to write tree");

            let read_tree: ObjectTree<PropNode> = TreeFileSystemReader::physical(base_path())
                .read()
                .await
                .expect("failed to read tree");
            dbg!(&read_tree);
        }
    }
}
