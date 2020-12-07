import {
  PropLink,
  PropObject,
  PropText,
  PropCode,
  PropAction,
} from "../../../components/prelude";

import { ComponentAndEntityObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface Ingress {
//   apiVersion: Ingress.version;
//   kind: Ingress.kind;
//   metadata: ObjectMeta | undefined;
//   spec: IngressSpec;
//   status: IngressStatus;
// }

// Depends on:
// - kubernetesMetadata
// - kubernetesIngressSpec
// - kubernetesIngressStatus

let kubernetesIngress = {
  typeName: "kubernetesIngress",
  displayTypeName: "Kubernetes Ingress Object",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: ComponentAndEntityObject) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "kubernetes",
      uiMenuDisplayName: "ingress",
    };

    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });

    // Properties
    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options(p: PropObject) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesIngress",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesIngress",
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
            p.baseDefaultValue = "Ingress";
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
        p.properties.addLink({
          name: "spec",
          label: "Ingress Spec",
          options(p: PropLink) {
            p.lookup = {
              typeName: "kubernetesIngressSpec",
            };
          },
        });
        p.properties.addLink({
          name: "status",
          label: "Ingress Status",
          options(p: PropLink) {
            p.lookup = {
              typeName: "kubernetesIngressStatus",
            };
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
            typeName: "kubernetesIngress",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesIngress",
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
};

export { kubernetesIngress };

registry.componentAndEntity(kubernetesIngress);
