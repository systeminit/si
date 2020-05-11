use crate::protobuf::KubernetesDeploymentEntityPropertiesKubernetesObject;
use si_cea::entity::prelude::*;
use std::convert::{TryFrom, TryInto};

pub use crate::protobuf::KubernetesDeploymentEntity;

impl KubernetesDeploymentEntity {
    // TODO: fix
    // codegen this method and account for the linked kubernetes_object_yaml property
    pub fn edit_kubernetes_object(
        &mut self,
        property: KubernetesDeploymentEntityPropertiesKubernetesObject,
    ) -> CeaResult<()> {
        self.properties_mut()?.kubernetes_object = Some(property);
        self.update_kubernetes_object_yaml_from_kubernetes_object()?;

        Ok(())
    }

    // TODO: fix
    // codegen this method and account for the linked kubernetes_object property
    pub fn edit_kubernetes_object_yaml(&mut self, property: String) -> CeaResult<()> {
        self.properties_mut()?.kubernetes_object_yaml = Some(property);
        self.update_kubernetes_object_from_kubernetes_object_yaml()?;

        Ok(())
    }

    // TODO: fix
    // we should be able to codegen this method to call in the "edit" method
    fn update_kubernetes_object_yaml_from_kubernetes_object(&mut self) -> CeaResult<()> {
        if let Some(ref kubernetes_object) = self.properties()?.kubernetes_object {
            self.properties_mut()?.kubernetes_object_yaml = Some(kubernetes_object.try_into()?);
        }
        Ok(())
    }

    // TODO: fix
    // we should be able to codegen this method to call in the "edit" method
    fn update_kubernetes_object_from_kubernetes_object_yaml(&mut self) -> CeaResult<()> {
        if let Some(ref kubernetes_object_yaml) = self.properties()?.kubernetes_object_yaml {
            self.properties_mut()?.kubernetes_object = Some(kubernetes_object_yaml.try_into()?);
        }
        Ok(())
    }
}

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
