import {
  Component,
  PropObject,
  PropText,
  PropLink,
  PropNumber,
} from "../../components/prelude";

import { registry } from "../../componentRegistry";

registry.component({
  typeName: "kubernetes",
  displayTypeName: "Kubernetes",
  noStd: true,
  options(c: Component) {
    // Metadata
    c.internalOnly.addObject({
      name: "metadata",
      label: "Meta Data",
      options(p: PropObject) {
        p.properties.addText({
          name: "name",
          label: "Name",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addMap({
          name: "labels",
          label: "Labels",
        });
      },
    });

    // LabelSelector
    c.internalOnly.addObject({
      name: "selector",
      label: "Label Selector",
      options(p: PropObject) {
        p.properties.addMap({
          name: "matchLabels",
          label: "Match Labels",
        });
      },
    });

    c.internalOnly.addObject({
      name: "container",
      label: "Container",
      options(p: PropObject) {
        p.properties.addText({
          name: "name",
          label: "Name",
        });
        p.properties.addText({
          name: "image",
          label: "Image",
        });
        p.properties.addObject({
          name: "ports",
          label: "Ports",
          options(p: PropObject) {
            p.repeated = true;
            p.properties.addObject({
              name: "portValues",
              label: "Port Values",
              options(p: PropObject) {
                p.properties.addNumber({
                  name: "containerPort",
                  label: "Container Port",
                  options(p: PropNumber) {
                    p.numberKind = "uint32";
                  },
                });
              },
            });
          },
        });
      },
    });

    c.internalOnly.addObject({
      name: "podSpec",
      label: "Pod Spec",
      options(p: PropObject) {
        p.properties.addLink({
          name: "containers",
          label: "Containers",
          options(p: PropLink) {
            p.repeated = true;
            p.lookup = {
              component: "kubernetes",
              propType: "internalOnly",
              names: ["container"],
            };
          },
        });
      },
    });

    // PodTemplateSpec
    c.internalOnly.addObject({
      name: "podTemplateSpec",
      label: "Pod Template Spec",
      options(p: PropObject) {
        p.properties.addLink({
          name: "metadata",
          label: "Meta Data",
          options(p: PropLink) {
            p.lookup = {
              component: "kubernetes",
              propType: "internalOnly",
              names: ["metadata"],
            };
          },
        });
        p.properties.addLink({
          name: "spec",
          label: "Pod Spec",
          options(p: PropLink) {
            p.lookup = {
              component: "kubernetes",
              propType: "internalOnly",
              names: ["podSpec"],
            };
          },
        });
      },
    });
  },
});
