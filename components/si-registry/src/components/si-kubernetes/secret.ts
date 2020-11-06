import {
  PropObject,
  PropText,
  PropLink,
  PropEnum,
  PropCode,
  PropAction,
} from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "kubernetesSecret",
  displayTypeName: "Kubernetes Secret Object",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.inputType("kubernetesCluster");
    c.entity.inputType("kubernetesNamespace");
    c.entity.inputType("dockerHubCredential");

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
        p.baseDefaultValue = "v1.15";
      },
    });

    // Properties
    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options(p: PropObject) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesNamespace",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesNamespace",
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
            p.baseDefaultValue = "Secret";
            p.baseValidation = p
              .validation()
              .min(3)
              .max(10)
              .required();
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
        p.properties.addMap({
          name: "data",
          label: "Data",
        });
        p.properties.addMap({
          name: "stringData",
          label: "StringData",
        });
        p.properties.addBool({
          name: "immutable",
          label: "immutable",
        });
        p.properties.addText({
          name: "type",
          label: "type",
        });
      },
    });
    c.properties.addCode({
      name: "kubernetesObjectYaml",
      label: "Kubernetes Object YAML",
      options(p: PropCode) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesNamespace",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesNamespace",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.language = "yaml";
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "apply",
      label: "Apply",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
  },
});
