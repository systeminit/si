import {
  PropObject,
  PropText,
  PropLink,
  PropNumber,
} from "../../components/prelude";

import { registry } from "../../registry";

registry.base({
  typeName: "kubernetesMetadata",
  displayTypeName: "Kubernetes Meta Data",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "name",
      label: "Name",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addMap({
      name: "labels",
      label: "Labels",
    });
  },
});

registry.base({
  typeName: "kubernetesSelector",
  displayTypeName: "Kubernetes Label Selector",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addMap({
      name: "matchLabels",
      label: "Match Labels",
    });
  },
});

registry.base({
  typeName: "kubernetesContainer",
  displayTypeName: "Kubernetes Container Definition",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "name",
      label: "Name",
    });
    c.fields.addText({
      name: "image",
      label: "Image",
    });
    c.fields.addObject({
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

registry.base({
  typeName: "kubernetesPodSpec",
  displayTypeName: "Kubernetes Pod Spec",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addLink({
      name: "containers",
      label: "Containers",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesContainer",
        };
      },
    });
  },
});

registry.base({
  typeName: "kubernetesPodTemplateSpec",
  displayTypeName: "Kubernetes Pod Template Spec",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addLink({
      name: "metadata",
      label: "Meta Data",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesMetadata",
        };
      },
    });
    c.fields.addLink({
      name: "spec",
      label: "Pod Spec",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesPodSpec",
        };
      },
    });
  },
});
