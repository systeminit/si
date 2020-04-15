import {
  Component,
  PropObject,
  PropText,
  PropLink,
  PropNumber,
  PropCode,
} from "@/components/prelude";
import { registry } from "@/componentRegistry";

registry.component({
  typeName: "kubernetesDeployment",
  displayTypeName: "Kubernetes Deployment Object",
  siPathName: "si-kubernetes",
  options(c: Component) {
    // Constraints
    c.constraints.addText({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
    });

    // Properties
    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options(p: PropObject) {
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
              component: "kubernetes",
              propType: "internalOnly",
              names: ["metadata"],
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
                  component: "kubernetes",
                  propType: "internalOnly",
                  names: ["labelSelector"],
                };
              },
            });
            p.properties.addLink({
              name: "template",
              label: "Pod Template Spec",
              options(p: PropLink) {
                p.lookup = {
                  component: "kubernetes",
                  propType: "internalOnly",
                  names: ["podTemplateSpec"],
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
        p.language = "yaml";
      },
    });
  },
});
