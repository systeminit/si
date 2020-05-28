import {
  PropObject,
  PropText,
  PropLink,
  PropNumber,
  PropEnum,
  PropCode,
} from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "kubernetesDeployment",
  displayTypeName: "Kubernetes Deployment Object",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    // Constraints
    c.constraints.addEnum({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
      options(p: PropEnum) {
        p.variants = ["v1.12", "v1.13", "v1.14", "v1.15"];
      },
    });

    // Properties
    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options(p: PropObject) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.properties.addText({
          name: "apiVersion",
          label: "API Version",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "kind",
          label: "Kind",
          options(p: PropText) {
            p.required = true;
            p.baseDefaultValue = "Deployment";
          },
        });
        p.properties.addLink({
          name: "metadata",
          label: "Metadata",
          options(p: PropLink) {
            p.lookup = {
              typeName: "kubernetesMetadata",
            };
          },
        });
        p.properties.addObject({
          name: "spec",
          label: "Deployment Spec",
          options(p: PropObject) {
            p.properties.addNumber({
              name: "replicas",
              label: "Replicas",
              options(p: PropNumber) {
                p.numberKind = "uint32";
              },
            });
            p.properties.addLink({
              name: "selector",
              label: "Selector",
              options(p: PropLink) {
                p.lookup = {
                  typeName: "kubernetesSelector",
                };
              },
            });
            p.properties.addLink({
              name: "template",
              label: "Pod Template Spec",
              options(p: PropLink) {
                p.lookup = {
                  typeName: "kubernetesPodTemplateSpec",
                };
              },
            });
          },
        });
      },
    });

    c.properties.addCode({
      name: "kubernetesObjectYaml",
      label: "Kubernetes Object YAML",
      options(p: PropCode) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesDeploymentEntity",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.language = "yaml";
      },
    });
  },
});