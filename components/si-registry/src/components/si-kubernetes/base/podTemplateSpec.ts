import { PropLink } from "../../../components/prelude";

import { registry } from "../../../registry";

// Depends on:
// - kubernetesMetadata
// - kubernetesPodSpec

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
