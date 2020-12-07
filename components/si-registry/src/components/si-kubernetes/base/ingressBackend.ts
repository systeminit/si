import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressBackend {
//   resource: TypedLocalObjectReference
//   service: IngressServiceBackend
// }

// Depends on:
// - kubernetesTypedLocalObjectReference
// - kubernetesIngressServiceBackend

let kubernetesIngressBackend = {
  typeName: "kubernetesIngressBackend",
  displayTypeName: "Kubernetes Ingress Backend",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addLink({
      name: "resource",
      label: "Resource",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesTypedLocalObjectReference",
        };
      },
    });

    c.fields.addLink({
      name: "service",
      label: "Service",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesIngressServiceBackend",
        };
      },
    });
  },
};
export { kubernetesIngressBackend };

registry.base(kubernetesIngressBackend);
