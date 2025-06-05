mod v1;
mod v2;

use std::{
    fmt,
    ops::{
        Deref,
        DerefMut,
    },
};

use naxum_api_types::{
    ApiVersionsWrapper,
    ApiWrapper,
    RequestId,
    UpgradeError,
};
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    EnumDiscriminants,
    EnumIs,
    EnumString,
    VariantNames,
};
pub use v1::{
    JobArgsV1,
    JobExecutionRequestV1,
};
use v2::JobArgsV2Discriminants;
pub use v2::{
    JobArgsV2,
    JobExecutionRequestV2,
};

pub type JobExecutionRequestVCurrent = JobExecutionRequestV2;

pub type JobArgsVCurrent = JobArgsV2;
pub type JobArgsVCurrentDiscriminants = JobArgsV2Discriminants;

#[derive(Clone, Eq, Serialize, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum JobExecutionRequest {
    V2(JobExecutionRequestV2),
}

impl ApiWrapper for JobExecutionRequest {
    type VersionsTarget = JobExecutionRequestVersions;
    type Current = JobExecutionRequestVCurrent;

    const MESSAGE_TYPE: &'static str = "JobExecutionRequest";

    fn id(&self) -> naxum_api_types::RequestId {
        match self {
            Self::V2(JobExecutionRequestVCurrent { id, .. }) => *id,
        }
    }

    fn new_current(current: Self::Current) -> Self {
        Self::V2(current)
    }
}

impl fmt::Debug for JobExecutionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V2(inner) => inner.fmt(f),
        }
    }
}

impl Deref for JobExecutionRequest {
    type Target = JobExecutionRequestVCurrent;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::V2(inner) => inner,
        }
    }
}

impl DerefMut for JobExecutionRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::V2(inner) => inner,
        }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, EnumDiscriminants, EnumIs, Eq, PartialEq, VariantNames)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[strum_discriminants(strum(serialize_all = "camelCase"), derive(AsRefStr, EnumString))]
pub enum JobExecutionRequestVersions {
    V1(JobExecutionRequestV1),
    V2(JobExecutionRequestV2),
}

impl ApiVersionsWrapper for JobExecutionRequestVersions {
    type Target = JobExecutionRequest;

    fn id(&self) -> RequestId {
        match self {
            Self::V1(JobExecutionRequestV1 { id, .. }) => *id,
            Self::V2(JobExecutionRequestV2 { id, .. }) => *id,
        }
    }

    fn into_current_version(self) -> Result<Self::Target, UpgradeError> {
        match self {
            Self::V1(inner) => Ok(Self::Target::V2(JobExecutionRequestV2 {
                id: inner.id,
                workspace_id: inner.workspace_id,
                change_set_id: inner.change_set_id,
                args: match inner.args {
                    JobArgsV1::Action { action_id } => JobArgsV2::Action { action_id },
                    JobArgsV1::DependentValuesUpdate => JobArgsV2::DependentValuesUpdate,
                    JobArgsV1::Validation {
                        attribute_value_ids,
                    } => JobArgsV2::Validation {
                        attribute_value_ids,
                    },
                },
                is_job_blocking: inner.is_job_blocking,
            })),
            Self::V2(inner) => Ok(Self::Target::V2(inner)),
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

    use super::JobExecutionRequestVersionsDiscriminants;

    /// Tests that a versioned object will always serialize to the same representation, no matter
    /// what future versions or changes to the object.
    ///
    /// NOTE: It is imperative that incrementatl refactorings do not lead to a version-incompatible
    /// or serialize-incompatible change. If a test fails and it's because there is a diff to the
    /// commiteed `.snap` file, this should be considered a failed refactoring. The remediation is
    /// to *not* update the `.snap` file but rather to fix the code so that the `.snap` format is
    /// 100% preserved.
    pub fn assert_serialize(
        name: &str,
        version: JobExecutionRequestVersionsDiscriminants,
        serialize: impl Serialize,
    ) {
        insta::with_settings!({
            snapshot_path => format!("job_execution_request/snapshots-{}", version.as_ref()),
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

    pub fn assert_deserialize<T>(
        snapshot_name: &str,
        version: JobExecutionRequestVersionsDiscriminants,
        expected: T,
    ) where
        T: fmt::Debug + DeserializeOwned + PartialEq,
    {
        let actual: T = read_from_snapshot(
            snapshot_name,
            &format!("job_execution_request/snapshots-{}", version.as_ref()),
        )
        .expect("failed to deserialize from snapshot");

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
