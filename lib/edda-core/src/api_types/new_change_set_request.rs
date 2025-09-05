use acceptable::{
    AllVersions,
    Container,
    CurrentContainer,
    IntoContainer,
    UpgradeError,
};
use serde::Deserialize;

mod v1;

pub use self::v1::NewChangeSetRequestV1;

#[remain::sorted]
#[derive(AllVersions, CurrentContainer, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum NewChangeSetRequestAllVersions {
    #[acceptable(current)]
    V1(NewChangeSetRequestV1),
}

impl IntoContainer for NewChangeSetRequestAllVersions {
    type Container = NewChangeSetRequest;

    fn into_container(self) -> Result<Self::Container, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Container::new(inner)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        error::Error,
        fmt,
        fs::File,
        io::{
            self,
            BufRead as _,
            BufReader,
            Read as _,
        },
        path::Path,
    };

    use serde::{
        Serialize,
        de::DeserializeOwned,
    };

    /// Tests that a versioned object will always serialize to the same representation, no matter
    /// what future versions or changes to the object.
    ///
    /// NOTE: It is imperative that incrementatl refactorings do not lead to a version-incompatible
    /// or serialize-incompatible change. If a test fails and it's because there is a diff to the
    /// commiteed `.snap` file, this should be considered a failed refactoring. The remediation is
    /// to *not* NewChangeSet the `.snap` file but rather to fix the code so that the `.snap` format is
    /// 100% preserved.
    pub fn assert_serialize(name: &str, version: u64, serialize: impl Serialize) {
        insta::with_settings!({
            snapshot_path => format!("new_change_set_request/snapshots-v{version}"),
            prepend_module_to_snapshot => false,
            omit_expression => true,
            description => concat!(
                "\n",
                "\n",
                "!!!\n",
                "!!! System Initiative Developers:\n",
                "!!!\n",
                "!!! IMPORTANT:\n",
                "!!!\n",
                "!!!     The contents of this snapshot should *never* be modified as it\n",
                "!!!     represents the serialization of a versioned Rust type. If a tests fails\n",
                "!!!     with this warning, then something about a Rust type has changed\n",
                "!!!     the wire serialization of this type and would represent a potential\n",
                "!!!     production outage or data corruption.\n",
                "!!!\n",
                "!!!     Consider this an erroneous behavioral change of the Rust code and *not*\n",
                "!!!     an out-of-date snapshot or fixture!\n",
                "!!!\n",
                "\n",
                "\n",
            ),
        }, {
            insta::assert_json_snapshot!(name, serialize);
        });
    }

    pub fn assert_deserialize<T>(snapshot_name: &str, version: u64, expected: T)
    where
        T: fmt::Debug + DeserializeOwned + PartialEq,
    {
        let actual: T = read_from_snapshot(
            snapshot_name,
            &format!("new_change_set_request/snapshots-v{version}"),
        )
        .expect("failed to deserialize from snapshot");
        dbg!(&actual);
        assert_eq!(actual, expected);
    }

    fn read_from_snapshot<T>(name: &str, path: &str) -> Result<T, Box<dyn Error>>
    where
        T: fmt::Debug + DeserializeOwned + PartialEq,
    {
        let glob = format!("{path}/{name}.snap");

        let mut maybe_obj_result = None;
        insta::glob!(&glob, |path| {
            let bytes = read_snapshot_content(path).unwrap();

            maybe_obj_result = Some(serde_json::from_slice(&bytes).map_err(Into::into));
        });

        match maybe_obj_result {
            Some(object_result) => object_result,
            None => Err(Box::new(io::Error::other(format!(
                "snapshot not found: {glob}"
            )))),
        }
    }

    // Implementation is adapted from the [`insta::Snapshot::from_file`] function.
    //
    // See: <https://github.com/mitsuhiko/insta/blob/62bb0a3/insta/src/snapshot.rs#L339-L421>
    fn read_snapshot_content(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut f = BufReader::new(File::open(path)?);

        // Skip through the snapshot metadata, that being the first YAML document, where YAML
        // documents are delimited with a `"---"` line
        {
            let mut buf = String::new();
            f.read_line(&mut buf)?;
            if buf.trim_end() == "---" {
                loop {
                    let read = f.read_line(&mut buf)?;
                    if read == 0 {
                        break;
                    }
                    if buf[buf.len() - read..].trim_end() == "---" {
                        break;
                    }
                }
            }
        }

        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }
}
