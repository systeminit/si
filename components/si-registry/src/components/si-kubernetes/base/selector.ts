import { registry } from "../../../registry";

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
