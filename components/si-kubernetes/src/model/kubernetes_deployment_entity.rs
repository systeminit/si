use crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject;
use si_cea::entity::prelude::*;
use std::convert::TryFrom;

pub use crate::protobuf::KubernetesDeploymentEntity;

impl TryFrom<&KubernetesDeploymentEntityPropertiesKubernetesObject> for String {
    type Error = CeaError;

    fn try_from(
        value: &KubernetesDeploymentEntityPropertiesKubernetesObject,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(serde_yaml::to_string(value)?)
    }
}

impl TryFrom<&String> for KubernetesDeploymentEntityPropertiesKubernetesObject {
    type Error = CeaError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(serde_yaml::from_str(value)?)
    }
}
